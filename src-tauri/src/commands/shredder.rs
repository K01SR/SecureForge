use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::path::Path;
use sih149_core::wiper::file_wiper::FileWiper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredConfig {
    /// List of absolute file/directory paths to shred
    pub paths: Vec<String>,
    /// Number of overwrite passes (recommend 3 for DoD-style)
    pub passes: u32,
    /// Number of random renames per file (scrubs directory-entry history)
    pub renames: u32,
    /// Attempt slack space scrubbing (will error if unsupported on this FS)
    pub scrub_slack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredFileResult {
    pub path: String,
    pub bytes_wiped: u64,
    pub passes_completed: u32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredProgress {
    pub current_file: String,
    pub files_done: u32,
    pub files_total: u32,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredResult {
    pub total_files: u32,
    pub total_bytes: u64,
    pub failed_files: u32,
    pub results: Vec<ShredFileResult>,
}

#[tauri::command]
pub async fn shred_files(config: ShredConfig, app_handle: AppHandle) -> Result<ShredResult, String> {
    let config_bg = config.clone();
    let app_handle_bg = app_handle.clone();

    tokio::task::spawn_blocking(move || -> Result<ShredResult, String> {
        let wiper = FileWiper::new(config_bg.passes, config_bg.renames, config_bg.scrub_slack);

        // Flatten directory entries into individual file paths so we can report
        // accurate per-file progress to the frontend.
        let mut all_targets: Vec<String> = Vec::new();
        for path_str in &config_bg.paths {
            let p = Path::new(path_str);
            if p.is_dir() {
                collect_files(p, &mut all_targets);
            } else {
                all_targets.push(path_str.clone());
            }
        }

        let total = all_targets.len() as u32;
        let mut results = Vec::new();
        let mut total_bytes: u64 = 0;
        let mut failed: u32 = 0;

        for (idx, path_str) in config_bg.paths.iter().enumerate() {
            let path = Path::new(path_str);

            let _ = app_handle_bg.emit("shred-progress", ShredProgress {
                current_file: path_str.clone(),
                files_done: idx as u32,
                files_total: total,
                percent: (idx as f32 / total.max(1) as f32) * 100.0,
            });

            if path.is_dir() {
                match wiper.wipe_directory(path) {
                    Ok(file_results) => {
                        for fr in file_results {
                            total_bytes += fr.bytes_wiped;
                            results.push(ShredFileResult {
                                path: fr.path,
                                bytes_wiped: fr.bytes_wiped,
                                passes_completed: fr.passes_completed,
                                success: fr.success,
                                error: None,
                            });
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        results.push(ShredFileResult {
                            path: path_str.clone(),
                            bytes_wiped: 0,
                            passes_completed: 0,
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            } else {
                match wiper.wipe_file(path) {
                    Ok(fr) => {
                        total_bytes += fr.bytes_wiped;
                        results.push(ShredFileResult {
                            path: fr.path,
                            bytes_wiped: fr.bytes_wiped,
                            passes_completed: fr.passes_completed,
                            success: fr.success,
                            error: None,
                        });
                    }
                    Err(e) => {
                        failed += 1;
                        results.push(ShredFileResult {
                            path: path_str.clone(),
                            bytes_wiped: 0,
                            passes_completed: 0,
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        let _ = app_handle_bg.emit("shred-progress", ShredProgress {
            current_file: String::new(),
            files_done: total,
            files_total: total,
            percent: 100.0,
        });

        Ok(ShredResult {
            total_files: total,
            total_bytes,
            failed_files: failed,
            results,
        })
    })
    .await
    .map_err(|e| format!("Shred task panicked: {}", e))?
}

/// Recursively collect file paths under a directory for progress counting.
fn collect_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_files(&p, out);
            } else {
                out.push(p.to_string_lossy().into_owned());
            }
        }
    }
}
