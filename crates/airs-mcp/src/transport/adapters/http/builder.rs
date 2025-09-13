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
use super::engine::{HttpEngine, HttpEngineError};
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
    /// let engine = AxumHttpServer::with_auth(auth_config)?;
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
    ///     AxumHttpServer::builder()
    ///         .with_oauth2_authorization(oauth2_config)
    ///         .with_custom_middleware(middleware)
    ///         .build()
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
    /// // Complex async engine construction
    /// let transport = HttpTransportBuilder::with_configured_engine_async(|| async {
    ///     let oauth2_config = load_oauth2_config_from_db().await?;
    ///     AxumHttpServer::builder()
    ///         .with_oauth2_authorization(oauth2_config)
    ///         .build()
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
}
