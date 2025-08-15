//! Axum HTTP Server Architecture for MCP Transport
//!
//! This module provides a clean, modular architecture for HTTP server implementation
//! using Axum framework. It follows SOLID principles with proper separation of concerns.

pub mod handlers;
pub mod mcp_handlers;
pub mod mcp_operations;
pub mod server;

// Re-export key types for convenience
pub use handlers::*;
pub use mcp_handlers::{McpHandlers, McpHandlersBuilder};
pub use server::AxumHttpServer;
