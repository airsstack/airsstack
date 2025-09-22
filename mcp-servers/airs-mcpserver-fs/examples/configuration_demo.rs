//! Configuration testing example for AIRS MCP-FS
//!
//! This example demonstrates the new configuration management system including:
//! - Multi-environment configuration loading
//! - Environment variable overrides
//! - Configuration validation
//! - Different configuration formats

#![allow(clippy::uninlined_format_args)]

use airs_mcpserver_fs::config::{ConfigEnvironment, ConfigurationLoader, Settings};
use std::env;

fn main() -> anyhow::Result<()> {
    println!("🔧 AIRS MCP-FS Configuration Management Demo");
    println!("===========================================\n");

    // 1. Demonstrate environment detection
    demo_environment_detection()?;

    // 2. Demonstrate configuration loading with different environments
    demo_environment_specific_loading()?;

    // 3. Demonstrate environment variable overrides
    demo_environment_variable_overrides()?;

    // 4. Demonstrate configuration validation
    demo_configuration_validation()?;

    // 5. Demonstrate loading from specific files
    demo_file_specific_loading()?;

    println!("\n✅ Configuration management demo completed successfully!");
    Ok(())
}

fn demo_environment_detection() -> anyhow::Result<()> {
    println!("📍 Environment Detection Demo");
    println!("-----------------------------");

    // Show current environment detection
    let current_env = ConfigEnvironment::detect();
    println!("Detected environment: {}", current_env.as_str());
    println!("Config filename: {}", current_env.config_filename());

    // Show different detection scenarios
    println!("\nEnvironment variable scenarios:");

    let scenarios = [
        ("AIRS_MCP_FS_ENV", "production"),
        ("NODE_ENV", "development"),
        ("ENVIRONMENT", "staging"),
    ];

    for (var_name, var_value) in scenarios {
        println!("  {} = {} → {}", var_name, var_value, {
            env::set_var(var_name, var_value);
            let env = ConfigEnvironment::detect();
            env::remove_var(var_name);
            env.as_str()
        });
    }

    println!();
    Ok(())
}

fn demo_environment_specific_loading() -> anyhow::Result<()> {
    println!("🌍 Environment-Specific Configuration Loading");
    println!("---------------------------------------------");

    let environments = [
        ConfigEnvironment::Development,
        ConfigEnvironment::Staging,
        ConfigEnvironment::Production,
    ];

    for env in environments {
        println!("\n📋 Loading {} configuration:", env.as_str());

        let loader =
            ConfigurationLoader::with_environment(env).with_config_dir("./examples/config");

        match loader.load() {
            Ok((settings, source_info)) => {
                println!("  ✅ Loaded successfully");
                println!("     Environment: {}", source_info.environment);
                println!("     Config files: {:?}", source_info.files);
                println!(
                    "     Environment vars: {} overrides",
                    source_info.env_vars.len()
                );
                println!("     Server name: {}", settings.server.name);
                println!(
                    "     Max file size: {} bytes",
                    settings.binary.max_file_size
                );
                println!(
                    "     Write requires policy: {}",
                    settings.security.operations.write_requires_policy
                );
                println!(
                    "     Security policies: {} defined",
                    settings.security.policies.len()
                );
            }
            Err(e) => {
                println!("  ⚠️  Failed to load: {}", e);
                println!("     (This is normal if config files don't exist)");
            }
        }
    }

    println!();
    Ok(())
}

fn demo_environment_variable_overrides() -> anyhow::Result<()> {
    println!("🔀 Environment Variable Overrides Demo");
    println!("--------------------------------------");

    // Set some environment variable overrides
    env::set_var("AIRS_MCP_FS_SERVER__NAME", "env-override-server");
    env::set_var("AIRS_MCP_FS_BINARY__MAX_FILE_SIZE", "999999");
    env::set_var(
        "AIRS_MCP_FS_SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY",
        "true",
    );

    println!("Setting environment variables:");
    println!("  AIRS_MCP_FS_SERVER__NAME = env-override-server");
    println!("  AIRS_MCP_FS_BINARY__MAX_FILE_SIZE = 999999");
    println!("  AIRS_MCP_FS_SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY = true");

    let loader = ConfigurationLoader::with_environment(ConfigEnvironment::Development)
        .with_config_dir("./examples/config");

    match loader.load() {
        Ok((settings, source_info)) => {
            println!("\n✅ Configuration loaded with overrides:");
            println!("  Server name: {} (overridden)", settings.server.name);
            println!(
                "  Max file size: {} bytes (overridden)",
                settings.binary.max_file_size
            );
            println!(
                "  Write requires policy: {} (overridden)",
                settings.security.operations.write_requires_policy
            );
            println!("  Environment variables used: {:?}", source_info.env_vars);
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
        }
    }

    // Clean up environment variables
    env::remove_var("AIRS_MCP_FS_SERVER__NAME");
    env::remove_var("AIRS_MCP_FS_BINARY__MAX_FILE_SIZE");
    env::remove_var("AIRS_MCP_FS_SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY");

    println!();
    Ok(())
}

fn demo_configuration_validation() -> anyhow::Result<()> {
    println!("✅ Configuration Validation Demo");
    println!("--------------------------------");

    let config_files = [
        ("Base config", "./examples/config/config.toml"),
        ("Development", "./examples/config/development.toml"),
        ("Production", "./examples/config/production.toml"),
        ("Staging", "./examples/config/staging.toml"),
    ];

    for (name, path) in config_files {
        println!("\n🔍 Validating {} ({})", name, path);

        match ConfigurationLoader::validate_file(path) {
            Ok(issues) => {
                if issues.is_empty() {
                    println!("  ✅ No validation issues found");
                } else {
                    println!("  ⚠️  Found {} validation issue(s):", issues.len());
                    for issue in issues.iter().take(3) {
                        // Show first 3 issues
                        println!("     - {}", issue);
                    }
                    if issues.len() > 3 {
                        println!("     ... and {} more", issues.len() - 3);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Validation failed: {}", e);
                println!("     (This is normal if the file doesn't exist)");
            }
        }
    }

    println!();
    Ok(())
}

fn demo_file_specific_loading() -> anyhow::Result<()> {
    println!("📄 File-Specific Loading Demo");
    println!("-----------------------------");

    // Try to load the production configuration specifically
    match ConfigurationLoader::load_from_file("./examples/config/production.toml") {
        Ok(settings) => {
            println!("✅ Loaded production.toml directly:");
            println!("  Server: {}", settings.server.name);
            println!("  Max file size: {} bytes", settings.binary.max_file_size);
            println!(
                "  Allowed paths: {} patterns",
                settings.security.filesystem.allowed_paths.len()
            );
            println!(
                "  Denied paths: {} patterns",
                settings.security.filesystem.denied_paths.len()
            );
            println!("  Security policies: {}", settings.security.policies.len());

            // Show some example paths
            if !settings.security.filesystem.allowed_paths.is_empty() {
                println!(
                    "  Example allowed path: {}",
                    settings.security.filesystem.allowed_paths[0]
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to load production.toml: {}", e);
            println!("   (This is normal if the example config files don't exist)");
        }
    }

    // Demonstrate the default Settings::load() method
    println!("\n🔄 Using Settings::load() (current method):");
    match Settings::load() {
        Ok(settings) => {
            println!("  ✅ Settings loaded successfully");
            println!("     Server: {}", settings.server.name);
            println!("     Environment: Auto-detected");
        }
        Err(e) => {
            println!("  ❌ Failed to load settings: {}", e);
        }
    }

    println!();
    Ok(())
}
