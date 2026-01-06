use clap::Parser;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions as semconv;
use tracing_subscriber::prelude::*;

#[derive(Debug, Parser, Clone)]
#[command(name = "varnishotel")]
#[command(about = "Exports Varnish telemetry to OpenTelemetry compatible destinations", long_about = None)]
pub struct App {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = App::parse();

    let app_name = env!("CARGO_CRATE_NAME");
    let app_version = env!("CARGO_PKG_VERSION");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", app_name).into()),
        )
        .init();

    tracing::info!(app_name, app_version);

    let instrumentation_resource = Resource::builder()
        .with_attributes([
            KeyValue::new(semconv::resource::SERVICE_NAME, "varnish"),
            KeyValue::new(semconv::resource::SERVICE_VERSION, app_version),
        ])
        .build();

    let instrumentation_scope = opentelemetry::InstrumentationScope::builder("varnish")
        .with_version(app_version)
        .with_schema_url(opentelemetry_semantic_conventions::SCHEMA_URL)
        .with_attributes([])
        .build();

    tracing::debug!("setting up tracer");

    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    let trace_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .with_resource(instrumentation_resource.clone())
        .build();

    let trace_propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();

    let mut trace_scope = opentelemetry::InstrumentationScope::default();
    trace_scope.clone_from(&instrumentation_scope);

    global::set_text_map_propagator(trace_propagator);
    global::set_tracer_provider(trace_provider.clone());

    {
        tokio::spawn(async move {
            tracing::info!("starting varnishtrace");

            let recv = varnishotel_varnishtrace::VarnishlogReceiver::new();
            match recv.execute(trace_scope).await {
                Ok(()) => {}
                Err(err) => tracing::error!("error in varnishtrace: {err}"),
            }
        });
    }

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("shutting down tracer");
            trace_provider.shutdown()?;
        }
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}
