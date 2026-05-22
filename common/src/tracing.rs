use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_tracing() -> WorkerGuard {
    let env_filter = EnvFilter::from_default_env();

    let stdout_layer = tracing_subscriber::fmt::layer().with_ansi(true).compact();

    let file_appender = tracing_appender::rolling::daily("/app/logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    let _ = dotenvy::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .expect("please provide the env var OTEL_EXPORTER_OTLP_ENDPOINT");

    let exporter = SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create OTLP span exporter");

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());
    let service_name =
        dotenvy::var("OTEL_SERVICE_NAME").expect("OTEL_SERVICE_NAME env var needs to be a thing");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(provider.tracer(service_name));

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .with(telemetry_layer)
        .init();

    guard
}
