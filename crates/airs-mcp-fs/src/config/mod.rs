//! Configuration management for AIRS MCP-FS
//! 
//! Handles loading and validation of user settings, security policies, and runtime configuration.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports  
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod settings;

// Public API re-exports
pub use settings::{
    BinaryConfig, SecurityConfig, ServerConfig, Settings,
    FilesystemConfig, OperationConfig, SecurityPolicy, RiskLevel,
};
