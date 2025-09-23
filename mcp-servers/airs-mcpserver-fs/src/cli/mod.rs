//! CLI module for AIRS MCP-FS
//!
//! Provides command-line interface coordination and routing for the MCP filesystem server.
//! This module serves as the main CLI coordinator, handling argument parsing, logging setup,
//! and routing commands to appropriate handlers.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod args;
pub mod coordinator;
pub mod handlers;
pub mod logging;

// Public API re-exports
pub use args::{Cli, Commands};
pub use coordinator::run;
