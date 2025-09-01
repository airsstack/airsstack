//! MCP-Compliant Transport Layer
//!
//! This module implements the transport abstraction aligned with the official
//! Model Context Protocol (MCP) specification, providing event-driven message
//! handling that matches the patterns used in official TypeScript and Python SDKs.
//!
//! # Architecture
//!
//! The MCP-compliant transport layer is built around:
//! - **JsonRpcMessage**: Core message type matching MCP specification
//! - **MessageHandler**: Event-driven protocol logic (separation of concerns)
//! - **Transport**: Event-driven transport interface
//! - **MessageContext**: Session and metadata management
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
//! ```rust
//! use airs_mcp::transport::mcp::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
//! use std::sync::Arc;
//! use async_trait::async_trait;
//!
//! # struct MyHandler;
//! # #[async_trait]
//! # impl MessageHandler for MyHandler {
//! #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {}
//! #     async fn handle_error(&self, error: TransportError) {}
//! #     async fn handle_close(&self) {}
//! # }
//! # struct MyTransport;
//! # impl MyTransport {
//! #     fn new() -> Self { Self }
//! # }
//! # #[async_trait]
//! # impl Transport for MyTransport {
//! #     type Error = TransportError;
//! #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn close(&mut self) -> Result<(), Self::Error> { Ok(()) }
//! #     async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> { Ok(()) }
//! #     fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {}
//! #     fn session_id(&self) -> Option<String> { None }
//! #     fn set_session_context(&mut self, session_id: Option<String>) {}
//! #     fn is_connected(&self) -> bool { true }
//! #     fn transport_type(&self) -> &'static str { "test" }
//! # }
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let handler = Arc::new(MyHandler);
//!     let mut transport = MyTransport::new();
//!     
//!     // Set up event-driven message handling
//!     transport.set_message_handler(handler);
//!     
//!     // Start the transport (begins listening for messages)
//!     transport.start().await?;
//!     
//!     // Send a message
//!     let message = JsonRpcMessage::new_notification("ping", None);
//!     transport.send(message).await?;
//!     
//!     // Transport automatically calls handler.handle_message() for incoming messages
//!     // No blocking receive() calls needed
//!     
//!     // Clean up
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::base::jsonrpc::message::JsonRpcMessage as LegacyJsonRpcMessage;

/// Core JSON-RPC message type aligned with MCP specification
///
/// This structure matches the official MCP specification's JSON-RPC message format,
/// supporting all message types: requests, responses, notifications, and errors.
///
/// The design is intentionally flat to match the MCP TypeScript/Python SDK patterns
/// and avoid the complexity of our current trait-based message hierarchy.
///
/// # JSON-RPC 2.0 Compliance
///
/// All messages include the required `jsonrpc: "2.0"` field as per specification.
/// The `id` field distinguishes between requests/responses (present) and notifications (absent).
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::mcp::JsonRpcMessage;
/// use serde_json::json;
///
/// // Request message
/// let request = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: Some(json!(1)),
///     method: Some("initialize".to_string()),
///     params: Some(json!({"protocolVersion": "2024-11-05"})),
///     result: None,
///     error: None,
/// };
///
/// // Response message  
/// let response = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: Some(json!(1)),
///     method: None,
///     params: None,
///     result: Some(json!({"protocolVersion": "2024-11-05"})),
///     error: None,
/// };
///
/// // Notification message
/// let notification = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: None,
///     method: Some("initialized".to_string()),
///     params: None,
///     result: None,
///     error: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcMessage {
    /// JSON-RPC protocol version (always "2.0")
    pub jsonrpc: String,

    /// Message ID for correlation (present for requests/responses, absent for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonValue>,

    /// Method name for requests and notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Parameters for requests and notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<JsonValue>,

    /// Result for successful responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,

    /// Error for failed responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcMessage {
    /// Create a new request message
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional parameters
    /// * `id` - Request ID for correlation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let request = JsonRpcMessage::new_request(
    ///     "ping",
    ///     None,
    ///     json!("ping-123")
    /// );
    ///
    /// assert_eq!(request.method.unwrap(), "ping");
    /// assert_eq!(request.id.unwrap(), json!("ping-123"));
    /// assert!(request.result.is_none());
    /// assert!(request.error.is_none());
    /// ```
    pub fn new_request(
        method: impl Into<String>,
        params: Option<JsonValue>,
        id: JsonValue,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a new success response message
    ///
    /// # Arguments
    ///
    /// * `result` - Response result data
    /// * `id` - Request ID for correlation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let response = JsonRpcMessage::new_response(
    ///     json!({"status": "ok"}),
    ///     json!(1)
    /// );
    ///
    /// assert!(response.result.is_some());
    /// assert!(response.method.is_none());
    /// assert!(response.error.is_none());
    /// ```
    pub fn new_response(result: JsonValue, id: JsonValue) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Create a new error response message
    ///
    /// # Arguments
    ///
    /// * `error` - Error details
    /// * `id` - Request ID for correlation (or null for parse errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::{JsonRpcMessage, JsonRpcError};
    /// use serde_json::json;
    ///
    /// let error = JsonRpcError {
    ///     code: -32601,
    ///     message: "Method not found".to_string(),
    ///     data: None,
    /// };
    ///
    /// let response = JsonRpcMessage::new_error(error, json!(1));
    ///
    /// assert!(response.error.is_some());
    /// assert!(response.result.is_none());
    /// ```
    pub fn new_error(error: JsonRpcError, id: JsonValue) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Create a new notification message
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let notification = JsonRpcMessage::new_notification(
    ///     "initialized",
    ///     Some(json!({"timestamp": "2024-01-01T00:00:00Z"}))
    /// );
    ///
    /// assert_eq!(notification.method.unwrap(), "initialized");
    /// assert!(notification.id.is_none());
    /// ```
    pub fn new_notification(method: impl Into<String>, params: Option<JsonValue>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        self.method.is_some() && self.id.is_some()
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        self.id.is_some()
            && self.method.is_none()
            && (self.result.is_some() || self.error.is_some())
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Standard JSON-RPC error structure
///
/// Follows the JSON-RPC 2.0 specification for error responses.
/// Standard error codes are defined in the specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code (integer)
    pub code: i64,

    /// Human-readable error message
    pub message: String,

    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

impl JsonRpcError {
    /// Standard JSON-RPC error codes
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;

    /// Create a new JSON-RPC error
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new JSON-RPC error with additional data
    pub fn with_data(code: i64, message: impl Into<String>, data: JsonValue) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(Self::INVALID_REQUEST, "Invalid Request")
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::with_data(
            Self::METHOD_NOT_FOUND,
            "Method not found",
            serde_json::json!({"method": method}),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(details: &str) -> Self {
        Self::with_data(
            Self::INVALID_PARAMS,
            "Invalid params",
            serde_json::json!({"details": details}),
        )
    }

    /// Create an internal error
    pub fn internal_error(details: &str) -> Self {
        Self::with_data(
            Self::INTERNAL_ERROR,
            "Internal error",
            serde_json::json!({"details": details}),
        )
    }
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

/// Transport-level error types
///
/// These errors represent transport layer failures, separate from
/// JSON-RPC protocol errors that are part of the message format.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Connection failed or was lost
    #[error("Connection error: {message}")]
    Connection { message: String },

    /// Message serialization/deserialization failed
    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// I/O error during transport operations
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Transport was closed
    #[error("Transport is closed")]
    Closed,

    /// Transport-specific error
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Timeout during operation
    #[error("Operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },
}

impl TransportError {
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a transport-specific error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        Self::Timeout { duration_ms }
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
/// use airs_mcp::transport::mcp::MessageContext;
/// use chrono::Utc;
///
/// let context = MessageContext::new("session-123".to_string())
///     .with_remote_addr("192.168.1.100:8080".to_string())
///     .with_user_agent("airs-mcp-client/1.0".to_string());
///
/// assert_eq!(context.session_id(), Some("session-123"));
/// assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
/// ```
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Session identifier (if applicable)
    session_id: Option<String>,

    /// Timestamp when message was received
    timestamp: DateTime<Utc>,

    /// Remote address/endpoint information
    remote_addr: Option<String>,

    /// Additional metadata
    metadata: HashMap<String, String>,
}

impl MessageContext {
    /// Create a new message context
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new message context without session ID
    pub fn without_session() -> Self {
        Self {
            session_id: None,
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
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
}

impl Default for MessageContext {
    fn default() -> Self {
        Self::without_session()
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
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::mcp::{MessageHandler, JsonRpcMessage, MessageContext, TransportError};
/// use async_trait::async_trait;
/// use std::sync::Arc;
///
/// struct EchoHandler;
///
/// #[async_trait]
/// impl MessageHandler for EchoHandler {
///     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {
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
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming JSON-RPC message
    ///
    /// This method is called for every message received by the transport,
    /// including requests, responses, and notifications.
    ///
    /// # Arguments
    ///
    /// * `message` - The JSON-RPC message received
    /// * `context` - Session and metadata information
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext);

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
/// ```rust
/// use airs_mcp::transport::mcp::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
/// use std::sync::Arc;
/// use async_trait::async_trait;
///
/// # struct MyHandler;
/// # #[async_trait]
/// # impl MessageHandler for MyHandler {
/// #     async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext) {}
/// #     async fn handle_error(&self, error: TransportError) {}
/// #     async fn handle_close(&self) {}
/// # }
/// # struct MyTransport;
/// # impl MyTransport {
/// #     fn new() -> Self { Self }
/// # }
/// # #[async_trait]
/// # impl Transport for MyTransport {
/// #     type Error = TransportError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn close(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> { Ok(()) }
/// #     fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {}
/// #     fn session_id(&self) -> Option<String> { None }
/// #     fn set_session_context(&mut self, session_id: Option<String>) {}
/// #     fn is_connected(&self) -> bool { true }
/// #     fn transport_type(&self) -> &'static str { "test" }
/// # }
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let handler = Arc::new(MyHandler);
///     let mut transport = MyTransport::new();
///     
///     // Set up event-driven message handling
///     transport.set_message_handler(handler);
///     
///     // Start the transport (begins listening for messages)
///     transport.start().await?;
///     
///     // Send a message
///     let message = JsonRpcMessage::new_notification("ping", None);
///     transport.send(message).await?;
///     
///     // Transport automatically calls handler.handle_message() for incoming messages
///     // No blocking receive() calls needed
///     
///     // Clean up
///     transport.close().await?;
///     
///     Ok(())
/// }
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
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error>;

    /// Set the message handler for incoming messages
    ///
    /// The transport will call the handler's methods for each incoming message,
    /// transport error, and transport closure event.
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler for incoming messages and events
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>);

    /// Get the current session ID (if applicable)
    ///
    /// For session-based transports, this returns the current session identifier.
    /// For single-connection transports like STDIO, this may return None.
    ///
    /// # Returns
    ///
    /// Session ID string or None if not applicable
    fn session_id(&self) -> Option<String>;

    /// Set session context for the transport
    ///
    /// This method allows associating a session ID with the transport,
    /// useful for session-aware transports like HTTP.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Optional session identifier
    fn set_session_context(&mut self, session_id: Option<String>);

    /// Check if the transport is currently connected
    ///
    /// # Returns
    ///
    /// `true` if transport is connected and can send/receive messages
    fn is_connected(&self) -> bool;

    /// Get the transport type identifier
    ///
    /// This is used for debugging, logging, and metrics collection.
    ///
    /// # Returns
    ///
    /// Static string identifying the transport type (e.g., "stdio", "http", "websocket")
    fn transport_type(&self) -> &'static str;
}

/// Compatibility bridge for existing JsonRpcMessage implementations
///
/// This allows gradual migration from the existing trait-based message system
/// to the new flat JsonRpcMessage structure without breaking existing code.
impl From<JsonRpcMessage> for Vec<u8> {
    fn from(message: JsonRpcMessage) -> Self {
        message.to_bytes().unwrap_or_default()
    }
}

impl TryFrom<Vec<u8>> for JsonRpcMessage {
    type Error = serde_json::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(&bytes)
    }
}

impl TryFrom<&[u8]> for JsonRpcMessage {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

/// Bridge to convert from legacy JsonRpcMessage trait implementations
///
/// This enables gradual migration by allowing conversion from existing
/// message types to the new MCP-compliant format.
impl JsonRpcMessage {
    /// Convert from any legacy JsonRpcMessage trait implementation
    pub fn from_legacy<T>(legacy_message: &T) -> Result<Self, serde_json::Error>
    where
        T: LegacyJsonRpcMessage,
    {
        let json = legacy_message.to_json()?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_message_request() {
        let request = JsonRpcMessage::new_request(
            "initialize",
            Some(json!({"protocolVersion": "2024-11-05"})),
            json!(1),
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method.as_ref().unwrap(), "initialize");
        assert_eq!(request.id.as_ref().unwrap(), &json!(1));
        assert!(request.is_request());
        assert!(!request.is_response());
        assert!(!request.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_response() {
        let response =
            JsonRpcMessage::new_response(json!({"protocolVersion": "2024-11-05"}), json!(1));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.method.is_none());
        assert_eq!(response.id.as_ref().unwrap(), &json!(1));
        assert!(response.result.is_some());
        assert!(!response.is_request());
        assert!(response.is_response());
        assert!(!response.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_notification() {
        let notification = JsonRpcMessage::new_notification(
            "initialized",
            Some(json!({"timestamp": "2024-01-01T00:00:00Z"})),
        );

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method.as_ref().unwrap(), "initialized");
        assert!(notification.id.is_none());
        assert!(!notification.is_request());
        assert!(!notification.is_response());
        assert!(notification.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_error() {
        let error = JsonRpcError::method_not_found("unknown_method");
        let error_response = JsonRpcMessage::new_error(error, json!(1));

        assert_eq!(error_response.jsonrpc, "2.0");
        assert!(error_response.method.is_none());
        assert_eq!(error_response.id.as_ref().unwrap(), &json!(1));
        assert!(error_response.error.is_some());
        assert!(error_response.is_error());
        assert!(error_response.is_response());
    }

    #[test]
    fn test_jsonrpc_message_serialization() {
        let request =
            JsonRpcMessage::new_request("test", Some(json!({"key": "value"})), json!("test-123"));

        let json = request.to_json().unwrap();
        let parsed = JsonRpcMessage::from_json(&json).unwrap();

        assert_eq!(request, parsed);
    }

    #[test]
    fn test_jsonrpc_error_standard_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_message_context() {
        let context = MessageContext::new("session-123")
            .with_remote_addr("192.168.1.100:8080".to_string())
            .with_user_agent("airs-mcp-client/1.0".to_string())
            .with_metadata("custom-header".to_string(), "custom-value".to_string());

        assert_eq!(context.session_id(), Some("session-123"));
        assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
        assert_eq!(
            context.get_metadata("user-agent"),
            Some("airs-mcp-client/1.0")
        );
        assert_eq!(context.get_metadata("custom-header"), Some("custom-value"));
    }

    #[test]
    fn test_transport_error_creation() {
        let conn_err = TransportError::connection("Connection refused");
        assert!(matches!(conn_err, TransportError::Connection { .. }));

        let timeout_err = TransportError::timeout(5000);
        assert!(matches!(
            timeout_err,
            TransportError::Timeout { duration_ms: 5000 }
        ));

        let transport_err = TransportError::transport("Custom transport error");
        assert!(matches!(transport_err, TransportError::Transport { .. }));
    }

    // Mock implementation for testing the Transport trait
    struct MockTransport {
        session_id: Option<String>,
        connected: bool,
        handler: Option<Arc<dyn MessageHandler>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                session_id: None,
                connected: false,
                handler: None,
            }
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        type Error = TransportError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.connected = true;
            Ok(())
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            self.connected = false;
            if let Some(handler) = &self.handler {
                handler.handle_close().await;
            }
            Ok(())
        }

        async fn send(&mut self, _message: JsonRpcMessage) -> Result<(), Self::Error> {
            if !self.connected {
                return Err(TransportError::Closed);
            }
            Ok(())
        }

        fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {
            self.handler = Some(handler);
        }

        fn session_id(&self) -> Option<String> {
            self.session_id.clone()
        }

        fn set_session_context(&mut self, session_id: Option<String>) {
            self.session_id = session_id;
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn transport_type(&self) -> &'static str {
            "mock"
        }
    }

    struct MockHandler {
        messages: Arc<tokio::sync::Mutex<Vec<JsonRpcMessage>>>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        #[allow(dead_code)]
        async fn get_messages(&self) -> Vec<JsonRpcMessage> {
            self.messages.lock().await.clone()
        }
    }

    #[async_trait]
    impl MessageHandler for MockHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
            self.messages.lock().await.push(message);
        }

        async fn handle_error(&self, _error: TransportError) {}

        async fn handle_close(&self) {}
    }

    #[tokio::test]
    async fn test_transport_trait() {
        let mut transport = MockTransport::new();
        let handler = Arc::new(MockHandler::new());

        // Test initial state
        assert!(!transport.is_connected());
        assert_eq!(transport.transport_type(), "mock");

        // Set handler and start
        transport.set_message_handler(handler.clone());
        transport.start().await.unwrap();
        assert!(transport.is_connected());

        // Test session management
        transport.set_session_context(Some("test-session".to_string()));
        assert_eq!(transport.session_id(), Some("test-session".to_string()));

        // Test sending messages
        let message = JsonRpcMessage::new_notification("test", None);
        transport.send(message).await.unwrap();

        // Test close
        transport.close().await.unwrap();
        assert!(!transport.is_connected());

        // Test sending after close fails
        let message = JsonRpcMessage::new_notification("test", None);
        assert!(transport.send(message).await.is_err());
    }
}
