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
use super::logging::{initialize_logging, LoggingMode};

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

/// Determine appropriate logging mode based on the CLI command
fn determine_logging_mode(cli: &Cli) -> LoggingMode {
    // For serve command (or default), use file logging to keep STDIO clean
    let is_serve_command = matches!(
        cli.command.as_ref().unwrap_or(&Commands::Serve {
            config_dir: None,
            logs_dir: None
        }),
        Commands::Serve { .. }
    );

    if is_serve_command {
        // Extract logs directory for server command
        let logs_dir_override = if let Some(Commands::Serve { logs_dir, .. }) = &cli.command {
            logs_dir.clone()
        } else {
            None
        };

        let log_dir = if let Some(custom_logs_dir) = logs_dir_override {
            custom_logs_dir.to_string_lossy().to_string()
        } else {
            std::env::var("AIRS_MCPSERVER_FS_LOG_DIR")
                .or_else(|_| std::env::var("AIRS_MCP_FS_LOG_DIR")) // Backward compatibility
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    format!("{home}/.airs-mcpserver-fs/logs")
                })
        };

        LoggingMode::File { log_dir }
    } else {
        // For CLI commands: use console logging
        LoggingMode::Console
    }
}
