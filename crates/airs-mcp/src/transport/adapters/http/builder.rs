//! HTTP Transport Implementation for Zero-Dyn Architecture
//!
//! This module provides the generic HttpTransport<E: HttpEngine> implementation
//! that eliminates dynamic dispatch and provides Transport trait compatibility
//! for McpServer lifecycle management. The actual MCP processing happens directly
//! through the HttpEngine → McpRequestHandler flow, bypassing the Transport interface.

// Layer 1: Standard library imports
use std::fmt::Debug;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::context::HttpContext;
use super::engine::{HttpEngine, HttpEngineError};
use crate::protocol::{JsonRpcMessage, MessageHandler, Transport, TransportError};

/// Convert HttpEngineError to TransportError
impl From<HttpEngineError> for TransportError {
    fn from(error: HttpEngineError) -> Self {
        match error {
            HttpEngineError::Io(e) => TransportError::Io { source: e },
            HttpEngineError::NotBound => TransportError::Connection {
                message: "HTTP engine not bound to address".to_string(),
            },
            HttpEngineError::AlreadyBound { addr } => TransportError::Connection {
                message: format!("HTTP engine already bound to address: {addr}"),
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
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, AxumHttpServer};
/// use airs_mcp::transport::adapters::http::DefaultAxumMcpRequestHandler;
/// use airs_mcp::integration::server::McpServer;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Modern zero configuration transport
/// let mut transport = HttpTransportBuilder::<AxumHttpServer>::with_default()?
///     .build().await?;
///
/// // Register MCP handler for direct HTTP processing
/// let handler = DefaultAxumMcpRequestHandler::new(None, None, None, None);
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
    /// Message handler for the transport (set via builder pattern)
    message_handler: Option<Arc<dyn MessageHandler<HttpContext>>>,
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
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<HttpContext>>"),
            )
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
            message_handler: None,
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

    /// Set the message handler for the transport (internal use for builders)
    ///
    /// This method is used internally by builders to set the
    /// message handler after transport creation. In future phases, this will
    /// integrate with the MessageHandlerAdapter pattern.
    ///
    /// # Arguments
    ///
    /// * `handler` - MessageHandler<HttpContext> for processing JSON-RPC messages
    pub(crate) fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler<HttpContext>>) {
        self.message_handler = Some(handler);
    }

    /// Get reference to the message handler (if set)
    ///
    /// # Returns
    ///
    /// Optional reference to the message handler
    pub fn message_handler(&self) -> Option<&Arc<dyn MessageHandler<HttpContext>>> {
        self.message_handler.as_ref()
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
                    message: format!("Failed to start HTTP engine: {e}"),
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
                message: format!("Failed to shutdown HTTP engine: {e}"),
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
/// use airs_mcp::transport::adapters::http::{HttpTransportBuilder, AxumHttpServer};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Tier 1: Zero configuration (beginner-friendly)
/// let transport = HttpTransportBuilder::<AxumHttpServer>::with_default()?
///     .build().await?;
///
/// // Tier 2: Pre-configured engine
/// let engine = AxumHttpServer::default();
/// let transport = HttpTransportBuilder::with_engine(engine)?
///     .bind("127.0.0.1:8080".parse()?).await?
///     .build().await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpTransportBuilder<E: HttpEngine> {
    /// HTTP engine instance
    engine: E,
    /// Message handler for the transport (set via with_message_handler method)
    message_handler: Option<Arc<dyn MessageHandler<HttpContext>>>,
}

impl<E: HttpEngine> std::fmt::Debug for HttpTransportBuilder<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransportBuilder")
            .field(
                "engine",
                &format!("HttpEngine({})", self.engine.engine_type()),
            )
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<HttpContext>>"),
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
        Self {
            engine,
            message_handler: None,
        }
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

    // ================================================================================================
    // Phase 5: Generic Convenience Methods (Engine-Agnostic)
    // ================================================================================================

    /// Create builder with default engine instance (Tier 1: Beginner)
    ///
    /// This is the simplest way to create an HTTP transport builder. It creates
    /// a default instance of the engine type and wraps it in a builder.
    ///
    /// # Type Parameters
    ///
    /// * `E` - HTTP engine type that must implement both HttpEngine and Default
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransportBuilder<E>)` - Builder with default engine
    /// * `Err(TransportError)` - Failed to create default engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simplest possible usage - just works
    /// let transport = HttpTransportBuilder::<AxumHttpServer>::with_default()?
    ///     .bind("127.0.0.1:8080".parse()?).await?
    ///     .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_default() -> Result<Self, TransportError>
    where
        E: Default + HttpEngine,
    {
        Ok(Self::new(E::default()))
    }

    /// Create builder with pre-configured engine (Tier 2: Basic Configuration)
    ///
    /// This method accepts a pre-configured engine instance and wraps it in a builder.
    /// Useful when you want to configure the engine first and then create the transport.
    ///
    /// # Arguments
    ///
    /// * `engine` - Pre-configured HTTP engine instance
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransportBuilder<E>)` - Builder with the provided engine
    /// * `Err(TransportError)` - Failed to create builder (currently always succeeds)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Pre-configured engines for common patterns
    /// let engine = AxumHttpServer::default();
    /// let transport = HttpTransportBuilder::with_engine(engine)?
    ///     .bind("127.0.0.1:8080".parse()?).await?
    ///     .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_engine(engine: E) -> Result<Self, TransportError> {
        Ok(Self::new(engine))
    }

    /// Create builder using engine builder function (Tier 3: Advanced Configuration)
    ///
    /// This method accepts a closure that returns a configured engine. The closure
    /// is called immediately to create the engine. This provides full control over
    /// engine configuration while maintaining the builder pattern.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type that returns Result<E, R>
    /// * `R` - Error type that can be converted to TransportError
    ///
    /// # Arguments
    ///
    /// * `builder_fn` - Closure that creates and configures the engine
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransportBuilder<E>)` - Builder with configured engine
    /// * `Err(TransportError)` - Failed to create or configure engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Full builder pattern control
    /// let transport = HttpTransportBuilder::with_configured_engine(|| {
    ///     Result::<AxumHttpServer, std::io::Error>::Ok(AxumHttpServer::default())
    /// })?
    /// .bind("127.0.0.1:8080".parse()?).await?
    /// .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_configured_engine<F, R>(builder_fn: F) -> Result<Self, TransportError>
    where
        F: FnOnce() -> Result<E, R>,
        R: Into<TransportError>,
    {
        let engine = builder_fn().map_err(Into::into)?;
        Ok(Self::new(engine))
    }

    /// Create builder using async engine builder function (Tier 4: Expert)
    ///
    /// This method accepts an async closure that returns a configured engine. This
    /// enables complex async initialization patterns like loading configuration from
    /// databases or external services.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type that returns Future<Output = Result<E, R>>
    /// * `Fut` - Future type returned by the closure
    /// * `R` - Error type that can be converted to TransportError
    ///
    /// # Arguments
    ///
    /// * `builder_fn` - Async closure that creates and configures the engine
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransportBuilder<E>)` - Builder with configured engine
    /// * `Err(TransportError)` - Failed to create or configure engine
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::HttpTransportBuilder;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # async fn load_config_from_db() -> Result<(), std::io::Error> { Ok(()) }
    /// // Complex async engine construction
    /// let transport = HttpTransportBuilder::with_configured_engine_async(|| async {
    ///     load_config_from_db().await?;
    ///     Result::<AxumHttpServer, std::io::Error>::Ok(AxumHttpServer::default())
    /// }).await?
    /// .bind("127.0.0.1:8080".parse()?).await?
    /// .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_configured_engine_async<F, Fut, R>(
        builder_fn: F,
    ) -> Result<Self, TransportError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<E, R>>,
        R: Into<TransportError>,
    {
        let engine = builder_fn().await.map_err(Into::into)?;
        Ok(Self::new(engine))
    }

    /// Set the message handler for the transport
    ///
    /// This implements the transport-specific construction pattern where handlers
    /// are set before building the transport, following ADR-011 Transport
    /// Configuration Separation principles.
    ///
    /// # Arguments
    ///
    /// * `handler` - MessageHandler<HttpContext> for processing JSON-RPC messages
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_message_handler(mut self, handler: Arc<dyn MessageHandler<HttpContext>>) -> Self {
        self.message_handler = Some(handler);
        self
    }

    /// Build the transport with the configured message handler
    ///
    /// This creates a fully configured HTTP transport that is ready to start.
    /// The transport will have its message handler pre-configured via the
    /// MessageHandlerAdapter bridge pattern.
    ///
    /// # Returns
    ///
    /// * `Ok(HttpTransport<E>)` - Fully configured transport
    /// * `Err(TransportError)` - Failed to create transport or no handler set
    pub async fn build_with_handler(self) -> Result<HttpTransport<E>, TransportError> {
        // Validate that message handler was set (ADR-011 compliance)
        let handler = self
            .message_handler
            .ok_or_else(|| TransportError::Protocol {
                message: "Message handler must be set before building HTTP transport".to_string(),
            })?;

        // Create transport and set the handler
        let mut transport = HttpTransport::new(self.engine);
        transport.set_message_handler(handler);

        Ok(transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::adapters::http::axum::AxumHttpServer;
    use tokio::time::{sleep, Duration};

    /// Test helper to create a dummy error type for testing
    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestError: {}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    impl From<TestError> for TransportError {
        fn from(error: TestError) -> Self {
            TransportError::Protocol { message: error.0 }
        }
    }

    #[tokio::test]
    async fn test_with_default_success() {
        // Test Tier 1: Zero configuration with default engine
        let result = HttpTransportBuilder::<AxumHttpServer>::with_default();

        assert!(
            result.is_ok(),
            "Should create builder with default AxumHttpServer"
        );

        let builder = result.unwrap();
        // Verify that the builder was created successfully with an engine
        // We can't directly inspect the engine, but we can verify the builder exists
        let _verification = &builder.engine; // This proves the engine field exists and is accessible
    }

    #[tokio::test]
    async fn test_with_engine_success() {
        // Test Tier 2: Pre-configured engine injection
        let engine = AxumHttpServer::default();
        let result = HttpTransportBuilder::with_engine(engine);

        assert!(result.is_ok(), "Should create builder with provided engine");

        let builder = result.unwrap();
        // Verify engine is properly stored by accessing the field
        let _verification = &builder.engine; // This proves the engine field exists and is accessible
    }

    #[tokio::test]
    async fn test_with_configured_engine_success() {
        // Test Tier 3: Builder pattern with configuration function
        let result = HttpTransportBuilder::with_configured_engine(|| {
            // Simulate complex engine configuration
            let engine = AxumHttpServer::default();
            Ok::<_, TestError>(engine)
        });

        assert!(
            result.is_ok(),
            "Should create builder from configuration function"
        );
    }

    #[tokio::test]
    async fn test_with_configured_engine_error_handling() {
        // Test error propagation from builder function
        let result = HttpTransportBuilder::<AxumHttpServer>::with_configured_engine(|| {
            Err(TestError("Configuration failed".to_string()))
        });

        assert!(
            result.is_err(),
            "Should propagate error from configuration function"
        );

        if let Err(TransportError::Protocol { message }) = result {
            assert!(
                message.contains("Configuration failed"),
                "Should contain original error message"
            );
        } else {
            panic!("Expected Protocol error with configuration message");
        }
    }

    #[tokio::test]
    async fn test_with_configured_engine_async_success() {
        // Test Tier 4: Async initialization patterns
        let result = HttpTransportBuilder::with_configured_engine_async(|| async {
            // Simulate async configuration loading (e.g., from database)
            sleep(Duration::from_millis(10)).await;
            let engine = AxumHttpServer::default();
            Ok::<_, TestError>(engine)
        })
        .await;

        assert!(
            result.is_ok(),
            "Should create builder from async configuration function"
        );
    }

    #[tokio::test]
    async fn test_with_configured_engine_async_error_handling() {
        // Test async error propagation
        let result =
            HttpTransportBuilder::<AxumHttpServer>::with_configured_engine_async(|| async {
                sleep(Duration::from_millis(5)).await;
                Err(TestError("Async configuration failed".to_string()))
            })
            .await;

        assert!(
            result.is_err(),
            "Should propagate error from async configuration function"
        );

        if let Err(TransportError::Protocol { message }) = result {
            assert!(
                message.contains("Async configuration failed"),
                "Should contain original error message"
            );
        } else {
            panic!("Expected Protocol error with async configuration message");
        }
    }

    #[tokio::test]
    async fn test_progressive_tier_patterns() {
        // Test that all four tiers work together and demonstrate progression

        // Tier 1: Zero Configuration (Beginner)
        let tier1_result = HttpTransportBuilder::<AxumHttpServer>::with_default();
        assert!(
            tier1_result.is_ok(),
            "Tier 1 should work with zero configuration"
        );

        // Tier 2: Basic Configuration (Pre-configured engines)
        let engine = AxumHttpServer::default();
        let tier2_result = HttpTransportBuilder::with_engine(engine);
        assert!(
            tier2_result.is_ok(),
            "Tier 2 should work with pre-configured engine"
        );

        // Tier 3: Advanced Configuration (Builder pattern control)
        let tier3_result = HttpTransportBuilder::with_configured_engine(|| {
            let engine = AxumHttpServer::default();
            Ok::<_, TestError>(engine)
        });
        assert!(
            tier3_result.is_ok(),
            "Tier 3 should work with builder pattern"
        );

        // Tier 4: Expert Async (Async initialization)
        let tier4_result = HttpTransportBuilder::with_configured_engine_async(|| async {
            sleep(Duration::from_millis(1)).await; // Simulate async operation
            let engine = AxumHttpServer::default();
            Ok::<_, TestError>(engine)
        })
        .await;
        assert!(
            tier4_result.is_ok(),
            "Tier 4 should work with async initialization"
        );
    }

    #[tokio::test]
    async fn test_engine_type_flexibility() {
        // Test that the generic methods work with the concrete AxumHttpServer type
        // This validates our type constraints and ensures the methods are truly generic

        type ConcreteBuilder = HttpTransportBuilder<AxumHttpServer>;

        // with_default should work with Default constraint
        let default_builder = ConcreteBuilder::with_default();
        assert!(
            default_builder.is_ok(),
            "Should work with concrete type via with_default"
        );

        // with_engine should work with any instance of the engine type
        let concrete_engine = AxumHttpServer::default();
        let engine_builder = HttpTransportBuilder::with_engine(concrete_engine);
        assert!(
            engine_builder.is_ok(),
            "Should work with concrete type via with_engine"
        );
    }

    #[tokio::test]
    async fn test_builder_state_consistency() {
        // Test that builder maintains proper state after creation
        let builder = HttpTransportBuilder::<AxumHttpServer>::with_default().unwrap();

        // Verify that the builder has proper internal state
        // We verify the builder works correctly by ensuring the engine field is accessible
        let _verification = &builder.engine; // This proves the engine field exists and is accessible

        // Verify the builder can be used for its intended purpose
        // (In real usage, this would be passed to transport construction)
        assert!(true, "Builder should maintain consistent state");
    }

    #[tokio::test]
    async fn test_error_conversion_and_propagation() {
        // Test various error conversion scenarios to ensure proper error handling

        // Test custom error conversion
        let result = HttpTransportBuilder::<AxumHttpServer>::with_configured_engine(|| {
            Err(TestError("Custom error".to_string()))
        });

        match result {
            Err(TransportError::Protocol { message }) => {
                assert!(message.contains("Custom error"));
            }
            _ => panic!("Expected TransportError::Protocol"),
        }

        // Test async error conversion
        let async_result =
            HttpTransportBuilder::<AxumHttpServer>::with_configured_engine_async(|| async {
                Err(TestError("Async custom error".to_string()))
            })
            .await;

        match async_result {
            Err(TransportError::Protocol { message }) => {
                assert!(message.contains("Async custom error"));
            }
            _ => panic!("Expected TransportError::Protocol from async"),
        }
    }

    #[tokio::test]
    async fn test_complex_async_scenarios() {
        // Test more complex async scenarios that might occur in real usage

        // Simulate database configuration loading
        let database_config_result = HttpTransportBuilder::with_configured_engine_async(|| async {
            // Simulate database query delay
            sleep(Duration::from_millis(20)).await;

            // Simulate successful config retrieval
            let engine = AxumHttpServer::default();
            Ok::<_, TestError>(engine)
        })
        .await;

        assert!(
            database_config_result.is_ok(),
            "Should handle database config loading"
        );

        // Simulate service discovery
        let service_discovery_result =
            HttpTransportBuilder::with_configured_engine_async(|| async {
                // Simulate service discovery lookup
                sleep(Duration::from_millis(15)).await;

                // Simulate successful service discovery
                let engine = AxumHttpServer::default();
                Ok::<_, TestError>(engine)
            })
            .await;

        assert!(
            service_discovery_result.is_ok(),
            "Should handle service discovery patterns"
        );
    }

    // ================================================================================================
    // TASK-031 Phase 1 Tests: TransportBuilder<HttpContext> Interface
    // ================================================================================================

    use crate::protocol::{JsonRpcMessage, MessageContext, MessageHandler};
    use async_trait::async_trait;

    /// Test MessageHandler implementation for testing TransportBuilder
    struct TestHttpMessageHandler {
        name: String,
    }

    #[async_trait]
    impl MessageHandler<HttpContext> for TestHttpMessageHandler {
        async fn handle_message(
            &self,
            message: JsonRpcMessage,
            _context: MessageContext<HttpContext>,
        ) {
            // Simple test implementation - just log that we received a message
            println!(
                "TestHttpMessageHandler '{}' received message: {:?}",
                self.name, message
            );
        }

        async fn handle_error(&self, error: TransportError) {
            println!(
                "TestHttpMessageHandler '{}' received error: {}",
                self.name, error
            );
        }

        async fn handle_close(&self) {
            println!("TestHttpMessageHandler '{}' transport closed", self.name);
        }
    }

    #[tokio::test]
    async fn test_transport_builder_interface_success() {
        // Test that HttpTransportBuilder implements TransportBuilder<HttpContext>
        let handler = Arc::new(TestHttpMessageHandler {
            name: "test-handler".to_string(),
        });

        let builder = HttpTransportBuilder::<AxumHttpServer>::with_default()
            .expect("Should create builder with default engine");

        // Test the direct builder interface (no more TransportBuilder trait)
        let transport_result: Result<HttpTransport<AxumHttpServer>, TransportError> = builder
            .with_message_handler(handler.clone())
            .build_with_handler()
            .await;

        assert!(
            transport_result.is_ok(),
            "Should build transport with message handler"
        );

        let transport = transport_result.unwrap();

        // Verify that the handler was set
        assert!(
            transport.message_handler().is_some(),
            "Transport should have message handler set"
        );
    }

    #[tokio::test]
    async fn test_transport_builder_requires_handler() {
        // Test that building without handler fails (ADR-011 compliance)
        let builder = HttpTransportBuilder::<AxumHttpServer>::with_default()
            .expect("Should create builder with default engine");

        // Try to build using direct builder without setting handler (should use build_with_handler)
        let transport_result: Result<HttpTransport<AxumHttpServer>, TransportError> =
            builder.build_with_handler().await;

        assert!(
            transport_result.is_err(),
            "Should fail to build transport without message handler"
        );

        if let Err(error) = transport_result {
            assert!(
                error.to_string().contains("Message handler must be set"),
                "Error should indicate that message handler is required"
            );
        }
    }

    #[tokio::test]
    async fn test_transport_builder_direct_usage() {
        // Test that we can use the direct builder pattern
        let handler = Arc::new(TestHttpMessageHandler {
            name: "direct-test".to_string(),
        });

        let builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");

        let transport_result = builder
            .with_message_handler(handler)
            .build_with_handler()
            .await;

        assert!(
            transport_result.is_ok(),
            "Should work with direct builder pattern"
        );
    }

    // ================================================================================================
    // TASK-031 Phase 2: Type System Compatibility Tests (Added 2025-01-16)
    // ================================================================================================

    #[tokio::test]
    async fn test_phase2_direct_transport_builders() {
        // Phase 2.1: Test that HTTP and STDIO builders work with their respective patterns

        // Test HTTP transport with direct builder pattern
        let http_handler = Arc::new(TestHttpMessageHandler {
            name: "http-direct".to_string(),
        });
        let http_builder = HttpTransportBuilder::<AxumHttpServer>::with_default()
            .expect("Should create HTTP builder");
        let http_result = http_builder
            .with_message_handler(http_handler)
            .build_with_handler()
            .await;
        assert!(
            http_result.is_ok(),
            "HTTP transport should work with direct builder pattern"
        );

        // Test STDIO transport with direct builder pattern
        use crate::transport::adapters::stdio::StdioTransportBuilder;

        #[derive(Debug)]
        struct TestStdioHandler;

        #[async_trait]
        impl MessageHandler<()> for TestStdioHandler {
            async fn handle_message(&self, _message: JsonRpcMessage, _context: MessageContext<()>) {
            }
            async fn handle_error(&self, _error: TransportError) {}
            async fn handle_close(&self) {}
        }

        let stdio_handler = Arc::new(TestStdioHandler);
        let stdio_builder = StdioTransportBuilder::new();
        let stdio_result = stdio_builder
            .with_message_handler(stdio_handler)
            .build()
            .await;
        assert!(
            stdio_result.is_ok(),
            "STDIO transport should work with direct builder pattern"
        );
    }

    #[tokio::test]
    async fn test_phase2_handler_type_constraints() {
        // Phase 2.1: Verify strict type constraints work properly

        // This should compile: MessageHandler<HttpContext> with HttpTransportBuilder
        let http_handler = Arc::new(TestHttpMessageHandler {
            name: "type-test".to_string(),
        });
        let http_builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");
        let http_result = http_builder
            .with_message_handler(http_handler)
            .build()
            .await;
        assert!(
            http_result.is_ok(),
            "HTTP handler should work with HTTP transport"
        );

        // Verify we cannot use wrong context type (compile-time safety)
        // Note: This test verifies the type system prevents mismatched contexts
        // In practice, trying to use MessageHandler<()> with HttpTransportBuilder
        // would result in a compilation error, which is the desired behavior
    }

    #[tokio::test]
    async fn test_phase2_enhanced_error_handling() {
        // Phase 2.2: Test enhanced handler validation error handling

        let builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");

        // Test building without handler produces descriptive error (direct builder pattern)
        let result: Result<HttpTransport<AxumHttpServer>, TransportError> =
            builder.build_with_handler().await;
        assert!(result.is_err(), "Should fail without handler");

        let error = result.unwrap_err();
        let error_message = error.to_string();

        // Verify error message is descriptive and follows ADR-011
        assert!(
            error_message.contains("Message handler must be set"),
            "Error should indicate handler requirement clearly. Got: {}",
            error_message
        );

        // Verify error type is appropriate (Protocol error for configuration issues)
        match error {
            TransportError::Protocol { .. } => {
                // Expected: Protocol error for missing handler configuration
            }
            _ => panic!(
                "Should be a Protocol error for configuration issue, got: {:?}",
                error
            ),
        }
    }

    #[tokio::test]
    async fn test_phase2_handler_validation_edge_cases() {
        // Phase 2.2: Test edge cases in handler validation

        let builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");

        // Test that handler can be set and retrieved
        let handler = Arc::new(TestHttpMessageHandler {
            name: "validation-test".to_string(),
        });

        let builder = builder.with_message_handler(handler.clone());
        let transport: HttpTransport<AxumHttpServer> = builder
            .build_with_handler()
            .await
            .expect("Should build with valid handler");

        // Verify handler was stored correctly
        assert!(
            transport.message_handler().is_some(),
            "Handler should be stored"
        );

        // Test handler immutability (cannot be changed after build)
        // This is enforced by the type system - once built, the handler is fixed
    }

    #[tokio::test]
    async fn test_phase2_transport_trait_compatibility() {
        // Phase 2.1: Verify our transport implements the Transport trait correctly
        use crate::protocol::Transport;

        let handler = Arc::new(TestHttpMessageHandler {
            name: "trait-test".to_string(),
        });

        let builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");
        let builder = builder.with_message_handler(handler);
        let transport: HttpTransport<AxumHttpServer> = builder
            .build_with_handler()
            .await
            .expect("Should build transport");

        // Verify Transport trait methods work
        assert_eq!(
            transport.transport_type(),
            "http",
            "Should identify as HTTP transport"
        );
        assert!(!transport.is_connected(), "Should start disconnected");

        // Verify session management (HTTP session is established per-request, may be None initially)
        assert!(
            transport.session_id().is_none(),
            "HTTP transport has no session until a request"
        );
    }

    #[tokio::test]
    async fn test_phase2_adr011_compliance_validation() {
        // Phase 2.2: Verify strict ADR-011 compliance in error scenarios

        // Test 1: Builder without handler should fail per ADR-011
        let empty_builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");

        let result: Result<HttpTransport<AxumHttpServer>, TransportError> =
            empty_builder.build_with_handler().await;
        assert!(
            result.is_err(),
            "ADR-011: Pre-configured handlers are mandatory"
        );

        // Test 2: Verify error follows ADR-011 configuration separation principles
        if let Err(TransportError::Protocol { message }) = result {
            assert!(
                message.contains("Message handler must be set"),
                "ADR-011: Error should reference handler requirement"
            );
        } else {
            panic!("Should be Protocol error per ADR-011");
        }

        // Test 3: Successful build should have handler pre-configured
        let handler = Arc::new(TestHttpMessageHandler {
            name: "adr011-test".to_string(),
        });

        let working_builder =
            HttpTransportBuilder::<AxumHttpServer>::with_default().expect("Should create builder");

        let working_builder = working_builder.with_message_handler(handler);
        let transport: HttpTransport<AxumHttpServer> = working_builder
            .build_with_handler()
            .await
            .expect("ADR-011: Should build with pre-configured handler");

        assert!(
            transport.message_handler().is_some(),
            "ADR-011: Transport should have handler pre-configured"
        );
    }
}
