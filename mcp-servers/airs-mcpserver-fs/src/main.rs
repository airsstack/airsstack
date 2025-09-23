//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//!
//! Binary entry point for the MCP server that provides secure filesystem operations
//! for Claude Desktop and other MCP-compatible AI tools.

// Layer 1: Standard library imports
// (None needed for minimal entry point)

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
// (None needed for CLI-only entry point)

// Layer 3b: Local crate modules (only through lib.rs gateway)
use airs_mcpserver_fs::cli;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}