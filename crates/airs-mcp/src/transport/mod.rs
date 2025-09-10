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
// buffer module removed - advanced buffer management not used by current transport implementations
pub mod error;
// mcp module removed - functionality consolidated into protocol module
// stdio module removed - use protocol-based transport instead
// streaming module removed - unnecessary complexity for MCP compliance
// traits module removed - use protocol::Transport instead of transport::traits::Transport
// zero_copy module removed - depends on deprecated Transport trait

// Re-export http module for backward compatibility
// This provides access to transport::http::* paths used by tests and examples
pub mod http {
    pub use crate::transport::adapters::http::*;
}

// Re-export key types for convenient access
pub use error::*;
// zero_copy module removed - use protocol-based transport implementations

// MCP-compliant transport re-exports moved to protocol module
// These types are now available via crate::protocol::*

// HTTP transport re-exports (via adapters for backward compatibility)
// Note: Complex types temporarily disabled for MCP-compliant simplification
pub use adapters::{
    /*AxumHttpServer,*/ BufferPool, BufferPoolStats,
    HttpTransportConfig, RequestParser,
};
