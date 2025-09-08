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
//! ```rust
//! use airs_mcp::protocol::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
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
//! #     async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> { Ok(()) }
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
//!     let message = JsonRpcMessage::from_notification("ping", None);
//!     transport.send(&message).await?;
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

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

// Layer 3: Internal module imports
use super::message::JsonRpcMessage;

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
/// use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError};
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
/// use airs_mcp::protocol::{Transport, MessageHandler, JsonRpcMessage, MessageContext, TransportError};
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
/// #     async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> { Ok(()) }
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
///     let message = JsonRpcMessage::from_notification("ping", None);
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
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error>;

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

// TODO(DEBT-ARCH): Add concrete transport implementations
// Reference: HTTP transport from transport/http/, STDIO transport
// TODO(DEBT-ARCH): Add message routing and correlation tracking
// Reference: src/correlation/ for correlation patterns
