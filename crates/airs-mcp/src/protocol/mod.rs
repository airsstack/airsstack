//! Protocol Layer - Unified JSON-RPC 2.0 and MCP Implementation
//!
//! This module consolidates JSON-RPC 2.0 foundation with MCP protocol layer
//! and transport abstractions into a single, coherent API surface.
//!
//! # Architecture
//!
//! This module is part of the protocol consolidation effort (TASK-028)
//! consolidating functionality from:
//! - `src/base/jsonrpc` (JSON-RPC 2.0 foundation)
//! - `src/shared/protocol` (MCP protocol layer)
//! - `src/transport/mcp` (MCP transport abstractions)
//!
//! ## Module Organization
//!
//! - `message`: JSON-RPC 2.0 and MCP message types with trait-based serialization
//! - `types`: MCP protocol-specific types and enumerations
//! - `transport`: Transport abstraction traits and implementations
//! - `errors`: Consolidated error types for all protocol operations
//! - `internal`: Internal implementation details and optimizations
//!
//! # Design Goals
//!
//! - **Zero Code Duplication**: Eliminate identical implementations across modules
//! - **Single Import Path**: Provide unified API surface for all protocol functionality
//! - **Backward Compatibility**: Maintain existing APIs through careful re-exports
//! - **Performance Preservation**: Maintain 8.5+ GiB/s throughput characteristics
//! - **Workspace Standards**: Follow all workspace standards for module organization

// Layer 1: Standard library imports
// (None required for module declarations)

// Layer 2: Third-party crate imports
// (None required for module declarations)

// Layer 3: Internal module imports
// (Will be added as consolidation proceeds)

// Module declarations (workspace standard: declarations only in mod.rs)
pub mod constants;
pub mod errors;
pub mod internal;
pub mod message;
pub mod transport;
pub mod types;

// Public re-exports (workspace standard: clean API surface)
// NOTE: Actual re-exports will be added during Phase 2 migration
// This ensures Phase 1 foundation compiles without breaking existing code

pub use constants::*;
pub use errors::*;
pub use message::*;
pub use transport::*;
pub use types::*;

// Internal modules are not re-exported (implementation details)
