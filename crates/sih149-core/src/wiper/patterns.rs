//! Overwrite pattern generators.
//!
//! Implements standard sanitization patterns:
//! - Zero fill (0x00)
//! - One fill (0xFF)
//! - Random fill (CSPRNG via `getrandom`)
//! - DoD 5220.22-M (3-pass and 7-pass variants)
//! - Gutmann 35-pass method
//! - NIST 800-88 Clear (single random pass + verify)
