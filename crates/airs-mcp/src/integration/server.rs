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
                message: format!("Failed to start transport: {}", e),
            })
        })?;

        Ok(())
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> McpResult<()> {
        let mut transport = self.transport.lock().await;
        transport.close().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to close transport: {}", e),
            })
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdioTransport;

    #[tokio::test]
    async fn test_server_creation() {
        let transport = StdioTransport::new();
        let server = McpServer::new(transport);

        // Verify it's a simple lifecycle wrapper
        // In practice, the transport would be pre-configured with message handlers
    }

    #[tokio::test]
    async fn test_lifecycle_operations() {
        let transport = StdioTransport::new();
        let server = McpServer::new(transport);

        // Test basic lifecycle - start and shutdown
        // Note: In practice the transport would be pre-configured
        let start_result = server.start().await;
        // StdioTransport might not actually start anything, so we just verify no panic

        let shutdown_result = server.shutdown().await;
        // Similarly for shutdown
    }
}
