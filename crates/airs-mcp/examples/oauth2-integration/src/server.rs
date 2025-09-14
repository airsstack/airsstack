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
    // Create temporary directory for filesystem provider
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    std::fs::write(
        temp_path.join("oauth2-test.txt"),
        "Hello from OAuth2 protected MCP server!",
    )?;
    std::fs::write(
        temp_path.join("config.json"),
        r#"{"server": "oauth2-mcp-test", "version": "1.0.0", "auth": "oauth2"}"#,
    )?;
    std::fs::write(
        temp_path.join("README.md"),
        "# OAuth2 MCP Integration Server\n\nThis file is accessible through OAuth2 protected resources.",
    )?;

    // Create MCP handlers with providers
    let resource_provider = FileSystemResourceProvider::new(&temp_path.canonicalize()?)
        .expect("Failed to create filesystem provider");
    let tool_provider = MathToolProvider::new();
    let prompt_provider = CodeReviewPromptProvider::new();
    let logging_handler = StructuredLoggingHandler::new();

    let handlers = AxumMcpRequestHandler::new(
        Some(resource_provider),
        Some(tool_provider),
        Some(prompt_provider),
        Some(logging_handler),
    );

    Ok((handlers, temp_dir))
}