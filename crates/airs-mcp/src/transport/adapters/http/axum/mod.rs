//! Axum HTTP Server Architecture for MCP Transport
//!
//! This module provides a clean, modular architecture for HTTP server implementation
//! using Axum framework. It follows SOLID principles with proper separation of concerns.

mod handlers;
mod mcp_handlers;
mod mcp_operations;
mod server;

// Re-export key types for convenience
pub use handlers::{create_router, McpSseQueryParams, ServerState, SseEvent};
pub use mcp_handlers::{McpHandlers, McpHandlersBuilder};
pub use server::AxumHttpServer;
