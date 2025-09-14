//! Server Setup Module
//!
//! This module handles the creation of the test environment and MCP handlers
//! for the OAuth2 integration example.

use tempfile::TempDir;

use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};
use airs_mcp::transport::adapters::http::AxumMcpRequestHandler;

/// Create test environment with filesystem resources and MCP handlers
///
/// Returns the MCP handlers and a TempDir guard that MUST be kept alive
/// for the entire lifetime of the FileSystemResourceProvider
pub async fn create_test_environment() -> Result<
    (
        AxumMcpRequestHandler<
            FileSystemResourceProvider,
            MathToolProvider,
            CodeReviewPromptProvider,
            StructuredLoggingHandler,
        >,
        TempDir,
    ),
    Box<dyn std::error::Error>,
> {
    use std::fs;
    use tracing::info;

    // Try using a persistent directory first to isolate the issue
    let persistent_test_dir = std::env::current_dir()?.join("oauth2_test_resources");

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

    // Create test files
    fs::write(
        persistent_test_dir.join("oauth2-test.txt"),
        "Hello from OAuth2 protected MCP server!",
    )?;
    fs::write(
        persistent_test_dir.join("config.json"),
        r#"{"server": "oauth2-mcp-test", "version": "1.0.0", "auth": "oauth2"}"#,
    )?;
    fs::write(
        persistent_test_dir.join("README.md"),
        "# OAuth2 MCP Integration Server\n\nThis file is accessible through OAuth2 protected resources.",
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
