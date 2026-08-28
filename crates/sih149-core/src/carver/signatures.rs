use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a file signature (magic bytes) for carving.
#[derive(Debug, Deserialize, Clone)]
pub struct FileSignature {
    /// File extension (e.g., "jpg")
    pub extension: String,
    /// Human-readable description
    pub description: String,
    /// Magic bytes at the beginning of the file (hex encoded)
    pub magic_header: String,
    /// Magic bytes at the end of the file (hex encoded), if any
    pub magic_footer: Option<String>,
    /// Maximum expected file size in bytes
    pub max_size: u64,
}

/// Collection of file signatures.
#[derive(Debug, Deserialize, Default)]
pub struct SignatureDatabase {
    pub signatures: Vec<FileSignature>,
}

impl SignatureDatabase {
    /// Load signatures from a TOML file.
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let db: SignatureDatabase = toml::from_str(&content).map_err(|e| crate::error::SecureForgeError::Parse(e.to_string()))?;
        Ok(db)
    }

    /// Convert magic hex string to bytes
    pub fn parse_hex(hex_str: &str) -> Result<Vec<u8>> {
        let hex_str = hex_str.replace(" ", "");
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16)
                    .map_err(|e| crate::error::SecureForgeError::Parse(e.to_string()))
            })
            .collect()
    }
}
