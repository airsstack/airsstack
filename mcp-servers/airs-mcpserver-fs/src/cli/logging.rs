//! Logging configuration for AIRS MCP-FS CLI
//!
//! Handles initialization of structured logging with support for both console and file output.
//! Provides environment-based filtering and appropriate output formatting for different modes.

// Layer 1: Standard library imports
use std::fs;

// Layer 2: Third-party crate imports
use anyhow::Result;
use tracing_subscriber;

// Layer 3: Internal module imports
// (None needed for logging configuration)

/// Logging output mode configuration
pub enum LoggingMode {
    /// Console logging with colors for CLI commands
    Console,
    /// File logging for MCP server mode to keep STDIO clean
    File { log_dir: String },
}

/// Initialize structured logging based on the specified mode
pub fn initialize_logging(mode: LoggingMode) -> Result<()> {
    match mode {
        LoggingMode::Console => {
            // For CLI commands: use standard console output with colors
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "airs_mcpserver_fs=info".into()),
                )
                .init();
            Ok(())
        }
        LoggingMode::File { log_dir } => {
            // Create log directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&log_dir) {
                eprintln!("Warning: Failed to create log directory {log_dir}: {e}");
            }

            let log_file = format!("{log_dir}/airs-mcpserver-fs.log");
            let file_appender = match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Warning: Failed to open log file {log_file}: {e}, disabling logging");
                    // If we can't create log file, disable logging completely for MCP mode
                    tracing_subscriber::fmt()
                        .with_env_filter("off")
                        .with_writer(std::io::sink)
                        .init();
                    return Ok(());
                }
            };

            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "airs_mcpserver_fs=info".into()),
                )
                .with_ansi(false)
                .with_writer(file_appender)
                .init();
            
            Ok(())
        }
    }
}