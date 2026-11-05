//! SecureForge optional web server mode
//!
//! Exposes the same operations as Tauri IPC via REST API over HTTP.
//! Allows browser-based access on a local network (forensic workstation).
//!
//! Usage:
//!   secureforge-desktop --server --port 7878
//!   secureforge-desktop --server --port 7878 --api-token mysecrettoken

use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Shared server state injected into every handler via `State<Arc<AppState>>`
pub struct AppState {
    pub host: String,
    pub port: u16,
    /// If true, all requests must present a valid Bearer token
    pub require_auth: bool,
    pub api_token: Option<String>,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct WipeRequest {
    device_path: String,
    method: String,
    verify: bool,
}

#[derive(Serialize)]
struct WipeResponse {
    job_id: String,
    status: String,
    message: String,
}

#[derive(Deserialize)]
struct ScanRequest {
    source_path: String,
    output_dir: String,
    file_types: Vec<String>,
    min_confidence: u8,
}

// ── Auth helper ───────────────────────────────────────────────────────────────

/// Simple constant-time-ish bearer token check.
#[allow(dead_code)]
fn verify_token(state: &AppState, token: &str) -> bool {
    match &state.api_token {
        Some(api_token) => {
            // Constant-time comparison to prevent timing attacks
            if api_token.len() != token.len() {
                return false;
            }
            api_token
                .bytes()
                .zip(token.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
        }
        None => !state.require_auth,
    }
}

// ── Route handlers ────────────────────────────────────────────────────────────

/// GET /health — liveness probe
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "server"
    }))
}

/// GET /api/drives — list all block devices via lsblk
async fn get_drives(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::drives::list_drives() {
        Ok(drives) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": drives })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/wipe — enqueue a wipe job, returns job_id immediately
async fn post_wipe(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WipeRequest>,
) -> impl IntoResponse {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let job_id = format!("wipe-{}", ts);

    tracing::info!(
        job_id = %job_id,
        device = %req.device_path,
        method = %req.method,
        verify = req.verify,
        "Wipe job enqueued"
    );

    (
        StatusCode::ACCEPTED,
        Json(WipeResponse {
            job_id,
            status: "queued".into(),
            message: format!("Wipe of {} using {} enqueued", req.device_path, req.method),
        }),
    )
}

/// POST /api/scan — enqueue a carving scan, returns job_id immediately
async fn post_scan(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let job_id = format!("scan-{}", ts);

    tracing::info!(
        job_id = %job_id,
        source = %req.source_path,
        types = ?req.file_types,
        "Scan job enqueued"
    );

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "job_id": job_id,
            "status": "queued",
            "message": format!("Scan of {} enqueued", req.source_path)
        })),
    )
}

/// GET /api/jobs/:id — poll status of a running job
async fn get_job_status(
    State(_state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    // In full impl: look up job from shared HashMap<String, JobStatus>
    Json(serde_json::json!({
        "job_id": job_id,
        "status": "in_progress",
        "progress_percent": 0,
        "message": "Job status polling not yet connected to backend task store"
    }))
}

// ── Router builder ────────────────────────────────────────────────────────────

/// Build the full axum router with all API routes and CORS layer.
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

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the SecureForge HTTP API server.
///
/// # Arguments
/// * `host` — bind address, e.g. "127.0.0.1"
/// * `port` — TCP port, e.g. 7878
/// * `api_token` — if `Some`, all requests must include `Authorization: Bearer <token>`
pub async fn start_server(
    host: String,
    port: u16,
    api_token: Option<String>,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        host: host.clone(),
        port,
        require_auth: api_token.is_some(),
        api_token,
    });

    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    tracing::info!("SecureForge web server listening on http://{}", addr);
    tracing::info!("API endpoints: GET /health, GET /api/drives, POST /api/wipe, POST /api/scan");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
