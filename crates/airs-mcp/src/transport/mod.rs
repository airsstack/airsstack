//! Transport Abstraction Layer
//!
//! This module provides transport abstractions for JSON-RPC communication.
//! The transport layer sits between the correlation manager and the actual
//! communication protocols (STDIO, HTTP, WebSocket, etc.).
//!
//! # Architecture
//!
//! The transport layer is built around the `Transport` trait, which defines
//! the core operations for sending and receiving messages:
//!
//! ```rust
//! use airs_mcp::transport::Transport;
//!
//! async fn example_usage<T: Transport>(mut transport: T) -> Result<(), T::Error> {
//!     // Send a message
//!     transport.send(b"Hello, world!").await?;
//!     
//!     // Receive a response
//!     let response = transport.receive().await?;
//!     
//!     // Close the connection
//!     transport.close().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Design Principles
//!
//! - **Async-native**: All operations return futures for integration with Tokio
//! - **Error flexibility**: Associated Error type for transport-specific error handling
//! - **Generic messages**: Uses `&[u8]` for maximum flexibility and zero-copy potential
//! - **Resource management**: Explicit `close()` method for proper cleanup
//! - **Thread safety**: All implementations must be `Send + Sync`
//!
//! # Transports
//!
//! Currently implemented transports:
//! - **STDIO**: Standard input/output for MCP communication (primary)
//!
//! Planned transports:
//! - **HTTP**: RESTful JSON-RPC over HTTP
//! - **WebSocket**: Real-time bidirectional communication
//! - **TCP**: Direct socket communication
//!
//! # Error Handling
//!
//! Each transport defines its own error type that must implement:
//! - `std::error::Error`
//! - `Send + Sync + 'static`
//!
//! This allows for transport-specific error variants while maintaining
//! a consistent interface.

pub mod error;
pub mod stdio;
pub mod traits;

pub use error::TransportError;
pub use stdio::StdioTransport;
pub use traits::Transport;
