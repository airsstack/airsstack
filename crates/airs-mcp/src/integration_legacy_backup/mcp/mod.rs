//! High-level MCP (Model Context Protocol) client and server implementations
//!
//! This module provides high-level, ergonomic APIs for building MCP clients and servers.

pub mod client;
pub mod constants;
pub mod error;
pub mod server;

pub use client::{ConnectionState, McpClient, McpClientBuilder, McpClientConfig};
pub use constants::*;
pub use error::{McpError, McpResult};
pub use server::{
    LoggingHandler, McpServer, McpServerBuilder, McpServerConfig, PromptProvider, ResourceProvider,
    ToolProvider,
};
