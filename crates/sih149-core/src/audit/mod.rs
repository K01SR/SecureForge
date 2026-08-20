//! Tamper-evident audit trail and report signing.
//!
//! Every operation generates a structured JSON audit entry.
//! Entries are chained via SHA-256 hashes (each entry includes
//! the hash of the previous entry), forming a tamper-evident log.
//!
//! Optional features:
//! - RFC 3161 trusted timestamping via external TSA
//! - PKI digital signatures (Ed25519/RSA) in Expert Mode

pub mod hashchain;
pub mod schema;
pub mod signing;
