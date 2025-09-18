//! STDIO MCP Server Integration Example
//!
//! This example demonstrates a complete MCP server implementation using proper
//! STDIO transport layer integration with modular architecture.
//!
//! ## Architecture
//!
//! Following the Example Module Architecture Standard (§4.3):
//! - **handlers/**: MCP protocol logic with MessageHandler<()> trait implementation
//! - **providers/**: Provider setup and test environment management
//! - **transport/**: STDIO transport integration using StdioTransportBuilder
//! - **utilities**: Logging and configuration utilities
//!
//! ## Features
//!
//! - **Proper Transport Integration**: Uses StdioTransport with MessageHandler trait
//! - **Event-Driven Processing**: Asynchronous message handling via transport layer
//! - **Standard Providers**: FileSystem resources, Math tools, Code review prompts, Structured logging
//! - **MCP Protocol**: Full MCP 2024-11-05 protocol implementation
//! - **Modular Architecture**: Clean separation of concerns across modules
//!
//! ## Usage
//!
//! ```bash
//! # Run the server
//! cargo run --bin stdio-server
//!
//! # Test with MCP client
//! echo '{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}' | cargo run --bin stdio-server
//! ```

use airs_mcp::protocol::Transport;

// Local module declarations (avoid cross-crate imports for editor stability)
mod handlers;
mod providers;
mod transport;
mod utilities;

// Bring required items into scope from local modules (Layer 3)
use providers::create_test_environment;
use transport::create_stdio_transport;
use utilities::init_logging;

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tracing::info;

// Layer 3: Internal module imports
// (imports handled via lib.rs)

/// Main server entry point - STDIO MCP Server with Transport Integration
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    info!("🚀 Starting STDIO MCP Server (Transport Integration Mode)");

    // Create test environment and MCP handler
    let (handler, _temp_dir) = create_test_environment().await?;
    let handler = Arc::new(handler);

    info!("📦 MCP providers initialized");
    info!("🌟 MCP server starting with STDIO transport integration");
    info!("📋 Available capabilities:");
    info!("   • Tools: Math operations and calculations");
    info!("   • Resources: Filesystem access to test directory");
    info!("   • Prompts: Code review templates");
    info!("   • Logging: Structured logging with configurable levels");
    info!("");
    info!("🔧 Transport Integration:");
    info!("   • StdioTransportBuilder: Pre-configured transport creation");
    info!("   • MessageHandler<()>: Event-driven message processing");
    info!("   • Async Processing: Background task with proper shutdown");
    info!("");
    info!("💡 Usage:");
    info!("   Send JSON-RPC requests to stdin, receive responses on stdout");
    info!("   Example: echo '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{{}}}}' | ./stdio-server");
    info!("");
    info!("🛠️  Environment variables:");
    info!("   • STDIO_LOG_LEVEL: Log level (trace, debug, info, warn, error)");
    info!("   • STDIO_LOG_STRUCTURED: Enable structured logging");

    // Create and start STDIO transport with proper integration
    info!("✅ Creating STDIO transport with handler integration");
    let mut transport = create_stdio_transport(handler).await?;

    info!("🔌 Starting transport layer");
    transport.start().await?;

    info!("🎯 STDIO MCP server ready for requests");

    // Wait for stdin to close (EOF) - the transport will handle this automatically
    // and the background task will complete when stdin is closed
    transport.wait_for_completion().await?;

    info!("🔌 Closing transport");
    transport.close().await?;

    info!("✅ STDIO MCP server shutdown complete");
    Ok(())
}
