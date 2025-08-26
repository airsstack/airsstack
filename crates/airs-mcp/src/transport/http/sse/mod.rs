//! HTTP Server-Sent Events (SSE) Transport Implementation
//!
//! This module provides SSE transport for legacy MCP client compatibility.
//!
//! ⚠️  **DEPRECATION NOTICE**: This transport is provided for ecosystem
//! transition support. New implementations should use HTTP Streamable transport.
//!
//! # Architecture
//!
//! Uses dual-endpoint pattern:
//! - `POST /messages` - JSON request/response with session creation
//! - `GET /sse` - Server-Sent Events streaming with session correlation
//!
//! # Migration Support
//!
//! Built-in migration tools help transition to HTTP Streamable:
//! ```rust
//! use airs_mcp::transport::http::sse::{HttpSseConfig, MigrationMode};
//!
//! let config = HttpSseConfig::new()
//!     .migration_mode(MigrationMode::Active)
//!     .encourage_migration();
//! ```
//!
//! # Performance Characteristics
//!
//! Intentionally conservative performance targets for legacy compatibility:
//! - ~10,000 req/sec throughput
//! - ~1,000 concurrent connections
//! - ~1-2ms latency
//! - Built-in migration incentives
//!
//! # Examples
//!
//! ```rust
//! use airs_mcp::transport::http::sse::{HttpSseConfig, MigrationMode};
//! use std::time::Duration;
//!
//! // Basic SSE configuration
//! let config = HttpSseConfig::new();
//!
//! // SSE configuration with migration encouragement
//! let config = HttpSseConfig::new()
//!     .migration_mode(MigrationMode::Active)
//!     .heartbeat_interval(Duration::from_secs(30))
//!     .encourage_migration();
//! ```

pub mod config;
pub mod constants;
pub mod transport;

// Re-export public types
pub use config::{
    DeprecationConfig, DeprecationPhase, HttpSseConfig, MigrationMode, SseEndpointConfig,
};

// Re-export constants for easy access
pub use constants::{
    DEFAULT_MESSAGES_ENDPOINT, DEFAULT_SSE_ENDPOINT,
    cache_control, content_types, events, headers,
};

// Re-export transport types
pub use transport::{HttpSseTransport, SseBroadcaster, SseEvent};
