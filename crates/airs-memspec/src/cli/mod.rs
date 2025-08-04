// CLI module for airs-memspec
// Handles command-line interface, argument parsing, and command dispatch

use clap::Parser;

pub mod args;
pub mod commands;

pub use args::{Cli, Commands, GlobalArgs, TaskAction};

/// Main CLI entry point
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Set up logging based on global args
    setup_logging(&cli.global)?;

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Install {
            target,
            force,
            template,
        } => commands::install::run(&cli.global, target, force, template),
        Commands::Status {
            detailed,
            sub_project,
        } => commands::status::run(&cli.global, detailed, sub_project),
        Commands::Context { set, show, list } => {
            commands::context::run(&cli.global, set, show, list)
        }
        Commands::Tasks { action } => commands::tasks::run(&cli.global, action),
    }
}

/// Set up logging and output formatting based on global arguments
fn setup_logging(global: &GlobalArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Configure colored output
    if global.no_color {
        colored::control::set_override(false);
    }

    // TODO: Set up proper logging based on verbose/quiet flags
    // For now, just store the configuration

    Ok(())
}
