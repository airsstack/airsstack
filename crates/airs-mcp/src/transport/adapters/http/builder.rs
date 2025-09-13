//! HTTP Transport Implementation for Zero-Dyn Architecture
//!
//! This module provides the generic HttpTransport<E: HttpEngine> implementation
//! that eliminates dynamic dispatch and provides Transport trait compatibility
//! for McpServer lifecycle management. The actual MCP processing happens directly
//! through the HttpEngine → McpRequestHandler flow, bypassing the Transport interface.

// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::engine::{HttpEngine, HttpEngineError, McpRequestHandler, ResponseMode};
use crate::protocol::{JsonRpcMessage, Transport, TransportError};

/// Convert HttpEngineError to TransportError
impl From<HttpEngineError> for TransportError {
    fn from(error: HttpEngineError) -> Self {
        match error {
            HttpEngineError::Io(e) => TransportError::Io { source: e },
            HttpEngineError::NotBound => TransportError::Connection {
                message: "HTTP engine not bound to address".to_string(),
            },
            HttpEngineError::AlreadyBound { addr } => TransportError::Connection {
                message: format!("HTTP engine already bound to address: {}", addr),
            },
            HttpEngineError::AlreadyRunning => TransportError::Connection {
                message: "HTTP engine already running".to_string(),
            },
            HttpEngineError::Authentication { message } => TransportError::Protocol { message },
            HttpEngineError::Engine { message } => TransportError::Protocol { message },
        }
    }
}

/// Generic HTTP Transport implementing the Transport trait for McpServer compatibility
///
/// This transport eliminates dynamic dispatch by being generic over HttpEngine,
/// achieving zero-cost abstraction while providing the lifecycle interface
/// required by McpServer. The actual MCP request/response handling is done
/// directly by the HttpEngine → McpRequestHandler flow.
///
/// # Architecture
///
/// ```text
/// McpServer<HttpTransport<E>> -> HttpTransport<E> -> HttpEngine -> McpRequestHandler
/// (lifecycle wrapper)          (Transport trait)   (HTTP server)   (Direct MCP processing)
/// ```
///
/// # Usage with McpServer
///
/// McpServer only calls start() and shutdown() on the transport. All HTTP
/// request processing happens directly through the engine's registered handler.
///
/// # Type Parameters
///
/// * `E` - HTTP engine implementation (e.g., AxumHttpEngine)
///
/// # Usage
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
/// use airs_mcp::integration::server::McpServer;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Phase 4: Basic zero-dyn transport for testing
/// let mut transport = HttpTransportBuilder::with_placeholder_engine()
///     .build().await?;
///
/// // Register MCP handler for direct HTTP processing
/// let handler = (); // Placeholder handler for Phase 4
/// transport.register_mcp_handler(handler);
///
/// // Use with McpServer (just lifecycle wrapper)
/// let server = McpServer::new(transport);
/// server.start().await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpTransport<E: HttpEngine> {
    /// HTTP engine (concrete type - zero dynamic dispatch)
    engine: E,
    /// Session context for current active session
    session_id: Option<String>,
    /// Connection state
    is_connected: bool,
}

impl<E: HttpEngine> Debug for HttpTransport<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransport")
            .field(
                "engine",
                &format!("HttpEngine({})", self.engine.engine_type()),
            )
            .field("session_id", &self.session_id)
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl<E: HttpEngine> HttpTransport<E> {
    /// Create a new HTTP transport with the given engine
    ///
    /// This constructor creates a transport that wraps the provided HTTP engine
    /// and implements the Transport trait for McpServer compatibility.
    ///
    /// # Arguments
    ///
    /// * `engine` - HTTP engine implementation (zero dynamic dispatch)
    ///
    /// # Returns
    ///
    /// New HttpTransport instance ready for use with McpServer
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            session_id: None,
            is_connected: false,
        }
    }

    /// Register an MCP request handler with the underlying engine
    ///
    /// This method delegates to the engine's register_mcp_handler method,
    /// maintaining the zero-cost abstraction while providing a convenient
    /// interface for handler registration.
    ///
    /// # Arguments
    ///
    /// * `handler` - MCP request handler (concrete type specific to engine)
    pub fn register_mcp_handler(&mut self, handler: E::Handler) {
        self.engine.register_mcp_handler(handler);
    }

    /// Get access to the underlying HTTP engine
    ///
    /// This method provides direct access to the engine for advanced
    /// configuration or engine-specific operations.
    ///
    /// # Returns
    ///
    /// Reference to the underlying HTTP engine
    pub fn engine(&self) -> &E {
        &self.engine
    }

    /// Get mutable access to the underlying HTTP engine
    ///
    /// This method provides mutable access to the engine for configuration
    /// and setup operations.
    ///
    /// # Returns
    ///
    /// Mutable reference to the underlying HTTP engine
    pub fn engine_mut(&mut self) -> &mut E {
        &mut self.engine
    }
}

#[async_trait]
impl<E: HttpEngine> Transport for HttpTransport<E> {
    type Error = TransportError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Bind and start the HTTP engine
        if let Some(addr) = self.engine.local_addr() {
            self.engine
                .start()
                .await
                .map_err(|e| TransportError::Connection {
                    message: format!("Failed to start HTTP engine: {}", e),
                })?;

            self.is_connected = true;
            tracing::info!("HTTP transport started on {}", addr);
            Ok(())
        } else {
            Err(TransportError::Connection {
                message: "HTTP engine not bound to address".to_string(),
            })
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Shutdown the HTTP engine
        self.engine
            .shutdown()
            .await
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to shutdown HTTP engine: {}", e),
            })?;

        self.is_connected = false;
        self.session_id = None;

        tracing::info!("HTTP transport closed");
        Ok(())
    }

    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> {
        // For HTTP server transports used with McpServer, send() is not typically called
        // since McpServer only uses start() and shutdown() lifecycle methods.
        // HTTP request/response handling is done directly through the HttpEngine → McpRequestHandler flow.
        // This implementation is provided for Transport trait completeness.

        tracing::warn!(
            "HttpTransport::send() called - this is unusual for HTTP server usage with McpServer"
        );
        tracing::debug!("Message that would be sent: {:?}", message);

        // Return success since the message would have nowhere meaningful to go
        // in an HTTP server context - HTTP doesn't "send" messages, it responds to requests
        Ok(())
    }

    fn session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    fn set_session_context(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    fn is_connected(&self) -> bool {
        self.is_connected && self.engine.is_running()
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }
}

/// Builder for creating HTTP transports with specific engine types
///
/// This builder implements the zero-dyn pattern by being generic over the
/// HTTP engine type, eliminating dynamic dispatch while providing a convenient
/// builder interface.
///
/// # Type Parameters
///
/// * `E` - HTTP engine implementation (e.g., AxumHttpEngine)
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Phase 4: With placeholder engine for testing/development
/// let transport = HttpTransportBuilder::with_placeholder_engine()
///     .build().await?;
///
/// // Phase 5: With concrete engine configuration (to be implemented)
/// // let transport = HttpTransportBuilder::with_axum_engine(connection_manager, config).await?
/// //     .configure_engine(|engine| {
/// //         engine.register_middleware(custom_middleware);
/// //     })
/// //     .bind("127.0.0.1:8080".parse()?).await?
/// //     .build().await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpTransportBuilder<E: HttpEngine> {
    /// HTTP engine instance
    engine: E,
}

impl<E: HttpEngine> std::fmt::Debug for HttpTransportBuilder<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransportBuilder")
            .field(
                "engine",
                &format!("HttpEngine({})", self.engine.engine_type()),
            )
            .finish()
    }
}

impl<E: HttpEngine> HttpTransportBuilder<E> {
    /// Create a new HTTP transport builder with the given engine
    ///
    /// # Arguments
    ///
    /// * `engine` - HTTP engine implementation
    ///
    /// # Returns
    ///
    /// New HttpTransportBuilder instance
    pub fn new(engine: E) -> Self {
        Self { engine }
    }

    /// Build the transport with the configured engine
    ///
    /// This creates a fully configured HttpTransport that wraps the engine
    /// and implements the Transport trait for McpServer compatibility.
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransport<E>)` - Successfully created transport
    /// * `Err(TransportError)` - Failed to create transport
    pub async fn build(self) -> Result<HttpTransport<E>, TransportError> {
        Ok(HttpTransport::new(self.engine))
    }

    /// Access the underlying engine for configuration
    ///
    /// This method provides access to the engine for configuration
    /// before building the transport.
    ///
    /// # Returns
    ///
    /// Reference to the underlying HTTP engine
    pub fn engine(&self) -> &E {
        &self.engine
    }

    /// Access the underlying engine mutably for configuration
    ///
    /// This method provides mutable access to the engine for configuration
    /// before building the transport.
    ///
    /// # Returns
    ///
    /// Mutable reference to the underlying HTTP engine
    pub fn engine_mut(&mut self) -> &mut E {
        &mut self.engine
    }

    /// Configure the engine with a closure
    ///
    /// This method allows for fluent configuration of the underlying engine
    /// using a closure, enabling convenient method chaining.
    ///
    /// # Arguments
    ///
    /// * `config_fn` - Closure that configures the engine
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn configure_engine<F>(mut self, config_fn: F) -> Self
    where
        F: FnOnce(&mut E),
    {
        config_fn(&mut self.engine);
        self
    }

    /// Bind the HTTP engine to a specific address
    ///
    /// This method provides a convenient way to bind the engine to an address
    /// as part of the builder pattern.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully bound, ready for building
    /// * `Err(TransportError)` - Failed to bind to address
    pub async fn bind(mut self, addr: std::net::SocketAddr) -> Result<Self, TransportError> {
        self.engine.bind(addr).await.map_err(TransportError::from)?;
        Ok(self)
    }
}

// ================================================================================================
// Factory Methods (Phase 5 - Authentication Integration)
// ================================================================================================

impl HttpTransportBuilder<()> {
    /// Create a builder with a placeholder engine for testing and development
    ///
    /// This method creates a builder with a minimal placeholder engine implementation
    /// that satisfies the HttpEngine trait for testing and basic development.
    /// For production use, see the factory methods below.
    ///
    /// # Note
    ///
    /// This placeholder engine provides minimal functionality and should not be used
    /// in production. It exists to support the zero-dyn architecture during Phase 4
    /// development and testing.
    ///
    /// # Returns
    ///
    /// HttpTransportBuilder with placeholder engine
    pub fn with_placeholder_engine() -> HttpTransportBuilder<()> {
        HttpTransportBuilder::new(())
    }

    /// Create a builder with a default HTTP engine (legacy alias)
    ///
    /// This method is an alias for `with_placeholder_engine()` to maintain
    /// backward compatibility during the Phase 4 to Phase 5 transition.
    ///
    /// # Deprecated
    ///
    /// Use `with_placeholder_engine()` for clarity, or wait for Phase 5
    /// factory methods like `with_axum_engine()`.
    ///
    /// # Returns
    ///
    /// HttpTransportBuilder with placeholder engine
    #[deprecated(
        since = "phase-4",
        note = "Use with_placeholder_engine() or wait for Phase 5 factory methods"
    )]
    pub fn with_default_engine() -> HttpTransportBuilder<()> {
        Self::with_placeholder_engine()
    }
}

// ================================================================================================
// Phase 5 Factory Methods (To Be Implemented)
// ================================================================================================

// TODO(PHASE-5): Factory methods for concrete engine types
//
// These factory methods will be implemented in Phase 5 to provide convenient
// constructors for common HTTP engine configurations:
//
// impl HttpTransportBuilder<AxumHttpServer<NoAuth>> {
//     /// Create a builder with an Axum HTTP engine (no authentication)
//     pub async fn with_axum_engine(
//         connection_manager: Arc<HttpConnectionManager>,
//         config: HttpTransportConfig,
//     ) -> Result<HttpTransportBuilder<AxumHttpServer<NoAuth>>, TransportError> {
//         let engine = AxumHttpServer::new(connection_manager, config).await?;
//         Ok(HttpTransportBuilder::new(engine))
//     }
// }
//
// impl HttpTransportBuilder<AxumHttpServer<OAuth2StrategyAdapter<JwtValidator, ScopeValidator>, ScopePolicy<ScopeContext>, ScopeContext>> {
//     /// Create a builder with an Axum HTTP engine configured for OAuth2
//     pub async fn with_oauth2_engine(
//         connection_manager: Arc<HttpConnectionManager>,
//         config: HttpTransportConfig,
//         oauth2_adapter: OAuth2StrategyAdapter<JwtValidator, ScopeValidator>,
//         auth_config: OAuth2AuthConfig,
//     ) -> Result<Self, TransportError> {
//         let engine = AxumHttpServer::new(connection_manager, config).await?
//             .with_oauth2_authorization(oauth2_adapter, auth_config);
//         Ok(HttpTransportBuilder::new(engine))
//     }
// }
//
// impl HttpTransportBuilder<AxumHttpServer<ApiKeyStrategyAdapter<InMemoryApiKeyValidator>, ApiKeyPolicy<ApiKeyContext>, ApiKeyContext>> {
//     /// Create a builder with an Axum HTTP engine configured for API key authentication
//     pub async fn with_apikey_engine(
//         connection_manager: Arc<HttpConnectionManager>,
//         config: HttpTransportConfig,
//         apikey_adapter: ApiKeyStrategyAdapter<InMemoryApiKeyValidator>,
//         auth_config: ApiKeyAuthConfig,
//     ) -> Result<Self, TransportError> {
//         let engine = AxumHttpServer::new(connection_manager, config).await?
//             .with_apikey_authorization(apikey_adapter, auth_config);
//         Ok(HttpTransportBuilder::new(engine))
//     }
// }
//
// impl HttpTransportBuilder<AxumHttpServer<CustomAuth, CustomPolicy, CustomContext>> {
//     /// Create a builder with a custom authentication engine
//     pub async fn with_custom_auth_engine<A, P, C>(
//         connection_manager: Arc<HttpConnectionManager>,
//         config: HttpTransportConfig,
//         auth_adapter: A,
//         auth_config: AuthConfig,
//     ) -> Result<HttpTransportBuilder<AxumHttpServer<A, P, C>>, TransportError>
//     where
//         A: HttpAuthStrategyAdapter,
//         P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
//         C: AuthzContext + Clone,
//     {
//         let engine = AxumHttpServer::new(connection_manager, config).await?
//             .with_custom_authorization(auth_adapter, auth_config);
//         Ok(HttpTransportBuilder::new(engine))
//     }
// }

// Placeholder implementation for unit type (temporary)
#[async_trait]
impl HttpEngine for () {
    type Error = HttpEngineError;
    type Config = ();
    type Handler = ();

    fn new(_config: Self::Config) -> Result<Self, Self::Error> {
        Ok(())
    }

    async fn bind(&mut self, _addr: std::net::SocketAddr) -> Result<(), HttpEngineError> {
        Ok(())
    }

    async fn start(&mut self) -> Result<(), HttpEngineError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), HttpEngineError> {
        Ok(())
    }

    fn register_mcp_handler(&mut self, _handler: Self::Handler) {}

    fn register_authentication<S, T, D>(
        &mut self,
        _auth_manager: crate::authentication::AuthenticationManager<S, T, D>,
    ) -> Result<(), HttpEngineError>
    where
        S: crate::authentication::AuthenticationStrategy<T, D>,
        T: Send + Sync,
        D: Send + Sync + 'static,
    {
        Ok(())
    }

    fn register_middleware(&mut self, _middleware: Box<dyn super::engine::HttpMiddleware>) {}

    fn is_bound(&self) -> bool {
        true
    }

    fn is_running(&self) -> bool {
        false
    }

    fn local_addr(&self) -> Option<std::net::SocketAddr> {
        Some("127.0.0.1:8080".parse().unwrap())
    }

    fn engine_type(&self) -> &'static str {
        "placeholder"
    }
}

// Placeholder implementation for McpRequestHandler
#[async_trait]
impl McpRequestHandler for () {
    async fn handle_mcp_request(
        &self,
        _session_id: String,
        _request_data: Vec<u8>,
        _response_mode: ResponseMode,
        _auth_context: Option<super::engine::AuthenticationContext>,
    ) -> Result<super::engine::HttpResponse, HttpEngineError> {
        Ok(super::engine::HttpResponse::json(b"{}".to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_transport_generic_builder() {
        // Test the new generic builder pattern
        let transport_result = HttpTransportBuilder::with_placeholder_engine()
            .build()
            .await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_engine_access() {
        // Test engine access methods
        let mut builder = HttpTransportBuilder::with_placeholder_engine();

        // Test engine access
        assert_eq!(builder.engine().engine_type(), "placeholder");

        // Test mutable engine access
        let _engine_mut = builder.engine_mut();

        let transport = builder.build().await.unwrap();
        assert_eq!(transport.engine().engine_type(), "placeholder");
    }

    #[tokio::test]
    async fn test_http_transport_session_management() {
        // Test session ID management
        let mut transport = HttpTransportBuilder::with_placeholder_engine()
            .build()
            .await
            .unwrap();

        assert_eq!(transport.session_id(), None);

        transport.set_session_context(Some("test-session".to_string()));
        assert_eq!(transport.session_id(), Some("test-session".to_string()));

        transport.set_session_context(None);
        assert_eq!(transport.session_id(), None);
    }

    #[tokio::test]
    async fn test_zero_dyn_architecture() {
        // Test that we've achieved zero dynamic dispatch
        // The generic HttpTransport<E> should have no dyn traits

        let transport = HttpTransportBuilder::with_placeholder_engine()
            .build()
            .await
            .unwrap();

        // This compiles without dyn traits, proving zero-cost abstraction
        let _engine_ref: &() = transport.engine();

        // Transport trait implementation works
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_mcp_handler_registration() {
        // Test MCP handler registration with concrete types (no dyn)
        let mut transport = HttpTransportBuilder::with_placeholder_engine()
            .build()
            .await
            .unwrap();

        // Register handler - this uses concrete types, no dynamic dispatch
        transport.register_mcp_handler(());

        // Handler registration should work without errors
        assert_eq!(transport.transport_type(), "http");
    }

    #[tokio::test]
    async fn test_deprecated_with_default_engine() {
        // Test backward compatibility with the deprecated method
        #[allow(deprecated)]
        let transport_result = HttpTransportBuilder::with_default_engine().build().await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "http");
        assert!(!transport.is_connected());
    }
}
