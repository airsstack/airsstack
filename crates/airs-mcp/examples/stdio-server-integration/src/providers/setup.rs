//! Provider Setup and Test Environment
//!
//! This module contains functions for creating and configuring all MCP providers
//! and setting up test environments.

// Layer 1: Standard library imports
use std::fs;

// Layer 2: Third-party crate imports
use tempfile::TempDir;
use tracing::info;

// Layer 3: Internal module imports
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};

use crate::handlers::McpHandler;

/// Create test environment with filesystem resources for STDIO server
///
/// Creates a test directory with sample files and returns the configured
/// MCP message handler with all providers enabled.
///
/// # Returns
///
/// A tuple containing:
/// - `McpHandler`: Configured handler with all providers
/// - `Option<TempDir>`: Optional temporary directory (None for persistent directory)
///
/// # Errors
///
/// Returns an error if:
/// - Failed to create test directory
/// - Failed to create test files
/// - Failed to initialize providers
pub async fn create_test_environment(
) -> Result<(McpHandler, Option<TempDir>), Box<dyn std::error::Error>> {
    // Create persistent test directory instead of temporary one
    // This ensures files persist between server invocations (important for STDIO one-shot pattern)
    let test_path = std::env::current_dir()?.join("test_resources");

    // Create directory if it doesn't exist
    if !test_path.exists() {
        fs::create_dir_all(&test_path)?;
        info!(
            "📂 Created persistent test directory: {}",
            test_path.display()
        );
    } else {
        info!("📂 Using existing test directory: {}", test_path.display());
    }

    // Create test files if they don't exist (avoid overwriting)
    let text_file = test_path.join("stdio-test.txt");
    if !text_file.exists() {
        fs::write(
            &text_file,
            "Hello from STDIO MCP server! This file demonstrates resource access.",
        )?;
    }

    let config_file = test_path.join("config.json");
    if !config_file.exists() {
        fs::write(
            &config_file,
            r#"{"server": "stdio-mcp-server", "version": "1.0.0", "transport": "stdio"}"#,
        )?;
    }

    let readme_file = test_path.join("README.md");
    if !readme_file.exists() {
        fs::write(
            &readme_file,
            "# STDIO MCP Server\n\nThis file is accessible through the filesystem resources provider.",
        )?;
    }

    let yaml_file = test_path.join("sample.yaml");
    if !yaml_file.exists() {
        fs::write(
            &yaml_file,
            "# Sample YAML file\nname: stdio-test\nversion: 1.0\ntools:\n  - math\n  - system\n  - text",
        )?;
    }

    // Create subdirectory with more files
    let subdir = test_path.join("examples");
    if !subdir.exists() {
        fs::create_dir_all(&subdir)?;
    }

    let example_file = subdir.join("example.txt");
    if !example_file.exists() {
        fs::write(&example_file, "This is an example file in a subdirectory.")?;
    }

    info!("📂 Verified test files in directory");

    // Verify files exist
    let files: Vec<_> = fs::read_dir(&test_path)?.filter_map(|e| e.ok()).collect();
    info!("📂 Files in test directory: {} files", files.len());

    // Get canonical path for resource provider
    let canonical_path = test_path.canonicalize()?;
    info!("📂 Canonical path: {}", canonical_path.display());

    // Create all providers following the modular pattern
    let resource_provider = FileSystemResourceProvider::new(&canonical_path)
        .expect("Failed to create filesystem resource provider");
    let tool_provider = MathToolProvider::new();
    let prompt_provider = CodeReviewPromptProvider::new();
    let logging_handler = StructuredLoggingHandler::new();

    info!("📦 Created all MCP providers");

    // Create MCP message handler
    let handler = McpHandler::new(
        resource_provider,
        tool_provider,
        prompt_provider,
        logging_handler,
    );

    // Return None for temp_dir since we're using a persistent directory
    Ok((handler, None))
}