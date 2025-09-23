//! Config command handler for AIRS MCP-FS
//!
//! Handles generation of example configuration files for different environments.

// Layer 1: Standard library imports
use std::fs;
use std::path::PathBuf;
use std::process;

// Layer 2: Third-party crate imports
use anyhow::Result;
use tracing::{error, info};

// Layer 3: Internal module imports
// (None needed for config generation)

/// Handle the config command - generate configuration files for specified environment
pub async fn handle_config(output: PathBuf, env: &str, force: bool) -> Result<()> {
    info!("🔧 Generating configuration files for environment: {}", env);

    // Create output directory if it doesn't exist
    if !output.exists() {
        fs::create_dir_all(&output)?;
        info!("📁 Created directory: {}", output.display());
    }

    let config_content = match env {
        "development" => include_str!("../../../examples/config/development.toml"),
        "staging" => include_str!("../../../examples/config/staging.toml"),
        "production" => include_str!("../../../examples/config/production.toml"),
        _ => include_str!("../../../examples/config/config.toml"),
    };

    let config_file = output.join(format!("{env}.toml"));

    if config_file.exists() && !force {
        error!(
            "❌ Configuration file already exists: {}",
            config_file.display()
        );
        error!("   Use --force to overwrite or choose a different output directory");
        process::exit(1);
    }

    fs::write(&config_file, config_content)?;
    info!("✅ Generated configuration: {}", config_file.display());

    // Also copy the base config.toml if generating development
    if env == "development" {
        let base_config = output.join("config.toml");
        if !base_config.exists() || force {
            let base_content = include_str!("../../../examples/config/config.toml");
            fs::write(&base_config, base_content)?;
            info!("✅ Generated base configuration: {}", base_config.display());
        }
    }

    info!("🎉 Configuration generation complete!");
    info!("📖 Next steps:");
    info!(
        "   1. Edit {} to customize your settings",
        config_file.display()
    );
    info!("   2. Update allowed_paths for your specific use case");
    info!("   3. Run: airs-mcpserver-fs serve");
    info!("   4. See CONFIGURATION.md for detailed setup instructions");

    Ok(())
}
