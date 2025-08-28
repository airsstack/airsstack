//! Model Context Protocol integration for AIRS MCP-FS
//!
//! Provides MCP server implementation, tool registration, and STDIO transport for Claude Desktop.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod handlers;
pub mod server;
pub mod types;

// Public API re-exports
pub use server::{DefaultFilesystemMcpServer, FilesystemMcpServer, McpServer};
pub use types::OperationType;
