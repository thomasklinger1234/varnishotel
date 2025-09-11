use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[command(name = "varnishotel")]
#[command(about = "Exports Varnish telemetry to OpenTelemetry compatible destinations", long_about = None)]
pub struct App {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = App::parse();

    Ok(())
}
