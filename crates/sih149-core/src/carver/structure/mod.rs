//! File structure validators for carving accuracy.
//!
//! After signature-based carving extracts a candidate file,
//! these validators parse the internal structure to confirm
//! the file is valid and determine its true boundaries.

pub mod jpeg;
pub mod pdf;
pub mod png;
pub mod sqlite;
pub mod zip;
