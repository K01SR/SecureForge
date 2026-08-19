//! Secure data erasure engine.
//!
//! Implements NIST SP 800-88 Rev. 2 sanitization methods:
//! - **Clear**: Logical overwrite (zero fill, random, DoD 5220.22-M)
//! - **Purge**: Firmware-level commands (NVMe Crypto/Block Erase, ATA Secure Erase)
//!
//! Includes post-wipe verification via entropy analysis and
//! bad sector flagging for NIST compliance.

pub mod file_wiper;
pub mod firmware;
pub mod metadata;
pub mod patterns;
pub mod verify;
