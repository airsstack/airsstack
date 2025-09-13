//! Axum HTTP Server Architecture for MCP Transport
//!
//! This module provides a clean, modular architecture for HTTP server implementation
//! using Axum framework. It follows SOLID principles with proper separation of concerns.

mod builder;
mod handlers;
mod server;

// Re-export key types for convenience
pub use builder::AxumHttpServerBuilder;
pub use handlers::{create_router, McpSseQueryParams, ServerState, SseEvent};
pub use server::{AxumHttpServer, NoAuth};
