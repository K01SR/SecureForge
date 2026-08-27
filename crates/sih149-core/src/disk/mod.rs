pub mod block_device;
pub mod raw_image;

use crate::error::Result;
use std::io::{Read, Seek, Write};

/// Trait representing a disk source that can be read from, written to, and sought.
pub trait DiskSource: Read + Write + Seek + Send + Sync {
    /// Get the total size of the disk source in bytes.
    fn size(&self) -> Result<u64>;
    
    /// Get the sector size of the disk source in bytes.
    fn sector_size(&self) -> Result<u32>;
}
