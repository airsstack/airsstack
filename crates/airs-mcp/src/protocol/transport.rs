//! MCP Transport Abstractions and Event-Driven Architecture
//!
//! This module provides the event-driven transport abstraction aligned with the
//! official Model Context Protocol (MCP) specification, providing sophisticated
//! message handling that matches the patterns used in official TypeScript and Python SDKs.
//!
//! # Architecture
//!
//! The MCP-compliant transport layer is built around:
//! - **MessageHandler**: Event-driven protocol logic (separation of concerns)  
//! - **Transport**: Event-driven transport interface
//! - **MessageContext**: Session and metadata management
//! - **TransportError**: Comprehensive error handling
//!
//! # Design Philosophy
//!
//! - **Event-Driven**: Uses callbacks instead of blocking receive() operations
//! - **Specification-Aligned**: Matches official MCP SDK patterns exactly
//! - **Clean Separation**: Transport handles delivery, MessageHandler handles protocol
//! - **Natural Correlation**: Uses JSON-RPC message IDs, no artificial mechanisms
//! - **Session-Aware**: Supports multi-session transports like HTTP
//!
//! # Examples
//!
//! ## Generic MessageHandler Pattern
//!
//! ```rust
//! use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError};
//! use std::sync::Arc;
//! use async_trait::async_trait;
//!
//! // Example with unit context (for STDIO transport)
//! struct EchoHandler;
//!
//! #[async_trait]
//! impl MessageHandler<()> for EchoHandler {
//!     async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
//!         println!("Received message: {:?}", message);
//!     }
//!     
//!     async fn handle_error(&self, error: TransportError) {
//!         eprintln!("Transport error: {:?}", error);
//!     }
//!     
//!     async fn handle_close(&self) {
//!         println!("Transport closed");
//!     }
//! }
//! ```

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

// Layer 3: Internal module imports
use super::message::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};
use super::types::{ProtocolVersion, ServerCapabilities, ServerConfig, ServerInfo};

/// Transport error types for comprehensive error handling
///
/// This enum covers all possible transport-level errors that can occur
/// during MCP message handling, providing specific error types for
/// different failure scenarios.
#[derive(Error, Debug)]
pub enum TransportError {
    /// Connection-related errors
    #[error("Connection error: {message}")]
    Connection { message: String },

    /// I/O operation errors
    #[error("I/O error: {source}")]
    Io { source: std::io::Error },

    /// Message serialization/deserialization errors
    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },

    /// Protocol-level errors
    #[error("Protocol error: {message}")]
    Protocol { message: String },

    /// Timeout errors
    #[error("Timeout error: {message}")]
    Timeout { message: String },

    /// Authentication/authorization errors
    #[error("Authentication error: {message}")]
    Auth { message: String },

    /// Request-specific timeout errors (client operation)
    #[error("Request timeout after {duration:?}")]
    RequestTimeout { duration: Duration },

    /// Response parsing errors (client receiving invalid response)
    #[error("Invalid response format: {message}")]
    InvalidResponse { message: String },

    /// Client not ready for requests (not connected, initializing, etc.)
    #[error("Client not ready: {reason}")]
    NotReady { reason: String },

    /// Generic transport errors
    #[error("Transport error: {message}")]
    Other { message: String },
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::Io { source: error }
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(error: serde_json::Error) -> Self {
        TransportError::Serialization { source: error }
    }
}

impl TransportError {
    /// Create a request timeout error with the specified duration
    ///
    /// This convenience constructor is useful for client implementations
    /// when a request times out after waiting for the specified duration.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration after which the request timed out
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::TransportError;
    /// use std::time::Duration;
    ///
    /// let timeout_error = TransportError::request_timeout(Duration::from_secs(30));
    /// ```
    pub fn request_timeout(duration: Duration) -> Self {
        Self::RequestTimeout { duration }
    }

    /// Create an invalid response error with the specified message
    ///
    /// This convenience constructor is useful for client implementations
    /// when they receive a response that cannot be parsed or is malformed.
    ///
    /// # Arguments
    ///
    /// * `message` - Description of what made the response invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::TransportError;
    ///
    /// let invalid_response = TransportError::invalid_response("Missing required 'result' field");
    /// ```
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse {
            message: message.into(),
        }
    }

    /// Create a not ready error with the specified reason
    ///
    /// This convenience constructor is useful for client implementations
    /// when they cannot process requests due to not being ready.
    ///
    /// # Arguments
    ///
    /// * `reason` - Description of why the client is not ready
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::TransportError;
    ///
    /// let not_ready = TransportError::not_ready("Transport not connected");
    /// ```
    pub fn not_ready(reason: impl Into<String>) -> Self {
        Self::NotReady {
            reason: reason.into(),
        }
    }
}

/// Message context for session and metadata management
///
/// This structure carries session information and metadata for each message,
/// enabling proper handling of multi-session transports like HTTP.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::MessageContext;
/// use chrono::Utc;
///
/// // Default generic context (for STDIO)
/// let context = MessageContext::<()>::new("session-123".to_string())
///     .with_remote_addr("192.168.1.100:8080".to_string())
///     .with_user_agent("airs-mcp-client/1.0".to_string());
///
/// assert_eq!(context.session_id(), Some("session-123"));
/// assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
/// ```
#[derive(Debug, Clone)]
pub struct MessageContext<T = ()> {
    /// Session identifier (if applicable)
    session_id: Option<String>,

    /// Timestamp when message was received
    timestamp: DateTime<Utc>,

    /// Remote address/endpoint information
    remote_addr: Option<String>,

    /// Additional metadata
    metadata: HashMap<String, String>,

    /// Transport-specific data (generic for different transport types)
    transport_data: Option<T>,
}

impl<T> MessageContext<T> {
    /// Create a new message context with transport-specific data
    pub fn new_with_transport_data(session_id: impl Into<String>, transport_data: T) -> Self {
        Self {
            session_id: Some(session_id.into()),
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
            transport_data: Some(transport_data),
        }
    }

    /// Create a new message context without transport data (for simple transports)
    pub fn new(session_id: impl Into<String>) -> Self
    where
        T: Default,
    {
        Self {
            session_id: Some(session_id.into()),
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
            transport_data: None,
        }
    }

    /// Create a new message context without session ID or transport data
    pub fn without_session() -> Self
    where
        T: Default,
    {
        Self {
            session_id: None,
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
            transport_data: None,
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get message timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get remote address
    pub fn remote_addr(&self) -> Option<&str> {
        self.remote_addr.as_deref()
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Set remote address
    pub fn with_remote_addr(mut self, addr: String) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Convenience method to add user agent
    pub fn with_user_agent(self, user_agent: String) -> Self {
        self.with_metadata("user-agent".to_string(), user_agent)
    }

    /// Convenience method to add content type
    pub fn with_content_type(self, content_type: String) -> Self {
        self.with_metadata("content-type".to_string(), content_type)
    }

    /// Get transport-specific data
    ///
    /// Returns a reference to the transport-specific data if it exists.
    /// This allows handlers to access transport-specific context information.
    pub fn transport_data(&self) -> Option<&T> {
        self.transport_data.as_ref()
    }

    /// Set transport-specific data
    ///
    /// Adds or updates transport-specific data for this context.
    pub fn with_transport_data(mut self, data: T) -> Self {
        self.transport_data = Some(data);
        self
    }

    /// Check if transport data is available
    ///
    /// Returns true if this context contains transport-specific data.
    pub fn has_transport_data(&self) -> bool {
        self.transport_data.is_some()
    }
}

/// Event-driven message handler trait
///
/// This trait defines the interface for handling MCP protocol logic,
/// providing clean separation between transport (message delivery) and
/// protocol (MCP semantics) concerns.
///
/// The event-driven design matches the official MCP specification patterns
/// and eliminates the complexity of blocking receive() operations.
///
/// The generic type parameter `T` represents transport-specific context data
/// that can be included with each message (e.g., HTTP request details).
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError};
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait]
/// impl MessageHandler<()> for EchoHandler {
///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
///         println!("Received message: {:?}", message);
///         // Echo logic would go here
///     }
///
///     async fn handle_error(&self, error: TransportError) {
///         eprintln!("Transport error: {}", error);
///     }
///
///     async fn handle_close(&self) {
///         println!("Transport closed");
///     }
/// }
/// ```
#[async_trait]
pub trait MessageHandler<T = ()>: Send + Sync {
    /// Handle an incoming JSON-RPC message
    ///
    /// This method is called for every message received by the transport,
    /// including requests, responses, and notifications.
    ///
    /// # Arguments
    ///
    /// * `message` - The JSON-RPC message received
    /// * `context` - Session and metadata information with transport-specific data
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<T>);

    /// Handle a transport-level error
    ///
    /// This method is called when the transport encounters an error that
    /// doesn't result in a valid JSON-RPC message (e.g., connection failures).
    ///
    /// # Arguments
    ///
    /// * `error` - The transport error that occurred
    async fn handle_error(&self, error: TransportError);

    /// Handle transport closure
    ///
    /// This method is called when the transport is closed, either gracefully
    /// or due to an error. It provides an opportunity for cleanup.
    async fn handle_close(&self);
}

/// MCP-compliant transport trait
///
/// This trait defines the event-driven transport interface aligned with the
/// official MCP specification. It replaces the blocking receive() pattern
/// with event-driven message handling via MessageHandler callbacks.
///
/// # Design Principles
///
/// - **Event-Driven**: Uses MessageHandler callbacks instead of blocking receive()
/// - **Session-Aware**: Supports multi-session transports (e.g., HTTP)
/// - **Lifecycle Management**: Explicit start/close for resource management
/// - **Natural Correlation**: Uses JSON-RPC message IDs, no artificial mechanisms
/// - **Transport Agnostic**: Works with STDIO, HTTP, WebSocket, etc.
///
/// # Examples
///
/// See specific transport implementations:
/// - `transport::adapters::stdio::StdioTransport` for STDIO communication
/// - `transport::adapters::http::HttpTransport` for HTTP-based communication
///
/// ```rust
/// use airs_mcp::protocol::{Transport, JsonRpcMessage};
/// use async_trait::async_trait;
///
/// // Transport implementations provide event-driven message handling
/// // via the MessageHandler pattern - see transport adapters for examples
/// ```
#[async_trait]
pub trait Transport: Send + Sync {
    /// Transport-specific error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Start the transport and begin listening for messages
    ///
    /// This method initializes the transport and begins accepting incoming
    /// messages. For connection-based transports, this establishes the connection.
    /// For server transports, this starts listening for connections.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport started successfully
    /// * `Err(Self::Error)` - Failed to start transport
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Close the transport and clean up resources
    ///
    /// This method gracefully shuts down the transport, closes connections,
    /// and releases resources. It should be idempotent.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport closed successfully
    /// * `Err(Self::Error)` - Error during closure (resources may still be cleaned up)
    async fn close(&mut self) -> Result<(), Self::Error>;

    /// Send a JSON-RPC message through the transport
    ///
    /// This method sends a message through the transport. For connection-based
    /// transports, this sends over the active connection. For HTTP transports,
    /// this may initiate a new request/response cycle.
    ///
    /// # Arguments
    ///
    /// * `message` - JSON-RPC message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(Self::Error)` - Failed to send message
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error>;

    /// Get the current session ID (if applicable)
    ///
    /// For session-based transports, this returns the current session identifier.
    /// For single-connection transports like STDIO, this may return None.
    ///
    /// # Returns
    ///
    /// * `Some(String)` - Current session ID
    /// * `None` - No session or single-connection transport
    fn session_id(&self) -> Option<String>;

    /// Set session context for the transport
    ///
    /// For session-based transports, this sets the current session context.
    /// This method allows the transport to track which session is being used.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier to set (None to clear)
    fn set_session_context(&mut self, session_id: Option<String>);

    /// Check if the transport is currently connected
    ///
    /// # Returns
    ///
    /// * `true` - Transport is connected and ready to send/receive
    /// * `false` - Transport is disconnected or not ready
    fn is_connected(&self) -> bool;

    /// Get the transport type identifier
    ///
    /// This returns a string identifying the transport type (e.g., "stdio", "http", "websocket").
    /// Useful for logging and debugging.
    ///
    /// # Returns
    ///
    /// Static string identifying the transport type
    fn transport_type(&self) -> &'static str;
}

/// Client-oriented transport interface for request-response communication
///
/// This trait provides a clean, synchronous interface for client-side MCP communication,
/// focusing on the natural request-response pattern that clients expect. Unlike the
/// server-oriented `Transport` trait which uses event-driven `MessageHandler` patterns,
/// `TransportClient` provides direct request-response semantics.
///
/// # Design Philosophy
///
/// - **Request-Response Pattern**: Direct mapping to client mental model
/// - **Synchronous Flow**: No complex correlation mechanisms needed
/// - **Simple Interface**: Single method for core operation
/// - **Transport Agnostic**: Each implementation handles its own details
/// - **Error Clarity**: Clear error types for client scenarios
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{TransportClient, JsonRpcRequest, RequestId};
/// use serde_json::json;
///
/// async fn example_usage<T: TransportClient>(mut client: T) -> Result<(), Box<dyn std::error::Error>> {
///     // Create a request
///     let request = JsonRpcRequest::new(
///         "initialize",
///         Some(json!({"capabilities": {}})),
///         RequestId::new_string("init-1")
///     );
///     
///     // Send request and receive response directly
///     let response = client.call(request).await?;
///     
///     println!("Received response: {:?}", response);
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait TransportClient: Send + Sync {
    /// Transport-specific error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send a JSON-RPC request and receive the response
    ///
    /// This is the core method of the client interface, providing direct
    /// request-response semantics. The implementation handles all transport-specific
    /// details including connection management, serialization, and correlation.
    ///
    /// # Arguments
    ///
    /// * `request` - The JSON-RPC request to send
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcResponse)` - The response from the server
    /// * `Err(Self::Error)` - Transport or protocol error
    ///
    /// # Error Handling
    ///
    /// Implementations should map transport-specific errors to appropriate
    /// error types, providing clear context for debugging and error handling.
    async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error>;

    /// Check if the transport client is ready to send requests
    ///
    /// This method allows callers to verify that the transport is in a state
    /// where requests can be sent successfully.
    ///
    /// # Returns
    ///
    /// * `true` - Client is ready to send requests
    /// * `false` - Client is not ready (not connected, initializing, etc.)
    fn is_ready(&self) -> bool;

    /// Get the transport type identifier
    ///
    /// Returns a string identifying the transport type for logging and debugging.
    ///
    /// # Returns
    ///
    /// Static string identifying the transport type (e.g., "stdio", "http")
    fn transport_type(&self) -> &'static str;

    /// Close the client transport and clean up resources
    ///
    /// This method gracefully shuts down the client transport and releases
    /// any resources. It should be idempotent and safe to call multiple times.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport closed successfully
    /// * `Err(Self::Error)` - Error during closure (resources may still be cleaned up)
    async fn close(&mut self) -> Result<(), Self::Error>;
}

/// Type alias for boxed transport clients with standardized error handling
///
/// This type alias provides a convenient way to work with transport clients
/// when you need trait objects, using the standard `TransportError` type.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{BoxedTransportClient, TransportError};
///
/// async fn use_any_client(mut client: BoxedTransportClient) -> Result<(), TransportError> {
///     // Work with any transport client implementation
///     // ...
///     Ok(())
/// }
/// ```
pub type BoxedTransportClient = Box<dyn TransportClient<Error = TransportError>>;

/// Result type for transport client operations
///
/// This type alias provides a convenient shorthand for transport client results
/// using the standard `TransportError` type.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{TransportClientResult, JsonRpcResponse};
///
/// fn process_response(response: TransportClientResult<JsonRpcResponse>) {
///     match response {
///         Ok(resp) => println!("Success: {:?}", resp),
///         Err(err) => eprintln!("Error: {}", err),
///     }
/// }
/// ```
pub type TransportClientResult<T> = Result<T, TransportError>;

/// Transport configuration trait for type-safe transport settings
///
/// This trait provides a standardized interface for transport-specific configuration
/// management while maintaining access to universal MCP core requirements.
///
/// Reference: ADR-011 Transport Configuration Separation
pub trait TransportConfig: Send + Sync {
    /// Set or update the MCP server configuration
    ///
    /// This method allows updating the core MCP requirements (server info,
    /// capabilities, protocol version) that are common across all transports.
    fn set_server_config(&mut self, server_config: ServerConfig);

    /// Get reference to the MCP server configuration
    ///
    /// Returns None if no server configuration has been set.
    fn server_config(&self) -> Option<&ServerConfig>;

    /// Get effective MCP capabilities for this transport
    ///
    /// This method combines the base capabilities from the server config
    /// with any transport-specific capability modifications.
    fn effective_capabilities(&self) -> ServerCapabilities;

    /// Get server info from server config
    ///
    /// Convenience method that extracts server info from the server config.
    fn server_info(&self) -> Option<&ServerInfo> {
        self.server_config().map(|c| &c.server_info)
    }

    /// Get protocol version from server config
    ///
    /// Convenience method that extracts protocol version from the server config.
    fn protocol_version(&self) -> Option<&ProtocolVersion> {
        self.server_config().map(|c| &c.protocol_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::{JsonRpcRequest, JsonRpcResponse, RequestId};
    use serde_json::json;

    /// Mock TransportClient implementation for testing
    ///
    /// This simple mock demonstrates that the TransportClient trait
    /// provides a clean, implementable interface for client operations.
    struct MockTransportClient {
        ready: bool,
        should_fail: bool,
    }

    impl MockTransportClient {
        fn new() -> Self {
            Self {
                ready: true,
                should_fail: false,
            }
        }

        fn with_failure() -> Self {
            Self {
                ready: true,
                should_fail: true,
            }
        }

        fn not_ready() -> Self {
            Self {
                ready: false,
                should_fail: false,
            }
        }
    }

    #[async_trait]
    impl TransportClient for MockTransportClient {
        type Error = TransportError;

        async fn call(
            &mut self,
            request: JsonRpcRequest,
        ) -> Result<JsonRpcResponse, TransportError> {
            if !self.ready {
                return Err(TransportError::not_ready("Mock client not ready"));
            }

            if self.should_fail {
                return Err(TransportError::request_timeout(Duration::from_secs(30)));
            }

            // Mock successful response
            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(request.id),
                result: Some(json!({"status": "success", "method": request.method})),
                error: None,
            })
        }

        fn is_ready(&self) -> bool {
            self.ready
        }

        fn transport_type(&self) -> &'static str {
            "mock"
        }

        async fn close(&mut self) -> Result<(), TransportError> {
            self.ready = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_transport_client_basic_call() {
        let mut client = MockTransportClient::new();

        assert!(client.is_ready());
        assert_eq!(client.transport_type(), "mock");

        let request = JsonRpcRequest::new(
            "test_method",
            Some(json!({"param": "value"})),
            RequestId::new_string("test-1"),
        );

        let response = client.call(request.clone()).await.unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(request.id));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_transport_client_not_ready() {
        let mut client = MockTransportClient::not_ready();

        assert!(!client.is_ready());

        let request = JsonRpcRequest::new("test_method", None, RequestId::new_string("test-2"));

        let result = client.call(request).await;
        assert!(result.is_err());

        if let Err(TransportError::NotReady { reason }) = result {
            assert_eq!(reason, "Mock client not ready");
        } else {
            panic!("Expected NotReady error");
        }
    }

    #[tokio::test]
    async fn test_transport_client_timeout() {
        let mut client = MockTransportClient::with_failure();

        let request = JsonRpcRequest::new("test_method", None, RequestId::new_string("test-3"));

        let result = client.call(request).await;
        assert!(result.is_err());

        if let Err(TransportError::RequestTimeout { duration }) = result {
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!("Expected RequestTimeout error");
        }
    }

    #[tokio::test]
    async fn test_transport_client_close() {
        let mut client = MockTransportClient::new();

        assert!(client.is_ready());

        client.close().await.unwrap();

        assert!(!client.is_ready());
    }

    #[test]
    fn test_convenience_error_constructors() {
        let timeout_error = TransportError::request_timeout(Duration::from_secs(10));
        if let TransportError::RequestTimeout { duration } = timeout_error {
            assert_eq!(duration, Duration::from_secs(10));
        } else {
            panic!("Expected RequestTimeout error");
        }

        let invalid_response = TransportError::invalid_response("Bad JSON");
        if let TransportError::InvalidResponse { message } = invalid_response {
            assert_eq!(message, "Bad JSON");
        } else {
            panic!("Expected InvalidResponse error");
        }

        let not_ready = TransportError::not_ready("Connecting");
        if let TransportError::NotReady { reason } = not_ready {
            assert_eq!(reason, "Connecting");
        } else {
            panic!("Expected NotReady error");
        }
    }
}
