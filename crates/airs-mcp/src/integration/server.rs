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
        use std::io::Cursor;
        use tokio::io::BufReader;

        // Create a proper test message handler
        let handler = Arc::new(TestMessageHandler);

        // Create mock I/O streams for true lifecycle testing
        // Use array instead of vec for input data
        let input_data = [
            r#"{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}"#,
            "\n",
            r#"{"jsonrpc":"2.0","method":"ping","id":2}"#,
            "\n",
        ]
        .concat()
        .into_bytes();

        let reader = BufReader::new(Cursor::new(input_data));
        let writer = Vec::<u8>::new(); // Simple Vec writer for testing

        // Build transport with custom I/O for real lifecycle testing
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .with_custom_io(reader, writer)
            .build()
            .await
            .expect("Failed to build transport with mock I/O");

        let server = McpServer::new(transport);

        // Test initial state
        assert!(!server.transport.lock().await.is_connected());

        // Test REAL server lifecycle operations with mock I/O
        let start_result = server.start().await;
        assert!(
            start_result.is_ok(),
            "Server start should succeed with mock I/O: {start_result:?}"
        );

        // Verify transport is now connected
        assert!(server.transport.lock().await.is_connected());

        // Allow time for message processing
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        // Test server shutdown with real operations
        let shutdown_result = server.shutdown().await;
        assert!(
            shutdown_result.is_ok(),
            "Server shutdown should succeed: {shutdown_result:?}"
        );

        // Verify transport is disconnected after shutdown
        assert!(!server.transport.lock().await.is_connected());

        // NOTE: This test now performs REAL lifecycle operations including:
        // - Real transport.start() with message processing
        // - Real I/O stream handling (via Vec<u8> writer)
        // - Real transport.close() with proper cleanup
        // - No blocking on stdin - completes in milliseconds
        // - Tests actual server.start() and server.shutdown() operations
    }
}
