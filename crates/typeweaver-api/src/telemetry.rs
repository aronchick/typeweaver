use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialise tracing with optional OTLP export.
///
/// When `OTEL_EXPORTER_OTLP_ENDPOINT` is set the tracer will attempt to
/// export spans via gRPC; otherwise only the fmt subscriber is used.
pub fn init_tracer() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().compact();

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let exporter = match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
        {
            Ok(e) => e,
            Err(err) => {
                eprintln!("warn: failed to build OTLP exporter: {err}");
                registry.init();
                return;
            }
        };

        let resource = Resource::new(vec![KeyValue::new("service.name", "typeweaver-api")]);

        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_resource(resource)
            .build();

        let tracer = opentelemetry::trace::TracerProvider::tracer(&provider, "typeweaver-api");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        registry.with(otel_layer).init();
    } else {
        registry.init();
    }
}
