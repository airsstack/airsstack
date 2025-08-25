//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//!
//! Binary entry point for the MCP server that provides secure filesystem operations
//! for Claude Desktop and other MCP-compatible AI tools.

// Layer 1: Standard library imports
use std::process;

// Layer 2: Third-party crate imports
use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::transport::StdioTransport;
use anyhow::Result;
use tracing::{error, info};

// Layer 3: Internal module imports
use airs_mcp_fs::{DefaultFilesystemMcpServer, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging with environment filter support
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "airs_mcp_fs=info".into()),
        )
        .init();

    info!(
        "🚀 Starting AIRS MCP-FS server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration settings
    let settings = match Settings::load() {
        Ok(settings) => {
            info!("✅ Configuration loaded successfully");
            settings
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Create STDIO transport for Claude Desktop integration
    let transport = match StdioTransport::new().await {
        Ok(transport) => {
            info!("✅ STDIO transport initialized for Claude Desktop");
            transport
        }
        Err(e) => {
            error!("❌ Failed to create STDIO transport: {}", e);
            process::exit(1);
        }
    };

    // Initialize filesystem MCP server
    let filesystem_server = match DefaultFilesystemMcpServer::with_default_handlers(settings).await
    {
        Ok(server) => {
            info!("✅ Filesystem MCP server initialized with security manager");
            server
        }
        Err(e) => {
            error!("❌ Failed to initialize filesystem server: {}", e);
            process::exit(1);
        }
    };

    // Build complete MCP server with STDIO transport and filesystem tools
    let mcp_server = match McpServerBuilder::new()
        .server_info("airs-mcp-fs", env!("CARGO_PKG_VERSION"))
        .with_tool_provider(filesystem_server)
        .build(transport)
        .await
    {
        Ok(server) => {
            info!("✅ Complete MCP server built successfully");
            info!("📋 Available tools: read_file, write_file, list_directory");
            info!("🔗 Ready for Claude Desktop connections via STDIO");
            server
        }
        Err(e) => {
            error!("❌ Failed to build MCP server: {}", e);
            process::exit(1);
        }
    };

    // Start the server event loop
    info!("🎯 Starting MCP server event loop...");
    if let Err(e) = mcp_server.run().await {
        error!("💥 MCP server runtime error: {}", e);
        process::exit(1);
    }

    info!("🛑 AIRS MCP-FS server shutdown completed");
    Ok(())
}
