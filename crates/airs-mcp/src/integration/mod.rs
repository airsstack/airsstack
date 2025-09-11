//! Integration Layer - High-level MCP (Model Context Protocol) implementations
//!
//! This module provides high-level, ergonomic APIs for building MCP clients and servers.
//! It combines the modern protocol transport layer with MCP-specific abstractions.

pub mod client;
pub mod constants;
pub mod error;
pub mod server;

// Re-export MCP-specific types
// Re-export MCP-specific types
pub use client::{McpClient, McpClientBuilder, McpClientConfig, McpSessionState};
pub use constants::*;
pub use error::{McpError, McpResult};
pub use server::{LoggingHandler, McpServer};

// Re-export integration error types for backwards compatibility
pub use error::{IntegrationError, IntegrationResult};

// Placeholder exports for removed legacy components
// These provide type aliases to the modern MCP implementations
pub type JsonRpcClient<T> = McpClient<T>;
pub type JsonRpcServer<T> = McpServer<T>;

// Handler types - these can be implemented using the modern MessageHandler pattern
pub trait Handler: Send + Sync {}
pub trait NotificationHandler: Handler {}
pub trait RequestHandler: Handler {}

// Message router - replaced by modern protocol transport event handling
pub struct MessageRouter;

impl MessageRouter {
    pub fn new(_config: RouteConfig) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Default)]
pub struct RouteConfig;

// Implement the handler traits for empty structs to maintain API compatibility
impl Handler for () {}
impl NotificationHandler for () {}
impl RequestHandler for () {}
