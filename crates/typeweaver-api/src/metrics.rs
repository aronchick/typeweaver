use prometheus::{Histogram, HistogramOpts, IntCounter, IntCounterVec, IntGauge, Opts, Registry};

/// Application-level Prometheus metrics.
pub struct Metrics {
    pub registry: Registry,
    pub requests_total: IntCounter,
    pub request_duration: Histogram,
    pub fonts_ingested_total: IntCounter,
    pub bench_runs_total: IntCounter,
    pub ocr_runs_total: IntCounter,
    pub registry_size: IntGauge,
    pub api_calls_total: IntCounterVec,
    pub active_requests: IntGauge,
    pub upload_bytes_total: IntCounter,
}

impl Metrics {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let registry = Registry::new();

        let requests_total =
            IntCounter::new("typeweaver_requests_total", "Total HTTP requests served").unwrap();
        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "typeweaver_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
        )
        .unwrap();
        let fonts_ingested_total =
            IntCounter::new("typeweaver_fonts_ingested_total", "Total fonts ingested").unwrap();
        let bench_runs_total =
            IntCounter::new("typeweaver_bench_runs_total", "Total benchmark runs").unwrap();
        let ocr_runs_total =
            IntCounter::new("typeweaver_ocr_runs_total", "Total OCR scoring runs").unwrap();
        let registry_size =
            IntGauge::new("typeweaver_registry_size", "Number of fonts in registry").unwrap();
        let api_calls_total = IntCounterVec::new(
            Opts::new("typeweaver_api_calls_total", "Total API calls by endpoint"),
            &["endpoint"],
        )
        .unwrap();
        let active_requests =
            IntGauge::new("typeweaver_active_requests", "Currently in-flight requests").unwrap();
        let upload_bytes_total =
            IntCounter::new("typeweaver_upload_bytes_total", "Total bytes uploaded via ingest")
                .unwrap();

        registry
            .register(Box::new(requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();
        registry
            .register(Box::new(fonts_ingested_total.clone()))
            .unwrap();
        registry
            .register(Box::new(bench_runs_total.clone()))
            .unwrap();
        registry
            .register(Box::new(ocr_runs_total.clone()))
            .unwrap();
        registry
            .register(Box::new(registry_size.clone()))
            .unwrap();
        registry
            .register(Box::new(api_calls_total.clone()))
            .unwrap();
        registry
            .register(Box::new(active_requests.clone()))
            .unwrap();
        registry
            .register(Box::new(upload_bytes_total.clone()))
            .unwrap();

        Self {
            registry,
            requests_total,
            request_duration,
            fonts_ingested_total,
            bench_runs_total,
            ocr_runs_total,
            registry_size,
            api_calls_total,
            active_requests,
            upload_bytes_total,
        }
    }
}
