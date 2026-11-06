use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // output_dir and min_confidence consumed by future scan engine
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

#[tauri::command]
pub async fn start_scan(_config: ScanConfig, app_handle: AppHandle) -> Result<ScanResult, String> {
    SCAN_CANCEL_FLAG.store(false, Ordering::SeqCst);
    
    let total_sectors = 100_000;
    let mut current_sector = 0;
    
    while current_sector < total_sectors {
        if SCAN_CANCEL_FLAG.load(Ordering::SeqCst) {
            return Err("Scan cancelled".to_string());
        }
        
        current_sector += 2000;
        let percent = (current_sector as f32 / total_sectors as f32) * 100.0;
        
        let progress = ScanProgress {
            sector_current: current_sector,
            sector_total: total_sectors,
            percent,
            files_found: 5,
            speed_mbps: 200.0,
        };
        
        app_handle.emit("scan-progress", progress).map_err(|e| e.to_string())?;
        sleep(Duration::from_millis(50)).await;
    }
    
    Ok(ScanResult {
        total_files: 5,
        total_size_bytes: 1024 * 1024,
        duration_secs: 10,
        entropy_heatmap: vec![0.1, 0.5, 0.9],
        files: vec![],
    })
}

#[tauri::command]
pub fn cancel_scan() -> Result<(), String> {
    SCAN_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn get_file_hex_preview(_file_path: String, _offset: u64, _length: usize) -> Result<String, String> {
    Ok("00 01 02 03".to_string())
}
