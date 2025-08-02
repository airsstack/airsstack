// CLI arguments and global options
// Defines the main CLI structure using clap derive macros

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// AI-focused memory bank management tool
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

        /// Check specific sub-project
        #[arg(short = 's', long = "sub-project")]
        #[arg(help = "Check status of specific sub-project")]
        sub_project: Option<String>,
    },

    /// Manage and display current project context
    #[command(name = "context")]
    #[command(about = "Manage project context and active sub-project")]
    Context {
        /// Set active sub-project
        #[arg(short = 's', long = "set")]
        #[arg(help = "Set the active sub-project")]
        set: Option<String>,

        /// Show current context
        #[arg(long = "show")]
        #[arg(help = "Display current context information")]
        show: bool,

        /// List available sub-projects
        #[arg(short = 'l', long = "list")]
        #[arg(help = "List all available sub-projects")]
        list: bool,
    },

    /// Handle task management and tracking operations
    #[command(name = "tasks")]
    #[command(about = "Manage tasks and track progress")]
    Tasks {
        /// Action to perform
        #[command(subcommand)]
        action: TaskAction,
    },
}

/// Task management actions
#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// List tasks with optional filtering
    #[command(name = "list")]
    #[command(about = "List tasks with optional filtering")]
    List {
        /// Filter tasks by status
        #[arg(short = 's', long = "status")]
        #[arg(help = "Filter by task status")]
        #[arg(value_parser = ["all", "active", "pending", "completed", "blocked"])]
        status: Option<String>,

        /// Filter tasks by sub-project
        #[arg(short = 'p', long = "project")]
        #[arg(help = "Filter by sub-project")]
        project: Option<String>,
    },

    /// Add a new task
    #[command(name = "add")]
    #[command(about = "Add a new task")]
    Add {
        /// Task title
        #[arg(help = "Task title")]
        title: String,

        /// Sub-project for the task
        #[arg(short = 'p', long = "project")]
        #[arg(help = "Sub-project for this task")]
        project: Option<String>,

        /// Task description
        #[arg(short = 'd', long = "description")]
        #[arg(help = "Detailed task description")]
        description: Option<String>,
    },

    /// Update an existing task
    #[command(name = "update")]
    #[command(about = "Update an existing task")]
    Update {
        /// Task ID to update
        #[arg(help = "Task ID (e.g., task_001)")]
        task_id: String,

        /// New status for the task
        #[arg(short = 's', long = "status")]
        #[arg(help = "Update task status")]
        #[arg(value_parser = ["pending", "in_progress", "completed", "blocked"])]
        status: Option<String>,

        /// Progress note
        #[arg(short = 'n', long = "note")]
        #[arg(help = "Add a progress note")]
        note: Option<String>,
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
