//! SecureForge optional web server mode
//! Exposes the same operations as Tauri IPC via REST API over HTTPS
//! Allows browser-based access on local network (e.g. forensic workstation)

use axum::{
    Router,
    routing::{get, post},
    Json, extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::commands::drives::DriveInfo;
use serde::{Serialize, Deserialize};

pub struct AppState {
    pub host: String,
    pub port: u16,
    pub require_auth: bool,
    pub api_token: Option<String>,
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION"), "mode": "server"}))
}

async fn verify_token(state: &AppState, token: &str) -> bool {
    if let Some(ref api_token) = state.api_token {
        // Simple constant time-ish comparison for demonstration
        api_token == token
    } else {
        !state.require_auth
    }
}

use crate::commands::drives::DriveInfo;

async fn get_drives(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = crate::commands::drives::list_drives();
    match result {
        Ok(drives) => (StatusCode::OK, Json(serde_json::json!({"status": "success", "data": drives}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": e}))),
    }
}
