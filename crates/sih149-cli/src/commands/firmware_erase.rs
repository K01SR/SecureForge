use clap::Args;
use std::path::Path;
use sih149_core::wiper::firmware::{
    FirmwareMethod, ata_detect_frozen, detect_nvme_capabilities,
    detect_hpa_dco, nvme_crypto_erase, nvme_block_erase,
    ata_secure_erase, ata_enhanced_erase,
};
use crate::display;

/// Firmware-level secure erase: NVMe sanitize or ATA secure erase.
///
/// This goes deeper than software-layer overwrites. The drive's internal
/// controller handles the erase, including reallocated/remapped sectors
/// that the OS has no visibility into.
///
/// REQUIRES:
///   Linux: root (CAP_SYS_ADMIN), `nvme-cli` or `hdparm` packages installed
///   The target drive must NOT be the boot/system drive
#[derive(Args)]
pub struct FirmwareEraseArgs {
    /// Target block device (e.g. /dev/sdb, /dev/nvme1n1)
    #[arg(short, long)]
    pub device: String,

    /// Erase method:
    ///   nvme-crypto  — NVMe sanitize crypto erase (destroys encryption key)
    ///   nvme-block   — NVMe sanitize block erase (rewrites all blocks)
    ///   ata          — ATA Security Erase (single pass)
    ///   ata-enhanced — ATA Enhanced Security Erase (includes remapped sectors)
    ///   auto         — detect capabilities and choose the strongest available
    #[arg(short, long, default_value = "auto")]
    pub method: String,

    /// ATA security password (required for ATA methods, ignored for NVMe)
    #[arg(long, default_value = "secureforge_tmp")]
    pub ata_password: String,

    /// Skip confirmation — DANGER: this is irreversible firmware-level destruction
    #[arg(long)]
    pub yes: bool,
}

pub fn run(args: &FirmwareEraseArgs) -> anyhow::Result<()> {
    let device = Path::new(&args.device);

    if !args.yes {
        println!("═══════════════════════════════════════════════════");
        println!("  FIRMWARE-LEVEL ERASE — IRREVERSIBLE");
        println!("  Device : {}", args.device);
        println!("  Method : {}", args.method);
        println!("═══════════════════════════════════════════════════");
        println!();
        println!("This command issues a FIRMWARE secure erase command directly");
        println!("to the drive. The drive's controller erases ALL sectors,");
        println!("including reallocated blocks invisible to the OS.");
        println!();
        println!("There is NO undo. ALL data will be PERMANENTLY destroyed.");
        println!();
        print!("Type ERASE-FIRMWARE to continue: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut input = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(std::io::stdin()), &mut input)?;
        if input.trim() != "ERASE-FIRMWARE" {
            display::print_warn("Aborted by user.");
            return Ok(());
        }
    }

    display::print_section("Firmware Erase");

    let is_nvme = args.device.contains("nvme");
    let method = if args.method == "auto" {
        detect_best_method(device, is_nvme)?
    } else {
        parse_method(&args.method)?
    };

    println!("Device : {}", args.device);
    println!("Method : {:?}", method);

    // Pre-flight checks
    if matches!(method, FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase) {
        match ata_detect_frozen(device) {
            Ok(true) => {
                anyhow::bail!(
                    "Drive is in FROZEN security state — the BIOS locked it at boot.\n\
                     Power-cycle (not reboot) the machine without triggering another BIOS freeze,\n\
                     then retry. On some systems, hot-unplugging and reinserting breaks the freeze."
                );
            }
            Ok(false) => {}
            Err(e) => display::print_warn(&format!("Could not check frozen state: {} — continuing", e)),
        }

        match detect_hpa_dco(device) {
            Ok(info) => {
                if info.hpa_enabled {
                    display::print_warn(
                        "Host Protected Area (HPA) detected — hidden sectors exist that this erase may not reach. \
                         Use hdparm --yes-i-know-what-i-am-doing --dco-restore to expand before erasing for full coverage."
                    );
                }
                if info.dco_enabled {
                    display::print_warn(
                        "Device Configuration Overlay (DCO) detected — drive may report fewer sectors than it contains."
                    );
                }
            }
            Err(e) => display::print_warn(&format!("Could not check HPA/DCO: {} — continuing", e)),
        }
    }

    if matches!(method, FirmwareMethod::NvmeCryptoErase | FirmwareMethod::NvmeBlockErase) {
        match detect_nvme_capabilities(device) {
            Ok(caps) => {
                if matches!(method, FirmwareMethod::NvmeCryptoErase) && !caps.sanitize_supported {
                    display::print_warn(
                        "Crypto erase not confirmed via id-ctrl SANICAP — the drive may still support it \
                         (parse is heuristic). Proceeding, but verify the sanitize log page after completion."
                    );
                }
            }
            Err(e) => display::print_warn(&format!("Could not query NVMe capabilities: {} — continuing", e)),
        }
    }

    println!("Issuing firmware erase command…");

    let result = match method {
        FirmwareMethod::NvmeCryptoErase => nvme_crypto_erase(device)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
        FirmwareMethod::NvmeBlockErase => nvme_block_erase(device)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
        FirmwareMethod::AtaSecureErase => ata_secure_erase(device, &args.ata_password)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
        FirmwareMethod::AtaEnhancedErase => ata_enhanced_erase(device, &args.ata_password)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
    };

    println!();
    println!("Command output:\n{}", result.command_output.trim());
    println!();
    println!("Duration: {}s", result.duration_secs);

    if result.success {
        display::print_success("Firmware erase completed successfully.");
    } else {
        display::print_error("Firmware erase reported a failure. Check command output above.");
        anyhow::bail!("Firmware erase failed");
    }

    Ok(())
}

fn detect_best_method(device: &Path, is_nvme: bool) -> anyhow::Result<FirmwareMethod> {
    if is_nvme {
        match detect_nvme_capabilities(device) {
            Ok(caps) if caps.sanitize_supported => {
                println!("Auto-detected: NVMe crypto erase (SANICAP supported).");
                Ok(FirmwareMethod::NvmeCryptoErase)
            }
            _ => {
                println!("Auto-detected: NVMe block erase (falling back from crypto erase).");
                Ok(FirmwareMethod::NvmeBlockErase)
            }
        }
    } else {
        println!("Auto-detected: ATA Enhanced Security Erase (covers remapped sectors).");
        Ok(FirmwareMethod::AtaEnhancedErase)
    }
}

fn parse_method(s: &str) -> anyhow::Result<FirmwareMethod> {
    match s {
        "nvme-crypto"  => Ok(FirmwareMethod::NvmeCryptoErase),
        "nvme-block"   => Ok(FirmwareMethod::NvmeBlockErase),
        "ata"          => Ok(FirmwareMethod::AtaSecureErase),
        "ata-enhanced" => Ok(FirmwareMethod::AtaEnhancedErase),
        other => anyhow::bail!(
            "Unknown firmware erase method '{}'. Valid: nvme-crypto, nvme-block, ata, ata-enhanced, auto",
            other
        ),
    }
}
