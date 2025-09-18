//! STDIO Transport Integration
//!
//! This module provides STDIO transport creation and configuration
//! using the proper StdioTransportBuilder pattern.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (none needed for this module)

// Layer 3: Internal module imports
use airs_mcp::protocol::TransportError;
use airs_mcp::transport::adapters::stdio::{StdioTransport, StdioTransportBuilder};
use crate::handlers::McpHandler;

/// Create STDIO transport with the provided MCP handler
///
/// This function demonstrates the proper transport integration pattern
/// using the pre-configured StdioTransportBuilder.
///
/// # Arguments
///
/// * `handler` - Arc-wrapped MCP handler implementing MessageHandler<()>
///
/// # Returns
///
/// A configured StdioTransport ready to start processing messages
///
/// # Errors
///
/// Returns `TransportError` if transport creation fails
///
/// # Examples
///
/// ```rust,no_run
/// use stdio_server_integration::{create_test_environment, create_stdio_transport};
/// use std::sync::Arc;
/// use airs_mcp::protocol::Transport;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let (handler, _temp_dir) = create_test_environment().await?;
/// let handler = Arc::new(handler);
///
/// let mut transport = create_stdio_transport(handler).await?;
/// transport.start().await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_stdio_transport(
    handler: Arc<McpHandler>
) -> Result<StdioTransport, TransportError> {
    StdioTransportBuilder::new()
        .with_message_handler(handler)
        .build()
        .await
}