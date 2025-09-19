//! HTTP Transport Module
//!
//! This module handles the HTTP transport layer configuration and setup
//! for the API key authenticated MCP server.

pub mod server;

pub use server::HttpApiKeyServer;
