use crate::carver::confidence::{Confidence, ConfidenceScorer};
use crate::carver::scanner::SectorScanner;
use crate::carver::structure::{jpeg, pdf, png, sqlite, zip};
use crate::disk::DiskSource;
use crate::error::Result;
use std::io::SeekFrom;
use tracing::{debug, info};

pub struct CarvingEngine {
    scanner: SectorScanner,
}

impl CarvingEngine {
    pub fn new(scanner: SectorScanner) -> Self {
        Self { scanner }
    }

    /// Run the carving engine on a disk source, returning a list of recovered files and their offsets.
    pub fn carve<D: DiskSource>(&self, disk: &mut D) -> Result<Vec<(u64, Confidence, String)>> {
        let size = disk.size()?;
        let sector_size = disk.sector_size()? as usize;
        
        let chunk_size = 1024 * 1024 * 16; // 16 MB chunks
        let mut buffer = vec![0u8; chunk_size];
        
        let mut offset = 0;
        let mut results = Vec::new();

        info!("Starting carving on disk of size {}", size);

        disk.seek(SeekFrom::Start(0))?;

        while offset < size {
            let to_read = std::cmp::min(chunk_size as u64, size - offset) as usize;
            disk.read_exact(&mut buffer[..to_read]).unwrap_or_default(); // In reality handle errors gracefully
            
            let hits = self.scanner.scan_buffer(&buffer[..to_read], offset, sector_size);
            
            for hit in hits {
                debug!("Found {} at offset {}", hit.signature.extension, hit.offset);
                
                let mut scorer = ConfidenceScorer::new();
                scorer.has_header = true;

                // Validate structure if supported
                if hit.offset >= offset && hit.offset < offset + to_read as u64 {
                    let local_offset = (hit.offset - offset) as usize;
                    let file_data = &buffer[local_offset..std::cmp::min(local_offset + hit.signature.max_size as usize, to_read)];
                    
                    let valid = match hit.signature.extension.as_str() {
                        "jpg" | "jpeg" => jpeg::validate_jpeg(file_data).unwrap_or(false),
                        "png" => png::validate_png(file_data).unwrap_or(false),
                        "zip" => zip::validate_zip(file_data).unwrap_or(false),
                        "pdf" => pdf::validate_pdf(file_data).unwrap_or(false),
                        "sqlite" | "db" => sqlite::validate_sqlite(file_data).unwrap_or(false),
                        _ => false,
                    };

                    scorer.structure_valid = valid;
                }

                results.push((hit.offset, scorer.calculate(), hit.signature.extension));
            }

            offset += to_read as u64;
        }

        info!("Carving completed. Found {} potential files.", results.len());
        Ok(results)
    }
}
