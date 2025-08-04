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
//! - **Performance-optimized**: Advanced buffer management for high-throughput scenarios
//! - **Thread safety**: All implementations must be `Send + Sync`
//!
//! # Buffer Management
//!
//! The transport layer includes advanced buffer management features:
//!
//! - **Buffer Pooling**: Reusable buffer allocation to minimize GC pressure
//! - **Zero-Copy Operations**: Avoid unnecessary data copying where possible
//! - **Streaming Support**: Efficient handling of partial reads and writes
//! - **Backpressure Management**: Flow control to prevent memory exhaustion
//!
//! ```rust
//! use airs_mcp::transport::buffer::{BufferManager, BufferConfig};
//!
//! async fn buffer_example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = BufferConfig::default();
//!     let buffer_manager = BufferManager::new(config);
//!     
//!     let mut buffer = buffer_manager.acquire_read_buffer().await?;
//!     // Use buffer for I/O operations...
//!     // Buffer automatically returns to pool when dropped
//!     Ok(())
//! }
//! ```
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

// Export main transport components
pub mod buffer;
pub mod error;
pub mod stdio;
pub mod traits;

// Re-export main types for convenience
pub use buffer::{BufferConfig, BufferManager, BufferMetrics, PooledBuffer, StreamingBuffer};
pub use error::TransportError;
pub use stdio::StdioTransport;
pub use traits::Transport;
