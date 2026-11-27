//! Firmware-level secure erase commands.
//! Wraps `hdparm` (ATA) and `nvme-cli` (NVMe) via subprocess.
//!
//! WARNING: every function here that performs an erase is genuinely
//! destructive and irreversible against real hardware. Requires root
//! (CAP_SYS_ADMIN) and the `hdparm`/`nvme-cli` packages installed.
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq)]
pub enum FirmwareMethod {
    NvmeCryptoErase,
    NvmeBlockErase,
    AtaSecureErase,
    AtaEnhancedErase,
}

#[derive(Debug)]
pub struct FirmwareEraseResult {
    pub method: FirmwareMethod,
    pub success: bool,
    pub command_output: String,
    pub duration_secs: u64,
}

#[derive(Debug)]
pub struct NvmeCapabilities {
    pub sanitize_supported: bool,
}

#[derive(Debug)]
pub struct HpaInfo {
    pub hpa_enabled: bool,
    pub dco_enabled: bool,
}

/// Validates that a device path is a safe block-device target for
/// firmware-level commands. Only accepts canonical absolute paths under
/// /dev with no single-dot/double-dot traversal, no shell metacharacters,
/// and requires the target to actually exist as a block device.
fn valid_block_device(device: &Path) -> bool {
    let s = device.to_string_lossy();
    // Must be an absolute /dev path, not a relative/arbitrary path.
    if !s.starts_with("/dev/") {
        return false;
    }
    // Refuse traversal and shell-unsafe characters outright.
    if s.contains("..") || s.contains('\0') {
        return false;
    }
    if s.chars().any(|c| matches!(c, ' ' | ';' | '&' | '|' | '`' | '$' | '(' | ')' | '<' | '>' | '\'' | '"' | '\\')) {
        return false;
    }
    // Must actually be a block device node (not a regular file, not a directory).
    use std::os::unix::fs::FileTypeExt;
    match std::fs::symlink_metadata(device) {
        Ok(md) => md.file_type().is_block_device(),
        Err(_) => false,
    }
}

fn run_command(cmd: &str, device: &Path, args: &[&str]) -> Result<(bool, String), CoreError> {
    if !valid_block_device(device) {
        return Err(CoreError::Wiper(format!(
            "Refusing to run '{}' on unsafe target: {} (must be a valid /dev block device path)",
            cmd,
            device.display()
        )));
    }
    let device_str = device.to_string_lossy();
    let mut full_args: Vec<&str> = Vec::with_capacity(args.len() + 1);
    full_args.extend_from_slice(args);
    full_args.push(&device_str);

    let output = Command::new(cmd)
        .args(&full_args)
        .output()
        .map_err(|e| CoreError::Wiper(format!("Failed to run {} {:?}: {} (is {} installed and are you running as root?)", cmd, full_args, e, cmd)))?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok((output.status.success(), combined))
}

/// Checks the NVMe Sanitize log page for crypto-erase support.
/// Parses `nvme id-ctrl` SANICAP bits via `nvme-cli`'s human-readable
/// output — brittle against nvme-cli version differences; treat a `false`
/// result conservatively (may be a parse miss, not definitive absence).
pub fn detect_nvme_capabilities(device: &Path) -> Result<NvmeCapabilities, CoreError> {
    let (ok, output) = run_command("nvme", device, &["id-ctrl", "-H"])?;
    if !ok {
        return Err(CoreError::Wiper(format!("nvme id-ctrl failed: {}", output)));
    }
    // Human-readable id-ctrl output includes a line like:
    // "[0:0] : 0x1  Crypto Erase Supported"
    let sanitize_supported = output.to_lowercase().contains("crypto erase supported");
    Ok(NvmeCapabilities { sanitize_supported })
}

/// NVMe Sanitize with Crypto Erase (sanact=2). Cryptographically destroys
/// the media encryption key — data becomes unrecoverable even without
/// overwriting every block, but only if the drive actually implements
/// full-disk encryption at rest (verify via detect_nvme_capabilities first).
pub fn nvme_crypto_erase(device: &Path) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();
    let (success, output) = run_command("nvme", device, &["sanitize", "--sanact=2"])?;
    Ok(FirmwareEraseResult {
        method: FirmwareMethod::NvmeCryptoErase,
        success,
        command_output: output,
        duration_secs: started.elapsed().as_secs(),
    })
}

/// NVMe Sanitize with Block Erase (sanact=1). Physically erases every
/// block at the flash-translation-layer level. Slower than crypto erase
/// but doesn't depend on the drive's encryption implementation.
pub fn nvme_block_erase(device: &Path) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();
    let (success, output) = run_command("nvme", device, &["sanitize", "--sanact=1"])?;
    Ok(FirmwareEraseResult {
        method: FirmwareMethod::NvmeBlockErase,
        success,
        command_output: output,
        duration_secs: started.elapsed().as_secs(),
    })
}

/// Checks whether the ATA drive is in a "frozen" security state — BIOS/
/// firmware sometimes freezes the security feature set at boot, which
/// blocks secure-erase commands until the drive is power-cycled (not just
/// rebooted) with no intervening freeze command from the OS.
pub fn ata_detect_frozen(device: &Path) -> Result<bool, CoreError> {
    let (ok, output) = run_command("hdparm", device, &["-I"])?;
    if !ok {
        return Err(CoreError::Wiper(format!("hdparm -I failed: {}", output)));
    }
    Ok(output.contains("frozen"))
}

/// ATA Security Erase (single pass, drive-internal). Requires setting a
/// temporary security password first, per the ATA spec — hdparm handles
/// both steps here as two calls.
pub fn ata_secure_erase(device: &Path, password: &str) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();

    let (set_ok, set_out) = run_command("hdparm", device, &[
        "--user-master", "u", "--security-set-pass", password
    ])?;
    if !set_ok {
        return Ok(FirmwareEraseResult {
            method: FirmwareMethod::AtaSecureErase,
            success: false,
            command_output: format!("security-set-pass failed: {}", set_out),
            duration_secs: started.elapsed().as_secs(),
        });
    }

    let (erase_ok, erase_out) = run_command("hdparm", device, &[
        "--user-master", "u", "--security-erase", password
    ])?;

    Ok(FirmwareEraseResult {
        method: FirmwareMethod::AtaSecureErase,
        success: erase_ok,
        command_output: format!("{}\n{}", set_out, erase_out),
        duration_secs: started.elapsed().as_secs(),
    })
}

/// ATA Enhanced Security Erase — writes a vendor-defined pattern to every
/// user-addressable sector, including reallocated/reassigned sectors that
/// plain Secure Erase can miss. Not all drives support the enhanced mode;
/// check via `hdparm -I` for "supported: enhanced erase" before calling.
pub fn ata_enhanced_erase(device: &Path, password: &str) -> Result<FirmwareEraseResult, CoreError> {
    let started = Instant::now();

    let (set_ok, set_out) = run_command("hdparm", device, &[
        "--user-master", "u", "--security-set-pass", password
    ])?;
    if !set_ok {
        return Ok(FirmwareEraseResult {
            method: FirmwareMethod::AtaEnhancedErase,
            success: false,
            command_output: format!("security-set-pass failed: {}", set_out),
            duration_secs: started.elapsed().as_secs(),
        });
    }

    let (erase_ok, erase_out) = run_command("hdparm", device, &[
        "--user-master", "u", "--security-erase-enhanced", password
    ])?;

    Ok(FirmwareEraseResult {
        method: FirmwareMethod::AtaEnhancedErase,
        success: erase_ok,
        command_output: format!("{}\n{}", set_out, erase_out),
        duration_secs: started.elapsed().as_secs(),
    })
}

/// Detects Host Protected Area / Device Configuration Overlay — hidden
/// regions some drives reserve that standard erase commands don't always
/// reach. Parsing is heuristic (hdparm's text output isn't a stable API);
/// treat this as advisory, not a compliance guarantee.
pub fn detect_hpa_dco(device: &Path) -> Result<HpaInfo, CoreError> {
    let (n_ok, n_out) = run_command("hdparm", device, &["-N"])?;
    let hpa_enabled = n_ok && n_out.contains("HPA is enabled");

    let (dco_ok, dco_out) = run_command("hdparm", device, &["--dco-identify"])?;
    let dco_enabled = dco_ok && !dco_out.to_lowercase().contains("no dco");

    Ok(HpaInfo { hpa_enabled, dco_enabled })
}
