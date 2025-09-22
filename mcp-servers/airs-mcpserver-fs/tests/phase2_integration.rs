//! Integration test to verify Phase 2 STDIO transport implementation

#![allow(clippy::expect_used)]

// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::McpServer;
use airs_mcp::providers::ToolProvider;
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;

// Layer 3b: Local crate modules
use airs_mcpserver_fs::mcp::FilesystemMessageHandler;
use airs_mcpserver_fs::{DefaultFilesystemMcpServer, Settings};

#[tokio::test]
async fn test_phase2_message_handler_integration() {
    // Test that we can create a FilesystemMcpServer
    let settings = Settings::default();
    let filesystem_server = DefaultFilesystemMcpServer::with_default_handlers(settings)
        .await
        .expect("Failed to create filesystem server");

    // Test that we can create a MessageHandler wrapper
    let message_handler = std::sync::Arc::new(FilesystemMessageHandler::new(std::sync::Arc::new(
        filesystem_server,
    )));

    // Test that we can build the transport with MessageHandler
    let transport = StdioTransportBuilder::new()
        .with_message_handler(message_handler)
        .build()
        .await
        .expect("Failed to build STDIO transport");

    // Test that we can create the complete MCP server
    let _mcp_server = McpServer::new(transport);

    // Test passed - we can create the complete server with new architecture
    println!("✅ Phase 2 MessageHandler integration test passed!");
    println!("🎯 MCP server with FilesystemMessageHandler created successfully");
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
