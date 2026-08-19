//! Filesystem-aware metadata scrubbing.
//!
//! After overwriting file content, residual metadata (filenames,
//! timestamps, allocation records) can still reveal information.
//! This module provides per-filesystem scrubbers.

pub mod ext4;
pub mod fat;
pub mod ntfs;
