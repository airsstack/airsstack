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
use crate::authentication::AuthContext;
use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::transport::adapters::http::auth::middleware::{HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest, HttpAuthStrategyAdapter};
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::connection_manager::HttpConnectionManager;
use crate::transport::adapters::http::engine::{
    HttpEngine, HttpEngineError, HttpMiddleware, McpRequestHandler,
};
use crate::transport::adapters::http::session::SessionManager;
use crate::transport::error::TransportError;

use super::handlers::{create_router, ServerState};
use super::mcp_handlers::{McpHandlers, McpHandlersBuilder};

/// Zero-cost no authentication adapter for default server behavior
///
/// This type serves as the default authentication strategy for AxumHttpServer,
/// providing a zero-cost abstraction when no authentication is required.
/// It implements HttpAuthStrategyAdapter with no-op behavior.
#[derive(Debug, Clone, Default)]
pub struct NoAuth;

#[async_trait]
impl HttpAuthStrategyAdapter for NoAuth {
    type RequestType = ();
    type AuthData = ();

    fn auth_method(&self) -> &'static str {
        "none"
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, crate::transport::adapters::http::auth::oauth2::error::HttpAuthError> {
        // NoAuth always returns a successful authentication with empty data
        use crate::authentication::AuthMethod;
        Ok(AuthContext::new(AuthMethod::new("none"), ()))
    }

    fn should_skip_path(&self, _path: &str) -> bool {
        true // NoAuth skips authentication for all paths
    }
}

/// HTTP server implementation using Axum framework
///
/// This server provides a clean, modular architecture for handling MCP JSON-RPC
/// requests over HTTP with proper session management and connection tracking.
/// It implements the HttpEngine trait for pluggable HTTP framework abstraction.
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter (defaults to NoAuth for backward compatibility)
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
/// // Default server with no authentication
/// // let server = AxumHttpServer::new(...).await?;
/// ```
pub struct AxumHttpServer<A = NoAuth>
where
    A: HttpAuthStrategyAdapter,
{
    /// Server state shared across handlers
    state: ServerState<A>,
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

impl AxumHttpServer<NoAuth> {
    /// Create a new Axum HTTP server with the specified configuration and handlers
    /// 
    /// Creates a server with NoAuth (no authentication) as the default.
    /// Use with_authentication() to add authentication to the server.
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
            auth_middleware: None, // NoAuth has no middleware
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

    /// Add authentication to the server (zero-cost type conversion)
    ///
    /// Converts the server from NoAuth to a specific authentication strategy.
    /// This is a zero-cost conversion that happens at compile time.
    ///
    /// # Type Parameters
    /// * `A` - Authentication strategy adapter
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    /// * `config` - Authentication middleware configuration
    ///
    /// # Returns
    /// * AxumHttpServer with the specified authentication strategy
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    /// 
    /// // Example usage (requires proper setup)
    /// // let server = AxumHttpServer::new(
    /// //     connection_manager,
    /// //     session_manager, 
    /// //     jsonrpc_processor,
    /// //     mcp_handlers,
    /// //     config,
    /// // ).await?
    /// // .with_authentication(oauth_adapter, HttpAuthConfig::default());
    /// ```
    pub fn with_authentication<A>(
        self,
        adapter: A,
        auth_config: HttpAuthConfig,
    ) -> AxumHttpServer<A>
    where
        A: HttpAuthStrategyAdapter,
    {
        let auth_middleware = HttpAuthMiddleware::new(adapter, auth_config);
        
        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            session_manager: self.state.session_manager,
            jsonrpc_processor: self.state.jsonrpc_processor,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: Some(auth_middleware),
        };
        
        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }

}

/// Generic implementation for AxumHttpServer with any authentication strategy
impl<A> AxumHttpServer<A>
where
    A: HttpAuthStrategyAdapter,
{
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
impl<A> HttpEngine for AxumHttpServer<A>
where
    A: HttpAuthStrategyAdapter,
{
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

    async fn create_test_server() -> AxumHttpServer<NoAuth> {
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
