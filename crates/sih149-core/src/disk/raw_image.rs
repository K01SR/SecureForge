//! Raw disk image file reader.
//!
//! Supports `.dd`, `.raw`, and `.img` files — flat byte-for-byte
//! copies of a block device. Implements the [`DiskSource`] trait
//! with memory-mapped or buffered sequential access.
