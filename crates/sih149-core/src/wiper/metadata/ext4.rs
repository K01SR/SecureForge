//! ext2/ext3/ext4 metadata scrubber.
//!
//! Targets:
//! - Inode fields (timestamps, size, block pointers, extended attributes)
//! - Directory entries (dirent records in parent directory blocks)
//! - Journal references (ext3/ext4 journal transaction records)
