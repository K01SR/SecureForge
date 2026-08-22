//! SecureForge Tauri desktop application backend.
//!
//! Registers IPC command handlers that bridge the React frontend
//! to the sih149-core engine. Also provides an optional HTTP server
//! mode (via axum) for browser-based access.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // TODO: Initialize Tauri app with command handlers
    println!("SecureForge Desktop v{}", env!("CARGO_PKG_VERSION"));
}
