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

// Re-export all public types for convenient access
pub use context::MessageContext;
pub use error::TransportError;
pub use message::{JsonRpcError, JsonRpcMessage};
pub use transport::{MessageHandler, Transport};

// Internal modules
mod compat;
mod context;
mod error;
mod message;
mod transport;
