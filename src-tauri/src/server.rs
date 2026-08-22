//! Optional HTTP/HTTPS server mode.
//!
//! When launched with `sih149 --mode server`, this module starts
//! an axum web server that serves the React frontend as static files
//! and exposes the same IPC commands as REST API endpoints.
//!
//! Endpoints mirror Tauri commands:
//! - GET  /api/drives
//! - POST /api/wipe
//! - POST /api/recover
//! - GET  /api/reports
//! - POST /api/auth/expert
