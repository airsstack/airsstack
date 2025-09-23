//! Command line argument definitions for AIRS MCP-FS
//!
//! This module contains all CLI argument structures and command definitions using clap.
//! It provides a clean separation of argument parsing concerns from command handling logic.

// Layer 1: Standard library imports
use std::path::PathBuf;

// Layer 2: Third-party crate imports
use clap::{Parser, Subcommand};

// Layer 3: Internal module imports
// (None needed for pure argument definitions)

#[derive(Parser)]
#[command(name = "airs-mcpserver-fs")]
#[command(about = "AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Setup AIRS MCP server directory structure
    Setup {
        /// Custom configuration directory (default: ~/.airs-mcpserver-fs/config)
        #[arg(long, help = "Custom configuration directory")]
        config_dir: Option<PathBuf>,

        /// Custom logs directory (default: ~/.airs-mcpserver-fs/logs)
        #[arg(long, help = "Custom logs directory")]
        logs_dir: Option<PathBuf>,

        /// Whether to overwrite existing directories
        #[arg(long)]
        force: bool,
    },
    /// Generate example configuration files
    Config {
        /// Output directory for configuration files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Environment to generate config for
        #[arg(short, long, default_value = "development")]
        env: String,

        /// Whether to overwrite existing files
        #[arg(long)]
        force: bool,
    },
    /// Run the MCP server (default when no command specified)
    Serve {
        /// Custom configuration directory (overrides AIRS_MCPSERVER_FS_CONFIG_DIR)
        #[arg(long, help = "Custom configuration directory")]
        config_dir: Option<PathBuf>,

        /// Custom logs directory (overrides AIRS_MCPSERVER_FS_LOG_DIR)
        #[arg(long, help = "Custom logs directory")]
        logs_dir: Option<PathBuf>,
    },
}

impl Default for Commands {
    fn default() -> Self {
        Commands::Serve {
            config_dir: None,
            logs_dir: None,
        }
    }
}