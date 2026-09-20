use serde::Deserialize;
use std::path::Path;
use std::fs;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
struct SignatureFile {
    signature: Vec<TomlSignatureDef>,
}

#[derive(Debug, Deserialize)]
struct TomlSignatureDef {
    name: String, 
    category: String,
    header: String, 
    footer: Option<String>,
    max_size: String,  // "50MB", "2GB" etc
    extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct TomlSignature {
    pub name: String, 
    pub category: String,
    pub header_bytes: Vec<u8>, 
    pub footer_bytes: Option<Vec<u8>>,
    pub max_size_bytes: u64, 
    pub extensions: Vec<String>,
}

pub fn load_signatures_from_file(path: &Path) -> Result<Vec<TomlSignature>, CoreError> {
    let content = fs::read_to_string(path).map_err(CoreError::Io)?;
    let sig_file: SignatureFile = toml::from_str(&content)
        .map_err(|e| CoreError::Parse(e.to_string()))?;
        
    let mut sigs = Vec::new();
    for def in sig_file.signature {
        let max_size_bytes = parse_size_string(&def.max_size)?;
        let header_bytes = decode_escape_sequence(&def.header);
        let footer_bytes = def.footer.map(|f| decode_escape_sequence(&f));
        
        sigs.push(TomlSignature {
            name: def.name,
            category: def.category,
            header_bytes,
            footer_bytes,
            max_size_bytes,
            extensions: def.extensions.unwrap_or_default(),
        });
    }
    
    Ok(sigs)
}

pub fn load_signatures_from_dir(dir: &Path) -> Result<Vec<TomlSignature>, CoreError> {
    let mut all_sigs = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(CoreError::Io)? {
            let entry = entry.map_err(CoreError::Io)?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(mut sigs) = load_signatures_from_file(&path) {
                    all_sigs.append(&mut sigs);
                }
            }
        }
    }
    Ok(all_sigs)
}

pub fn parse_size_string(s: &str) -> Result<u64, CoreError> {
    let s = s.trim().to_uppercase();
    let mut num_str = String::new();
    let mut unit_str = String::new();
    
    for c in s.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if c.is_ascii_alphabetic() {
            unit_str.push(c);
        }
    }
    
    let value: u64 = num_str.parse().map_err(|_| CoreError::Parse(format!("Invalid size number: {}", s)))?;
    
    let multiplier = match unit_str.as_str() {
        "B" | "" => 1,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024 * 1024 * 1024 * 1024,
        _ => return Err(CoreError::Parse(format!("Invalid size unit: {}", unit_str))),
    };
    
    Ok(value * multiplier)
}

pub fn decode_escape_sequence(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&'x') = chars.peek() {
                chars.next(); // consume 'x'
                let mut hex = String::new();
                if let Some(h1) = chars.next() { hex.push(h1); }
                if let Some(h2) = chars.next() { hex.push(h2); }
                
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                }
            } else {
                bytes.push(b'\\');
            }
        } else {
            bytes.extend_from_slice(c.to_string().as_bytes());
        }
    }
    
    bytes
}
