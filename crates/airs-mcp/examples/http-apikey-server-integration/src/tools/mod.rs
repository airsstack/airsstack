//! Standardized Tool Providers
//!
//! This module provides the standardized tool set for HTTP API Key server examples:
//! - **file_ops**: File operations (read, write, list, create directories)
//! - **system_info**: System information (OS, environment, processes)
//! - **utilities**: Utility functions (echo, timestamp, health check)

// Layer 1: Standard library imports
use std::fs;

// Layer 2: Third-party crate imports
use tempfile::TempDir;
use tracing::info;

// Layer 3: Internal module imports
use airs_mcp::{
    providers::{
        CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
        StructuredLoggingHandler,
    },
    transport::adapters::http::AxumMcpRequestHandler,
};

pub mod file_ops;
pub mod system_info;
pub mod utilities;

/// Type alias for the MCP handlers used in this server
/// 
/// This simplifies the complex generic type signature for AxumMcpRequestHandler
/// with our specific provider combination.
pub type McpHandlers = AxumMcpRequestHandler<
    FileSystemResourceProvider,
    MathToolProvider,
    CodeReviewPromptProvider,
    StructuredLoggingHandler,
>;

/// Create test environment with filesystem resources and MCP handlers
///
/// Returns the MCP handlers and a TempDir guard that MUST be kept alive
/// for the entire lifetime of the FileSystemResourceProvider
pub async fn create_test_environment() -> Result<(McpHandlers, TempDir), Box<dyn std::error::Error>> {
    // Try using a persistent directory first for HTTP API Key testing
    let persistent_test_dir = std::env::current_dir()?.join("test_resources");

    // Clean up any existing test directory
    if persistent_test_dir.exists() {
        fs::remove_dir_all(&persistent_test_dir)?;
    }

    // Create the test directory
    fs::create_dir_all(&persistent_test_dir)?;

    info!(
        "📂 Created persistent test directory: {}",
        persistent_test_dir.display()
    );
    info!(
        "📂 Persistent test directory exists: {}",
        persistent_test_dir.exists()
    );

    // Create test files for HTTP API key server
    fs::write(
        persistent_test_dir.join("api-info.txt"),
        "Hello from HTTP API Key protected MCP server! This file demonstrates authenticated resource access.",
    )?;
    fs::write(
        persistent_test_dir.join("server-config.json"),
        r#"{"server": "http-apikey-mcp-server", "version": "1.0.0", "transport": "http", "auth": "api-key"}"#,
    )?;
    fs::write(
        persistent_test_dir.join("README.md"),
        "# HTTP API Key MCP Server\n\nThis file is accessible through the authenticated filesystem resources provider.\n\n## Authentication\n\nAll requests require a valid API key.",
    )?;

    // Create examples subdirectory
    let examples_dir = persistent_test_dir.join("examples");
    fs::create_dir_all(&examples_dir)?;

    fs::write(
        examples_dir.join("authentication.txt"),
        "Example of API key authentication:\n\n1. X-API-Key header\n2. Authorization Bearer\n3. Query parameter",
    )?;

    info!("📂 Created test files in persistent directory");

    // Verify files exist
    let files: Vec<_> = fs::read_dir(&persistent_test_dir)?.collect();
    info!("📂 Files in persistent directory: {} files", files.len());

    // Create the resource provider using the persistent directory
    let canonical_path = persistent_test_dir.canonicalize()?;
    info!("📂 Canonical path: {}", canonical_path.display());
    info!("📂 Canonical path exists: {}", canonical_path.exists());

    let resource_provider = FileSystemResourceProvider::new(&canonical_path)
        .expect("Failed to create filesystem provider");
    let tool_provider = MathToolProvider::new();
    let prompt_provider = CodeReviewPromptProvider::new();
    let logging_handler = StructuredLoggingHandler::new();

    // Create a dummy TempDir for the return value (it won't be used for cleanup)
    let dummy_temp_dir = TempDir::new()?;

    let handlers = AxumMcpRequestHandler::new(
        Some(resource_provider),
        Some(tool_provider),
        Some(prompt_provider),
        Some(logging_handler),
    );

    // Return handlers and dummy temp dir
    Ok((handlers, dummy_temp_dir))
}