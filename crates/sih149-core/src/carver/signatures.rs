//! File signature database and TOML loader.
//!
//! Loads file type signatures from:
//! 1. Built-in Rust constants (common types)
//! 2. TOML definition files (`plugins/signatures/*.toml`)
//! 3. Lua plugin scripts (`plugins/scripts/*.lua`)
//!
//! Each signature specifies: name, category, header bytes,
//! optional footer bytes, and maximum file size.
