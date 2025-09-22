//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//!
//! This crate provides secure filesystem operations for Claude Desktop and other
//! MCP-compatible AI tools through a human-in-the-loop approval workflow.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod binary;
pub mod config;
pub mod filesystem;
pub mod mcp;
pub mod security;

// Public API re-exports
pub use config::Settings;
pub use filesystem::FileOperation;
pub use mcp::{DefaultFilesystemMcpServer, FilesystemMcpServer, OperationType};
pub use security::{ApprovalDecision, SecurityManager};
