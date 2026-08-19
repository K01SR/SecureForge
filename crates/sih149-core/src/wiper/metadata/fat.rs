//! FAT12/FAT16/FAT32/exFAT metadata scrubber.
//!
//! Targets:
//! - Directory entry (32-byte record: filename, timestamps, first cluster)
//! - Long Filename (LFN) directory entries
//! - FAT chain entries (cluster allocation table)
//! - exFAT allocation bitmap
