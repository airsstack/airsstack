//! Protocol Types - MCP-Specific Types and Enumerations
//!
//! This module consolidates MCP protocol types from:
//! - `src/shared/protocol/types/` (MCP protocol-specific types)
//!
//! # Consolidation Strategy
//!
//! **Phase 2 Migration Plan:**
//! - Migrate MCP-specific types (Uri, ProtocolVersion, ClientInfo, etc.) from `shared/protocol/types/`
//! - Preserve type safety and validation patterns
//! - Maintain backward compatibility through careful re-exports
//!
//! # Architecture Goals
//!
//! - **Type Safety**: Strong typing for all MCP protocol concepts
//! - **Validation**: Built-in validation for protocol constraints
//! - **Serialization**: Consistent JSON serialization/deserialization
//! - **Documentation**: Comprehensive documentation for all types

// Layer 1: Standard library imports
// (None required for current placeholder implementation)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (Will be added during Phase 2 migration)

// PHASE 1: Placeholder implementations
// These will be replaced with actual consolidated implementations in Phase 2

/// Placeholder for MCP URI type
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Uri {
    // Implementation will be added in Phase 2
}

/// Placeholder for MCP Protocol Version
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    // Implementation will be added in Phase 2
}

/// Placeholder for MCP Client Information
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    // Implementation will be added in Phase 2
}

/// Placeholder for MCP Server Information
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    // Implementation will be added in Phase 2
}

/// Placeholder for MCP Capabilities
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capabilities {
    // Implementation will be added in Phase 2
}
