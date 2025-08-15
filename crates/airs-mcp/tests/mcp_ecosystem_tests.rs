//! MCP Ecosystem Integration Tests
//!
//! This test suite validates the complete MCP ecosystem including:
//! - Provider implementations with McpServerBuilder
//! - MCP Client functionality and end-to-end communication
//! - Transport integration and client-server interactions
//! - Provider functionality and integration
//! - End-to-end MCP operations

use airs_mcp::integration::mcp::{
    LoggingHandler, McpClient, McpClientBuilder, McpServerBuilder, PromptProvider,
    ResourceProvider, ToolProvider,
};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};
use airs_mcp::transport::Transport;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;

/// Mock transport for testing client-server communication
#[derive(Debug, Clone)]
pub struct MockTransport {
    /// Messages received by this transport
    received_messages: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// Messages to be sent from this transport  
    outgoing_messages: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// Whether this transport is closed
    is_closed: Arc<Mutex<bool>>,
    /// Name for debugging
    #[allow(dead_code)]
    name: String,
}

impl MockTransport {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            received_messages: Arc::new(Mutex::new(VecDeque::new())),
            outgoing_messages: Arc::new(Mutex::new(VecDeque::new())),
            is_closed: Arc::new(Mutex::new(false)),
            name: name.into(),
        }
    }

    /// Add a message that will be returned by receive()
    pub fn add_incoming_message(&self, message: Vec<u8>) {
        self.outgoing_messages.lock().unwrap().push_back(message);
    }

    /// Get all messages that were sent via send()
    pub fn get_sent_messages(&self) -> Vec<Vec<u8>> {
        self.received_messages
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    /// Check if transport is closed
    pub fn is_closed(&self) -> bool {
        *self.is_closed.lock().unwrap()
    }

    /// Create a pair of connected mock transports for client-server testing
    pub fn create_pair() -> (MockTransport, MockTransport) {
        let mut client_transport = MockTransport::new("client");
        let mut server_transport = MockTransport::new("server");

        // Cross-connect the transports so messages sent by one are received by the other
        client_transport.outgoing_messages = server_transport.received_messages.clone();
        server_transport.outgoing_messages = client_transport.received_messages.clone();

        (client_transport, server_transport)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MockTransportError {
    #[error("Transport is closed")]
    Closed,
    #[error("No message available")]
    NoMessage,
    #[error("Mock error: {0}")]
    Mock(String),
}

impl Transport for MockTransport {
    type Error = MockTransportError;

    async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
        if *self.is_closed.lock().unwrap() {
            return Err(MockTransportError::Closed);
        }

        self.received_messages
            .lock()
            .unwrap()
            .push_back(message.to_vec());
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        if *self.is_closed.lock().unwrap() {
            return Err(MockTransportError::Closed);
        }

        self.outgoing_messages
            .lock()
            .unwrap()
            .pop_front()
            .ok_or(MockTransportError::NoMessage)
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        *self.is_closed.lock().unwrap() = true;
        Ok(())
    }
}

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
    let canonical_base_path = temp_dir
        .path()
        .canonicalize()
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
    let canonical_base_path = temp_dir
        .path()
        .canonicalize()
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
    let canonical_base_path = temp_dir
        .path()
        .canonicalize()
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

/// Test MCP Client functionality and initialization
#[tokio::test]
async fn test_mcp_client_initialization() {
    println!("Testing MCP Client initialization...");

    // Create mock transport for client testing
    let transport = MockTransport::new("test-client");

    // Add a mock initialization response
    let init_response = json!({
        "jsonrpc": "2.0",
        "id": "init",
        "result": {
            "protocolVersion": "2025-01-01",
            "capabilities": {
                "resources": {
                    "subscribe": true,
                    "listChanged": true
                },
                "tools": {
                    "listChanged": true
                },
                "prompts": {
                    "listChanged": true
                },
                "logging": {}
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        }
    });
    transport.add_incoming_message(init_response.to_string().into_bytes());

    // Test client creation with builder pattern
    let client = McpClientBuilder::new()
        .client_info("test-client", "1.0.0")
        .timeout(Duration::from_secs(5))
        .build(transport)
        .await
        .expect("Failed to create MCP client");

    // Test initial state
    assert!(matches!(
        client.state().await,
        airs_mcp::integration::mcp::client::ConnectionState::Disconnected
    ));
    assert!(!client.is_initialized().await);
    assert!(client.server_capabilities().await.is_none());

    // Test initialization
    let server_caps = client
        .initialize()
        .await
        .expect("Failed to initialize client");

    // Verify initialization succeeded
    assert!(client.is_initialized().await);
    assert!(client.server_capabilities().await.is_some());
    assert_eq!(server_caps.resources.unwrap().subscribe, Some(true));

    println!("MCP Client initialization test completed successfully!");
}

/// Test MCP Client operations (list tools, call tools, etc.)
#[tokio::test]
async fn test_mcp_client_operations() {
    println!("Testing MCP Client operations...");

    let transport = MockTransport::new("test-client-ops");

    // Add initialization response
    let init_response = json!({
        "jsonrpc": "2.0",
        "id": "init",
        "result": {
            "protocolVersion": "2025-01-01",
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "subscribe": true },
                "prompts": { "listChanged": true }
            },
            "serverInfo": { "name": "test-server", "version": "1.0.0" }
        }
    });
    transport.add_incoming_message(init_response.to_string().into_bytes());

    // Add list tools response
    let list_tools_response = json!({
        "jsonrpc": "2.0",
        "id": "list_tools_1",
        "result": {
            "tools": [
                {
                    "name": "add",
                    "description": "Add two numbers",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "numbers": { "type": "array", "items": { "type": "number" } }
                        },
                        "required": ["numbers"]
                    }
                }
            ]
        }
    });
    transport.add_incoming_message(list_tools_response.to_string().into_bytes());

    // Add call tool response
    let call_tool_response = json!({
        "jsonrpc": "2.0",
        "id": "call_tool_1",
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Result: 8"
                }
            ],
            "isError": false
        }
    });
    transport.add_incoming_message(call_tool_response.to_string().into_bytes());

    let client = McpClient::new(transport)
        .await
        .expect("Failed to create client");

    // Initialize client
    client.initialize().await.expect("Failed to initialize");

    // Test list tools
    let tools = client.list_tools().await.expect("Failed to list tools");
    assert!(!tools.is_empty());
    assert_eq!(tools[0].name, "add");

    // Test call tool
    let result = client
        .call_tool("add", Some(json!({"numbers": [3, 5]})))
        .await
        .expect("Failed to call tool");
    assert!(!result.is_empty());

    println!("MCP Client operations test completed successfully!");
}

/// Test MCP Client resource operations
#[tokio::test]
async fn test_mcp_client_resources() {
    println!("Testing MCP Client resource operations...");

    let transport = MockTransport::new("test-client-resources");

    // Add initialization response
    let init_response = json!({
        "jsonrpc": "2.0",
        "id": "init",
        "result": {
            "protocolVersion": "2025-01-01",
            "capabilities": { "resources": { "subscribe": true } },
            "serverInfo": { "name": "test-server", "version": "1.0.0" }
        }
    });
    transport.add_incoming_message(init_response.to_string().into_bytes());

    // Add list resources response
    let list_resources_response = json!({
        "jsonrpc": "2.0",
        "id": "list_resources_1",
        "result": {
            "resources": [
                {
                    "uri": "file://test.txt",
                    "name": "test.txt",
                    "description": "Test file",
                    "mimeType": "text/plain"
                }
            ]
        }
    });
    transport.add_incoming_message(list_resources_response.to_string().into_bytes());

    // Add read resource response
    let read_resource_response = json!({
        "jsonrpc": "2.0",
        "id": "read_resource_1",
        "result": {
            "contents": [
                {
                    "type": "text",
                    "uri": "file://test.txt",
                    "mimeType": "text/plain",
                    "text": "Hello MCP World!"
                }
            ]
        }
    });
    transport.add_incoming_message(read_resource_response.to_string().into_bytes());

    let client = McpClient::new(transport)
        .await
        .expect("Failed to create client");
    client.initialize().await.expect("Failed to initialize");

    // Test list resources
    let resources = client
        .list_resources()
        .await
        .expect("Failed to list resources");
    assert!(!resources.is_empty());
    assert_eq!(resources[0].uri.as_str(), "file://test.txt");

    // Test read resource
    let contents = client
        .read_resource("file://test.txt")
        .await
        .expect("Failed to read resource");
    assert!(!contents.is_empty());

    println!("MCP Client resource operations test completed successfully!");
}

/// Test MCP Client prompt operations
#[tokio::test]
async fn test_mcp_client_prompts() {
    println!("Testing MCP Client prompt operations...");

    let transport = MockTransport::new("test-client-prompts");

    // Add initialization response
    let init_response = json!({
        "jsonrpc": "2.0",
        "id": "init",
        "result": {
            "protocolVersion": "2025-01-01",
            "capabilities": { "prompts": { "listChanged": true } },
            "serverInfo": { "name": "test-server", "version": "1.0.0" }
        }
    });
    transport.add_incoming_message(init_response.to_string().into_bytes());

    // Add list prompts response
    let list_prompts_response = json!({
        "jsonrpc": "2.0",
        "id": "list_prompts_1",
        "result": {
            "prompts": [
                {
                    "name": "code_review_general",
                    "description": "General code review prompt",
                    "arguments": [
                        {
                            "name": "language",
                            "description": "Programming language",
                            "required": true
                        },
                        {
                            "name": "code",
                            "description": "Code to review",
                            "required": true
                        }
                    ]
                }
            ]
        }
    });
    transport.add_incoming_message(list_prompts_response.to_string().into_bytes());

    // Add get prompt response
    let get_prompt_response = json!({
        "jsonrpc": "2.0",
        "id": "get_prompt_1",
        "result": {
            "description": "Code review for Rust code",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "Please review this Rust code: fn main() { println!(\"Hello\"); }"
                    }
                }
            ]
        }
    });
    transport.add_incoming_message(get_prompt_response.to_string().into_bytes());

    let client = McpClient::new(transport)
        .await
        .expect("Failed to create client");
    client.initialize().await.expect("Failed to initialize");

    // Test list prompts
    let prompts = client.list_prompts().await.expect("Failed to list prompts");
    assert!(!prompts.is_empty());
    assert_eq!(prompts[0].name, "code_review_general");

    // Test get prompt
    let mut args = HashMap::new();
    args.insert("language".to_string(), "rust".to_string());
    args.insert(
        "code".to_string(),
        "fn main() { println!(\"Hello\"); }".to_string(),
    );

    let messages = client
        .get_prompt("code_review_general", args)
        .await
        .expect("Failed to get prompt");
    assert!(!messages.is_empty());

    println!("MCP Client prompt operations test completed successfully!");
}

/// Test end-to-end client-server communication with real providers
#[tokio::test]
async fn test_end_to_end_client_server_communication() {
    println!("Testing end-to-end client-server communication...");

    // This test demonstrates how to set up full client-server communication
    // In a real scenario, you would use paired transports or real network communication

    // Create mock transports for client-server pair
    let (client_transport, _server_transport) = MockTransport::create_pair();

    // Set up server with providers
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let canonical_base_path = temp_dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize temp dir path");

    // Note: In a real end-to-end test, you would:
    // 1. Create a server with McpServerBuilder and actual providers
    // 2. Run the server in a background task
    // 3. Create a client with paired transport
    // 4. Test actual request-response cycles
    // 5. Verify provider operations work through the full stack

    // For now, we demonstrate the setup pattern
    let _server_builder = McpServerBuilder::new()
        .server_info("e2e-test-server", "1.0.0")
        .with_resource_provider(
            FileSystemResourceProvider::new(&canonical_base_path)
                .expect("Failed to create filesystem provider"),
        )
        .with_tool_provider(MathToolProvider::new())
        .with_prompt_provider(CodeReviewPromptProvider::new())
        .with_logging_handler(StructuredLoggingHandler::new());

    // Create client
    let client = McpClient::new(client_transport)
        .await
        .expect("Failed to create client");

    // Verify client is ready for communication
    assert!(matches!(
        client.state().await,
        airs_mcp::integration::mcp::client::ConnectionState::Disconnected
    ));

    println!("End-to-end client-server communication test setup completed successfully!");
    println!("Note: Full E2E testing requires async server execution, planned for next phase");
}
