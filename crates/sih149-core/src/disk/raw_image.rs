use crate::disk::DiskSource;
use crate::error::Result;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// A raw image (.dd / .img) reader/writer.
pub struct RawImage {
    file: File,
    size: u64,
}

impl RawImage {
    /// Open a raw image file at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;

        let size = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;

        Ok(Self { file, size })
    }
}

impl Read for RawImage {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for RawImage {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for RawImage {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl DiskSource for RawImage {
    fn size(&self) -> Result<u64> {
        Ok(self.size)
    }

    fn sector_size(&self) -> Result<u32> {
        Ok(512) // Default sector size
    }
}
