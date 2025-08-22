//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//! 
//! Binary entry point for the MCP server that provides secure filesystem operations
//! for Claude Desktop and other MCP-compatible AI tools.

// Layer 1: Standard library imports
use std::process;

// Layer 2: Third-party crate imports
use anyhow::Result;
use tracing::{error, info};

// Layer 3: Internal module imports
use airs_mcp_fs::{McpServer, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging with environment filter support
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "airs_mcp_fs=info".into())
        )
        .init();

    info!("Starting AIRS MCP-FS server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration settings
    let settings = match Settings::load() {
        Ok(settings) => {
            info!("Configuration loaded successfully");
            settings
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Initialize and run MCP server
    match McpServer::new(settings).await {
        Ok(server) => {
            info!("MCP server initialized successfully");
            
            if let Err(e) = server.run().await {
                error!("MCP server error: {}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            error!("Failed to initialize MCP server: {}", e);
            process::exit(1);
        }
    }

    info!("AIRS MCP-FS server shutting down");
    Ok(())
}
