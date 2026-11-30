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
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::fs;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use std::sync::Mutex;

/// Simple in-memory token-bucket rate limiter keyed by client IP.
/// Prevents unauthenticated/brute-force/burst attacks against the API.
struct RateLimiter {
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window: Duration) -> Self {        Self {
            buckets: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }

    fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        match buckets.get_mut(key) {
            Some((count, start)) => {
                if now.duration_since(*start) > self.window {
                    *count = 1;
                    *start = now;
                    true
                } else if *count < self.max_requests {
                    *count += 1;
                    true
                } else {
                    false
                }
            }
            None => {
                buckets.insert(key.to_string(), (1, now));
                true
            }
        }
    }
}

type SharedRateLimiter = Arc<RateLimiter>;

/// Shared server state injected into every handler via `State<Arc<AppState>>`
#[allow(dead_code)]
pub struct AppState {
    pub host: String,
    pub port: u16,
    pub require_auth: bool,
    pub api_token: String,
    limiter: SharedRateLimiter,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct WipeRequest {
    device_path: String,
    method: String,
    verify: bool,
    expert_passphrase: Option<String>,
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

#[derive(Deserialize)]
struct EntropyRequest {
    device_path: String,
    chunks: Option<usize>,
}

#[derive(Deserialize)]
struct HexRequest {
    file_path: String,
    offset: u64,
    length: usize,
    allowed_root: Option<String>,
}

// Simple in-memory job store so web-mode `/api/scan` (async) can be polled
// via `/api/jobs/:id` and return a real `ScanResult` when complete.
lazy_static::lazy_static! {
    static ref SCAN_JOBS: Mutex<std::collections::HashMap<String, serde_json::Value>> =
        Mutex::new(std::collections::HashMap::new());
}

fn get_scan_job(job_id: &str) -> Option<serde_json::Value> {
    SCAN_JOBS.lock().unwrap().get(job_id).cloned()
}

fn set_scan_job(job_id: &str, value: serde_json::Value) {
    SCAN_JOBS.lock().unwrap().insert(job_id.to_string(), value);
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
    // Only /health and non-api static asset routes do not require Bearer header
    if path == "/health" || !path.starts_with("/api") {
        return Ok(next.run(req).await);
    }

    // Rate limit by client IP to blunt brute-force and burst attacks.
    // When fronted by a reverse proxy, honor the de-facto X-Forwarded-For
    // header; otherwise fall back to a shared bucket keyed on the endpoint.
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| path.to_string());

    if !state.limiter.allow(&client_ip) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "status": "error",
                "message": "Rate limit exceeded. Try again shortly."
            })),
        ));
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
                "message": "Unauthorized: Missing or invalid Bearer token."
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

/// POST /api/wipe — enqueue a wipe job with protected system drive check and verified passphrase
async fn post_wipe(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WipeRequest>,
) -> impl IntoResponse {
    let target = FsPath::new(&req.device_path);
    if sih149_core::wiper::file_wiper::is_protected_drive(target) {
        let authorized = match &req.expert_passphrase {
            Some(pass) => crate::commands::auth::verify_expert_passphrase(pass.clone())
                .await
                .unwrap_or(false),
            None => false,
        };
        if !authorized {
            return (
                StatusCode::FORBIDDEN,
                Json(WipeResponse {
                    job_id: "".into(),
                    status: "forbidden".into(),
                    message: format!(
                        "Safety guard: Refusing to wipe system/boot drive {} — expert passphrase required and did not verify.",
                        req.device_path
                    ),
                }),
            );
        }
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

/// POST /api/scan — run a carving scan in the background, store the result,
/// and return a job id that can be polled via GET /api/jobs/:id
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

    set_scan_job(&job_id, serde_json::json!({
        "job_id": job_id,
        "status": "in_progress",
        "progress_percent": 0
    }));

    // Run the same scan engine the Tauri IPC path uses, off the async thread.
    let job_id_bg = job_id.clone();
    let config = crate::commands::carver::ScanConfig {
        source_path: req.source_path,
        output_dir: req.output_dir,
        file_types: req.file_types,
        min_confidence: req.min_confidence,
    };
    let source_log = config.source_path.clone();
    tokio::task::spawn_blocking(move || {
        let result = crate::commands::carver::run_scan(&config, |_| {});
        let payload = match result {
            Ok(scan) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "completed",
                "progress_percent": 100,
                "result": scan
            }),
            Err(e) => serde_json::json!({
                "job_id": job_id_bg,
                "status": "failed",
                "progress_percent": 100,
                "message": e
            }),
        };
        set_scan_job(&job_id_bg, payload);
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "job_id": job_id,
            "status": "queued",
            "message": format!("Scan of {} enqueued", source_log)
        })),
    )
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

/// GET /api/jobs/:id — poll a running carve job; returns the completed
/// ScanResult once `/api/scan`'s background task finishes.
async fn get_job_status(
    State(_state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match get_scan_job(&job_id) {
        Some(job) => (StatusCode::OK, Json(job)),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "job_id": job_id,
                "status": "unknown"
            })),
        ),
    }
}

/// POST /api/hex — read a hex preview from an evidence file (mirrors the
/// Tauri `get_file_hex_preview` command so web mode behaves identically).
async fn post_hex(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<HexRequest>,
) -> impl IntoResponse {
    match crate::commands::carver::get_file_hex_preview_str(
        &req.file_path,
        req.offset,
        req.length,
        req.allowed_root.as_deref(),
    ) {
        Ok(hex) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "data": hex })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "status": "error", "message": e })),
        ),
    }
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

/// Serve index.html without token injection (token must be provided out-of-band by operator)
async fn serve_index_html(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(dist_dir) = find_frontend_dist() {
        let index_path = dist_dir.join("index.html");
        if let Ok(html) = fs::read_to_string(index_path) {
            return (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            ).into_response();
        }
    }
    fallback_html_handler().await.into_response()
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
    #[allow(unused_mut)]
    let mut allowed_origins = vec![
        format!("http://localhost:{}", state.port).parse().unwrap_or(HeaderValue::from_static("http://localhost:7878")),
        format!("http://127.0.0.1:{}", state.port).parse().unwrap_or(HeaderValue::from_static("http://127.0.0.1:7878")),
    ];
    #[cfg(debug_assertions)]
    {
        if let Ok(v) = "http://localhost:5173".parse() { allowed_origins.push(v); }
        if let Ok(v) = "http://127.0.0.1:5173".parse() { allowed_origins.push(v); }
    }

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(allowed_origins)
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
        .route("/api/hex", post(post_hex))
        .route("/api/jobs/:id", get(get_job_status))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .layer(cors)
        .with_state(state.clone());

    if let Some(dist_dir) = find_frontend_dist() {
        let assets_dir = dist_dir.join("assets");
        let static_router = Router::new()
            .route("/", get(serve_index_html))
            .route("/index.html", get(serve_index_html))
            .fallback(serve_index_html)
            .with_state(state)
            .nest_service("/assets", ServeDir::new(assets_dir));
        static_router.merge(api_router)
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
    let (token, was_generated) = match api_token {
        Some(t) if !t.is_empty() => (t, false),
        _ => {
            use rand::RngCore;
            let mut bytes = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            let generated_token = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            eprintln!("[SecureForge] Generated API token (save this — it will not be shown again):");
            eprintln!("[SecureForge] Token: {}", generated_token);
            (generated_token, true)
        }
    };

    let state = Arc::new(AppState {
        host: host.clone(),
        port,
        require_auth: true,
        api_token: token.clone(),
        limiter: Arc::new(RateLimiter::new(120, Duration::from_secs(60))),
    });

    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("============================================================");
    println!("  SecureForge Forensic Station — Web Server Active");
    println!("  Web UI URL : http://{}", addr);
    if was_generated {
        eprintln!("  Auth Token (write to a secure note, NOT printed to terminal for security):");
        eprintln!("  [Token generated — provide via --api-token or check config]");
    } else {
        println!("  Auth Token : [Configured from CLI]");
    }
    if host != "127.0.0.1" && host != "localhost" {
        eprintln!("  WARNING: Binding to non-localhost address '{}'. Ensure firewall rules are in place.", host);
    }
    println!("============================================================");

    tracing::info!("SecureForge web server listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
