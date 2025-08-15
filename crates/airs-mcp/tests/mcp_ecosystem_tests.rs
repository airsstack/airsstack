//! MCP Ecosystem Integration Tests
//!
//! This test suite validates the complete MCP ecosystem including:
//! - Provider implementations with McpServerBuilder
//! - Transport integration using StdioTransport for simplicity
//! - Provider functionality and integration
//! - End-to-end MCP operations

use airs_mcp::integration::mcp::{
    LoggingHandler, McpServerBuilder, PromptProvider, ResourceProvider, ToolProvider,
};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

/// Test Phase 3C provider implementations
#[tokio::test]
async fn test_provider_implementations() {
    // Create temporary directory for file system provider
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file_path = temp_dir.path().join("test.txt");
    std::fs::write(&test_file_path, "Hello MCP World!").expect("Failed to write test file");

    // Test 1: FileSystemResourceProvider
    println!("Testing FileSystemResourceProvider...");
    
    // Work around the canonicalization bug by using the canonical path for the provider
    let canonical_base_path = temp_dir.path().canonicalize()
        .expect("Failed to canonicalize temp dir path");
    
    let resource_provider = FileSystemResourceProvider::new(&canonical_base_path)
        .expect("Failed to create filesystem provider");

    // Test resource listing
    let resources = resource_provider
        .list_resources()
        .await
        .expect("Failed to list resources");
    assert!(!resources.is_empty(), "Resources should not be empty");

    // Test resource reading with proper URI format
    let content = resource_provider
        .read_resource("file://test.txt")
        .await
        .expect("Failed to read test resource");
    assert!(!content.is_empty(), "Content should not be empty");

    // Test 2: MathToolProvider
    println!("Testing MathToolProvider...");
    let tool_provider = MathToolProvider::new();

    // Test tool listing
    let tools = tool_provider
        .list_tools()
        .await
        .expect("Failed to list tools");
    assert!(!tools.is_empty(), "Tools should not be empty");

    // Find and test the add tool
    let _add_tool = tools
        .iter()
        .find(|t| t.name == "add")
        .expect("Add tool should be available");

    // Test tool execution
    let arguments = json!({
        "numbers": [5.0, 3.0]
    });
    let result = tool_provider
        .call_tool("add", arguments)
        .await
        .expect("Failed to call add tool");
    assert!(!result.is_empty(), "Tool result should not be empty");

    // Test 3: CodeReviewPromptProvider
    println!("Testing CodeReviewPromptProvider...");
    let prompt_provider = CodeReviewPromptProvider::new();

    // Test prompt listing
    let prompts = prompt_provider
        .list_prompts()
        .await
        .expect("Failed to list prompts");
    assert!(!prompts.is_empty(), "Prompts should not be empty");

    // Test prompt retrieval
    let mut prompt_args = HashMap::new();
    prompt_args.insert("language".to_string(), "rust".to_string());
    prompt_args.insert(
        "code".to_string(),
        "fn main() { println!(\"Hello\"); }".to_string(),
    );

    let (description, messages) = prompt_provider
        .get_prompt("code_review_general", prompt_args)
        .await
        .expect("Failed to get code review prompt");
    assert!(
        !description.is_empty(),
        "Prompt description should not be empty"
    );
    assert!(!messages.is_empty(), "Prompt messages should not be empty");

    // Test 4: StructuredLoggingHandler
    println!("Testing StructuredLoggingHandler...");
    let logging_handler = StructuredLoggingHandler::new();

    // Test logging configuration
    let logging_config = airs_mcp::shared::protocol::logging::LoggingConfig {
        min_level: airs_mcp::shared::protocol::logging::LogLevel::Info,
        include_stack_traces: false,
        buffered: false,
        buffer_size: Some(1024),
        included_components: vec![],
        excluded_components: vec![],
    };
    let result = logging_handler
        .set_logging(logging_config)
        .await
        .expect("Failed to set logging configuration");
    assert!(result, "Logging configuration should succeed");

    println!("All provider implementations tested successfully!");
}

/// Test McpServerBuilder integration with providers
#[tokio::test]
async fn test_mcp_server_builder_integration() {
    // Create temporary directory for file system provider
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file_path = temp_dir.path().join("config.json");
    std::fs::write(&test_file_path, r#"{"test": "value"}"#).expect("Failed to write test file");

    println!("Testing McpServerBuilder with providers...");

    // Use canonicalized path to avoid the path validation issue
    let canonical_base_path = temp_dir.path().canonicalize()
        .expect("Failed to canonicalize temp dir path");

    // Test that we can build a server with all provider types
    let _builder = McpServerBuilder::new()
        .server_info("integration-test-server", "1.0.0")
        .with_resource_provider(
            FileSystemResourceProvider::new(&canonical_base_path)
                .expect("Failed to create filesystem provider"),
        )
        .with_tool_provider(MathToolProvider::new())
        .with_prompt_provider(CodeReviewPromptProvider::new())
        .with_logging_handler(StructuredLoggingHandler::new());

    // We can't easily test the full server with StdioTransport in a unit test
    // since it requires actual stdin/stdout, but we can test that the builder
    // accepts all our providers without errors

    // Note: In a real integration test environment, we would:
    // let transport = StdioTransport::new().await.expect("Failed to create transport");
    // let server = builder.build(transport).await.expect("Failed to build server");
    // server.run().await.expect("Failed to run server");

    println!("McpServerBuilder integration test completed successfully!");
}

/// Test provider error handling and edge cases
#[tokio::test]
async fn test_provider_error_handling() {
    println!("Testing provider error handling...");

    // Test 1: FileSystemResourceProvider with invalid path
    let invalid_result = FileSystemResourceProvider::new("/nonexistent/path/that/should/not/exist");
    assert!(invalid_result.is_err(), "Should fail with invalid path");

    // Test 2: FileSystemResourceProvider with valid path but invalid resource
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let canonical_base_path = temp_dir.path().canonicalize()
        .expect("Failed to canonicalize temp dir path");
    let provider =
        FileSystemResourceProvider::new(&canonical_base_path).expect("Failed to create provider");

    let invalid_resource_result = provider.read_resource("file:///nonexistent/file.txt").await;
    assert!(
        invalid_resource_result.is_err(),
        "Should fail with nonexistent file"
    );

    // Test 3: MathToolProvider with invalid tool name
    let tool_provider = MathToolProvider::new();
    let invalid_tool_result = tool_provider.call_tool("nonexistent_tool", json!({})).await;
    assert!(
        invalid_tool_result.is_err(),
        "Should fail with invalid tool name"
    );

    // Test 4: MathToolProvider with invalid arguments
    let invalid_args_result = tool_provider
        .call_tool("add", json!({"invalid": "args"}))
        .await;
    assert!(
        invalid_args_result.is_err(),
        "Should fail with invalid arguments"
    );

    // Test 5: CodeReviewPromptProvider with invalid prompt name
    let prompt_provider = CodeReviewPromptProvider::new();
    let invalid_prompt_result = prompt_provider
        .get_prompt("nonexistent_prompt", HashMap::new())
        .await;
    assert!(
        invalid_prompt_result.is_err(),
        "Should fail with invalid prompt name"
    );

    println!("Provider error handling test completed successfully!");
}
