//! MCP Protocol Message Handlers
//!
//! This module contains the MCP message handler that implements the
//! `MessageHandler<()>` trait for proper transport integration.

pub mod mcp_handler;

pub use mcp_handler::McpHandler;
