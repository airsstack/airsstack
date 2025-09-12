//! HTTP Engine Abstraction
//!
//! This module provides a pluggable HTTP engine architecture that allows different
//! HTTP frameworks (Axum, Rocket, Warp, etc.) to be used with the same Transport
//! interface while maintaining MCP protocol compliance.
//!
//! # Design Philosophy
//!
//! **Separation of Concerns**: HTTP framework specifics are separated from MCP
//! transport logic through clean abstraction layers.
//!
//! **Framework Choice**: Teams can use their preferred HTTP framework based on
//! performance, familiarity, or ecosystem requirements.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    MCP Protocol Layer                      │
//! │             (McpServer, MessageHandler)                    │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │ JsonRpcMessage, MessageContext
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │                 Transport Interface                        │
//! │              (HttpServerTransport<E>)                      │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │ HttpEngine trait
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │               HTTP Engine Layer                            │
//! │      (AxumHttpEngine, Future: RocketHttpEngine, etc.)     │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │ Framework-specific implementation
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │              HTTP Framework                                │
//! │              (Axum, Future: Rocket, Warp)                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```

// Layer 1: Standard library imports
use std::net::SocketAddr;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::authentication::{AuthenticationManager, AuthenticationStrategy};

/// Error type for HTTP engine operations
#[derive(Debug, thiserror::Error)]
pub enum HttpEngineError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Engine is not bound to an address
    #[error("Engine not bound to address")]
    NotBound,

    /// Engine is already bound
    #[error("Engine already bound to address: {addr}")]
    AlreadyBound { addr: SocketAddr },

    /// Engine is already running
    #[error("Engine already running")]
    AlreadyRunning,

    /// Authentication configuration error
    #[error("Authentication error: {message}")]
    Authentication { message: String },

    /// Framework-specific error
    #[error("Engine error: {message}")]
    Engine { message: String },
}

/// Response mode for HTTP responses
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseMode {
    /// Standard JSON response
    Json,
    /// Server-Sent Events streaming response
    ServerSentEvents,
    /// Custom streaming response
    Streaming,
}

/// HTTP response data
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Response body as bytes
    pub body: Vec<u8>,
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// Response mode
    pub mode: ResponseMode,
}

impl HttpResponse {
    /// Create a new JSON response
    pub fn json(body: Vec<u8>) -> Self {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Self {
            body,
            status: 200,
            headers,
            mode: ResponseMode::Json,
        }
    }

    /// Create a Server-Sent Events response
    pub fn sse(body: Vec<u8>) -> Self {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "text/event-stream".to_string());
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("connection".to_string(), "keep-alive".to_string());

        Self {
            body,
            status: 200,
            headers,
            mode: ResponseMode::ServerSentEvents,
        }
    }

    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        let body = format!(r#"{{"error": "{message}"}}"#).into_bytes();
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Self {
            body,
            status,
            headers,
            mode: ResponseMode::Json,
        }
    }
}

/// MCP request handler interface for HTTP engines
#[async_trait]
pub trait McpRequestHandler: Send + Sync {
    /// Handle an MCP request and return an HTTP response
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique session identifier for this request
    /// * `request_data` - Raw request body as bytes
    /// * `response_mode` - Expected response format (JSON, SSE, Streaming)
    /// * `auth_context` - Authentication context (if available)
    ///
    /// # Returns
    ///
    /// * `Ok(HttpResponse)` - Successful response
    /// * `Err(HttpEngineError)` - Request handling error
    async fn handle_mcp_request(
        &self,
        session_id: String,
        request_data: Vec<u8>,
        response_mode: ResponseMode,
        auth_context: Option<AuthenticationContext>,
    ) -> Result<HttpResponse, HttpEngineError>;
}

/// Placeholder authentication context trait
///
/// This will be replaced with the actual authentication implementation
/// in Phase 5 (Multi-Method Authentication Enhancement)
#[derive(Debug, Clone)]
pub struct AuthenticationContext {
    /// Authentication method used
    pub method: String,
    /// User or client identifier
    pub identity: String,
    /// Additional authentication metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// HTTP middleware trait for extensible middleware support
pub trait HttpMiddleware: Send + Sync {
    /// Get the middleware name for debugging
    fn name(&self) -> &'static str;
}

/// Core HTTP engine trait for pluggable HTTP frameworks
#[async_trait]
pub trait HttpEngine: Send + Sync {
    /// Engine-specific error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Engine configuration type
    type Config: Clone + Send + Sync;

    /// MCP request handler type - eliminates dynamic dispatch
    type Handler: McpRequestHandler + Send + Sync + 'static;

    /// Create a new HTTP engine with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Engine-specific configuration
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created engine
    /// * `Err(Self::Error)` - Configuration or initialization error
    fn new(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Bind the engine to a network address
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully bound
    /// * `Err(HttpEngineError)` - Binding failed
    async fn bind(&mut self, addr: SocketAddr) -> Result<(), HttpEngineError>;

    /// Start the HTTP server
    ///
    /// This begins accepting HTTP requests and routing them through
    /// the registered MCP handler.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server started successfully
    /// * `Err(HttpEngineError)` - Failed to start server
    async fn start(&mut self) -> Result<(), HttpEngineError>;

    /// Gracefully shutdown the HTTP server
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server shut down successfully
    /// * `Err(HttpEngineError)` - Shutdown failed
    async fn shutdown(&mut self) -> Result<(), HttpEngineError>;

    /// Register the MCP request handler
    ///
    /// # Arguments
    ///
    /// * `handler` - MCP request handler implementation (concrete type, no dynamic dispatch)
    fn register_mcp_handler(&mut self, handler: Self::Handler);

    /// Register authentication manager
    ///
    /// This method allows registering an authentication manager with the HTTP engine.
    /// The manager handles authentication for all incoming requests.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Authentication strategy type
    /// * `T` - Request type (typically HttpAuthRequest)
    /// * `D` - Authentication data type
    ///
    /// # Arguments
    ///
    /// * `auth_manager` - Authentication manager with configured strategy
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Authentication manager registered
    /// * `Err(HttpEngineError)` - Registration failed
    fn register_authentication<S, T, D>(
        &mut self,
        auth_manager: AuthenticationManager<S, T, D>,
    ) -> Result<(), HttpEngineError>
    where
        S: AuthenticationStrategy<T, D>,
        T: Send + Sync,
        D: Send + Sync + 'static;

    /// Register custom HTTP middleware
    ///
    /// # Arguments
    ///
    /// * `middleware` - Custom middleware implementation
    fn register_middleware(&mut self, middleware: Box<dyn HttpMiddleware>);

    /// Check if the engine is bound to an address
    fn is_bound(&self) -> bool;

    /// Check if the engine is currently running
    fn is_running(&self) -> bool;

    /// Get the local address the engine is bound to
    fn local_addr(&self) -> Option<SocketAddr>;

    /// Get the engine type identifier
    fn engine_type(&self) -> &'static str;
}
