//! Mock HTTP MCP Server
//!
//! A lightweight HTTP MCP server for testing the HTTP client implementation.

pub mod responses;
pub mod server;

pub use server::MockHttpServer;
