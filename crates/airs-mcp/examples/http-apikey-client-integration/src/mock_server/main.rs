//! Mock HTTP Server Binary
//!
//! Standalone binary for running the lightweight HTTP MCP mock server.

// Layer 1: Standard library imports
use std::env;

// Layer 2: Third-party crate imports
use clap::Parser;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

// Layer 3: Internal module imports - Import from the current module
mod server;
mod responses;

use server::{MockHttpServer, MockServerConfig};

/// Mock HTTP MCP Server CLI
#[derive(Parser)]
#[command(name = "http-mock-server")]
#[command(about = "Lightweight HTTP MCP mock server for testing")]
#[command(version)]
struct Cli {
    /// Server port
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable fault injection for testing
    #[arg(long)]
    fault_injection: bool,

    /// Add artificial delay in milliseconds
    #[arg(long, default_value = "0")]
    delay_ms: u64,

    /// Custom API keys (comma-separated)
    #[arg(long)]
    api_keys: Option<String>,

    /// Enable debug mode
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting HTTP MCP Mock Server");

    // Build configuration
    let mut config = if env::var("MOCK_FROM_ENV").is_ok() {
        info!("Loading configuration from environment variables");
        MockServerConfig::from_env()
    } else {
        MockServerConfig::default()
    };

    // Override with CLI arguments
    config.port = cli.port;
    config.host = cli.host;
    config.fault_injection = cli.fault_injection;
    config.delay_ms = cli.delay_ms;
    config.debug_mode = cli.debug || cli.verbose;

    // Custom API keys
    if let Some(keys) = cli.api_keys {
        config.api_keys = keys
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    info!("Mock server configuration: {:?}", config);

    // Create and start the server
    let server = MockHttpServer::with_config(config);
    
    match server.start().await {
        Ok(()) => {
            info!("Mock server shut down gracefully");
        }
        Err(e) => {
            error!("Mock server error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}