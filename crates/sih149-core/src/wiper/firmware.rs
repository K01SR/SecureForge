//! Firmware-level secure erase commands
//! Wraps hdparm and nvme-cli via subprocess
use std::process::{Command, Output};
use std::path::Path;
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
