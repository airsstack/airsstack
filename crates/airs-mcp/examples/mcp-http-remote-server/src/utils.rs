//! Utility functions for MCP HTTP Remote Server

// Layer 1: Standard library imports
use std::env;

// Layer 2: Third-party crate imports
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Initialize logging with appropriate configuration
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set default log level if not specified
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "mcp_http_remote_server=info,airs_mcp=info");
    }

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mcp_http_remote_server=info,airs_mcp=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("📋 Logging initialized");
    Ok(())
}
