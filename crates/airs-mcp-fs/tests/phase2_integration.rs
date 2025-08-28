//! Integration test to verify Phase 2 STDIO transport implementation

#![allow(clippy::expect_used)]

use airs_mcp::integration::mcp::{McpServerBuilder, ToolProvider};
use airs_mcp::transport::StdioTransport;
use airs_mcp_fs::{DefaultFilesystemMcpServer, Settings};

#[tokio::test]
async fn test_phase2_mcp_server_builder_integration() {
    // Test that we can create a FilesystemMcpServer
    let settings = Settings::default();
    let filesystem_server = DefaultFilesystemMcpServer::with_default_handlers(settings)
        .await
        .expect("Failed to create filesystem server");

    // Test that we can create an STDIO transport
    let transport = StdioTransport::new()
        .await
        .expect("Failed to create STDIO transport");

    // Test that we can build the complete MCP server
    let mcp_server = McpServerBuilder::new()
        .server_info("test-airs-mcp-fs", "0.1.0")
        .with_tool_provider(filesystem_server)
        .build(transport)
        .await
        .expect("Failed to build MCP server");

    // Verify the server is not initialized yet (normal state)
    assert!(!mcp_server.is_initialized().await);

    // Test that server.run() would be callable (don't actually run it as it would block)
    // This confirms our integration is complete
    println!("✅ Phase 2 STDIO transport integration test passed!");
    println!("🎯 MCP server with FilesystemMcpServer tool provider created successfully");
}

#[tokio::test]
async fn test_phase2_filesystem_tool_provider_functionality() {
    // Create filesystem server
    let settings = Settings::default();
    let filesystem_server = DefaultFilesystemMcpServer::with_default_handlers(settings)
        .await
        .expect("Failed to create filesystem server");

    // Test the ToolProvider trait methods directly
    let tools = filesystem_server
        .list_tools()
        .await
        .expect("Failed to list tools");

    // Verify the expected tools are available
    assert_eq!(tools.len(), 3);

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"read_file"));
    assert!(tool_names.contains(&"write_file"));
    assert!(tool_names.contains(&"list_directory"));

    println!("✅ Phase 2 filesystem tool provider functionality test passed!");
    println!("📋 Available tools: {tool_names:?}");
}
