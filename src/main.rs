use clap::Parser;
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

    Ok(())
}
