//! High-level MCP Server API
//!
//! This module provides a simplified MCP server that acts as a lifecycle
//! wrapper around pre-configured MCP transports.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::error::{McpError, McpResult};
use crate::protocol::transport::Transport;
use crate::protocol::LoggingConfig;

/// Trait for handling logging operations
#[async_trait]
pub trait LoggingHandler: Send + Sync {
    /// Set logging configuration
    async fn set_logging(&self, config: LoggingConfig) -> McpResult<bool>;
}

/// High-level MCP server lifecycle wrapper
///
/// This is a simplified server that acts as a lifecycle wrapper around
/// pre-configured MCP transports. The transport implementations handle
/// their own message routing and MCP protocol logic internally.
///
/// # Architecture
///
/// ```text
/// McpServer<T> -> Transport (pre-configured with providers)
/// (lifecycle)     (handles MCP protocol internally)
/// ```
///
/// # Usage
///
/// The server wraps a pre-configured transport that handles all the MCP
/// protocol details internally. The server just manages lifecycle.
pub struct McpServer<T: Transport> {
    /// Underlying pre-configured transport
    transport: Arc<Mutex<T>>,
}

impl<T: Transport + 'static> McpServer<T> {
    /// Create a new MCP server with a pre-configured transport
    pub fn new(transport: T) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
        }
    }

    /// Start the server
    ///
    /// Starts the pre-configured transport which should handle all MCP
    /// protocol logic internally.
    pub async fn start(&self) -> McpResult<()> {
        let mut transport = self.transport.lock().await;
        transport.start().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to start transport: {e}"),
            })
        })?;

        Ok(())
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> McpResult<()> {
        let mut transport = self.transport.lock().await;
        transport.close().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to close transport: {e}"),
            })
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcMessage, MessageHandler};
    use crate::transport::adapters::stdio::{StdioMessageContext, StdioTransportBuilder};

    // Simple test message handler for integration tests
    #[derive(Debug)]
    struct TestMessageHandler;

    #[async_trait]
    impl MessageHandler<()> for TestMessageHandler {
        async fn handle_message(&self, _message: JsonRpcMessage, _context: StdioMessageContext) {
            // Simple test handler - just ignore messages
        }

        async fn handle_error(&self, _error: crate::protocol::TransportError) {
            // Simple test handler - just ignore errors
        }

        async fn handle_close(&self) {
            // Simple test handler - no cleanup needed
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        let handler = Arc::new(TestMessageHandler);
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .build()
            .await
            .unwrap();

        let server = McpServer::new(transport);

        // Verify it's a simple lifecycle wrapper - server should be created successfully
        assert!(!server.transport.lock().await.is_connected());
    }

    #[tokio::test]
    async fn test_lifecycle_operations() {
        let handler = Arc::new(TestMessageHandler);
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .build()
            .await
            .unwrap();

        let server = McpServer::new(transport);

        // NOTE: We don't test actual start() because STDIO transport
        // blocks on stdin reading in test environment. This is a known
        // limitation documented in the transport's own tests.
        //
        // Instead, we test that the server can be created successfully
        // and basic operations work without hanging.

        // Test server creation and basic state
        assert!(!server.transport.lock().await.is_connected());

        // Test shutdown when not running (should be safe)
        let shutdown_result = server.shutdown().await;
        assert!(
            shutdown_result.is_ok(),
            "Server shutdown should succeed even when not started"
        );
    }
}
