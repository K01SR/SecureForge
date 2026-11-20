use serde::{Deserialize, Serialize};
use std::path::Path;
use sih149_core::wiper::firmware::{
    FirmwareMethod,
    ata_detect_frozen, detect_nvme_capabilities, detect_hpa_dco,
    nvme_crypto_erase, nvme_block_erase, ata_secure_erase, ata_enhanced_erase,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareEraseConfig {
    /// Block device path, e.g. /dev/sdb or /dev/nvme1n1
    pub device_path: String,
    /// "auto", "nvme-crypto", "nvme-block", "ata", or "ata-enhanced"
    pub method: String,
    /// ATA security password (only used for ATA methods)
    pub ata_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareCapabilities {
    pub is_nvme: bool,
    pub nvme_sanitize_supported: bool,
    pub ata_frozen: bool,
    pub hpa_enabled: bool,
    pub dco_enabled: bool,
    pub recommended_method: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareEraseResult {
    pub method_used: String,
    pub success: bool,
    pub command_output: String,
    pub duration_secs: u64,
    pub warnings: Vec<String>,
}

/// Called before erase to show the user what we found and what will happen.
#[tauri::command]
pub fn detect_firmware_capabilities(device_path: String) -> Result<FirmwareCapabilities, String> {
    let device = Path::new(&device_path);
    let is_nvme = device_path.contains("nvme");
    let mut warnings = Vec::new();

    let nvme_sanitize_supported = if is_nvme {
        match detect_nvme_capabilities(device) {
            Ok(caps) => caps.sanitize_supported,
            Err(e) => {
                warnings.push(format!("Could not query NVMe capabilities: {} (nvme-cli installed?)", e));
                false
            }
        }
    } else {
        false
    };

    let ata_frozen = if !is_nvme {
        match ata_detect_frozen(device) {
            Ok(frozen) => {
                if frozen {
                    warnings.push(
                        "Drive is in FROZEN security state. Power-cycle (not reboot) \
                         the machine to unfreeze before issuing a secure erase.".to_string()
                    );
                }
                frozen
            }
            Err(e) => {
                warnings.push(format!("Could not check ATA frozen state: {} (hdparm installed?)", e));
                false
            }
        }
    } else {
        false
    };

    let (hpa_enabled, dco_enabled) = if !is_nvme {
        match detect_hpa_dco(device) {
            Ok(info) => {
                if info.hpa_enabled {
                    warnings.push(
                        "Host Protected Area (HPA) detected — hidden sectors may survive the erase. \
                         Use hdparm --yes-i-know-what-i-am-doing --dco-restore first for full coverage."
                            .to_string()
                    );
                }
                if info.dco_enabled {
                    warnings.push(
                        "Device Configuration Overlay (DCO) detected — drive may report \
                         fewer sectors than it physically contains."
                            .to_string()
                    );
                }
                (info.hpa_enabled, info.dco_enabled)
            }
            Err(e) => {
                warnings.push(format!("Could not check HPA/DCO: {}", e));
                (false, false)
            }
        }
    } else {
        (false, false)
    };

    let recommended_method = if is_nvme {
        if nvme_sanitize_supported { "nvme-crypto" } else { "nvme-block" }
    } else if !ata_frozen {
        "ata-enhanced"
    } else {
        "ata"
    }
    .to_string();

    Ok(FirmwareCapabilities {
        is_nvme,
        nvme_sanitize_supported,
        ata_frozen,
        hpa_enabled,
        dco_enabled,
        recommended_method,
        warnings,
    })
}

#[tauri::command]
pub async fn start_firmware_erase(config: FirmwareEraseConfig) -> Result<FirmwareEraseResult, String> {
    tokio::task::spawn_blocking(move || -> Result<FirmwareEraseResult, String> {
        let device = Path::new(&config.device_path);
        let is_nvme = config.device_path.contains("nvme");
        let mut warnings = Vec::new();

        let method = match config.method.as_str() {
            "nvme-crypto"  => FirmwareMethod::NvmeCryptoErase,
            "nvme-block"   => FirmwareMethod::NvmeBlockErase,
            "ata"          => FirmwareMethod::AtaSecureErase,
            "ata-enhanced" => FirmwareMethod::AtaEnhancedErase,
            "auto" => {
                if is_nvme {
                    match detect_nvme_capabilities(device) {
                        Ok(caps) if caps.sanitize_supported => FirmwareMethod::NvmeCryptoErase,
                        _ => {
                            warnings.push("NVMe sanitize capability not confirmed — using block erase.".to_string());
                            FirmwareMethod::NvmeBlockErase
                        }
                    }
                } else {
                    FirmwareMethod::AtaEnhancedErase
                }
            }
            other => return Err(format!("Unknown method '{}'. Use: auto, nvme-crypto, nvme-block, ata, ata-enhanced", other)),
        };

        // Safety check: refuse if ATA drive is frozen (command will fail anyway, but give a clear message)
        if matches!(method, FirmwareMethod::AtaSecureErase | FirmwareMethod::AtaEnhancedErase) {
            if let Ok(true) = ata_detect_frozen(device) {
                return Err(
                    "Drive is in FROZEN ATA security state — power-cycle the machine (not reboot) \
                     before issuing a secure erase command.".to_string()
                );
            }
        }

        let generated_pass: String = {
            use rand::RngCore;
            let mut bytes = [0u8; 8];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            bytes.iter().map(|b| format!("{:02x}", b)).collect()
        };
        let ata_password = config.ata_password.as_deref().unwrap_or(&generated_pass);

        let result = match &method {
            FirmwareMethod::NvmeCryptoErase => nvme_crypto_erase(device)
                .map_err(|e| e.to_string())?,
            FirmwareMethod::NvmeBlockErase  => nvme_block_erase(device)
                .map_err(|e| e.to_string())?,
            FirmwareMethod::AtaSecureErase  => ata_secure_erase(device, ata_password)
                .map_err(|e| e.to_string())?,
            FirmwareMethod::AtaEnhancedErase => ata_enhanced_erase(device, ata_password)
                .map_err(|e| e.to_string())?,
        };

        let method_label = match &method {
            FirmwareMethod::NvmeCryptoErase  => "nvme-crypto",
            FirmwareMethod::NvmeBlockErase   => "nvme-block",
            FirmwareMethod::AtaSecureErase   => "ata",
            FirmwareMethod::AtaEnhancedErase => "ata-enhanced",
        };

        Ok(FirmwareEraseResult {
            method_used: method_label.to_string(),
            success: result.success,
            command_output: result.command_output,
            duration_secs: result.duration_secs,
            warnings,
        })
    })
    .await
    .map_err(|e| format!("Firmware erase task panicked: {}", e))?
}
