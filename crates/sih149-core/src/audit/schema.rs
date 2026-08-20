//! Serde-serializable audit data structures.
//!
//! Defines the JSON schema for:
//! - Erasure audit records (drive info, method, verification, hashes)
//! - Recovery audit records (source, carved files, confidence scores)
//! - Hash chain entries (entry hash, previous hash, timestamp)
//! - Certificate of destruction fields (NIST 800-88 Rev. 2 compliant)
