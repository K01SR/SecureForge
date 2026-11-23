use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::DiskSource;
use sih149_core::wiper::patterns::get_dod_pattern;
use sih149_core::wiper::verify::verify_wipe;
use std::io::{Seek, SeekFrom, Write};
use std::time::Instant;

use crate::commands::auth::verify_expert_passphrase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeConfig {
    pub device_path: String,
    pub method: String,
    pub verify: bool,
    pub expert_passphrase: Option<String>,
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

    let path = std::path::Path::new(&config.device_path);
    if sih149_core::wiper::file_wiper::is_protected_drive(path) {
        let authorized = match &config.expert_passphrase {
            Some(pass) => verify_expert_passphrase(pass.clone()).await.unwrap_or(false),
            None => false,
        };
        if !authorized {
            return Err(format!(
                "Safety guard: Refusing to wipe system/boot drive {} — expert passphrase required and did not verify.",
                config.device_path
            ));
        }
    }

    let app_handle_bg = app_handle.clone();
    let config_bg = config.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<WipeResult, String> {
        let mut disk = BlockDevice::open(&config_bg.device_path)
            .map_err(|e| format!("Failed to open {}: {}", config_bg.device_path, e))?;
        let size = disk.size().map_err(|e| e.to_string())?;
        let chunk_size: u64 = 1024 * 1024;

        let passes: Vec<u8> = match config_bg.method.as_str() {
            "zero" => vec![1],
            "dod3" | "dod" => vec![1, 2, 3],
            _ => vec![3],
        };

        let started = Instant::now();
        let total_bytes = size * passes.len() as u64;
        let mut bytes_done_all_passes: u64 = 0;

        for (pass_idx, pass) in passes.iter().enumerate() {
            let pattern_fn = get_dod_pattern(*pass);
            disk.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            let mut written: u64 = 0;

            while written < size {
                if WIPE_CANCEL_FLAG.load(Ordering::SeqCst) {
                    return Err("Wipe cancelled".to_string());
                }

                let this_chunk = std::cmp::min(chunk_size, size - written) as usize;
                let buf = pattern_fn(this_chunk);
                disk.write_all(&buf).map_err(|e| e.to_string())?;
                written += this_chunk as u64;
                bytes_done_all_passes += this_chunk as u64;

                let elapsed = started.elapsed().as_secs_f32().max(0.001);
                let speed_mbps = (bytes_done_all_passes as f32 / elapsed) / (1024.0 * 1024.0);
                let percent = (bytes_done_all_passes as f32 / total_bytes as f32) * 100.0;
                let remaining_bytes = total_bytes.saturating_sub(bytes_done_all_passes);
                let eta_seconds = if speed_mbps > 0.0 {
                    (remaining_bytes as f32 / (speed_mbps * 1024.0 * 1024.0)) as u64
                } else {
                    0
                };

                let progress = WipeProgress {
                    sector_current: bytes_done_all_passes / 512,
                    sector_total: total_bytes / 512,
                    percent,
                    speed_mbps,
                    eta_seconds,
                    phase: format!("Wiping (pass {}/{})", pass_idx + 1, passes.len()),
                };
                let _ = app_handle_bg.emit("wipe-progress", progress);
            }
            disk.flush().map_err(|e| e.to_string())?;
        }

        let mut verified = false;
        if config_bg.verify {
            let last_pass = *passes.last().unwrap();
            let is_random_pass = last_pass == 3;
            let pattern_fn = get_dod_pattern(last_pass);
            verified = verify_wipe(&mut disk, pattern_fn, 10, is_random_pass)
                .map_err(|e| e.to_string())?;
            if !verified {
                return Err("Verification failed: residual data pattern detected".to_string());
            }
        }

        Ok(WipeResult {
            success: true,
            sectors_wiped: size / 512,
            bad_sectors: 0,
            duration_secs: started.elapsed().as_secs(),
            method_used: config_bg.method,
            verified,
        })
    })
    .await
    .map_err(|e| format!("Wipe task panicked: {}", e))??;

    Ok(result)
}

#[tauri::command]
pub fn cancel_wipe() -> Result<(), String> {
    WIPE_CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn estimate_wipe_time(_device_path: String, _method: String) -> Result<u64, String> {
    Ok(3600)
}
