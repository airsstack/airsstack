//! Base layer implementations
//!
//! This module contains foundational components that other layers build upon.
//! 
//! # Modules
//! 
//! - `jsonrpc`: Complete JSON-RPC 2.0 implementation with trait-based message handling

pub mod jsonrpc;

// Re-export for internal use
pub use jsonrpc::*;