use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::raw_image::RawImage;
use sih149_core::carver::engine::CarvingEngine;
use sih149_core::carver::scanner::SectorScanner;
use sih149_core::carver::signatures::SignatureDatabase;
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub source_path: String,
    pub output_dir: String,
    pub file_types: Vec<String>,
    pub min_confidence: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarvedFile {
    pub id: String,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub confidence: u8,
    pub offset_bytes: u64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub sector_current: u64,
    pub sector_total: u64,
    pub percent: f32,
    pub files_found: u32,
    pub speed_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: u32,
    pub total_size_bytes: u64,
    pub duration_secs: u64,
    pub entropy_heatmap: Vec<f32>,
    pub files: Vec<CarvedFile>,
}

lazy_static::lazy_static! {
    static ref SCAN_CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

/// Loads signatures from TOML directory with built-in fallbacks
fn load_signature_db(signatures_dir: &Path) -> Result<SignatureDatabase, String> {
    let mut signatures = Vec::new();

    if signatures_dir.is_dir() {
        if let Ok(toml_sigs) = sih149_core::plugins::toml_loader::load_signatures_from_dir(signatures_dir) {
            for sig in toml_sigs {
                let magic_header = sig.header_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                let magic_footer = sig.footer_bytes.map(|fb| fb.iter().map(|b| format!("{:02x}", b)).collect::<String>());
                let extension = sig.extensions.into_iter().next().unwrap_or_else(|| sig.name.to_lowercase());
                signatures.push(sih149_core::carver::signatures::FileSignature {
                    extension,
                    description: sig.name,
                    magic_header,
                    magic_footer,
                    max_size: sig.max_size_bytes,
                });
            }
        }
    }

    if signatures.is_empty() {
        signatures = vec![
            sih149_core::carver::signatures::FileSignature {
                extension: "jpg".to_string(),
                description: "JPEG Image".to_string(),
                magic_header: "ffd8ff".to_string(),
                magic_footer: Some("ffd9".to_string()),
                max_size: 50 * 1024 * 1024,
            },
            sih149_core::carver::signatures::FileSignature {
                extension: "png".to_string(),
                description: "PNG Image".to_string(),
                magic_header: "89504e470d0a1a0a".to_string(),
                magic_footer: Some("49454e44ae426082".to_string()),
                max_size: 100 * 1024 * 1024,
            },
            sih149_core::carver::signatures::FileSignature {
                extension: "pdf".to_string(),
                description: "PDF Document".to_string(),
                magic_header: "25504446".to_string(),
                magic_footer: Some("2525454f46".to_string()),
                max_size: 100 * 1024 * 1024,
            },
            sih149_core::carver::signatures::FileSignature {
                extension: "zip".to_string(),
                description: "ZIP Archive".to_string(),
                magic_header: "504b0304".to_string(),
                magic_footer: None,
                max_size: 1024 * 1024 * 1024,
            },
            sih149_core::carver::signatures::FileSignature {
                extension: "sqlite".to_string(),
                description: "SQLite Database".to_string(),
                magic_header: "53514c69746520666f726d6174203300".to_string(),
                magic_footer: None,
                max_size: 500 * 1024 * 1024,
            },
        ];
    }

    Ok(SignatureDatabase { signatures })
}

#[tauri::command]
pub async fn start_scan(config: ScanConfig, app_handle: AppHandle) -> Result<ScanResult, String> {
    // IPC input validation: refuse empty paths, cap file-types count, and
    // keep min_confidence in a sane range to prevent excessive resource use.
    if config.source_path.trim().is_empty() {
        return Err("source_path must not be empty".to_string());
    }
    if config.output_dir.trim().is_empty() {
        return Err("output_dir must not be empty".to_string());
    }
    if config.file_types.len() > 32 {
        return Err("file_types list too large (max 32)".to_string());
    }
    if config.min_confidence > 100 {
        return Err("min_confidence must be between 0 and 100".to_string());
    }
    if config.source_path.len() > 4096 || config.output_dir.len() > 4096 {
        return Err("path arguments too long".to_string());
    }

    SCAN_CANCEL_FLAG.store(false, Ordering::SeqCst);

    let app_handle_bg = app_handle.clone();
    let config_bg = config.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<ScanResult, String> {
        let signatures_dir = Path::new("plugins/signatures");
        let sig_db = load_signature_db(signatures_dir)?;
        let scanner = SectorScanner::new(sig_db).map_err(|e| e.to_string())?;
        let engine = CarvingEngine::new(scanner);

        let started = Instant::now();

        let source_path = Path::new(&config_bg.source_path);
        let is_block_device = config_bg.source_path.starts_with("/dev");

        let _ = app_handle_bg.emit("scan-progress", ScanProgress {
            sector_current: 0,
            sector_total: 100,
            percent: 0.0,
            files_found: 0,
            speed_mbps: 0.0,
        });

        let carve_hits = if is_block_device {
            let mut disk = BlockDevice::open(source_path)
                .map_err(|e| format!("Failed to open {}: {}", config_bg.source_path, e))?;
            engine.carve(&mut disk).map_err(|e| e.to_string())?
        } else {
            let mut disk = RawImage::open(source_path)
                .map_err(|e| format!("Failed to open image {}: {}", config_bg.source_path, e))?;
            engine.carve(&mut disk).map_err(|e| e.to_string())?
        };

        if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
            return Err("Scan cancelled".to_string());
        }

        std::fs::create_dir_all(&config_bg.output_dir).map_err(|e| e.to_string())?;

        let mut files = Vec::new();
        let min_confidence = config_bg.min_confidence;
        for (offset, confidence, extension) in &carve_hits {
            let confidence_pct: u8 = match confidence {
                sih149_core::carver::confidence::Confidence::Low => 40,
                sih149_core::carver::confidence::Confidence::Medium => 65,
                sih149_core::carver::confidence::Confidence::High => 90,
                sih149_core::carver::confidence::Confidence::Absolute => 100,
            };
            if confidence_pct < min_confidence {
                continue;
            }
            if !config_bg.file_types.is_empty() && !config_bg.file_types.contains(extension) {
                continue;
            }
            files.push(CarvedFile {
                id: Uuid::new_v4().to_string(),
                filename: format!("carved_{:016x}.{}", offset, extension),
                file_type: extension.clone(),
                size_bytes: 0,
                confidence: confidence_pct,
                offset_bytes: *offset,
                category: extension.clone(),
            });
        }

        let elapsed = started.elapsed().as_secs_f32().max(0.001);
        let _ = app_handle_bg.emit("scan-progress", ScanProgress {
            sector_current: 100,
            sector_total: 100,
            percent: 100.0,
            files_found: files.len() as u32,
            speed_mbps: 100.0 / elapsed,
        });

        Ok(ScanResult {
            total_files: files.len() as u32,
            total_size_bytes: 0,
            duration_secs: started.elapsed().as_secs(),
            entropy_heatmap: vec![0.1, 0.4, 0.8],
            files,
        })
    })
    .await
    .map_err(|e| format!("Scan task panicked: {}", e))??;

    Ok(result)
}

#[tauri::command]
pub fn cancel_scan() -> Result<(), String> {
    SCAN_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn get_file_hex_preview(
    file_path: String,
    offset: u64,
    length: usize,
    allowed_root: Option<String>,
) -> Result<String, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;

    let path = Path::new(&file_path);
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve {}: {}", file_path, e))?;

    if let Some(ref root_str) = allowed_root {
        if !root_str.is_empty() {
            let root = Path::new(root_str)
                .canonicalize()
                .map_err(|e| format!("Invalid allowed_root: {}", e))?;
            if !resolved.starts_with(&root) {
                return Err(format!(
                    "Access denied: {} is outside the permitted case directory {}",
                    file_path, root_str
                ));
            }
        }
    }

    if sih149_core::wiper::file_wiper::is_protected_path(&resolved) {
        return Err(format!("Access denied: protected system path: {}", file_path));
    }

    let mut file = File::open(&resolved).map_err(|e| format!("Failed to open {}: {}", file_path, e))?;
    file.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
    let len = length.min(4096).max(16);
    let mut buf = vec![0u8; len];
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(n);
    let hex = buf.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
    Ok(hex)
}
