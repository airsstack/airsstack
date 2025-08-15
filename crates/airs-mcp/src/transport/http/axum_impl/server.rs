//! Axum HTTP Server Implementation
//!
//! This module provides the main HTTP server implementation using Axum framework
//! with clean separation of concerns and proper dependency injection.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;

use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::transport::error::TransportError;
use crate::transport::http::config::HttpTransportConfig;
use crate::transport::http::connection_manager::HttpConnectionManager;
use crate::transport::http::session::SessionManager;

use super::handlers::{create_router, ServerState};
use super::mcp_handlers::{McpHandlers, McpHandlersBuilder};

/// HTTP server implementation using Axum framework
///
/// This server provides a clean, modular architecture for handling MCP JSON-RPC
/// requests over HTTP with proper session management and connection tracking.
pub struct AxumHttpServer {
    /// Server state shared across handlers
    state: ServerState,
    /// TCP listener for accepting connections
    listener: Option<TcpListener>,
}

impl AxumHttpServer {
    /// Create a new Axum HTTP server with the specified configuration and handlers
    pub async fn new(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        mcp_handlers: Arc<McpHandlers>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config: config.clone(),
        };

        Ok(Self {
            state,
            listener: None,
        })
    }

    /// Create a new Axum HTTP server with empty MCP handlers (for testing/development)
    pub async fn new_with_empty_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(McpHandlersBuilder::new().build());

        Self::new(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config,
        )
        .await
    }

    /// Create a new Axum HTTP server using a handlers builder
    pub async fn with_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        handlers_builder: McpHandlersBuilder,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(handlers_builder.build());

        Self::new(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config,
        )
        .await
    }

    /// Bind the server to the specified address
    pub async fn bind(&mut self, addr: SocketAddr) -> Result<(), TransportError> {
        let listener = TcpListener::bind(addr).await.map_err(TransportError::Io)?;

        self.listener = Some(listener);
        Ok(())
    }

    /// Start the HTTP server and begin accepting connections
    pub async fn serve(self) -> Result<(), TransportError> {
        let app = create_router(self.state);

        let listener = self.listener.ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(TransportError::Io)?;

        Ok(())
    }

    /// Check if the server is bound to an address
    pub fn is_bound(&self) -> bool {
        self.listener.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::jsonrpc::concurrent::ProcessorConfig;
    use crate::correlation::manager::{CorrelationConfig, CorrelationManager};
    use crate::transport::http::connection_manager::HealthCheckConfig;
    use crate::transport::http::session::SessionConfig;

    async fn create_test_server() -> AxumHttpServer {
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        AxumHttpServer::new_with_empty_handlers(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            config,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_axum_server_creation() {
        let server = create_test_server().await;
        assert!(!server.is_bound());
    }

    #[tokio::test]
    async fn test_axum_server_bind() {
        let mut server = create_test_server().await;
        let addr = "127.0.0.1:0".parse().unwrap();

        server.bind(addr).await.unwrap();
        assert!(server.is_bound());
    }
}
