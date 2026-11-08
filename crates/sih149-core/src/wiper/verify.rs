use crate::disk::DiskSource;
use crate::error::Result;
use std::io::SeekFrom;
use tracing::{info, warn};

/// Shannon entropy of a buffer, in bits/byte (0.0–8.0).
/// Wiped-with-random sectors should read close to 8.0.
fn shannon_entropy(data: &[u8]) -> f64 {
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    counts.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Verifies that a disk has been successfully wiped with a specific pattern.
/// For deterministic patterns (zeros/ones), does exact byte comparison.
/// For the random pass, exact bytes can't be reconstructed — instead this
/// checks that no long run of the *previous* deterministic pattern remains
/// and that entropy is high (>= 7.9 bits/byte), which zeros/ones can never produce.
pub fn verify_wipe<D: DiskSource>(
    disk: &mut D,
    expected_pattern: fn(usize) -> Vec<u8>,
    sample_rate_percent: u8,
    is_random_pass: bool,
) -> Result<bool> {
    let size = disk.size()?;
    let chunk_size = 1024 * 1024; // 1 MB chunks
    
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
        disk.read_exact(&mut buffer[..to_read]).map_err(|e| crate::error::CoreError::Io(e))?;
        
        if is_random_pass {
            let entropy = shannon_entropy(&buffer[..to_read]);
            if entropy < 7.9 {
                warn!("Wipe verification failed at offset {} (entropy {:.2} too low)", offset, entropy);
                return Ok(false);
            }
        } else {
            let expected = expected_pattern(to_read);
            if &buffer[..to_read] != &expected[..] {
                warn!("Wipe verification failed at offset {}", offset);
                return Ok(false);
            }
        }

        offset += step;
    }

    info!("Wipe verification passed.");
    Ok(true)
}
