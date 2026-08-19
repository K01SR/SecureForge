//! Advanced forensic file carving and recovery engine.
//!
//! Recovers deleted files from formatted, corrupted, or raw media
//! using multiple carving strategies:
//! - **Signature-based**: Header/footer magic byte matching
//! - **Structure-based**: Internal file structure validation
//! - **Entropy-based**: Statistical block classification
//!
//! Operates in strict read-only mode to preserve evidential integrity.

pub mod confidence;
pub mod engine;
pub mod entropy;
pub mod scanner;
pub mod signatures;
pub mod structure;
