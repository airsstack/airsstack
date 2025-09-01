//! MCP Transport Trait Definitions
//!
//! Event-driven transport interface aligned with the official MCP specification.

use std::sync::Arc;

use async_trait::async_trait;

use super::{JsonRpcMessage, MessageContext, TransportError};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    use tokio::sync::Mutex;

    // Mock implementations for testing
    struct MockHandler {
        messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
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
