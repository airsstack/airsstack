//! HTTP Transport Implementation
//!
//! This module provides HTTP-based transport for JSON-RPC communication,
//! including both client and server implementations.
//!
//! # Available Transports
//!
//! - **HTTP Streamable** (Recommended): High-performance streaming transport
//! - **HTTP SSE** (Legacy): Server-Sent Events for ecosystem compatibility

pub mod axum;
pub mod buffer_pool;
pub mod client;
pub mod config;
pub mod connection_manager;
pub mod parser;
pub mod server;
pub mod session;
pub mod sse;

pub use axum::{AxumHttpServer, ServerState};
pub use buffer_pool::{BufferPool, BufferPoolStats, BufferStrategy, PooledBuffer};
pub use client::HttpClientTransport;
pub use config::HttpTransportConfig;
pub use connection_manager::{
    ConnectionHealth, ConnectionId, ConnectionInfo, ConnectionStats, ConnectionStatsSnapshot,
    HealthCheckConfig, HealthCheckResult, HttpConnectionManager,
};
pub use parser::RequestParser;
pub use server::HttpServerTransport;
pub use session::{
    extract_last_event_id, extract_session_id, SessionConfig, SessionId, SessionManager,
    SessionStatsSnapshot,
};

// SSE transport exports
pub use sse::{
    DeprecationConfig, DeprecationPhase, HttpSseConfig, MigrationMode, SseEndpointConfig,
    DEFAULT_MESSAGES_ENDPOINT, DEFAULT_SSE_ENDPOINT, cache_control, content_types, events, headers,
};
