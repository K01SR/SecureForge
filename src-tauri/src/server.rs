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
