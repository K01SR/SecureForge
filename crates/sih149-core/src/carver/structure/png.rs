//! PNG structure validator.
//!
//! Validates PNG chunk sequence:
//! - Signature: 89 50 4E 47 0D 0A 1A 0A
//! - IHDR chunk (must be first)
//! - IDAT chunks (image data)
//! - IEND chunk (must be last): 49 45 4E 44 AE 42 60 82
//!
//! Each chunk has a 4-byte length, 4-byte type, data, and CRC32.
