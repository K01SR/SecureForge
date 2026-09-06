//! SecureForge Tauri desktop application backend.
//!
//! Registers IPC command handlers that bridge the React frontend
//! to the sih149-core engine. Also provides an optional HTTP server
//! mode (via axum) for browser-based access.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::drives::*;
use commands::wiper::*;
use commands::carver::*;
use commands::auth::*;
use commands::reports::*;

fn main() {
    tracing::info!("SecureForge Desktop v{}", env!("CARGO_PKG_VERSION"));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_drives,
            get_drive_info,
            start_wipe,
            cancel_wipe,
            estimate_wipe_time,
            start_scan,
            cancel_scan,
            get_file_hex_preview,
            setup_expert_passphrase,
            verify_expert_passphrase,
            is_expert_configured,
            list_cases,
            export_report,
            get_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
