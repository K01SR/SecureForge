//! SecureForge Tauri desktop application backend.
//!
//! Registers IPC command handlers that bridge the React frontend
//! to the sih149-core engine. Also provides an optional HTTP server
//! mode (via axum) for browser-based access.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod server;

use commands::drives::*;
use commands::wiper::*;
use commands::carver::*;
use commands::auth::*;
use commands::reports::*;

fn main() -> anyhow::Result<()> {
    tracing::info!("SecureForge Desktop v{}", env!("CARGO_PKG_VERSION"));
    
    if std::env::args().any(|a| a == "--server") {
        let port: u16 = std::env::args()
            .find(|a| a.starts_with("--port="))
            .and_then(|a| a.split('=').nth(1).map(|s| s.parse().ok()).flatten())
            .unwrap_or(7878);
        let host = "127.0.0.1".to_string();
        tokio::runtime::Runtime::new()?.block_on(crate::server::start_server(host, port, None))?;
        return Ok(());
    }
    
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
