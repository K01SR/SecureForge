use crate::disk::DiskSource;
use crate::error::Result;
use std::io::SeekFrom;
use tracing::{info, warn};

/// Verifies that a disk has been successfully wiped with a specific pattern.
pub fn verify_wipe<D: DiskSource>(
    disk: &mut D,
    expected_pattern: fn(usize) -> Vec<u8>,
    sample_rate_percent: u8,
) -> Result<bool> {
    let size = disk.size()?;
    let chunk_size = 1024 * 1024; // 1 MB chunks
    
    // In full verification, we check 100%. In sampled, we check e.g. 10%
    let step = if sample_rate_percent >= 100 {
        chunk_size as u64
    } else {
        let skip = chunk_size as u64 * (100 - sample_rate_percent as u64) / (sample_rate_percent as u64).max(1);
        chunk_size as u64 + skip
    };

    let mut buffer = vec![0u8; chunk_size];
    let mut offset = 0;
    
    info!("Starting wipe verification on {} bytes with step {}", size, step);

    while offset < size {
        disk.seek(SeekFrom::Start(offset))?;
        
        let to_read = std::cmp::min(chunk_size as u64, size - offset) as usize;
        let expected = expected_pattern(to_read);
        
        disk.read_exact(&mut buffer[..to_read]).map_err(|e| crate::error::CoreError::Io(e))?;
        
        if &buffer[..to_read] != &expected[..] {
            warn!("Wipe verification failed at offset {}", offset);
            return Ok(false);
        }

        offset += step;
    }

    info!("Wipe verification passed.");
    Ok(true)
}
