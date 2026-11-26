use sha2::{Sha256, Digest};
use std::path::Path;
use crate::error::CoreError;
use std::fs;

/// Compare two ASCII hex digest strings in constant time (no early exit on
/// the first mismatching byte). This prevents a remote/IPC timing oracle
/// from recovering a digest byte-by-byte via `verify_file_hash`.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let len = a.len().max(b.len());
    let mut diff: u8 = 0;
    let mut i = 0;
    while i < len {
        let ai = a.get(i).copied().unwrap_or(0);
        let bi = b.get(i).copied().unwrap_or(0);
        diff |= ai ^ bi;
        i += 1;
    }
    diff == 0
}

/// Compute SHA-256 of a file
pub fn hash_file(path: &Path) -> Result<String, CoreError> {
    let bytes = fs::read(path).map_err(CoreError::Io)?;
    Ok(hash_bytes(&bytes))
}

/// Compute SHA-256 of arbitrary bytes  
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify a file's hash matches expected (constant-time comparison)
pub fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool, CoreError> {
    let hash = hash_file(path)?;
    Ok(constant_time_eq(&hash, expected_hash))
}

/// Simple report integrity record saved alongside PDF
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportManifest {
    pub report_path: String,
    pub report_hash: String,
    pub generated_at: String,
    pub case_id: String,
    pub chain_tip: String,
    pub hash_verified: bool,
}

impl ReportManifest {
    pub fn new(report_path: &Path, hash: String, case_id: String, chain_tip: String) -> Self {
        Self {
            report_path: report_path.to_string_lossy().into_owned(),
            report_hash: hash,
            generated_at: chrono::Utc::now().to_rfc3339(),
            case_id,
            chain_tip,
            hash_verified: true,
        }
    }
    
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::Parse(e.to_string()))?;
        fs::write(path, json).map_err(CoreError::Io)
    }
    
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        let json = fs::read_to_string(path).map_err(CoreError::Io)?;
        serde_json::from_str(&json).map_err(|e| CoreError::Parse(e.to_string()))
    }
}
