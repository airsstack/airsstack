//! Handler Traits for Dependency Injection
//!
//! This module defines the traits that enable dependency injection for MCP filesystem handlers.
//! These traits allow for better testability, loose coupling, and flexible handler implementations.

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::Value;

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::mcp::McpResult;
use airs_mcp::shared::protocol::Content;

/// Trait for file operation handlers
///
/// This trait defines the interface for handling file-related MCP tool operations.
/// Implementations should provide secure file reading and writing capabilities
/// with appropriate validation and error handling.
#[async_trait]
pub trait FileOperations: Send + Sync + std::fmt::Debug {
    /// Handle read_file tool execution with security validation and encoding detection
    async fn handle_read_file(&self, arguments: Value) -> McpResult<Vec<Content>>;

    /// Handle write_file tool execution with security validation and approval workflow
    async fn handle_write_file(&self, arguments: Value) -> McpResult<Vec<Content>>;
}

/// Trait for directory operation handlers
///
/// This trait defines the interface for handling directory-related MCP tool operations.
/// Implementations should provide secure directory listing capabilities with metadata
/// collection and recursive traversal support.
#[async_trait]
pub trait DirectoryOperations: Send + Sync + std::fmt::Debug {
    /// Handle list_directory tool execution with security validation and metadata
    async fn handle_list_directory(&self, arguments: Value) -> McpResult<Vec<Content>>;
}
