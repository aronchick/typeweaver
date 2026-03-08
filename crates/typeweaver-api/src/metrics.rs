use prometheus::{Histogram, HistogramOpts, IntCounter, IntGauge, Registry};

/// Application-level Prometheus metrics.
pub struct Metrics {
    pub registry: Registry,
    pub requests_total: IntCounter,
    pub request_duration: Histogram,
    pub fonts_ingested_total: IntCounter,
    pub bench_runs_total: IntCounter,
    pub ocr_runs_total: IntCounter,
    pub registry_size: IntGauge,
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

        Self {
            registry,
            requests_total,
            request_duration,
            fonts_ingested_total,
            bench_runs_total,
            ocr_runs_total,
            registry_size,
        }
    }
}
