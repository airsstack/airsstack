//! Setup command handler for AIRS MCP-FS
//!
//! Handles directory structure creation and initial setup for the MCP server.

// Layer 1: Standard library imports
use std::fs;
use std::path::PathBuf;
use std::process;

// Layer 2: Third-party crate imports
use anyhow::Result;
use tracing::{error, info};

// Layer 3: Internal module imports
// (None needed for setup operations)

/// Handle the setup command - create directory structure and sample configuration
pub async fn handle_setup(
    config_dir: Option<PathBuf>,
    logs_dir: Option<PathBuf>,
    force: bool,
) -> Result<()> {
    info!("🏗️ Setting up AIRS MCP Server directory structure");

    // Determine directories to create
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let default_base = format!("{home}/.airs-mcpserver-fs");

    let config_path = config_dir.unwrap_or_else(|| PathBuf::from(format!("{default_base}/config")));
    let logs_path = logs_dir.unwrap_or_else(|| PathBuf::from(format!("{default_base}/logs")));

    info!("📁 Configuration directory: {}", config_path.display());
    info!("📁 Logs directory: {}", logs_path.display());

    // Check if directories already exist
    if config_path.exists() && !force {
        error!(
            "❌ Configuration directory already exists: {}",
            config_path.display()
        );
        error!("   Use --force to overwrite or choose a different directory");
        process::exit(1);
    }

    if logs_path.exists() && !force {
        error!("❌ Logs directory already exists: {}", logs_path.display());
        error!("   Use --force to overwrite or choose a different directory");
        process::exit(1);
    }

    // Create directories
    if let Err(e) = fs::create_dir_all(&config_path) {
        error!("❌ Failed to create configuration directory: {}", e);
        process::exit(1);
    }
    info!(
        "✅ Created configuration directory: {}",
        config_path.display()
    );

    if let Err(e) = fs::create_dir_all(&logs_path) {
        error!("❌ Failed to create logs directory: {}", e);
        process::exit(1);
    }
    info!("✅ Created logs directory: {}", logs_path.display());

    // Create a sample configuration file
    let config_file = config_path.join("development.toml");
    if !config_file.exists() || force {
        let config_content = include_str!("../../../examples/config/development.toml");
        if let Err(e) = fs::write(&config_file, config_content) {
            error!("❌ Failed to create sample configuration: {}", e);
        } else {
            info!("✅ Created sample configuration: {}", config_file.display());
        }
    }

    info!("🎉 Setup complete!");
    info!("📖 Next steps:");
    info!(
        "   1. Edit {} to customize your settings",
        config_file.display()
    );
    info!(
        "   2. Set AIRS_MCPSERVER_FS_CONFIG_DIR={}",
        config_path.display()
    );
    info!("   3. Run: airs-mcpserver-fs serve");
    info!("   4. See CONFIGURATION.md for detailed setup instructions");

    Ok(())
}
