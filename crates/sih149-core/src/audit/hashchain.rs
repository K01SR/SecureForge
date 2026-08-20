//! SHA-256 append-only hash chain.
//!
//! Each audit entry's hash is computed as:
//!   SHA-256(entry_json + previous_entry_hash)
//!
//! This creates a chain where modifying any historical entry
//! breaks all subsequent hashes, providing tamper evidence.
