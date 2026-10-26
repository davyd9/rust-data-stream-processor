mod cli;
mod error;
mod processing;

use crate::cli::Cli;
use clap::Parser;

/// The main entry point for the application.
///
/// This function initializes the logger, parses command-line arguments, and
/// orchestrates the data processing stream. It uses `anyhow::Result` for
/// convenient top-level error handling.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger based on the RUST_LOG environment variable.
    env_logger::init();

    // Parse command-line arguments into the Cli struct.
    let cli = Cli::parse();
    log::info!("Configuration loaded: {:?}", cli);

    // Delegate the core logic to the processing module.
    // The `?` operator will propagate any errors up to be handled by anyhow.
    processing::process_stream(&cli).await?;

    log::info!("Processing completed successfully.");

    Ok(())
}
