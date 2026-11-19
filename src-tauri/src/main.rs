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
use commands::shredder::*;
use commands::firmware::*;

fn main() -> anyhow::Result<()> {
    tracing::info!("SecureForge Desktop v{}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--server") {
        let mut port: u16 = 7878;
        let mut host = "127.0.0.1".to_string();
        let mut api_token: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            if args[i] == "--port" && i + 1 < args.len() {
                if let Ok(p) = args[i + 1].parse() {
                    port = p;
                }
                i += 2;
                continue;
            } else if args[i].starts_with("--port=") {
                if let Some(val) = args[i].split('=').nth(1) {
                    if let Ok(p) = val.parse() {
                        port = p;
                    }
                }
            } else if args[i] == "--host" && i + 1 < args.len() {
                host = args[i + 1].clone();
                i += 2;
                continue;
            } else if args[i].starts_with("--host=") {
                if let Some(val) = args[i].split('=').nth(1) {
                    host = val.to_string();
                }
            } else if args[i] == "--api-token" && i + 1 < args.len() {
                api_token = Some(args[i + 1].clone());
                i += 2;
                continue;
            } else if args[i].starts_with("--api-token=") {
                if let Some(val) = args[i].split('=').nth(1) {
                    api_token = Some(val.to_string());
                }
            }
            i += 1;
        }

        tokio::runtime::Runtime::new()?.block_on(crate::server::start_server(host, port, api_token))?;
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
            shred_files,
            detect_firmware_capabilities,
            start_firmware_erase,
            setup_expert_passphrase,
            verify_expert_passphrase,
            is_expert_configured,
            list_cases,
            export_report,
            get_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
