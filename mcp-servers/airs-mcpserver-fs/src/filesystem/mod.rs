//! Filesystem operations and validation for AIRS MCP-FS
//! 
//! Provides secure filesystem operations with path validation and operation tracking.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod operations;
pub mod validation;

// Public API re-exports
pub use operations::FileOperation;
