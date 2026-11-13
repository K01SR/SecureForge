use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::path::Path;
use sih149_core::wiper::file_wiper::{FileWiper, is_protected_path};

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
    // Validate ALL targets before starting any work so the UI gets a single
    // clean error rather than partial progress + a mid-run failure.
    for path_str in &config.paths {
        let path = Path::new(path_str);
        if is_protected_path(path) {
            return Err(format!(
                "Refusing to shred protected system path: {} — remove it from the target list",
                path_str
            ));
        }
    }

    let config_bg = config.clone();
    let app_handle_bg = app_handle.clone();

    tokio::task::spawn_blocking(move || -> Result<ShredResult, String> {
        let wiper = FileWiper::new(config_bg.passes, config_bg.renames, config_bg.scrub_slack);

        // Flatten every top-level entry into individual file paths.
        // This is the ground truth for progress reporting AND for the
        // actual wipe loop — previously the loop iterated config_bg.paths
        // (top-level entries only) while total was derived from all_targets
        // (flattened count), making progress jump from 0%→50%→100% for
        // multi-file directories regardless of how many files they contained.
        let mut all_targets: Vec<String> = Vec::new();
        let mut dir_targets: Vec<String> = Vec::new(); // dirs need removal after files wiped

        for path_str in &config_bg.paths {
            let p = Path::new(path_str);
            if p.is_dir() {
                collect_files(p, &mut all_targets);
                dir_targets.push(path_str.clone());
            } else {
                all_targets.push(path_str.clone());
            }
        }

        let total = all_targets.len() as u32;
        let mut results = Vec::new();
        let mut total_bytes: u64 = 0;
        let mut failed: u32 = 0;

        // Iterate the flattened file list — one progress tick per real file.
        for (idx, path_str) in all_targets.iter().enumerate() {
            let path = Path::new(path_str);

            let _ = app_handle_bg.emit("shred-progress", ShredProgress {
                current_file: path_str.clone(),
                files_done: idx as u32,
                files_total: total,
                percent: (idx as f32 / total.max(1) as f32) * 100.0,
            });

            // Use the public wipe_file which re-checks is_protected_path —
            // belt-and-suspenders in case a symlink was created between the
            // pre-flight check above and now.
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

        // Remove now-empty directories (deepest first via reverse sort so
        // parent dirs are removed after their children).
        let mut dirs_to_remove: Vec<String> = Vec::new();
        for dir_str in &dir_targets {
            collect_dirs(Path::new(dir_str), &mut dirs_to_remove);
            dirs_to_remove.push(dir_str.clone());
        }
        dirs_to_remove.sort_by(|a, b| b.len().cmp(&a.len())); // deepest first
        for dir_str in &dirs_to_remove {
            let _ = std::fs::remove_dir(dir_str); // best-effort, ignore errors
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

/// Recursively collect individual file paths under `dir`.
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

/// Recursively collect directory paths under `dir` (not including `dir` itself).
fn collect_dirs(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_dirs(&p, out);
                out.push(p.to_string_lossy().into_owned());
            }
        }
    }
}
