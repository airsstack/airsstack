//! Transport Abstraction Layer
//!
//! This module provides transport abstractions for JSON-RPC communication.
//! The transport layer sits between the correlation manager and the actual
//! communication protocols.
//!
//! # Architecture
//!
//! The transport layer is organized around three key components:
//!
//! ## MCP-Compliant Transport
//! Pure MCP-specification compliant interfaces providing event-driven message handling.
//!
//! ## Transport Adapters
//! Adapters that bridge legacy transport implementations with MCP-compliant interfaces.
//!
//! ## Legacy Transport
//! The current Transport trait for backward compatibility.
//!
//! # Design Principles
//!
//! - **Async-native**: All operations return futures for integration with Tokio
//! - **Error flexibility**: Associated Error type for transport-specific error handling
//! - **Generic messages**: Uses byte arrays for maximum flexibility and zero-copy potential
//! - **Resource management**: Explicit close method for proper cleanup
//! - **Performance-optimized**: Advanced buffer management for high-throughput scenarios
//! - **Thread safety**: All implementations must be Send + Sync
//!
//! # Available Transports
//!
//! - **STDIO**: Standard input/output for MCP communication (primary)
//! - **HTTP**: HTTP Streamable Transport for MCP remote servers
//!
//! # Error Handling
//!
//! Each transport defines its own error type that must implement standard error traits
//! for consistent error handling across the transport layer.

// Export main transport components
pub mod adapters; // Transport adapters for legacy compatibility and MCP-compliant interfaces
pub mod buffer;
pub mod error;
pub mod mcp; // Pure MCP-specification compliant transport interfaces
pub mod stdio;
pub mod streaming;
pub mod traits;
pub mod zero_copy;

// Re-export http module for backward compatibility
// This provides access to transport::http::* paths used by tests and examples
pub mod http {
    pub use crate::transport::adapters::http::*;
}

// Re-export key types for convenient access
pub use buffer::*;
pub use error::*;
pub use stdio::*;
pub use streaming::*;
pub use traits::*;
pub use zero_copy::*;

// Adapter re-exports for convenience
pub use adapters::StdioTransportAdapter;

// MCP-compliant transport re-exports
pub use mcp::{
    JsonRpcError as McpJsonRpcError, JsonRpcMessage as McpJsonRpcMessage, MessageContext,
    MessageHandler, Transport as McpTransport, TransportError as McpTransportError,
};

// HTTP transport re-exports (via adapters for backward compatibility)
pub use adapters::{
    AxumHttpServer, BufferPool, BufferPoolStats, HttpClientTransport, HttpServerTransport,
    HttpTransportConfig, RequestParser,
};
