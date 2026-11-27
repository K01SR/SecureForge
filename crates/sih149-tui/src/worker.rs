use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::io::{Seek, SeekFrom, Write};
use crate::app::{App, WipeMethod, WipePhase};
use sih149_core::disk::block_device::BlockDevice;
use sih149_core::disk::DiskSource;
use sih149_core::wiper::patterns::get_dod_pattern;
use sih149_core::wiper::verify::verify_wipe;

pub fn start_wipe(app: Arc<Mutex<App>>, device_path: String, method: WipeMethod, do_verify: bool) {
    std::thread::spawn(move || {
        let size = {
            match BlockDevice::open(&device_path) {
                Ok(disk) => match disk.size() {
                    Ok(s) => s,
                    Err(e) => {
                        let mut a = app.lock().unwrap();
                        a.wipe_phase = WipePhase::Error(e.to_string());
                        a.push_log(format!("Error: {}", e));
                        return;
                    }
                },
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Cannot open {}: {}", device_path, e));
                    a.push_log(format!("Error: Cannot open {}: {}", device_path, e));
                    return;
                }
            }
        };

        let passes: Vec<u8> = match method {
            WipeMethod::Zero => vec![0],
            WipeMethod::Random => vec![3],
            WipeMethod::Dod3 => vec![1, 2, 3],
            WipeMethod::Dod7 => vec![1, 2, 3, 1, 2, 3, 3],
            WipeMethod::Nist => vec![0],
            WipeMethod::Gutmann => (1u8..=3).cycle().take(35).collect(),
        };

        let total_passes = passes.len();
        let chunk_size: usize = 4 * 1024 * 1024; // 4MB chunks
        let started = Instant::now();

        for (pass_idx, &pass_type) in passes.iter().enumerate() {
            let pattern_fn = get_dod_pattern(pass_type);

            let mut disk = match BlockDevice::open(&device_path) {
                Ok(d) => d,
                Err(e) => {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error(format!("Open error: {}", e));
                    return;
                }
            };

            if disk.seek(SeekFrom::Start(0)).is_err() {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Error("Seek error".to_string());
                return;
            }

            let mut written: u64 = 0;
            let mut last_speed_update = Instant::now();
            let mut bytes_since_update: u64 = 0;
            let mut speed_mbps: f64 = 0.0;

            while written < size {
                let this_chunk = std::cmp::min(chunk_size as u64, size - written) as usize;
                let buf = pattern_fn(this_chunk);
                if disk.write_all(&buf).is_err() {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Error("Write error".to_string());
                    return;
                }
                written += this_chunk as u64;
                bytes_since_update += this_chunk as u64;

                if last_speed_update.elapsed().as_millis() >= 250 {
                    let elapsed_s = last_speed_update.elapsed().as_secs_f64();
                    speed_mbps = (bytes_since_update as f64) / (elapsed_s * 1_048_576.0);
                    bytes_since_update = 0;
                    last_speed_update = Instant::now();
                }

                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Running {
                    pass: pass_idx + 1,
                    total_passes,
                    bytes_done: written,
                    bytes_total: size,
                    speed_mbps,
                    started,
                };
                a.push_log(format!("Pass {}/{}: {:.1}%", pass_idx + 1, total_passes, written as f64 / size as f64 * 100.0));
            }

            // Flush
            if let Ok(mut d2) = BlockDevice::open(&device_path) { let _ = d2.flush(); }
        }

        if do_verify {
            {
                let mut a = app.lock().unwrap();
                a.wipe_phase = WipePhase::Verifying;
            }
            let last_pass = *passes.last().unwrap();
            let is_random = last_pass == 3;
            let pattern_fn = get_dod_pattern(last_pass);
            let ok = BlockDevice::open(&device_path)
                .ok()
                .and_then(|mut d| verify_wipe(&mut d, pattern_fn, 10, is_random).ok())
                .unwrap_or(false);

            let elapsed = started.elapsed();
            let mut a = app.lock().unwrap();
            a.wipe_phase = WipePhase::Done { success: ok, elapsed };
            a.push_log(if ok { "Wipe verified OK".to_string() } else { "Verification FAILED".to_string() });
        } else {
            let elapsed = started.elapsed();
            let mut a = app.lock().unwrap();
            a.wipe_phase = WipePhase::Done { success: true, elapsed };
            a.push_log("Wipe complete".to_string());
        }
    });
}
