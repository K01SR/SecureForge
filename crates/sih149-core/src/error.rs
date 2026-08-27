use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecureForgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),
    #[error("Disk error: {0}")]
    Disk(String),
    #[error("Wiper error: {0}")]
    Wiper(String),
    #[error("Carver error: {0}")]
    Carver(String),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, SecureForgeError>;
