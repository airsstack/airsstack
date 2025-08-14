//! HTTP Transport Implementation
//!
//! This module provides HTTP-based transport for JSON-RPC communication,
//! including both client and server implementations.

pub mod axum_server;
pub mod buffer_pool;
pub mod client;
pub mod config;
pub mod connection_manager;
pub mod parser;
pub mod server;
pub mod session;

pub use axum_server::{AxumHttpServer, ServerState};
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
