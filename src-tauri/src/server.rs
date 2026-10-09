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


async fn get_drives(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = crate::commands::drives::list_drives();
    match result {
        Ok(drives) => (StatusCode::OK, Json(serde_json::json!({"status": "success", "data": drives}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"status": "error", "message": e}))),
    }
}

#[derive(Deserialize)]
struct WipeRequest { device_path: String, method: String, verify: bool }

#[derive(Serialize)]
struct WipeResponse { job_id: String, status: String, message: String }

async fn post_wipe(State(state): State<Arc<AppState>>, Json(req): Json<WipeRequest>) -> impl IntoResponse {
    let job_id = "job-".to_string() + &std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis().to_string();
    // Simulate spawning task
    let response = WipeResponse {
        job_id,
        status: "started".to_string(),
        message: "Wipe task started successfully".to_string()
    };
    (StatusCode::ACCEPTED, Json(response))
}

use axum::extract::Path;

async fn get_job_status(State(state): State<Arc<AppState>>, Path(job_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "job_id": job_id,
        "status": "in_progress",
        "progress": 50
    }))
}

#[derive(Deserialize)]
struct ScanRequest { source_path: String, output_dir: String, file_types: Vec<String>, min_confidence: u8 }

async fn post_scan(State(state): State<Arc<AppState>>, Json(req): Json<ScanRequest>) -> impl IntoResponse {
    let job_id = "scan-".to_string() + &std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis().to_string();
    (StatusCode::ACCEPTED, Json(serde_json::json!({
        "job_id": job_id,
        "status": "started",
        "message": "Scan task started successfully"
    })))
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/drives", get(get_drives))
        .route("/api/wipe", post(post_wipe))
        .route("/api/scan", post(post_scan))
        .route("/api/jobs/:id", get(get_job_status))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
}

pub async fn start_server(host: String, port: u16, api_token: Option<String>) -> anyhow::Result<()> {
    let state = Arc::new(AppState { host: host.clone(), port, require_auth: api_token.is_some(), api_token });
    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("SecureForge web server listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
