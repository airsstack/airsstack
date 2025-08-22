//! Binary file processing and format detection
//! 
//! Handles image processing, PDF processing, and file format detection for MCP operations.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod format;
pub mod processor;

// Public API re-exports
pub use format::FormatDetector;
pub use processor::BinaryProcessor;
