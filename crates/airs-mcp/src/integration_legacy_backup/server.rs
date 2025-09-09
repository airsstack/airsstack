//! JSON-RPC Server Implementation
//!
//! This module provides a high-level JSON-RPC server that handles incoming
//! requests and routes them to appropriate handlers.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::transport::Transport;
use crate::protocol::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

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

    /// Start the server with request and notification handlers
    pub async fn run<F, Fut, G, GFut>(
        &self,
        request_handler: F,
        notification_handler: G,
    ) -> IntegrationResult<()>
    where
        F: Fn(JsonRpcRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = JsonRpcResponse> + Send + 'static,
        G: Fn(JsonRpcNotification) -> GFut + Send + Sync + 'static,
        GFut: Future<Output = ()> + Send + 'static,
    {
        // Mark as running
        *self.running.write().await = true;

        // Get transport for receiving
        let transport = Arc::clone(&self.transport);
        let running = Arc::clone(&self.running);

        // Message processing loop - parse JSON-RPC directly
        while *running.read().await {
            let mut transport_guard = transport.write().await;

            match transport_guard.receive().await {
                Ok(data) => {
                    // Parse as JSON-RPC message directly
                    if let Ok(json_str) = std::str::from_utf8(&data) {
                        if let Ok(parsed) = serde_json::from_str::<JsonRpcMessage>(json_str) {
                            match parsed {
                                JsonRpcMessage::Request(request) => {
                                    // Process request with handler
                                    let response = request_handler(request).await;

                                    // Send response
                                    let response_data =
                                        serde_json::to_vec(&response).map_err(|e| {
                                            IntegrationError::other(format!(
                                                "Serialization failed: {e}"
                                            ))
                                        })?;

                                    if let Err(e) = transport_guard.send(&response_data).await {
                                        eprintln!("Failed to send response: {e}");
                                    }
                                }
                                JsonRpcMessage::Notification(notification) => {
                                    // Process notification with handler (no response expected)
                                    notification_handler(notification).await;
                                }
                                JsonRpcMessage::Response(_) => {
                                    // Ignore responses (we're a server, not a client)
                                    eprintln!("Received unexpected response message, ignoring");
                                }
                            }
                        } else {
                            eprintln!("Failed to parse JSON-RPC message");
                        }
                    } else {
                        eprintln!("Received non-UTF8 data");
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
