//! Serve command handler for AIRS MCP-FS
//!
//! Handles starting the MCP server with STDIO transport for Claude Desktop integration.

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use anyhow::Result;
use tracing::{error, info};

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::protocol::Transport;
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;

// Layer 3b: Local crate modules (only through lib.rs gateway)
use crate::{ConfigurationLoader, DefaultFilesystemMcpServer, FilesystemMessageHandler, Settings};

/// Handle the serve command - start the MCP server
pub async fn handle_serve(config_dir: Option<PathBuf>, _logs_dir: Option<PathBuf>) -> Result<()> {
    info!(
        "🚀 Starting AIRS MCP-FS server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration settings with custom config directory if provided
    let settings = if let Some(custom_config_dir) = config_dir {
        info!(
            "📁 Using custom configuration directory: {}",
            custom_config_dir.display()
        );

        // Temporarily set the environment variable for the configuration loader
        std::env::set_var("AIRS_MCPSERVER_FS_CONFIG_DIR", &custom_config_dir);

        // Create configuration loader with custom directory
        let loader = ConfigurationLoader::new().with_config_dir(custom_config_dir);
        let (settings, source_info) = loader.load().map_err(|e| {
            error!(
                "❌ Failed to load configuration from custom directory: {}",
                e
            );
            e
        })?;

        // Log configuration source information
        info!(
            "📋 Configuration loaded from {} environment",
            source_info.environment
        );
        if !source_info.files.is_empty() {
            info!("   Configuration files: {:?}", source_info.files);
        }

        settings
    } else {
        // Use standard configuration loading
        match Settings::load() {
            Ok(settings) => {
                info!("✅ Configuration loaded successfully");
                settings
            }
            Err(e) => {
                error!("❌ Failed to load configuration: {}", e);
                error!("💡 Try running: airs-mcpserver-fs config");
                process::exit(1);
            }
        }
    };

    // Create STDIO transport for Claude Desktop integration
    info!("🔌 Creating STDIO transport with MessageHandler integration");

    // Initialize filesystem MCP server
    let filesystem_server = match DefaultFilesystemMcpServer::with_default_handlers(settings).await
    {
        Ok(server) => {
            info!("✅ Filesystem MCP server initialized with security manager");
            Arc::new(server)
        }
        Err(e) => {
            error!("❌ Failed to initialize filesystem server: {}", e);
            process::exit(1);
        }
    };

    // Create MessageHandler wrapper for the server
    let message_handler = Arc::new(FilesystemMessageHandler::new(filesystem_server));
    info!("✅ MessageHandler wrapper created");

    // Create and configure STDIO transport with handler
    let mut transport = match StdioTransportBuilder::new()
        .with_message_handler(message_handler)
        .build()
        .await
    {
        Ok(transport) => {
            info!("✅ STDIO transport created with MessageHandler integration");
            transport
        }
        Err(e) => {
            error!("❌ Failed to create STDIO transport: {}", e);
            process::exit(1);
        }
    };

    info!("🚀 Starting AIRS MCP-FS server");
    info!("📋 Available capabilities:");
    info!("   • Tools: read_file, write_file, list_directory");
    info!("   • Security: Path validation, approval workflows, audit logging");
    info!("   • Transport: STDIO integration for Claude Desktop");
    info!("");
    info!("💡 Usage:");
    info!("   Connect via Claude Desktop MCP client configuration");
    info!("   Send JSON-RPC requests to stdin, receive responses on stdout");

    // Start the transport - this begins reading from stdin in background
    if let Err(e) = transport.start().await {
        error!("❌ Failed to start STDIO transport: {}", e);
        process::exit(1);
    }

    info!("✅ STDIO transport started, ready for MCP communication");
    info!("🎧 Server is now listening on stdin for JSON-RPC messages");

    // Wait for transport completion (blocks until stdin EOF or error)
    // This is the key fix - wait for the background stdin reader to complete
    if let Err(e) = transport.wait_for_completion().await {
        error!("❌ Transport error during operation: {}", e);
        process::exit(1);
    }

    info!("✅ AIRS MCP-FS server shutdown complete");
    Ok(())
}
