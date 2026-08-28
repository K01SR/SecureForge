use crate::carver::signatures::{FileSignature, SignatureDatabase};
use crate::error::Result;
use rayon::prelude::*;
use std::sync::Arc;

/// A hit found during scanning.
#[derive(Debug)]
pub struct ScanHit {
    /// Offset in bytes where the signature was found
    pub offset: u64,
    /// The signature that matched
    pub signature: FileSignature,
}

/// Scans a buffer for file signatures using multiple threads.
pub struct SectorScanner {
    signatures: Arc<Vec<(Vec<u8>, FileSignature)>>,
}

impl SectorScanner {
    /// Create a new scanner with the given signature database.
    pub fn new(db: SignatureDatabase) -> Result<Self> {
        let mut sigs = Vec::new();
        for sig in db.signatures {
            let header_bytes = SignatureDatabase::parse_hex(&sig.magic_header)?;
            sigs.push((header_bytes, sig));
        }
        Ok(Self {
            signatures: Arc::new(sigs),
        })
    }

    /// Scan a buffer (representing sectors) for signatures.
    /// `base_offset` is the absolute offset of this buffer on the disk.
    pub fn scan_buffer(&self, buffer: &[u8], base_offset: u64, sector_size: usize) -> Vec<ScanHit> {
        if buffer.is_empty() {
            return Vec::new();
        }

        // We only scan at sector boundaries.
        let num_sectors = buffer.len() / sector_size;
        
        (0..num_sectors)
            .into_par_iter()
            .filter_map(|sector_idx| {
                let offset = sector_idx * sector_size;
                if offset >= buffer.len() {
                    return None;
                }
                
                let sector_data = &buffer[offset..];
                
                for (header_bytes, sig) in self.signatures.iter() {
                    if sector_data.len() >= header_bytes.len() {
                        if &sector_data[..header_bytes.len()] == header_bytes.as_slice() {
                            return Some(ScanHit {
                                offset: base_offset + offset as u64,
                                signature: sig.clone(),
                            });
                        }
                    }
                }
                None
            })
            .collect()
    }
}
