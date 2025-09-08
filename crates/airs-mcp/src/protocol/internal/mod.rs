//! Internal Implementation Details
//!
//! This module contains internal implementation details, optimizations, and utilities
//! that support the public protocol API but are not part of the public interface.
//!
//! # Module Organization
//!
//! - `concurrent`: Concurrent processing utilities and optimizations
//! - `streaming`: Streaming protocol optimizations and buffering
//! - `context`: Context management and session handling
//!
//! # Consolidation Sources
//!
//! **Phase 2 Migration Plan:**
//! - `concurrent.rs` from `src/base/jsonrpc/concurrent.rs`
//! - `streaming.rs` from `src/base/jsonrpc/streaming.rs`
//! - `context.rs` from `src/transport/mcp/context.rs`

// Module declarations (workspace standard: declarations only in mod.rs)
pub mod concurrent;
pub mod context;
pub mod streaming;

// Internal modules are NOT re-exported (implementation details)
