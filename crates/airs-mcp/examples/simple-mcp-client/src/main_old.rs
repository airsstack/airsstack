//! Simple MCP Client Example with Real Server Interaction
//!
//! This example demonstrates actual client ↔ server communication using the 
//! AIRS MCP client library with a subprocess transport. You'll see real 
//! interactions through the high-level MCP client API.
//!
//! # Usage
//!
//! To test with the simple-mcp-server example:
//! ```bash
//! # Build the server first
//! cd ../simple-mcp-server && cargo build
//!
//! # Run this client (automatically spawns and connects to server)
//! cd ../simple-mcp-client
//! cargo run -- --server-path ../simple-mcp-server/target/debug/simple-mcp-server
//! ```
//!
//! To test with any other MCP server:
//! ```bash
//! cargo run -- --server-path /path/to/your/mcp-server
//! ```

use airs_mcp::integration::mcp::{McpClient, McpClientBuilder, McpError};
use airs_mcp::transport::{Transport, TransportError};
use async_trait::async_trait;
use serde_json::json;
use std::env;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

/// A transport that spawns and communicates with an MCP server subprocess
/// This implements the Transport trait so it can be used with McpClient
#[derive(Debug)]
pub struct SubprocessTransport {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
}

impl SubprocessTransport {
    /// Spawn a new MCP server and create a transport to communicate with it
    pub async fn spawn_server(server_path: impl AsRef<Path>) -> Result<Self, TransportError> {
        info!("🚀 Spawning MCP server: {}", server_path.as_ref().display());
        
        let mut child = Command::new(server_path.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Capture stderr to prevent interference
            .spawn()
            .map_err(|e| TransportError::connection_failed(format!("Failed to spawn server process: {}", e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| TransportError::connection_failed("Failed to get stdin handle from server process"))?;
        let stdout = BufReader::new(child.stdout.take()
            .ok_or_else(|| TransportError::connection_failed("Failed to get stdout handle from server process"))?);

        info!("✅ Server process spawned successfully (PID: {})", child.id().unwrap_or(0));

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Shutdown the server process
    pub async fn shutdown(mut self) -> Result<(), TransportError> {
        info!("� Shutting down server process...");
        
        // Close stdin to signal shutdown
        drop(self.stdin);
        
        // Wait for process to exit gracefully
        match tokio::time::timeout(Duration::from_secs(5), self.child.wait()).await {
            Ok(Ok(status)) => {
                info!("✅ Server process exited with status: {}", status);
            }
            Ok(Err(e)) => {
                warn!("⚠️  Error waiting for server process: {}", e);
            }
            Err(_) => {
                warn!("⚠️  Server process didn't exit gracefully, killing it...");
                let _ = self.child.kill().await;
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Transport for SubprocessTransport {
    type Error = TransportError;

    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        debug!("� Sending: {}", String::from_utf8_lossy(data));
        
        self.stdin.write_all(data).await
            .map_err(|e| TransportError::send_failed(format!("Failed to send data: {}", e)))?;
        self.stdin.write_all(b"\n").await
            .map_err(|e| TransportError::send_failed(format!("Failed to send newline: {}", e)))?;
        self.stdin.flush().await
            .map_err(|e| TransportError::send_failed(format!("Failed to flush: {}", e)))?;
        
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        let mut line = String::new();
        let bytes_read = self.stdout.read_line(&mut line).await
            .map_err(|e| TransportError::receive_failed(format!("Failed to read line: {}", e)))?;
        
        if bytes_read == 0 {
            return Err(TransportError::connection_closed());
        }

        // Remove trailing newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        debug!("📥 Received: {}", line);
        Ok(line.into_bytes())
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // For subprocess, we'll just close stdin which should signal the server to shutdown
        // The actual cleanup happens in shutdown() method
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see both our messages and library debug info
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // Use INFO to see our messages without too much noise
        .init();

    // Get server path from command line or use default
    let args: Vec<String> = env::args().collect();
    let server_path = if args.len() > 2 && args[1] == "--server-path" {
        args[2].clone()
    } else {
        // Default to simple-mcp-server example
        "../simple-mcp-server/target/debug/simple-mcp-server".to_string()
    };

    info!("🚀 Starting MCP Client Example using AIRS MCP Library");
    info!("📍 Server path: {}", server_path);

    // Verify the server executable exists
    if !Path::new(&server_path).exists() {
        error!("❌ Server executable not found: {}", server_path);
        error!("💡 Please build the server first with: cd ../simple-mcp-server && cargo build");
        return Err(format!("Server executable not found: {}", server_path).into());
    }

    // Create the subprocess transport
    let transport = SubprocessTransport::spawn_server(&server_path).await?;

    // Build MCP client using our custom transport
    info!("🔗 Creating MCP client with subprocess transport...");
    let client = McpClientBuilder::new()
        .client_info("simple-mcp-client", "0.1.0")
        .timeout(Duration::from_secs(30))
        .auto_retry(true, 3)
        .build(transport)
        .await?;

    info!("✅ MCP client created successfully using AIRS library");

    // Initialize connection with the server using the high-level API
    info!("🤝 Initializing MCP connection...");
    match client.initialize().await {
        Ok(server_capabilities) => {
            info!("✅ Connected to MCP server successfully!");
            info!("🎯 Server capabilities received: {:#?}", server_capabilities);
        }
        Err(McpError::Integration(source)) => {
            error!("❌ Integration error during initialization: {}", source);
            return Err(source.into());
        }
        Err(McpError::Protocol(protocol_err)) => {
            error!("❌ Protocol error during initialization: {}", protocol_err);
            return Err(format!("Protocol error: {}", protocol_err).into());
        }
        Err(e) => {
            error!("❌ Initialization failed: {}", e);
            return Err(e.into());
        }
    }

    // Now demonstrate all MCP operations using the high-level client API
    info!("🔄 Testing MCP operations using AIRS client library...");
    if let Err(e) = test_mcp_operations(&client).await {
        error!("❌ MCP operations failed: {}", e);
    }

    // The client will be dropped here, which handles cleanup
    info!("✅ MCP client example completed successfully!");
    Ok(())
}

/// Test various MCP operations using the high-level AIRS MCP client
async fn test_mcp_operations(
    client: &McpClient<SubprocessTransport>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 Testing MCP operations through AIRS client library...");

    // Test 1: List available resources
    info!("\n📂 Step 1: Discovering Resources");
    match client.list_resources().await {
        Ok(resources) => {
            if resources.is_empty() {
                info!("   No resources available");
            } else {
                info!("   ✅ Found {} resource(s):", resources.len());
                for resource in &resources {
                    info!("      • {} ({})", resource.name, resource.uri);
                    if let Some(description) = &resource.description {
                        info!("        Description: {}", description);
                    }
                }

                // Try to read the first resource using the client library
                if let Some(resource) = resources.first() {
                    info!("\n📖 Reading resource using AIRS client: {}", resource.uri);
                    match client.read_resource(resource.uri.to_string()).await {
                        Ok(content) => {
                            info!("   ✅ Resource content received:");
                            for content_item in content {
                                if let Some(text) = content_item.as_text() {
                                    let preview = text.chars().take(100).collect::<String>();
                                    info!("      📄 Content: {}", preview);
                                    if text.len() > 100 {
                                        info!("         ... ({} more characters)", text.len() - 100);
                                    }
                                }
                            }
                        }
                        Err(e) => warn!("   ⚠️  Failed to read resource: {}", e),
                    }
                }
            }
        }
        Err(e) => warn!("❌ Failed to list resources: {}", e),
    }

    // Test 2: List and call tools
    info!("\n🔧 Step 2: Discovering and Testing Tools");
    match client.list_tools().await {
        Ok(tools) => {
            if tools.is_empty() {
                info!("   No tools available");
            } else {
                info!("   ✅ Found {} tool(s):", tools.len());
                for tool in &tools {
                    info!("      • {}", tool.name);
                    if let Some(description) = &tool.description {
                        info!("        Description: {}", description);
                    }
                }

                // Try to call the first tool using the client library
                if let Some(tool) = tools.first() {
                    info!("\n⚙️  Calling tool using AIRS client: {}", tool.name);
                    let sample_input = match tool.name.as_str() {
                        "add" => json!({"a": 15, "b": 27}),
                        "greet" => json!({"name": "AIRS MCP Client"}),
                        _ => json!({"input": "test data"}),
                    };

                    match client.call_tool(&tool.name, Some(sample_input)).await {
                        Ok(result) => {
                            info!("   ✅ Tool execution successful:");
                            for content_item in result {
                                if let Some(text) = content_item.as_text() {
                                    info!("      🎯 Result: {}", text);
                                }
                            }
                        }
                        Err(e) => warn!("   ⚠️  Failed to call tool: {}", e),
                    }
                }
            }
        }
        Err(e) => warn!("❌ Failed to list tools: {}", e),
    }

    // Test 3: List and get prompts
    info!("\n💡 Step 3: Discovering and Testing Prompts");
    match client.list_prompts().await {
        Ok(prompts) => {
            if prompts.is_empty() {
                info!("   No prompts available");
            } else {
                info!("   ✅ Found {} prompt(s):", prompts.len());
                for prompt in &prompts {
                    info!("      • {}", prompt.name);
                    if let Some(description) = &prompt.description {
                        info!("        Description: {}", description);
                    }
                }

                // Try to get the first prompt using the client library
                if let Some(prompt) = prompts.first() {
                    info!("\n📝 Getting prompt using AIRS client: {}", prompt.name);
                    let mut sample_args = std::collections::HashMap::new();
                    sample_args.insert("language".to_string(), "rust".to_string());
                    sample_args.insert("code".to_string(), "fn main() { println!(\"Hello MCP!\"); }".to_string());

                    match client.get_prompt(&prompt.name, sample_args).await {
                        Ok(messages) => {
                            info!("   ✅ Prompt generated successfully:");
                            for message in messages {
                                info!("      📋 Role: {:?}", message.role);
                                if let Some(text) = message.content.as_text() {
                                    let preview = text.chars().take(150).collect::<String>();
                                    info!("      📄 Content: {}", preview);
                                    if text.len() > 150 {
                                        info!("         ... ({} more characters)", text.len() - 150);
                                    }
                                }
                            }
                        }
                        Err(e) => warn!("   ⚠️  Failed to get prompt: {}", e),
                    }
                }
            }
        }
        Err(e) => warn!("❌ Failed to list prompts: {}", e),
    }

    // Test 4: Check client state and capabilities using AIRS client
    info!("\n🔍 Step 4: Checking Client State");
    let state = client.state().await;
    let is_initialized = client.is_initialized().await;
    info!("   📊 Connection state: {:?}", state);
    info!("   🔗 Is initialized: {}", is_initialized);

    if let Some(server_caps) = client.server_capabilities().await {
        info!("   ✅ Server capabilities available:");
        info!("      📂 Resources: {}", server_caps.resources.is_some());
        info!("      🔧 Tools: {}", server_caps.tools.is_some());
        info!("      💡 Prompts: {}", server_caps.prompts.is_some());
        info!("      📝 Logging: {}", server_caps.logging.is_some());
    } else {
        info!("   ⚠️  Server capabilities: not available");
    }

    info!("\n🎉 All MCP operations completed using AIRS client library!");
    info!("💡 This demonstrates the high-level API hiding the JSON-RPC complexity");
    Ok(())
}
    info!("🔄 Starting MCP Protocol Demonstration");
    info!("💡 You'll see the actual JSON-RPC messages exchanged between client and server");
    
    // Step 1: Initialize the MCP connection
    info!("\n📋 Step 1: MCP Initialization Handshake");
    
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": "init-1",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "resources": {},
                "tools": {},
                "prompts": {}
            },
            "clientInfo": {
                "name": "simple-mcp-client",
                "version": "0.1.0"
            }
        }
    });
    
    transport.send_message(&init_request.to_string()).await?;
    let init_response = transport.receive_message().await?;
    
    info!("✅ Initialization successful! Server responded with capabilities.");
    
    // Parse the response to extract server capabilities
    let response: serde_json::Value = serde_json::from_str(&init_response)?;
    if let Some(capabilities) = response["result"]["capabilities"].as_object() {
        info!("🎯 Server Capabilities:");
        if capabilities.contains_key("resources") {
            info!("   ✓ Resources (file access, data sources)");
        }
        if capabilities.contains_key("tools") {
            info!("   ✓ Tools (executable functions)");
        }
        if capabilities.contains_key("prompts") {
            info!("   ✓ Prompts (templated interactions)");
        }
    }

    // Step 2: Send initialization notification
    info!("\n📋 Step 2: Completing Initialization");
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    transport.send_message(&initialized_notification.to_string()).await?;
    info!("✅ Initialization handshake completed");

    // Step 3: List available resources
    info!("\n📋 Step 3: Discovering Resources");
    let list_resources_request = json!({
        "jsonrpc": "2.0",
        "id": "list-resources-1",
        "method": "resources/list"
    });
    
    transport.send_message(&list_resources_request.to_string()).await?;
    let resources_response = transport.receive_message().await?;
    
    let resources: serde_json::Value = serde_json::from_str(&resources_response)?;
    if let Some(resource_list) = resources["result"]["resources"].as_array() {
        info!("📂 Found {} resource(s):", resource_list.len());
        for resource in resource_list {
            if let (Some(name), Some(uri)) = (resource["name"].as_str(), resource["uri"].as_str()) {
                info!("   • {} ({})", name, uri);
                if let Some(description) = resource["description"].as_str() {
                    info!("     Description: {}", description);
                }
            }
        }
        
        // Try to read the first resource
        if let Some(first_resource) = resource_list.first() {
            if let Some(uri) = first_resource["uri"].as_str() {
                info!("\n📖 Reading resource: {}", uri);
                
                let read_request = json!({
                    "jsonrpc": "2.0",
                    "id": "read-resource-1",
                    "method": "resources/read",
                    "params": {
                        "uri": uri
                    }
                });
                
                transport.send_message(&read_request.to_string()).await?;
                let read_response = transport.receive_message().await?;
                
                let content: serde_json::Value = serde_json::from_str(&read_response)?;
                if let Some(contents) = content["result"]["contents"].as_array() {
                    for content_item in contents {
                        if let Some(text) = content_item["text"].as_str() {
                            let preview = text.chars().take(100).collect::<String>();
                            info!("   📄 Content preview: {}", preview);
                            if text.len() > 100 {
                                info!("      ... ({} more characters)", text.len() - 100);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("📂 No resources available on this server");
    }

    // Step 4: List and call tools
    info!("\n� Step 4: Discovering and Testing Tools");
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": "list-tools-1",
        "method": "tools/list"
    });
    
    transport.send_message(&list_tools_request.to_string()).await?;
    let tools_response = transport.receive_message().await?;
    
    let tools: serde_json::Value = serde_json::from_str(&tools_response)?;
    if let Some(tool_list) = tools["result"]["tools"].as_array() {
        info!("🔧 Found {} tool(s):", tool_list.len());
        for tool in tool_list {
            if let Some(name) = tool["name"].as_str() {
                info!("   • {}", name);
                if let Some(description) = tool["description"].as_str() {
                    info!("     Description: {}", description);
                }
            }
        }
        
        // Try to call the first tool
        if let Some(first_tool) = tool_list.first() {
            if let Some(tool_name) = first_tool["name"].as_str() {
                info!("\n⚙️  Calling tool: {}", tool_name);
                
                // Prepare sample arguments based on tool name
                let arguments = match tool_name {
                    "add" => json!({"a": 15, "b": 27}),
                    "greet" => json!({"name": "MCP Protocol Demonstration"}),
                    _ => json!({"input": "test data"})
                };
                
                let call_request = json!({
                    "jsonrpc": "2.0",
                    "id": "call-tool-1",
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": arguments
                    }
                });
                
                transport.send_message(&call_request.to_string()).await?;
                let call_response = transport.receive_message().await?;
                
                let result: serde_json::Value = serde_json::from_str(&call_response)?;
                if let Some(content) = result["result"]["content"].as_array() {
                    for content_item in content {
                        if let Some(text) = content_item["text"].as_str() {
                            info!("   🎯 Tool result: {}", text);
                        }
                    }
                }
            }
        }
    } else {
        info!("🔧 No tools available on this server");
    }

    // Step 5: List and get prompts
    info!("\n� Step 5: Discovering and Testing Prompts");
    let list_prompts_request = json!({
        "jsonrpc": "2.0",
        "id": "list-prompts-1",
        "method": "prompts/list"
    });
    
    transport.send_message(&list_prompts_request.to_string()).await?;
    let prompts_response = transport.receive_message().await?;
    
    let prompts: serde_json::Value = serde_json::from_str(&prompts_response)?;
    if let Some(prompt_list) = prompts["result"]["prompts"].as_array() {
        info!("💡 Found {} prompt(s):", prompt_list.len());
        for prompt in prompt_list {
            if let Some(name) = prompt["name"].as_str() {
                info!("   • {}", name);
                if let Some(description) = prompt["description"].as_str() {
                    info!("     Description: {}", description);
                }
            }
        }
        
        // Try to get the first prompt
        if let Some(first_prompt) = prompt_list.first() {
            if let Some(prompt_name) = first_prompt["name"].as_str() {
                info!("\n📝 Getting prompt: {}", prompt_name);
                
                let get_request = json!({
                    "jsonrpc": "2.0",
                    "id": "get-prompt-1",
                    "method": "prompts/get",
                    "params": {
                        "name": prompt_name,
                        "arguments": {
                            "topic": "MCP Protocol",
                            "style": "detailed"
                        }
                    }
                });
                
                transport.send_message(&get_request.to_string()).await?;
                let get_response = transport.receive_message().await?;
                
                let result: serde_json::Value = serde_json::from_str(&get_response)?;
                if let Some(messages) = result["result"]["messages"].as_array() {
                    info!("   📋 Prompt contains {} message(s):", messages.len());
                    for message in messages {
                        if let Some(role) = message["role"].as_str() {
                            info!("     Role: {}", role);
                            if let Some(content) = message["content"].as_object() {
                                if let Some(text) = content["text"].as_str() {
                                    let preview = text.chars().take(150).collect::<String>();
                                    info!("     Content: {}", preview);
                                    if text.len() > 150 {
                                        info!("     ... ({} more characters)", text.len() - 150);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("💡 No prompts available on this server");
    }

    info!("\n🎉 MCP Protocol Demonstration Complete!");
    info!("💡 You've seen the complete request/response flow of the MCP protocol");
    Ok(())
}
