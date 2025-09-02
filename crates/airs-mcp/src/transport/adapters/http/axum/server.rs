//! Axum HTTP Server Implementation
//!
//! This module provides the main HTTP server implementation using Axum framework
//! with clean separation of concerns and proper dependency injection.
//!
//! The AxumHttpServer implements the HttpEngine trait to provide pluggable
//! HTTP framework abstraction while maintaining full MCP protocol support.

// Layer 1: Standard library imports
use std::net::SocketAddr;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

// Layer 3: Internal module imports
use crate::authentication::manager::AuthenticationManager;
use crate::authentication::strategy::AuthenticationStrategy;
use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::connection_manager::HttpConnectionManager;
use crate::transport::adapters::http::engine::{
    HttpEngine, HttpEngineError, HttpMiddleware, McpRequestHandler,
};
use crate::transport::adapters::http::session::SessionManager;
use crate::transport::error::TransportError;

use super::handlers::{create_router, ServerState};
use super::mcp_handlers::{McpHandlers, McpHandlersBuilder};

/// HTTP server implementation using Axum framework
///
/// This server provides a clean, modular architecture for handling MCP JSON-RPC
/// requests over HTTP with proper session management and connection tracking.
/// It implements the HttpEngine trait for pluggable HTTP framework abstraction.
pub struct AxumHttpServer {
    /// Server state shared across handlers
    state: ServerState,
    /// TCP listener for accepting connections
    listener: Option<TcpListener>,
    /// Local address the server is bound to
    local_addr: Option<SocketAddr>,
    /// Whether the server is currently running
    is_running: bool,
    /// Registered MCP request handler
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,
    /// Custom middleware
    middleware: Vec<Box<dyn HttpMiddleware>>,
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
        // Create SSE broadcast channel for HTTP Streamable support
        let (sse_broadcaster, _receiver) = broadcast::channel(1000);

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config: config.clone(),
            sse_broadcaster,
        };

        Ok(Self {
            state,
            listener: None,
            local_addr: None,
            is_running: false,
            mcp_handler: None,
            middleware: Vec::new(),
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
        self.local_addr = Some(listener.local_addr().map_err(TransportError::Io)?);
        self.listener = Some(listener);
        Ok(())
    }

    /// Start the HTTP server and begin accepting connections
    pub async fn serve(mut self) -> Result<(), TransportError> {
        let app = create_router(self.state);

        let listener = self.listener.ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        self.is_running = true;

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(TransportError::Io)?;

        self.is_running = false;
        Ok(())
    }

    /// Start the HTTP server using mutable reference (for HttpEngine trait)
    pub async fn start(&mut self) -> Result<(), TransportError> {
        if self.is_running {
            return Err(TransportError::Format {
                message: "Server is already running".into(),
            });
        }

        let app = create_router(self.state.clone());

        let listener = self.listener.take().ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        self.is_running = true;

        // Start server in the background - this is a simplified approach
        // for the HttpEngine trait interface
        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });

        Ok(())
    }

    /// Check if the server is bound to an address
    pub fn is_bound(&self) -> bool {
        self.listener.is_some()
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the local address the server is bound to
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }
}

// ================================================================================================
// HttpEngine Trait Implementation
// ================================================================================================

#[async_trait]
impl HttpEngine for AxumHttpServer {
    type Error = TransportError;
    type Config = HttpTransportConfig;

    /// Create a new HTTP engine with the given configuration
    fn new(_config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // Note: This simplified constructor is for HttpEngine trait compatibility.
        // However, AxumHttpServer requires complex async dependencies that cannot
        // be created in a synchronous constructor.
        //
        // For actual usage, use the `new()` method with proper dependency injection.
        // This limitation will be addressed in Phase 5 with proper async factory patterns.

        Err(TransportError::Format {
            message: "AxumHttpServer requires dependency injection - use new() with proper dependencies or async factory pattern".into(),
        })
    }

    /// Bind the engine to a network address
    async fn bind(&mut self, addr: SocketAddr) -> Result<(), HttpEngineError> {
        self.bind(addr).await.map_err(|e| match e {
            TransportError::Io(io_err) => HttpEngineError::Io(io_err),
            _ => HttpEngineError::Engine {
                message: format!("Bind failed: {e}"),
            },
        })
    }

    /// Start the HTTP server
    async fn start(&mut self) -> Result<(), HttpEngineError> {
        if self.is_running {
            return Err(HttpEngineError::AlreadyRunning);
        }

        self.start().await.map_err(|e| match e {
            TransportError::Io(io_err) => HttpEngineError::Io(io_err),
            _ => HttpEngineError::Engine {
                message: format!("Start failed: {e}"),
            },
        })
    }

    /// Gracefully shutdown the HTTP server
    async fn shutdown(&mut self) -> Result<(), HttpEngineError> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        // Note: For proper shutdown, we'd need to store the server handle
        // This will be improved in Phase 5
        Ok(())
    }

    /// Register the MCP request handler
    fn register_mcp_handler(&mut self, handler: Arc<dyn McpRequestHandler>) {
        self.mcp_handler = Some(handler);
    }

    /// Register authentication middleware
    fn register_authentication<S, T, D>(
        &mut self,
        _auth_manager: AuthenticationManager<S, T, D>,
    ) -> Result<(), HttpEngineError>
    where
        S: AuthenticationStrategy<T, D>,
        T: Send + Sync,
        D: Send + Sync + 'static,
    {
        // TODO: Implement authentication integration
        // For now, this is a placeholder that accepts the authentication manager
        // but doesn't store it. Full integration will be implemented in Phase 5.
        Ok(())
    }

    /// Register custom HTTP middleware
    fn register_middleware(&mut self, middleware: Box<dyn HttpMiddleware>) {
        self.middleware.push(middleware);
    }

    /// Check if the engine is bound to an address
    fn is_bound(&self) -> bool {
        self.listener.is_some()
    }

    /// Check if the engine is currently running
    fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the local address the engine is bound to
    fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Get the engine type identifier
    fn engine_type(&self) -> &'static str {
        "axum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::jsonrpc::concurrent::ProcessorConfig;
    use crate::correlation::manager::{CorrelationConfig, CorrelationManager};
    use crate::transport::adapters::http::connection_manager::HealthCheckConfig;
    use crate::transport::adapters::http::session::SessionConfig;

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
