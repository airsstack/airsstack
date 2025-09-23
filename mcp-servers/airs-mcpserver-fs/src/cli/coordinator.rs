//! CLI coordinator for AIRS MCP-FS
//!
//! Handles CLI coordination logic including argument parsing, logging setup, and command routing.
//! This module contains the actual implementation logic that was previously in mod.rs.

// Layer 1: Standard library imports
// (None needed for coordination logic)

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
use super::args::{Cli, Commands};
use super::handlers;
use super::logging::{determine_logging_mode, initialize_logging};

/// Main CLI entry point that coordinates all CLI operations
pub async fn run() -> Result<()> {
    use clap::Parser;

    let cli = Cli::parse();

    // Determine logging mode based on command
    let logging_mode = determine_logging_mode(&cli);

    // Initialize logging
    initialize_logging(logging_mode)?;

    // Route to appropriate handler
    match cli.command.unwrap_or_default() {
        Commands::Setup {
            config_dir,
            logs_dir,
            force,
        } => handlers::setup::handle_setup(config_dir, logs_dir, force).await,
        Commands::Config { output, env, force } => {
            handlers::config::handle_config(output, &env, force).await
        }
        Commands::Serve {
            config_dir,
            logs_dir,
        } => handlers::serve::handle_serve(config_dir, logs_dir).await,
    }
}
