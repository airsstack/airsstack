//! HTTP Transport Module
//!
//! This module provides HTTP transport implementations for the MCP protocol,
//! following the Single Responsibility Principle with separate client and server implementations.
//!
//! # Architecture
//!
//! The HTTP transport is built around validated architectural decisions:
//!
//! - **Single Responsibility**: Separate client and server transport implementations
//! - **Role-Specific Design**: Each transport correctly models its communication pattern
//! - **Performance Optimized**: Per-request parsing and optional buffer pooling
//! - **Configuration-Driven**: Builder pattern with progressive optimization
//!
//! # Modules
//!
//! - [`client`]: HTTP client transport for sending requests to MCP servers
//! - [`server`]: HTTP server transport for receiving requests from MCP clients  
//! - [`config`]: Configuration types and builders
//! - [`buffer_pool`]: Buffer pool implementation for high-throughput scenarios
//! - [`parser`]: Request/response parsing utilities
//!
//! # Usage Examples
//!
//! ## Client Transport
//!
//! ```rust
//! use airs_mcp::transport::http::{HttpTransportConfig, HttpClientTransport};
//! use std::time::Duration;
//!
//! let config = HttpTransportConfig::new()
//!     .request_timeout(Duration::from_secs(30))
//!     .enable_buffer_pool();
//!
//! let mut client = HttpClientTransport::new(config);
//! client.set_target("http://localhost:8080/mcp".parse().unwrap());
//! ```
//!
//! ## Server Transport (Phase 3)
//!
//! ```rust
//! use airs_mcp::transport::http::{HttpTransportConfig, HttpServerTransport};
//!
//! let config = HttpTransportConfig::new()
//!     .bind_address("0.0.0.0:8080".parse().unwrap())
//!     .max_connections(5000)
//!     .enable_buffer_pool();
//!
//! let server = HttpServerTransport::new(config);
//! ```
//!
//! # Protocol Compliance
//!
//! - **MCP March 2025 Specification**: 100% compliant implementation
//! - **Session Management**: Proper `Mcp-Session-Id` header handling
//! - **Connection Recovery**: `Last-Event-ID` support for streaming
//! - **Error Responses**: RFC-compliant JSON-RPC error formatting

// Sub-modules organized by responsibility
pub mod buffer_pool;
pub mod client;
pub mod config;
pub mod connection_manager;
pub mod parser;
pub mod server;
pub mod session;

// Re-export public configuration API
pub use config::{BufferPoolConfig, HttpTransportConfig, OptimizationStrategy, ParserConfig};

// Re-export buffer pool API
pub use buffer_pool::{BufferHandle, BufferPool, BufferPoolStats, BufferStrategy, PooledBuffer};

// Re-export parser API
pub use parser::{ParseMetrics, RequestParser};

// Re-export transport implementations
pub use client::HttpClientTransport;
pub use server::HttpServerTransport;

// Re-export connection management API
pub use connection_manager::{
    ConnectionHealth, ConnectionId, ConnectionInfo, ConnectionStatsSnapshot, HealthCheckConfig,
    HealthCheckResult, HttpConnectionManager,
};

// Re-export session management API
pub use session::{
    extract_last_event_id, extract_session_id, ClientInfo, SessionConfig, SessionContext,
    SessionId, SessionManager, SessionMetadata, SessionStatsSnapshot,
};
