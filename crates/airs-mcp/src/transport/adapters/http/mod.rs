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
pub mod axum; // Re-enabled with direct MessageHandler usage
pub mod buffer_pool;
pub mod builder; // NEW: Pre-configured transport builder
pub mod client;
pub mod config;
pub mod connection_manager;
pub mod context; // NEW: HTTP context for generic MessageHandler pattern
pub mod engine;
pub mod handlers; // NEW: Example MessageHandler<HttpContext> implementations
pub mod parser;
pub mod server;
pub mod session;
pub mod sse;

pub use auth::OAuth2StrategyAdapter;
pub use auth_request::HttpAuthRequest;
pub use axum::{AxumHttpServer, ServerState}; // Re-enabled with direct MessageHandler usage
pub use buffer_pool::{BufferPool, BufferPoolStats, BufferStrategy, PooledBuffer};
pub use builder::{HttpTransport, HttpTransportBuilder}; // NEW: Pre-configured transport pattern
pub use client::HttpClientTransport;
pub use config::HttpTransportConfig;
pub use connection_manager::{
    ConnectionHealth, ConnectionId, ConnectionInfo, ConnectionStats, ConnectionStatsSnapshot,
    HealthCheckConfig, HealthCheckResult, HttpConnectionManager,
};
pub use context::HttpContext; // NEW: HTTP context for generic MessageHandler pattern
pub use engine::{
    AuthenticationContext, HttpEngine, HttpEngineError, HttpMiddleware, HttpResponse,
    McpRequestHandler, ResponseMode,
};
pub use handlers::{EchoHttpHandler, McpHttpHandler, StaticFileHandler}; // NEW: Example HTTP message handlers
pub use parser::RequestParser;

// Type aliases for convenience (as per Phase 5.5.5 requirements)
/// Type alias for HTTP message handlers using HttpContext
pub type HttpMessageHandler = dyn crate::protocol::MessageHandler<HttpContext>;

/// Type alias for HTTP message context
pub type HttpMessageContext = crate::protocol::MessageContext<HttpContext>;
pub use server::HttpServerTransport;
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
