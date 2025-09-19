//! HTTP API Key MCP Server Integration Example
//!
//! This example demonstrates a complete HTTP MCP server with API key authentication
//! following the established pattern from the stdio-server example, but adapted for HTTP.
//!
//! This server provides a full MCP implementation with:
//! - HTTP transport layer with API key authentication
//! - Standardized tool set (file operations, system info, utilities)
//! - Comprehensive configuration and error handling

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use clap::Parser;
use tracing::info;

// Layer 3: Internal module imports

mod config;
mod tools;
mod transport;

use config::ServerConfig;
use tools::create_test_environment;
use transport::HttpApiKeyServer;

/// Command line arguments for the HTTP API Key server
#[derive(Parser)]
#[command(name = "http-apikey-server")]
#[command(about = "HTTP MCP Server with API Key Authentication")]
struct Args {
    /// Server port
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// API key for development (optional, uses default dev keys if not provided)
    #[arg(long)]
    api_key: Option<String>,

    /// Enable development mode with pre-configured keys
    #[arg(long, default_value = "true")]
    dev_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,airs_mcp=debug".into()),
        )
        .init();

    let args = Args::parse();

    info!("🚀 Starting HTTP MCP Server with API Key Authentication");
    info!("🔑 Authentication: API Key required");
    info!("🌐 Bind Address: localhost:{}", args.port);

    // Create server configuration
    let config = ServerConfig::new(args.port, args.dev_mode, args.api_key);

    // Create test environment and MCP handlers
    let (handlers, _temp_dir_guard) = create_test_environment().await?;

    info!("📦 MCP providers initialized");
    info!("🔧 Available tools:");
    info!("   • File operations: read_file, write_file, list_directory, create_directory");
    info!("   • System information: get_system_info, get_environment, get_process_info");
    info!("   • Utilities: echo, timestamp, health_check");

    // Create HTTP API Key server
    let server = HttpApiKeyServer::new(config)?;

    // Use tokio::select to ensure temp_dir stays alive until server is cancelled
    let result = tokio::select! {
        result = server.start(handlers) => {
            // Server finished normally
            result
        }
        _ = tokio::signal::ctrl_c() => {
            // Server was cancelled
            info!("🛑 Server cancelled by user");
            Ok(())
        }
    };

    info!("🎯 HTTP API Key server shutdown gracefully");
    result
}
