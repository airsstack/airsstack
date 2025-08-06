//! JSON-RPC Server Implementation
//!
//! This module provides a high-level JSON-RPC server that handles incoming
//! requests and routes them to appropriate handlers.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::base::jsonrpc::message::{JsonRpcRequest, JsonRpcResponse};
use crate::transport::Transport;

use super::error::{IntegrationError, IntegrationResult};

/// High-level JSON-RPC server
pub struct JsonRpcServer<T: Transport> {
    /// Underlying transport
    transport: Arc<RwLock<T>>,
    /// Whether the server is running
    running: Arc<RwLock<bool>>,
}

impl<T: Transport> JsonRpcServer<T> {
    /// Create a new JSON-RPC server with the given transport
    pub async fn new(transport: T) -> IntegrationResult<Self> {
        Ok(Self {
            transport: Arc::new(RwLock::new(transport)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the server with a request handler
    pub async fn run<F, Fut>(&self, handler: F) -> IntegrationResult<()>
    where
        F: Fn(JsonRpcRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JsonRpcResponse> + Send + 'static,
    {
        // Mark as running
        *self.running.write().await = true;

        // Get transport for receiving
        let transport = Arc::clone(&self.transport);
        let running = Arc::clone(&self.running);

        // Message processing loop
        while *running.read().await {
            let mut transport_guard = transport.write().await;

            match transport_guard.receive().await {
                Ok(data) => {
                    // Parse JSON-RPC request
                    match serde_json::from_slice::<JsonRpcRequest>(&data) {
                        Ok(request) => {
                            // Process request with handler
                            let response = handler(request).await;

                            // Send response
                            let response_data = serde_json::to_vec(&response).map_err(|e| {
                                IntegrationError::other(format!("Serialization failed: {e}"))
                            })?;

                            if let Err(e) = transport_guard.send(&response_data).await {
                                eprintln!("Failed to send response: {e}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON-RPC request: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to receive from transport: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> IntegrationResult<()> {
        *self.running.write().await = false;

        let mut transport = self.transport.write().await;
        transport
            .close()
            .await
            .map_err(|e| IntegrationError::other(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::StdioTransport;

    #[tokio::test]
    async fn test_server_creation() {
        let transport = StdioTransport::new().await.unwrap();
        let server = JsonRpcServer::new(transport).await.unwrap();
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn test_server_state_management() {
        let transport = StdioTransport::new().await.unwrap();
        let server = JsonRpcServer::new(transport).await.unwrap();

        // Initially not running
        assert!(!server.is_running().await);

        // Shutdown should work even when not running
        server.shutdown().await.unwrap();
        assert!(!server.is_running().await);
    }
}
