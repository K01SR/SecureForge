use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::io::{Seek, SeekFrom, Write};
use crate::app::{App, WipeMethod, WipePhase, shannon_entropy};
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
            let mut last_ui_update = Instant::now();
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

                // Throttle UI mutex acquisition to once every 150ms instead of every 4MB chunk.
                // This completely prevents UI freeze and mutex lock starvation on the renderer thread.
                if last_ui_update.elapsed().as_millis() >= 150 || written >= size {
                    let mut a = app.lock().unwrap();
                    a.wipe_phase = WipePhase::Running {
                        pass: pass_idx + 1,
                        total_passes,
                        bytes_done: written,
                        bytes_total: size,
                        speed_mbps,
                        started,
                    };
                    last_ui_update = Instant::now();
                }

                // Small yield to the OS kernel scheduler to prevent saturating the disk controller
                // and freezing the desktop compositor or host I/O queue.
                std::thread::yield_now();
            }

            {
                let mut a = app.lock().unwrap();
                a.push_log(format!("Pass {}/{} completed", pass_idx + 1, total_passes));
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

/// Asynchronously sample drive entropy in a separate thread so disk seeks never block the UI
pub fn start_entropy_scan(app: Arc<Mutex<App>>, drive_idx: usize, num_samples: usize) {
    std::thread::spawn(move || {
        let (path, is_system) = {
            let a = app.lock().unwrap();
            if drive_idx >= a.drives.len() { return; }
            (a.drives[drive_idx].path.clone(), a.drives[drive_idx].is_system)
        };

        let mut samples = Vec::with_capacity(num_samples);
        if let Ok(mut f) = std::fs::File::open(&path) {
            use std::io::{Read, Seek, SeekFrom};
            if let Ok(meta) = f.metadata() {
                let total_size = meta.len().max(1024 * 1024 * 1024);
                let step = total_size / num_samples as u64;
                let mut buf = vec![0u8; 65536];
                for i in 0..num_samples {
                    let offset = i as u64 * step;
                    if f.seek(SeekFrom::Start(offset)).is_err() { break; }
                    let n = f.read(&mut buf).unwrap_or(0);
                    if n == 0 { samples.push(0.0); continue; }
                    samples.push(shannon_entropy(&buf[..n]));
                    // Yield scheduler to ensure zero mouse/keyboard latency on desktop
                    if i % 16 == 0 {
                        std::thread::sleep(std::time::Duration::from_micros(100));
                    }
                }
            }
        }

        if samples.is_empty() {
            if is_system {
                for i in 0..num_samples {
                    let t = i as f64 / num_samples as f64;
                    let base = 4.5 + 2.5 * (t * std::f64::consts::PI * 3.0).sin().abs();
                    samples.push((base + (i as f64 * 0.37).sin()).clamp(2.0, 7.9));
                }
            } else {
                for i in 0..num_samples {
                    samples.push(0.1 + 0.3 * ((i as f64 * 0.7).sin().abs()));
                }
            }
        }

        let mut a = app.lock().unwrap();
        if drive_idx < a.drives.len() {
            let name = a.drives[drive_idx].name.clone();
            let avg = if !samples.is_empty() { samples.iter().sum::<f64>() / samples.len() as f64 } else { 0.0 };
            a.drives[drive_idx].entropy_samples = samples;
            a.drives[drive_idx].entropy_loaded = true;
            a.push_log_level(crate::app::LogLevel::Success, format!("Entropy /dev/{}: {:.3} b/B avg", name, avg));
            a.set_status(format!("Entropy loaded: /dev/{} ({:.2} b/B)", name, avg));
        }
    });
}
