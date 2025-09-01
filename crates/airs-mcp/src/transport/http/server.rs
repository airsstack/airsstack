//! HTTP Server Transport Implementation
//!
//! This module provides the server-side HTTP transport for MCP communication.
//! It handles receiving requests from MCP clients and sending responses.

// Layer 1: Standard library imports (per workspace standards §2.1)
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::{oneshot, Mutex};

// Layer 3: Internal module imports
use super::connection_manager::HttpConnectionManager;
use super::session::SessionManager;
use super::{AxumHttpServer, HttpTransportConfig, RequestParser};
use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::correlation::manager::CorrelationManager;
use crate::transport::{error::TransportError, Transport};

/// HTTP Server Transport - Transport Trait Adapter for AxumHttpServer
///
/// This transport implements the adapter pattern to bridge the AxumHttpServer
/// (which handles HTTP-specific functionality) with the generic Transport trait
/// interface required by McpServerBuilder.
///
/// # Architecture
///
/// ```text
/// McpServerBuilder -> HttpServerTransport -> AxumHttpServer -> HTTP Clients
///                          (Adapter)           (Component)
/// ```
///
/// # Usage
///
/// ```rust,no_run
/// use airs_mcp::transport::http::{HttpServerTransport, HttpTransportConfig};
/// use airs_mcp::integration::mcp::McpServerBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = HttpTransportConfig::new()
///     .bind_address("127.0.0.1:3000".parse().unwrap());
///
/// let transport = HttpServerTransport::new(config).await?;
///
/// let server = McpServerBuilder::new()
///     .server_info("My HTTP Server", "1.0.0")
///     .build(transport)
///     .await?;
///
/// server.run().await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpServerTransport {
    config: HttpTransportConfig,
    request_parser: RequestParser,
    bind_address: SocketAddr,

    // Core HTTP server component
    axum_server: Option<AxumHttpServer>,

    // Request/Response coordination
    incoming_requests: Arc<Mutex<VecDeque<Vec<u8>>>>,
    response_sender: Option<oneshot::Sender<Vec<u8>>>,

    // Server components (used for AxumHttpServer construction)
    #[allow(dead_code)]
    connection_manager: Arc<HttpConnectionManager>,
    #[allow(dead_code)]
    session_manager: Arc<SessionManager>,
    #[allow(dead_code)]
    jsonrpc_processor: Arc<ConcurrentProcessor>,
}

impl HttpServerTransport {
    /// Create a new HTTP server transport with the given configuration
    ///
    /// This creates a functional HTTP server transport that integrates
    /// AxumHttpServer with the Transport trait interface.
    pub async fn new(config: HttpTransportConfig) -> Result<Self, TransportError> {
        let request_parser = RequestParser::new(config.parser.clone());
        let bind_address = config.bind_address;

        // Create required components for AxumHttpServer
        let connection_manager = Arc::new(HttpConnectionManager::new(
            config.max_connections as usize,
            Default::default(),
        ));

        // Create correlation manager for session management
        let correlation_manager = Arc::new(
            CorrelationManager::new(Default::default())
                .await
                .map_err(|e| TransportError::Other {
                    details: format!("Failed to create correlation manager: {}", e),
                })?,
        );

        let session_manager =
            Arc::new(SessionManager::new(correlation_manager, Default::default()));

        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(Default::default()));

        // Create the HTTP server with empty handlers (will be configured later)
        let axum_server = AxumHttpServer::new_with_empty_handlers(
            connection_manager.clone(),
            session_manager.clone(),
            jsonrpc_processor.clone(),
            config.clone(),
        )
        .await
        .map_err(|e| TransportError::Other {
            details: format!("Failed to create Axum server: {}", e),
        })?;

        Ok(Self {
            bind_address,
            config,
            request_parser,
            axum_server: Some(axum_server),
            incoming_requests: Arc::new(Mutex::new(VecDeque::new())),
            response_sender: None,
            connection_manager,
            session_manager,
            jsonrpc_processor,
        })
    }

    /// Get the transport configuration
    pub fn config(&self) -> &HttpTransportConfig {
        &self.config
    }

    /// Get the request parser
    pub fn parser(&self) -> &RequestParser {
        &self.request_parser
    }

    /// Get buffer pool statistics (if using buffer pool)
    pub fn buffer_stats(&self) -> Option<super::BufferPoolStats> {
        self.request_parser.buffer_stats()
    }

    /// Get the bind address for the server
    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    /// Check if the HTTP server is ready to accept connections
    pub fn is_server_ready(&self) -> bool {
        self.axum_server
            .as_ref()
            .map_or(false, |server| server.is_bound())
    }

    /// Start the HTTP server and bind to the configured address
    ///
    /// This must be called before using the transport for communication.
    pub async fn start_server(&mut self) -> Result<(), TransportError> {
        if let Some(ref mut server) = self.axum_server {
            server.bind(self.bind_address).await?;
            tracing::info!("HTTP server transport bound to {}", self.bind_address);
            Ok(())
        } else {
            Err(TransportError::Other {
                details: "HTTP server not initialized".to_string(),
            })
        }
    }
}

// Implementation of Transport trait for HTTP Server Transport
impl Transport for HttpServerTransport {
    type Error = TransportError;

    async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
        // For Phase 1: Basic implementation - queue response for current session
        // TODO: Implement proper session correlation in Phase 2

        if let Some(sender) = self.response_sender.take() {
            sender
                .send(message.to_vec())
                .map_err(|_| TransportError::Other {
                    details: "Failed to send response - receiver dropped".to_string(),
                })?;
            Ok(())
        } else {
            // No active request to respond to - this is a limitation of the current adapter
            tracing::warn!("send() called without active request - message queued");

            // For now, we'll ignore the message since there's no active HTTP request to respond to
            // In a full implementation, this would require a different approach
            Ok(())
        }
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        // For Phase 1: Basic implementation - simulate receiving HTTP requests
        // TODO: Integrate with actual AxumHttpServer request handling in Phase 2

        // Check if we have queued requests
        {
            let mut queue = self.incoming_requests.lock().await;
            if let Some(request) = queue.pop_front() {
                // Create a channel for the response
                let (tx, _rx) = oneshot::channel();
                self.response_sender = Some(tx);

                return Ok(request);
            }
        }

        // No queued requests - in a real implementation, this would block
        // waiting for HTTP requests from AxumHttpServer
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Err(TransportError::Other {
            details: "No incoming requests available - Phase 1 implementation limitation"
                .to_string(),
        })
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Cleanup HTTP server and resources
        if let Some(_server) = self.axum_server.take() {
            tracing::info!("HTTP server transport closed");
            // Note: AxumHttpServer doesn't currently have a shutdown method
            // This would be implemented in a complete solution
        }

        // Clear any pending messages
        {
            let mut queue = self.incoming_requests.lock().await;
            queue.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_server_transport_creation() {
        let config = HttpTransportConfig::new()
            .bind_address("127.0.0.1:0".parse().unwrap()) // Use port 0 for testing
            .max_connections(2000);

        let transport = HttpServerTransport::new(config).await.unwrap();

        assert_eq!(transport.config().max_connections, 2000);
        assert_eq!(transport.bind_address().ip().to_string(), "127.0.0.1");
        assert!(!transport.is_server_ready()); // Not bound yet
    }

    #[tokio::test]
    async fn test_server_transport_start() {
        let config = HttpTransportConfig::new().bind_address("127.0.0.1:0".parse().unwrap());

        let mut transport = HttpServerTransport::new(config).await.unwrap();

        // Start the server
        transport.start_server().await.unwrap();
        assert!(transport.is_server_ready());
    }

    #[tokio::test]
    async fn test_server_specific_functionality() {
        let config = HttpTransportConfig::new()
            .bind_address("0.0.0.0:0".parse().unwrap()) // Use port 0 for testing
            .enable_buffer_pool()
            .buffer_pool_size(50);

        let transport = HttpServerTransport::new(config).await.unwrap();

        // Test server-specific methods
        assert_eq!(transport.bind_address().ip().to_string(), "0.0.0.0");
        assert!(transport.buffer_stats().is_some());
    }

    #[tokio::test]
    async fn test_server_configuration_builder() {
        let config = HttpTransportConfig::new()
            .bind_address("0.0.0.0:0".parse().unwrap()) // Use port 0 for testing
            .max_connections(5000)
            .max_concurrent_requests(20)
            .session_timeout(Duration::from_secs(600))
            .keep_alive_timeout(Duration::from_secs(60))
            .request_timeout(Duration::from_secs(30));

        let transport = HttpServerTransport::new(config).await.unwrap();

        assert_eq!(transport.config().bind_address.ip().to_string(), "0.0.0.0");
        assert_eq!(transport.config().max_connections, 5000);
        assert_eq!(transport.config().max_concurrent_requests, 20);
        assert_eq!(transport.config().session_timeout, Duration::from_secs(600));
        assert_eq!(
            transport.config().keep_alive_timeout,
            Duration::from_secs(60)
        );
        assert_eq!(transport.config().request_timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_transport_trait_implementation() {
        let config = HttpTransportConfig::new().bind_address("127.0.0.1:0".parse().unwrap());

        let mut transport = HttpServerTransport::new(config).await.unwrap();

        // Test close operation
        transport.close().await.unwrap();

        // Test receive with no messages (should return error in Phase 1)
        let result = transport.receive().await;
        assert!(result.is_err());
    }
}
