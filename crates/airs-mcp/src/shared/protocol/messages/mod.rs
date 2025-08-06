//! MCP Protocol Messages
//!
//! This module provides message types for all MCP protocol operations,
//! built on top of the existing JSON-RPC foundation.

pub mod capabilities;
pub mod initialization;

// Re-export public API
pub use capabilities::*;
pub use initialization::*;
