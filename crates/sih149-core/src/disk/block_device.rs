use crate::disk::DiskSource;
use crate::error::{Result, SecureForgeError};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// A Linux block device reader/writer.
pub struct BlockDevice {
    file: File,
    size: u64,
    sector_size: u32,
}

impl BlockDevice {
    /// Open a block device at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        // For simplicity, we just use file metadata. 
        // In reality, block devices need ioctl to get size.
        let metadata = file.metadata()?;
        let size = metadata.len();
        
        Ok(Self {
            file,
            size,
            sector_size: 512,
        })
    }
}

impl Read for BlockDevice {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for BlockDevice {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for BlockDevice {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl DiskSource for BlockDevice {
    fn size(&self) -> Result<u64> {
        Ok(self.size)
    }

    fn sector_size(&self) -> Result<u32> {
        Ok(self.sector_size)
    }
}
