pub mod metrics;
pub mod telemetry;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::extract::{FromRequestParts, Multipart, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use rust_embed::Embed;
use serde::Deserialize;
use tokio::sync::Mutex;

use typeweaver_bench::run_report;
use typeweaver_core::{BenchmarkProfile, REGISTRY_DIR_NAME};
use typeweaver_registry::{
    find_asset, ingest_dir, load_registry_at, registry_to_json, save_registry_at,
};

use crate::metrics::Metrics;

#[derive(Embed)]
#[folder = "static/"]
struct StaticAssets;

struct AppState {
    registry_root: PathBuf,
    metrics: Metrics,
    api_token: Option<String>,
    started_at: Instant,
}

// ---------------------------------------------------------------------------
// Bearer token auth extractor
// ---------------------------------------------------------------------------

struct ValidToken;

impl FromRequestParts<Arc<Mutex<AppState>>> for ValidToken {
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<Mutex<AppState>>,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = state.clone();
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        async move {
            let st = state.lock().await;
            let expected = st.api_token.clone();
            drop(st);

            match expected {
                None => Ok(ValidToken),
                Some(token) => {
                    let provided = auth_header
                        .as_deref()
                        .and_then(|h| h.strip_prefix("Bearer "));
                    if provided == Some(&token) {
                        Ok(ValidToken)
                    } else {
                        Err(error_response(StatusCode::UNAUTHORIZED, "unauthorized"))
                    }
                }
            }
        }
    }
}

/// Build the application router.
fn app(registry_root: PathBuf) -> Router {
    let api_token = std::env::var("TYPEWEAVER_API_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    let state = Arc::new(Mutex::new(AppState {
        registry_root,
        metrics: Metrics::new(),
        api_token,
        started_at: Instant::now(),
    }));

    Router::new()
        .route("/api/fonts/ingest", post(handle_ingest))
        .route("/api/fonts", get(handle_list))
        .route("/api/fonts/{id}", get(handle_get_font))
        .route("/api/fonts/{id}/file", get(handle_font_file))
        .route("/api/fonts/{id}/report", get(handle_report))
        .route("/healthz", get(handle_healthz))
        .route("/api/health", get(handle_healthz))
        .route("/okz", get(handle_okz))
        .route("/varz", get(handle_varz))
        .fallback(handle_static)
        .with_state(state)
}

/// Start the HTTP server.
pub async fn serve(
    registry_root: PathBuf,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = app(registry_root);
    let addr = format!("{host}:{port}");
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn registry_dir(root: &Path) -> PathBuf {
    root.join(REGISTRY_DIR_NAME)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn handle_ingest(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    mut multipart: Multipart,
) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let reg_root = st.registry_root.clone();
    let fonts_counter = st.metrics.fonts_ingested_total.clone();
    let registry_gauge = st.metrics.registry_size.clone();
    let upload_bytes = st.metrics.upload_bytes_total.clone();
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["ingest"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let upload_dir = reg_root.join("uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create upload dir: {e}"),
        );
    }

    let mut saved_count: u64 = 0;
    let mut total_bytes: u64 = 0;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("upload.ttf").to_string();
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                active_gauge.dec();
                duration_hist.observe(timer.elapsed().as_secs_f64());
                return error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("failed to read field: {e}"),
                );
            }
        };
        total_bytes += data.len() as u64;
        let dest = upload_dir.join(&file_name);
        if let Err(e) = std::fs::write(&dest, &data) {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to write {}: {e}", file_name),
            );
        }
        saved_count += 1;
    }

    upload_bytes.inc_by(total_bytes);

    if saved_count == 0 {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(StatusCode::BAD_REQUEST, "no files uploaded");
    }

    let registry = match ingest_dir(&upload_dir) {
        Ok(r) => r,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ingest failed: {e}"),
            );
        }
    };

    let reg_dir = registry_dir(&reg_root);
    if let Err(e) = save_registry_at(&reg_dir, &registry) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("save failed: {e}"),
        );
    }

    fonts_counter.inc_by(saved_count);
    registry_gauge.set(registry.assets.len() as i64);

    let body = format!(
        "{{\"ingested\": {}, \"total\": {}}}",
        saved_count,
        registry.assets.len()
    );
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    json_response(StatusCode::OK, &body)
}

async fn handle_list(State(state): State<Arc<Mutex<AppState>>>, _auth: ValidToken) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["list"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(_) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return json_response(StatusCode::OK, "{\"assets\": []}");
        }
    };

    let json = registry_to_json(&registry);
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    json_response(StatusCode::OK, &json)
}

async fn handle_get_font(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["get_font"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::NOT_FOUND, &format!("registry not found: {e}"));
        }
    };

    let resp = match find_asset(&registry, &id) {
        Ok(asset) => {
            let body = format!(
                "{{\"id\": \"{}\", \"file_name\": \"{}\", \"family_name\": {}, \"status\": \"{}\"}}",
                asset.id,
                asset.file_name,
                asset
                    .family_name
                    .as_ref()
                    .map_or("null".to_string(), |n| format!("\"{}\"", n)),
                asset.status.as_str()
            );
            json_response(StatusCode::OK, &body)
        }
        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("{e}")),
    };
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    resp
}

async fn handle_font_file(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["font_file"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::NOT_FOUND, &format!("registry not found: {e}"));
        }
    };

    let asset = match find_asset(&registry, &id) {
        Ok(asset) => asset,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::NOT_FOUND, &format!("{e}"));
        }
    };

    let bytes = match std::fs::read(&asset.path) {
        Ok(bytes) => bytes,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::NOT_FOUND,
                &format!("font file not readable: {e}"),
            );
        }
    };

    let mime = mime_guess::from_path(&asset.file_name)
        .first_or_octet_stream()
        .to_string();
    let mut resp = Response::new(axum::body::Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    if let Ok(val) = HeaderValue::from_str(&mime) {
        resp.headers_mut().insert("content-type", val);
    }
    resp.headers_mut().insert(
        "cache-control",
        HeaderValue::from_static("public, max-age=3600"),
    );
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    resp
}

#[derive(Deserialize)]
struct ReportQuery {
    profile: Option<String>,
}

async fn handle_report(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(query): Query<ReportQuery>,
) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    let bench_counter = st.metrics.bench_runs_total.clone();
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["report"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let profile = match &query.profile {
        Some(slug) => match BenchmarkProfile::from_slug(slug) {
            Ok(p) => p,
            Err(e) => {
                active_gauge.dec();
                duration_hist.observe(timer.elapsed().as_secs_f64());
                return error_response(StatusCode::BAD_REQUEST, &format!("{e}"));
            }
        },
        None => BenchmarkProfile::WebLightDefault,
    };

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::NOT_FOUND, &format!("registry not found: {e}"));
        }
    };

    let asset = match find_asset(&registry, &id) {
        Ok(a) => a.clone(),
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::NOT_FOUND, &format!("{e}"));
        }
    };

    let report = run_report(&asset, profile);
    bench_counter.inc();

    let json = report.to_json_pretty();
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    json_response(StatusCode::OK, &json)
}

/// Shallow liveness probe — no I/O, always 200. Used by load balancers.
async fn handle_okz() -> Response {
    json_response(StatusCode::OK, "{\"ok\":true}")
}

/// Deep health check — verifies registry is readable, reports uptime + font count.
async fn handle_healthz(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let st = state.lock().await;
    let uptime_seconds = st.started_at.elapsed().as_secs();
    let reg_dir = registry_dir(&st.registry_root);
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["healthz"])
        .inc();
    drop(st);

    let version = env!("CARGO_PKG_VERSION");

    let font_count = load_registry_at(&reg_dir)
        .map(|r| r.assets.len())
        .unwrap_or(0);

    let disk_free_bytes = get_disk_free_bytes();
    let memory_rss_bytes = get_memory_rss_bytes();

    let body = format!(
        "{{\"status\":\"ok\",\"uptime_seconds\":{},\"version\":\"{}\",\"font_count\":{},\"disk_free_bytes\":{},\"memory_rss_bytes\":{}}}",
        uptime_seconds, version, font_count, disk_free_bytes, memory_rss_bytes
    );
    json_response(StatusCode::OK, &body)
}

async fn handle_varz(State(state): State<Arc<Mutex<AppState>>>, _auth: ValidToken) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = st.metrics.registry.gather();
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["varz"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let resp = match encoder.encode_to_string(&metric_families) {
        Ok(body) => {
            let mut resp = Response::new(axum::body::Body::from(body));
            *resp.status_mut() = StatusCode::OK;
            resp.headers_mut().insert(
                "content-type",
                HeaderValue::from_static("text/plain; version=0.0.4"),
            );
            resp
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("metrics encode error: {e}"),
        ),
    };
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    resp
}

async fn handle_static(
    State(state): State<Arc<Mutex<AppState>>>,
    uri: axum::http::Uri,
) -> Response {
    {
        let st = state.lock().await;
        st.metrics.requests_total.inc();
        st.metrics
            .api_calls_total
            .with_label_values(&["static"])
            .inc();
    }

    let path = match resolve_static_path(uri.path()) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "not found"),
    };

    match StaticAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            let mut resp = Response::new(axum::body::Body::from(content.data.to_vec()));
            *resp.status_mut() = StatusCode::OK;
            if let Ok(val) = HeaderValue::from_str(&mime) {
                resp.headers_mut().insert("content-type", val);
            }
            resp
        }
        None => error_response(StatusCode::NOT_FOUND, "not found"),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolve_static_path(request_path: &str) -> Option<String> {
    let trimmed = request_path.trim_matches('/');
    let path = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        trimmed.to_string()
    };

    if StaticAssets::get(&path).is_some() {
        return Some(path);
    }

    if !path.contains('.') {
        let nested = format!("{path}/index.html");
        if StaticAssets::get(&nested).is_some() {
            return Some(nested);
        }
    }

    None
}

fn json_response(status: StatusCode, body: &str) -> Response {
    let mut resp = Response::new(axum::body::Body::from(body.to_string()));
    *resp.status_mut() = status;
    resp.headers_mut()
        .insert("content-type", HeaderValue::from_static("application/json"));
    resp
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let body = format!("{{\"error\": \"{}\"}}", message.replace('"', "\\\""));
    json_response(status, &body)
}

fn get_disk_free_bytes() -> u64 {
    std::process::Command::new("df")
        .args(["--output=avail", "-k", "/opt/typeweaver"])
        .output()
        .ok()
        .and_then(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .nth(1)
                .and_then(|line| line.trim().split_whitespace().next())
                .and_then(|val| val.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
        .unwrap_or(0)
}

fn get_memory_rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find(|line| line.starts_with("VmRSS:"))
                .and_then(|line| {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|val| val.parse::<u64>().ok())
                        .map(|kb| kb * 1024)
                })
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::resolve_static_path;

    #[test]
    fn resolves_root_and_nested_static_routes() {
        assert_eq!(resolve_static_path("/"), Some("index.html".to_string()));
        assert_eq!(
            resolve_static_path("/tool"),
            Some("tool/index.html".to_string())
        );
        assert_eq!(
            resolve_static_path("/tool/"),
            Some("tool/index.html".to_string())
        );
    }

    #[test]
    fn leaves_asset_paths_intact() {
        assert_eq!(
            resolve_static_path("/tool.css"),
            Some("tool.css".to_string())
        );
    }

    #[test]
    fn rejects_missing_static_routes() {
        assert_eq!(resolve_static_path("/missing"), None);
    }
}
