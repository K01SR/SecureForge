//! SecureForge optional web server mode
//!
//! Exposes the same operations as Tauri IPC via REST API over HTTP.
//! Allows browser-based access on a local network (forensic workstation).
//! Also serves the compiled React web interface (`src-ui/dist`).
//!
//! Usage:
//!   secureforge-desktop --server --port 7878
//!   secureforge-desktop --server --port 7878 --api-token mysecrettoken

use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::{Path, State},
    http::{StatusCode, HeaderValue},
    response::{IntoResponse, Html},
    middleware::{from_fn_with_state, Next},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{PathBuf, Path as FsPath};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

/// Shared server state injected into every handler via `State<Arc<AppState>>`
#[allow(dead_code)]
pub struct AppState {
    pub host: String,
    pub port: u16,
    pub require_auth: bool,
    pub api_token: String,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct WipeRequest {
    device_path: String,
    method: String,
    verify: bool,
    expert: Option<bool>,
}

#[derive(Serialize)]
struct WipeResponse {
    job_id: String,
    status: String,
    message: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScanRequest {
    source_path: String,
    output_dir: String,
    file_types: Vec<String>,
    min_confidence: u8,
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// Constant-time bearer token comparison to prevent timing side-channel attacks.
fn verify_token(state: &AppState, token: &str) -> bool {
    let expected = &state.api_token;
    if expected.is_empty() {
        return !state.require_auth;
    }
    if expected.len() != token.len() {
        return false;
    }
    expected
        .bytes()
        .zip(token.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// Authentication middleware for all `/api/*` routes (except `/health`).
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: Next,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let path = req.uri().path();
    // /health and static asset requests do not require authentication
    if path == "/health" || !path.starts_with("/api") {
        return Ok(next.run(req).await);
    }

    if !state.require_auth {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")))
        .map(str::trim);

    match token {
        Some(t) if verify_token(&state, t) => Ok(next.run(req).await),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": "error",
                "message": "Unauthorized: Missing or invalid Bearer token. Include 'Authorization: Bearer <TOKEN>' header."
            })),
        )),
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

/// POST /api/wipe — enqueue a wipe job with protected system drive check
async fn post_wipe(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WipeRequest>,
) -> impl IntoResponse {
    // Safety check: verify the target is not a protected system/boot drive
    let target = FsPath::new(&req.device_path);
    if sih149_core::wiper::file_wiper::is_protected_drive(target) && !req.expert.unwrap_or(false) {
        return (
            StatusCode::FORBIDDEN,
            Json(WipeResponse {
                job_id: "".into(),
                status: "forbidden".into(),
                message: format!(
                    "Safety guard: Refusing to wipe system/boot drive {} without expert authorization.",
                    req.device_path
                ),
            }),
        );
    }

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

/// POST /api/scan — enqueue a carving scan
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

#[derive(Deserialize)]
struct EntropyRequest {
    device_path: String,
    chunks: Option<usize>,
}

/// GET /api/cases — list all forensic cases from SQLite DB
async fn get_cases(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::reports::list_cases() {
        Ok(cases) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": cases })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/plugins — list all loaded TOML & Lua signature plugins
async fn get_plugins(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::commands::plugins::list_plugins() {
        Ok(plugins) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": plugins })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// POST /api/entropy — calculate real Shannon entropy over blocks
async fn post_entropy(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<EntropyRequest>,
) -> impl IntoResponse {
    match crate::commands::plugins::get_drive_entropy(req.device_path, req.chunks) {
        Ok(entropies) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": entropies })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
}

/// GET /api/jobs/:id — poll status of a running job
async fn get_job_status(
    State(_state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "job_id": job_id,
        "status": "in_progress",
        "progress_percent": 0,
        "message": "Job status polling not yet connected to backend task store"
    }))
}

/// Locate compiled static frontend directory
fn find_frontend_dist() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("src-ui/dist"),
        PathBuf::from("../src-ui/dist"),
        PathBuf::from("../../src-ui/dist"),
        PathBuf::from("/usr/local/share/secureforge/dist"),
        PathBuf::from("/usr/share/secureforge/dist"),
    ];
    for c in &candidates {
        if c.join("index.html").exists() {
            return Some(c.clone());
        }
    }
    None
}

/// Fallback HTML when frontend bundle has not yet been built
async fn fallback_html_handler() -> impl IntoResponse {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>SecureForge — Web Server Online</title>
    <style>
        body { background: #070c18; color: #f8fafc; font-family: system-ui, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .card { background: #0f172a; border: 1px solid #1e293b; padding: 2.5rem; border-radius: 1rem; max-width: 500px; text-align: center; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5); }
        h1 { color: #38bdf8; margin-top: 0; }
        code { background: #1e293b; color: #10b981; padding: 0.2rem 0.5rem; border-radius: 0.25rem; font-family: monospace; }
        p { color: #94a3b8; font-size: 0.9rem; line-height: 1.5; }
    </style>
</head>
<body>
    <div class="card">
        <h1>SecureForge Server Online</h1>
        <p>API is active on <code>/health</code> and <code>/api/*</code>.</p>
        <p>To serve the full desktop web UI, build the frontend with:</p>
        <p><code>npm --prefix src-ui run build</code></p>
    </div>
</body>
</html>"#)
}

// ── Router builder ────────────────────────────────────────────────────────────

/// Build the full axum router with all API routes, restricted CORS layer, and frontend file server.
pub fn build_router(state: Arc<AppState>) -> Router {
    // Restricted CORS layer (avoid wildcard / permissive on destructive API)
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin([
            format!("http://localhost:{}", state.port).parse().unwrap_or(HeaderValue::from_static("http://localhost:7878")),
            format!("http://127.0.0.1:{}", state.port).parse().unwrap_or(HeaderValue::from_static("http://127.0.0.1:7878")),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]);

    let api_router = Router::new()
        .route("/health", get(health))
        .route("/api/drives", get(get_drives))
        .route("/api/wipe", post(post_wipe))
        .route("/api/scan", post(post_scan))
        .route("/api/cases", get(get_cases))
        .route("/api/plugins", get(get_plugins))
        .route("/api/entropy", post(post_entropy))
        .route("/api/jobs/:id", get(get_job_status))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state.clone())
        .layer(cors);

    if let Some(dist_dir) = find_frontend_dist() {
        let index_html = dist_dir.join("index.html");
        let serve_dir = ServeDir::new(&dist_dir).not_found_service(ServeFile::new(index_html));
        api_router.fallback_service(serve_dir)
    } else {
        api_router.fallback(fallback_html_handler)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the SecureForge HTTP API & Web UI server.
pub async fn start_server(
    host: String,
    port: u16,
    api_token: Option<String>,
) -> anyhow::Result<()> {
    // Generate secure token if none supplied so server is always authenticated by default
    let (token, was_generated) = match api_token {
        Some(t) if !t.is_empty() => (t, false),
        _ => {
            use rand::RngCore;
            let mut bytes = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            let generated_token = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            (generated_token, true)
        }
    };

    let state = Arc::new(AppState {
        host: host.clone(),
        port,
        require_auth: true,
        api_token: token.clone(),
    });

    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("============================================================");
    println!("  SecureForge Forensic Station — Web Server Active");
    println!("  Web UI URL : http://{}", addr);
    if was_generated {
        println!("  Auth Token : {}", token);
        println!("  (Auto-generated: Include 'Authorization: Bearer {}' on API calls)", token);
    } else {
        println!("  Auth Token : [Configured from CLI]");
    }
    println!("============================================================");

    tracing::info!("SecureForge web server listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
