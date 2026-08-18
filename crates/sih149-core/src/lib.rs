//! # SecureForge Core Engine
//!
//! The core library powering all SecureForge operations:
//! - **disk**: Unified abstraction over block devices, raw images, and E01 files
//! - **wiper**: Secure data destruction (NIST 800-88 Clear/Purge methods)
//! - **carver**: Forensic file recovery via signature, structure, and entropy analysis
//! - **plugins**: User-extensible file signatures (TOML + Lua)
//! - **audit**: Tamper-evident hash chain and cryptographic report signing
//! - **classify**: Recovered file categorization and duplicate detection
//! - **db**: SQLite case management database

pub mod audit;
pub mod carver;
pub mod classify;
pub mod db;
pub mod disk;
pub mod plugins;
pub mod wiper;
