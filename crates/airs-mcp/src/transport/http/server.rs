//! HTTP Server Transport Implementation
//!
//! This module provides the server-side HTTP transport for MCP communication.
//! It handles receiving requests from MCP clients and sending responses.

// Layer 1: Standard library imports (per workspace standards §2.1)
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::{mpsc, oneshot, Mutex};

// Layer 3: Internal module imports
use super::connection_manager::HttpConnectionManager;
use super::session::{SessionId, SessionManager};
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

    // Phase 2: Session-aware message coordination
    incoming_requests: Arc<Mutex<mpsc::UnboundedReceiver<(SessionId, Vec<u8>)>>>,
    incoming_sender: mpsc::UnboundedSender<(SessionId, Vec<u8>)>,
    outgoing_responses: Arc<Mutex<HashMap<SessionId, oneshot::Sender<Vec<u8>>>>>,

    // Current session context for Transport trait operations
    current_session: Option<SessionId>,

    // Transport state
    is_closed: bool,

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

        // Create session coordination channels for Phase 2
        let (incoming_sender, incoming_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            bind_address,
            config,
            request_parser,
            axum_server: Some(axum_server),
            incoming_requests: Arc::new(Mutex::new(incoming_receiver)),
            incoming_sender,
            outgoing_responses: Arc::new(Mutex::new(HashMap::new())),
            current_session: None,
            is_closed: false,
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

    /// Get a sender for incoming HTTP requests (for HTTP handlers to use)
    ///
    /// This allows HTTP handlers to send incoming requests into the Transport coordination system.
    /// Each request is tagged with its session ID for proper correlation.
    pub fn get_request_sender(&self) -> mpsc::UnboundedSender<(SessionId, Vec<u8>)> {
        self.incoming_sender.clone()
    }

    /// Handle incoming HTTP request with session correlation
    ///
    /// This method is called by HTTP handlers to coordinate with the Transport interface.
    /// It sends the request through the coordination system and waits for the response.
    pub async fn handle_http_request(
        &self,
        session_id: SessionId,
        request_data: Vec<u8>,
    ) -> Result<Vec<u8>, TransportError> {
        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();

        // Store response channel
        {
            let mut responses = self.outgoing_responses.lock().await;
            responses.insert(session_id, response_tx);
        }

        // Send request through coordination system
        self.incoming_sender
            .send((session_id, request_data))
            .map_err(|_| TransportError::Other {
                details: "Failed to send request - transport receiver dropped".to_string(),
            })?;

        // Wait for response
        let response = response_rx.await.map_err(|_| TransportError::Other {
            details: "Failed to receive response - sender dropped".to_string(),
        })?;

        Ok(response)
    }

    /// Get the session manager for HTTP handlers to use
    pub fn get_session_manager(&self) -> &Arc<SessionManager> {
        &self.session_manager
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
        // Phase 2: Session-aware response sending
        if self.is_closed {
            return Err(TransportError::Other {
                details: "Transport is closed".to_string(),
            });
        }

        if let Some(session_id) = self.current_session {
            // Remove the response sender for this session
            let sender = {
                let mut responses = self.outgoing_responses.lock().await;
                responses.remove(&session_id)
            };

            if let Some(sender) = sender {
                sender
                    .send(message.to_vec())
                    .map_err(|_| TransportError::Other {
                        details: "Failed to send response - receiver dropped".to_string(),
                    })?;

                // Clear current session after successful send
                self.current_session = None;

                tracing::debug!("Sent response to session {}", session_id);
                Ok(())
            } else {
                Err(TransportError::Other {
                    details: format!("No response channel for session {}", session_id),
                })
            }
        } else {
            Err(TransportError::Other {
                details: "No active session for sending response".to_string(),
            })
        }
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        // Phase 2: Session-aware request receiving
        if self.is_closed {
            return Err(TransportError::Other {
                details: "Transport channel closed".to_string(),
            });
        }

        let message = {
            let mut receiver = self.incoming_requests.lock().await;
            receiver.recv().await
        };

        match message {
            Some((session_id, request_data)) => {
                // Set current session context
                self.current_session = Some(session_id);

                tracing::debug!("Received request from session {}", session_id);
                Ok(request_data)
            }
            None => {
                // Channel closed - transport is shutting down
                self.is_closed = true;
                Err(TransportError::Other {
                    details: "Transport channel closed".to_string(),
                })
            }
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Mark transport as closed
        self.is_closed = true;

        // Cleanup HTTP server and resources
        if let Some(_server) = self.axum_server.take() {
            tracing::info!("HTTP server transport closed");
            // Note: AxumHttpServer doesn't currently have a shutdown method
            // This would be implemented in a complete solution
        }

        // Clear current session context
        self.current_session = None;

        // Clear any pending response channels
        {
            let mut responses = self.outgoing_responses.lock().await;
            responses.clear();
        }

        // Note: incoming_sender will be dropped when the transport is dropped,
        // which will close the channel and cause receive() to return None

        tracing::info!("HTTP server transport session coordination shutdown complete");
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

        // Test receive with no messages (should block until channel closed)
        // We'll test that the channel closes properly after close() is called

        // First close the transport
        transport.close().await.unwrap();

        // Now receive should return an error because channel is closed
        let result = transport.receive().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Transport channel closed"));
    }

    #[tokio::test]
    async fn test_phase2_session_coordination() {
        use uuid::Uuid;

        let config = HttpTransportConfig::new().bind_address("127.0.0.1:0".parse().unwrap());
        let transport = HttpServerTransport::new(config).await.unwrap();

        // Test the session coordination interfaces
        let session_id = Uuid::new_v4();
        let test_request = b"test request data";

        // Test that we can get the coordination components
        let request_sender = transport.get_request_sender();
        let session_manager = transport.get_session_manager();

        // Test request queuing (this sends into the channel)
        request_sender
            .send((session_id, test_request.to_vec()))
            .unwrap();

        // Test basic transport state
        assert!(!transport.is_server_ready()); // Not bound yet
        assert_eq!(transport.bind_address().ip().to_string(), "127.0.0.1");

        // Verify session manager is available
        assert!(session_manager.get_session(session_id).is_none()); // Session not created through normal flow
    }
}
