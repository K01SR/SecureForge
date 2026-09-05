use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeConfig {
    pub device_path: String,
    pub method: String,
    pub verify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeProgress {
    pub sector_current: u64,
    pub sector_total: u64,
    pub percent: f32,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeResult {
    pub success: bool,
    pub sectors_wiped: u64,
    pub bad_sectors: u64,
    pub duration_secs: u64,
    pub method_used: String,
    pub verified: bool,
}

lazy_static::lazy_static! {
    static ref WIPE_CANCEL_FLAG: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

#[tauri::command]
pub async fn start_wipe(config: WipeConfig, app_handle: AppHandle) -> Result<WipeResult, String> {
    WIPE_CANCEL_FLAG.store(false, Ordering::SeqCst);
    
    let total_sectors = 100_000;
    let mut current_sector = 0;
    
    while current_sector < total_sectors {
        if WIPE_CANCEL_FLAG.load(Ordering::SeqCst) {
            return Err("Wipe cancelled".to_string());
        }
        
        current_sector += 1000;
        let percent = (current_sector as f32 / total_sectors as f32) * 100.0;
        
        let progress = WipeProgress {
            sector_current: current_sector,
            sector_total: total_sectors,
            percent,
            speed_mbps: 150.0,
            eta_seconds: 60,
            phase: "Wiping".to_string(),
        };
        
        app_handle.emit("wipe-progress", progress).map_err(|e| e.to_string())?;
        sleep(Duration::from_millis(50)).await;
    }
    
    Ok(WipeResult {
        success: true,
        sectors_wiped: total_sectors,
        bad_sectors: 0,
        duration_secs: 5,
        method_used: config.method,
        verified: config.verify,
    })
}

#[tauri::command]
pub fn cancel_wipe() -> Result<(), String> {
    WIPE_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn estimate_wipe_time(_device_path: String, _method: String) -> Result<u64, String> {
    // Dummy implementation for estimation
    Ok(3600)
}
