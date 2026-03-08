pub mod metrics;
pub mod telemetry;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Multipart, Query, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
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
}

/// Build the application router.
fn app(registry_root: PathBuf) -> Router {
    let state = Arc::new(Mutex::new(AppState {
        registry_root,
        metrics: Metrics::new(),
    }));

    Router::new()
        .route("/api/fonts/ingest", post(handle_ingest))
        .route("/api/fonts", get(handle_list))
        .route("/api/fonts/{id}", get(handle_get_font))
        .route("/api/fonts/{id}/report", get(handle_report))
        .route("/healthz", get(handle_healthz))
        .route("/api/health", get(handle_healthz))
        .route("/okz", get(handle_healthz))
        .route("/varz", get(handle_varz))
        .fallback(handle_static)
        .with_state(state)
}

/// Start the HTTP server.
pub async fn serve(registry_root: PathBuf, host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
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
    mut multipart: Multipart,
) -> Response {
    let st = state.lock().await;
    let reg_root = st.registry_root.clone();
    let fonts_counter = st.metrics.fonts_ingested_total.clone();
    let registry_gauge = st.metrics.registry_size.clone();
    drop(st);

    let upload_dir = reg_root.join("uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create upload dir: {e}"),
        );
    }

    let mut saved_count: u64 = 0;
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .unwrap_or("upload.ttf")
            .to_string();
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("failed to read field: {e}"),
                );
            }
        };
        let dest = upload_dir.join(&file_name);
        if let Err(e) = std::fs::write(&dest, &data) {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to write {}: {e}", file_name),
            );
        }
        saved_count += 1;
    }

    if saved_count == 0 {
        return error_response(StatusCode::BAD_REQUEST, "no files uploaded");
    }

    let registry = match ingest_dir(&upload_dir) {
        Ok(r) => r,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ingest failed: {e}"),
            );
        }
    };

    let reg_dir = registry_dir(&reg_root);
    if let Err(e) = save_registry_at(&reg_dir, &registry) {
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
    json_response(StatusCode::OK, &body)
}

async fn handle_list(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    drop(st);

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(_) => {
            return json_response(StatusCode::OK, "{\"assets\": []}");
        }
    };

    let json = registry_to_json(&registry);
    json_response(StatusCode::OK, &json)
}

async fn handle_get_font(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    drop(st);

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(e) => {
            return error_response(StatusCode::NOT_FOUND, &format!("registry not found: {e}"));
        }
    };

    match find_asset(&registry, &id) {
        Ok(asset) => {
            let body = format!(
                "{{\"id\": \"{}\", \"file_name\": \"{}\", \"family_name\": {}, \"status\": \"{}\"}}",
                asset.id,
                asset.file_name,
                asset.family_name.as_ref().map_or("null".to_string(), |n| format!("\"{}\"", n)),
                asset.status.as_str()
            );
            json_response(StatusCode::OK, &body)
        }
        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("{e}")),
    }
}

#[derive(Deserialize)]
struct ReportQuery {
    profile: Option<String>,
}

async fn handle_report(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Query(query): Query<ReportQuery>,
) -> Response {
    let st = state.lock().await;
    let reg_dir = registry_dir(&st.registry_root);
    let bench_counter = st.metrics.bench_runs_total.clone();
    drop(st);

    let profile = match &query.profile {
        Some(slug) => match BenchmarkProfile::from_slug(slug) {
            Ok(p) => p,
            Err(e) => {
                return error_response(StatusCode::BAD_REQUEST, &format!("{e}"));
            }
        },
        None => BenchmarkProfile::WebLightDefault,
    };

    let registry = match load_registry_at(&reg_dir) {
        Ok(r) => r,
        Err(e) => {
            return error_response(StatusCode::NOT_FOUND, &format!("registry not found: {e}"));
        }
    };

    let asset = match find_asset(&registry, &id) {
        Ok(a) => a.clone(),
        Err(e) => {
            return error_response(StatusCode::NOT_FOUND, &format!("{e}"));
        }
    };

    let report = run_report(&asset, profile);
    bench_counter.inc();

    let json = report.to_json_pretty();
    json_response(StatusCode::OK, &json)
}

async fn handle_healthz() -> Response {
    json_response(StatusCode::OK, "{\"status\": \"ok\"}")
}

async fn handle_varz(State(state): State<Arc<Mutex<AppState>>>) -> Response {
    let st = state.lock().await;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = st.metrics.registry.gather();
    drop(st);

    match encoder.encode_to_string(&metric_families) {
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
    }
}

async fn handle_static(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
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

fn json_response(status: StatusCode, body: &str) -> Response {
    let mut resp = Response::new(axum::body::Body::from(body.to_string()));
    *resp.status_mut() = status;
    resp.headers_mut().insert(
        "content-type",
        HeaderValue::from_static("application/json"),
    );
    resp
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let body = format!("{{\"error\": \"{}\"}}", message.replace('"', "\\\""));
    json_response(status, &body)
}
