//! Internal Implementation Details
//!
//! This module contains internal implementation details, optimizations, and utilities
//! that support the public protocol API but are not part of the public interface.
//!
//! # Module Organization
//!
//! - `context`: Context management and session handling
//!
//! # Consolidation Sources
//!
//! **Phase 2 Migration Plan:**
//! - `context.rs` from `src/transport/mcp/context.rs`
//!
//! **Removed Modules (0.2.0 Simplification):**
//! - `concurrent.rs` - Processors identified as over-engineering, removed for core focus
//! - `streaming.rs` - Advanced streaming functionality removed for 0.2.0 core release

// Module declarations (workspace standard: declarations only in mod.rs)
pub mod context;

// Internal modules are NOT re-exported (implementation details)
