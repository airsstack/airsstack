// CLI arguments and global options
// Defines the main CLI structure using clap derive macros

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// AI-focused memory bank management tool (read-only)
#[derive(Parser, Debug)]
#[command(name = "airs-memspec")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Global options
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Available commands
    #[command(subcommand)]
    pub command: Commands,
}

/// Global options available to all commands
#[derive(Parser, Debug)]
pub struct GlobalArgs {
    /// Path to the project directory
    #[arg(short = 'p', long = "path", global = true)]
    #[arg(help = "Path to project directory (default: current directory)")]
    pub path: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    #[arg(help = "Enable verbose output")]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long = "quiet", global = true)]
    #[arg(help = "Suppress non-essential output")]
    #[arg(conflicts_with = "verbose")]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long = "no-color", global = true)]
    #[arg(help = "Disable colored output")]
    pub no_color: bool,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Set up memory bank structure and initial configuration
    #[command(name = "install")]
    #[command(about = "Set up memory bank structure in the current project")]
    Install {
        /// Target directory for installation
        #[arg(help = "Directory to install memory bank (default: .copilot/instructions)")]
        target: Option<PathBuf>,

        /// Force installation even if files exist
        #[arg(short = 'f', long = "force")]
        #[arg(help = "Overwrite existing files")]
        force: bool,

        /// Template to use for installation
        #[arg(short = 't', long = "template")]
        #[arg(help = "Template name (default: basic)")]
        #[arg(value_parser = ["basic", "workspace", "multi-project"])]
        template: Option<String>,
    },

    /// Display memory bank status and project overview
    #[command(name = "status")]
    #[command(about = "Show current memory bank status and project overview")]
    Status {
        /// Show detailed information
        #[arg(short = 'd', long = "detailed")]
        #[arg(help = "Show detailed status information")]
        detailed: bool,

        /// Check specific project
        #[arg(long = "project")]
        #[arg(help = "Check status of specific project")]
        project: Option<String>,
    },

    /// Display current project context and active development focus
    #[command(name = "context")]
    #[command(about = "Show active development context and architectural decisions")]
    Context {
        /// Display workspace-level context
        #[arg(short = 'w', long = "workspace")]
        #[arg(help = "Show workspace-level context and integration points")]
        workspace: bool,

        /// Display specific sub-project context
        #[arg(long = "project")]
        #[arg(help = "Show context for specific sub-project")]
        project: Option<String>,
    },

    /// Handle task tracking and viewing operations (read-only)
    #[command(name = "tasks")]
    #[command(about = "View tasks and track progress (read-only access)")]
    Tasks {
        /// Action to perform
        #[command(subcommand)]
        action: TaskAction,
    },
}

/// Task tracking actions (read-only)
#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// List tasks with optional filtering
    #[command(name = "list")]
    #[command(about = "List tasks with smart filtering (15 most relevant) or custom filters")]
    List {
        /// Filter tasks by status
        #[arg(short = 's', long = "status")]
        #[arg(help = "Filter by task status")]
        #[arg(value_parser = ["all", "active", "pending", "completed", "blocked"])]
        status: Option<String>,

        /// Filter tasks by sub-project
        #[arg(long = "project")]
        #[arg(help = "Filter by sub-project")]
        project: Option<String>,

        /// Show all tasks (disable smart filtering)
        #[arg(long = "all")]
        #[arg(help = "Show all tasks (disable smart default filtering)")]
        show_all: bool,

        /// Include completed tasks in smart view
        #[arg(long = "completed")]
        #[arg(help = "Include completed tasks in results")]
        include_completed: bool,
    },

    /// Show detailed task information
    #[command(name = "show")]
    #[command(about = "Show detailed task information")]
    Show {
        /// Task ID to display
        #[arg(help = "Task ID (e.g., task_001)")]
        task_id: String,
    },
}
