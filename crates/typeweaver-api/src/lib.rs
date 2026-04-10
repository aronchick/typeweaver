pub mod metrics;
pub mod public_fonts;
pub mod telemetry;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::{FromRequestParts, Multipart, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use serde::Deserialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use typeweaver_bench::run_report;
use typeweaver_core::{BenchmarkProfile, REGISTRY_DIR_NAME};
use typeweaver_registry::{
    find_asset, ingest_dir, load_registry_at, registry_to_json, save_registry_at,
};

use crate::metrics::Metrics;
use crate::public_fonts::{
    PublicFontCatalog, load_public_font_catalog, resolve_public_font, search_public_font_catalog,
};

#[derive(Clone)]
struct CachedPublicFontCatalog {
    fetched_at: Instant,
    catalog: PublicFontCatalog,
}

struct AppState {
    registry_root: PathBuf,
    metrics: Metrics,
    api_token: Option<String>,
    started_at: Instant,
    http_client: reqwest::Client,
    public_font_catalog: Option<CachedPublicFontCatalog>,
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
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent(format!("TypeWeaver/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("public font http client");
    let state = Arc::new(Mutex::new(AppState {
        registry_root,
        metrics: Metrics::new(),
        api_token,
        started_at: Instant::now(),
        http_client,
        public_font_catalog: None,
    }));

    Router::new()
        .route("/api/fonts/ingest", post(handle_ingest))
        .route("/api/fonts/ingest-url", post(handle_ingest_url))
        .route("/api/public-fonts", get(handle_public_font_search))
        .route("/api/public-fonts/ingest", post(handle_public_font_ingest))
        .route("/api/fonts", get(handle_list))
        .route("/api/fonts/{id}", get(handle_get_font))
        .route("/api/fonts/{id}/file", get(handle_font_file))
        .route("/api/fonts/{id}/report", get(handle_report))
        .route("/healthz", get(handle_healthz))
        .route("/api/health", get(handle_healthz))
        .route("/okz", get(handle_okz))
        .route("/varz", get(handle_varz))
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

#[derive(Deserialize)]
struct IngestUrlRequest {
    url: String,
    declared_license: Option<String>,
}

#[derive(Deserialize)]
struct PublicFontSearchQuery {
    q: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct IngestPublicFontRequest {
    family: String,
    declared_license: Option<String>,
}

async fn handle_ingest_url(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    axum::Json(payload): axum::Json<IngestUrlRequest>,
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
        .with_label_values(&["ingest_url"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let remote_url = match reqwest::Url::parse(payload.url.trim()) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => url,
        Ok(_) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::BAD_REQUEST, "font URL must use http or https");
        }
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::BAD_REQUEST, &format!("invalid font URL: {e}"));
        }
    };

    let response = match reqwest::get(remote_url.clone()).await {
        Ok(response) => response,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::BAD_GATEWAY,
                &format!("could not fetch remote font: {e}"),
            );
        }
    };

    if !response.status().is_success() {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::BAD_GATEWAY,
            &format!("remote font returned {}", response.status()),
        );
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::BAD_GATEWAY,
                &format!("could not read remote font: {e}"),
            );
        }
    };

    let file_name = match remote_file_name(&remote_url, content_type.as_deref()) {
        Ok(file_name) => file_name,
        Err(message) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::BAD_REQUEST, &message);
        }
    };

    let upload_dir = reg_root.join("uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create upload dir: {e}"),
        );
    }

    let font_path = upload_dir.join(&file_name);
    if let Err(e) = std::fs::write(&font_path, &bytes) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write {}: {e}", file_name),
        );
    }

    if let Some(license) = payload
        .declared_license
        .as_deref()
        .map(str::trim)
        .filter(|license| !license.is_empty() && *license != "unknown")
    {
        let sidecar_name = license_sidecar_name(&file_name);
        let sidecar_path = upload_dir.join(sidecar_name);
        if let Err(e) = std::fs::write(&sidecar_path, license) {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to write license file: {e}"),
            );
        }
    }

    upload_bytes.inc_by(bytes.len() as u64);

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

    let saved_count = registry
        .assets
        .iter()
        .filter(|asset| asset.file_name == file_name)
        .count() as u64;
    fonts_counter.inc_by(saved_count.max(1));
    registry_gauge.set(registry.assets.len() as i64);

    let body = format!(
        "{{\"file_name\": \"{}\", \"total\": {}}}",
        file_name,
        registry.assets.len()
    );
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    json_response(StatusCode::OK, &body)
}

async fn handle_public_font_search(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    Query(query): Query<PublicFontSearchQuery>,
) -> Response {
    let timer = Instant::now();
    let st = state.lock().await;
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["public_font_search"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let catalog = cached_public_font_catalog(&state).await;
    let payload =
        search_public_font_catalog(&catalog, query.q.as_deref().unwrap_or(""), query.limit);
    let response = json_serialized_response(StatusCode::OK, &payload);
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    response
}

async fn handle_public_font_ingest(
    State(state): State<Arc<Mutex<AppState>>>,
    _auth: ValidToken,
    axum::Json(payload): axum::Json<IngestPublicFontRequest>,
) -> Response {
    let timer = Instant::now();
    let trimmed_family = payload.family.trim().to_string();
    if trimmed_family.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "font family is required");
    }

    let st = state.lock().await;
    let reg_root = st.registry_root.clone();
    let client = st.http_client.clone();
    let fonts_counter = st.metrics.fonts_ingested_total.clone();
    let registry_gauge = st.metrics.registry_size.clone();
    let upload_bytes = st.metrics.upload_bytes_total.clone();
    st.metrics.requests_total.inc();
    st.metrics
        .api_calls_total
        .with_label_values(&["public_font_ingest"])
        .inc();
    st.metrics.active_requests.inc();
    let duration_hist = st.metrics.request_duration.clone();
    let active_gauge = st.metrics.active_requests.clone();
    drop(st);

    let resolved = match resolve_public_font(
        &client,
        &trimmed_family,
        payload.declared_license.as_deref(),
    )
    .await
    {
        Ok(resolved) => resolved,
        Err(error) => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(StatusCode::BAD_GATEWAY, &error);
        }
    };

    let upload_dir = reg_root.join("uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create upload dir: {e}"),
        );
    }

    let font_path = upload_dir.join(&resolved.file_name);
    if let Err(e) = std::fs::write(&font_path, &resolved.bytes) {
        active_gauge.dec();
        duration_hist.observe(timer.elapsed().as_secs_f64());
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write {}: {e}", resolved.file_name),
        );
    }

    if let Some(license) = resolved.declared_license.as_deref() {
        let sidecar_name = license_sidecar_name(&resolved.file_name);
        let sidecar_path = upload_dir.join(sidecar_name);
        if let Err(e) = std::fs::write(&sidecar_path, license) {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to write license file: {e}"),
            );
        }
    }

    upload_bytes.inc_by(resolved.bytes.len() as u64);

    let registry = match ingest_dir(&upload_dir) {
        Ok(registry) => registry,
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

    let asset = match registry
        .assets
        .iter()
        .find(|asset| asset.file_name == resolved.file_name)
    {
        Some(asset) => asset,
        None => {
            active_gauge.dec();
            duration_hist.observe(timer.elapsed().as_secs_f64());
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "public font saved, but the registry entry was not found",
            );
        }
    };

    fonts_counter.inc();
    registry_gauge.set(registry.assets.len() as i64);

    let body = serde_json::json!({
        "family": resolved.family,
        "file_name": asset.file_name,
        "asset_id": asset.id,
        "status": asset.status.as_str(),
        "license_normalized": asset.license_normalized.as_str(),
        "source": resolved.source,
    });
    let response = json_serialized_response(StatusCode::OK, &body);
    active_gauge.dec();
    duration_hist.observe(timer.elapsed().as_secs_f64());
    response
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn cached_public_font_catalog(state: &Arc<Mutex<AppState>>) -> PublicFontCatalog {
    const FRESH_TTL: Duration = Duration::from_secs(30 * 60);
    const DEGRADED_TTL: Duration = Duration::from_secs(5 * 60);

    let (client, cached) = {
        let st = state.lock().await;
        (st.http_client.clone(), st.public_font_catalog.clone())
    };

    if let Some(cached) = cached {
        let ttl = if cached.catalog.degraded {
            DEGRADED_TTL
        } else {
            FRESH_TTL
        };
        if cached.fetched_at.elapsed() < ttl {
            return cached.catalog;
        }
    }

    let catalog = load_public_font_catalog(&client).await;
    let mut st = state.lock().await;
    st.public_font_catalog = Some(CachedPublicFontCatalog {
        fetched_at: Instant::now(),
        catalog: catalog.clone(),
    });
    catalog
}

fn json_response(status: StatusCode, body: &str) -> Response {
    let mut resp = Response::new(axum::body::Body::from(body.to_string()));
    *resp.status_mut() = status;
    resp.headers_mut()
        .insert("content-type", HeaderValue::from_static("application/json"));
    resp
}

fn json_serialized_response<T: serde::Serialize>(status: StatusCode, body: &T) -> Response {
    match serde_json::to_string(body) {
        Ok(json) => json_response(status, &json),
        Err(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("json encode error: {error}"),
        ),
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let body = format!("{{\"error\": \"{}\"}}", message.replace('"', "\\\""));
    json_response(status, &body)
}

fn remote_file_name(url: &reqwest::Url, content_type: Option<&str>) -> Result<String, String> {
    let base_name = url
        .path_segments()
        .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
        .map(sanitize_file_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("remote-font-{}", Uuid::new_v4().simple()));

    let existing_extension = base_name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase());
    if let Some(extension) = existing_extension.as_deref()
        && is_supported_font_extension(extension)
    {
        return Ok(base_name);
    }

    let extension = infer_font_extension(content_type)
        .ok_or_else(|| "font URL must end in .ttf, .otf, .woff, or .woff2".to_string())?;
    let stem = base_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(base_name.as_str());
    Ok(format!("{stem}.{extension}"))
}

fn sanitize_file_name(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
            sanitized.push(ch);
        } else if matches!(ch, ' ' | '+' | '%') {
            sanitized.push('-');
        }
    }

    let trimmed = sanitized.trim_matches(|ch| matches!(ch, '.' | '-'));
    let collapsed = trimmed
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    collapsed.chars().take(96).collect()
}

fn license_sidecar_name(file_name: &str) -> String {
    match file_name.rsplit_once('.') {
        Some((stem, _)) => format!("{stem}.license"),
        None => format!("{file_name}.license"),
    }
}

fn infer_font_extension(content_type: Option<&str>) -> Option<String> {
    match content_type?.split(';').next()?.trim() {
        "font/ttf" | "application/x-font-ttf" | "application/font-sfnt" => Some("ttf".to_string()),
        "font/otf" | "application/x-font-otf" | "application/vnd.ms-opentype" => {
            Some("otf".to_string())
        }
        "font/woff" | "application/font-woff" => Some("woff".to_string()),
        "font/woff2" | "application/font-woff2" => Some("woff2".to_string()),
        _ => None,
    }
}

fn is_supported_font_extension(extension: &str) -> bool {
    matches!(extension, "ttf" | "otf" | "woff" | "woff2")
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
                .and_then(|line| line.split_whitespace().next())
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
    use super::{infer_font_extension, remote_file_name, sanitize_file_name};

    #[test]
    fn sanitizes_remote_file_names() {
        assert_eq!(
            sanitize_file_name("Acme Sans Regular (2026).woff2"),
            "Acme-Sans-Regular-2026.woff2"
        );
        assert_eq!(sanitize_file_name("../../weird?.ttf"), "weird.ttf");
    }

    #[test]
    fn infers_font_extension_from_content_type() {
        assert_eq!(
            infer_font_extension(Some("font/woff2")),
            Some("woff2".to_string())
        );
        assert_eq!(
            infer_font_extension(Some("application/vnd.ms-opentype; charset=binary")),
            Some("otf".to_string())
        );
        assert_eq!(infer_font_extension(Some("text/css")), None);
    }

    #[test]
    fn keeps_supported_remote_extension() {
        let url = reqwest::Url::parse("https://example.com/fonts/OpenPixel-Regular.woff2").unwrap();
        assert_eq!(
            remote_file_name(&url, Some("font/woff2")).unwrap(),
            "OpenPixel-Regular.woff2"
        );
    }

    #[test]
    fn uses_content_type_when_remote_url_has_no_extension() {
        let url = reqwest::Url::parse("https://example.com/font?id=123").unwrap();
        let file_name = remote_file_name(&url, Some("font/ttf")).unwrap();
        assert!(file_name.starts_with("font"));
        assert!(file_name.ends_with(".ttf"));
    }
}
