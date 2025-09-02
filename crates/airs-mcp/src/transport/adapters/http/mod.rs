//! HTTP Transport Implementation
//!
//! This module provides HTTP-based transport for JSON-RPC communication,
//! including both client and server implementations, as well as MCP-compliant
//! transport adapters.
//!
//! # Available Transports
//!
//! - **HTTP Streamable** (Recommended): High-performance streaming transport
//! - **HTTP SSE** (Legacy): Server-Sent Events for ecosystem compatibility
//! - **MCP Transport Adapters** (New): Event-driven MCP-compliant interfaces

pub mod auth;
pub mod auth_request;
pub mod axum;
pub mod buffer_pool;
pub mod client;
pub mod client_adapter;
pub mod config;
pub mod connection_manager;
pub mod engine;
pub mod parser;
pub mod server;
pub mod server_adapter;
pub mod session;
pub mod sse;

pub use auth::OAuth2StrategyAdapter;
pub use auth_request::HttpAuthRequest;
pub use axum::{AxumHttpServer, ServerState};
pub use buffer_pool::{BufferPool, BufferPoolStats, BufferStrategy, PooledBuffer};
pub use client::HttpClientTransport;
pub use client_adapter::HttpClientTransportAdapter;
pub use config::HttpTransportConfig;
pub use connection_manager::{
    ConnectionHealth, ConnectionId, ConnectionInfo, ConnectionStats, ConnectionStatsSnapshot,
    HealthCheckConfig, HealthCheckResult, HttpConnectionManager,
};
pub use engine::{
    AuthenticationContext, HttpEngine, HttpEngineError, HttpMiddleware, HttpResponse,
    McpRequestHandler, ResponseMode,
};
pub use parser::RequestParser;
pub use server::HttpServerTransport;
pub use server_adapter::HttpServerTransportAdapter;
pub use session::{
    extract_last_event_id, extract_session_id, SessionConfig, SessionId, SessionManager,
    SessionStatsSnapshot,
};

// SSE transport exports
pub use sse::{
    cache_control, content_types, events, headers, DeprecationConfig, DeprecationPhase,
    HttpSseConfig, MigrationMode, SseEndpointConfig, DEFAULT_MESSAGES_ENDPOINT,
    DEFAULT_SSE_ENDPOINT,
};
