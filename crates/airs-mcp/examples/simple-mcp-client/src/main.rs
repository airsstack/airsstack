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
use std::future::Future;
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
            .map_err(|e| TransportError::other(format!("Failed to spawn server process: {}", e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| TransportError::other("Failed to get stdin handle from server process"))?;
        let stdout = BufReader::new(child.stdout.take()
            .ok_or_else(|| TransportError::other("Failed to get stdout handle from server process"))?);

        info!("✅ Server process spawned successfully (PID: {})", child.id().unwrap_or(0));

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Shutdown the server process
    pub async fn shutdown(mut self) -> Result<(), TransportError> {
        info!("🛑 Shutting down server process...");
        
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

impl Transport for SubprocessTransport {
    type Error = TransportError;

    fn send(&mut self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            debug!("📤 Sending: {}", String::from_utf8_lossy(data));
            
            self.stdin.write_all(data).await
                .map_err(|e| TransportError::other(format!("Failed to send data: {}", e)))?;
            self.stdin.write_all(b"\n").await
                .map_err(|e| TransportError::other(format!("Failed to send newline: {}", e)))?;
            self.stdin.flush().await
                .map_err(|e| TransportError::other(format!("Failed to flush: {}", e)))?;
            
            Ok(())
        }
    }

    fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send {
        async move {
            let mut line = String::new();
            let bytes_read = self.stdout.read_line(&mut line).await
                .map_err(|e| TransportError::other(format!("Failed to read line: {}", e)))?;
            
            if bytes_read == 0 {
                return Err(TransportError::closed());
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
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            // For subprocess, we'll just close stdin which should signal the server to shutdown
            // The actual cleanup happens in shutdown() method
            Ok(())
        }
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

                // Also test the second resource if available  
                if resources.len() > 1 {
                    let resource = &resources[1];
                    info!("\n📖 Reading second resource using AIRS client: {}", resource.uri);
                    match client.read_resource(resource.uri.to_string()).await {
                        Ok(content) => {
                            info!("   ✅ Second resource content received:");
                            for content_item in content {
                                if let Some(text) = content_item.as_text() {
                                    let preview = text.chars().take(150).collect::<String>();
                                    info!("      📄 Content: {}", preview);
                                    if text.len() > 150 {
                                        info!("         ... ({} more characters)", text.len() - 150);
                                    }
                                }
                            }
                        }
                        Err(e) => warn!("   ⚠️  Failed to read second resource: {}", e),
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

                // Also test the second tool if available
                if tools.len() > 1 {
                    let tool = &tools[1];
                    info!("\n⚙️  Calling second tool using AIRS client: {}", tool.name);
                    let sample_input = match tool.name.as_str() {
                        "add" => json!({"a": 100, "b": 200}),
                        "greet" => json!({"name": "Rust Developer"}),
                        _ => json!({"input": "test data"}),
                    };

                    match client.call_tool(&tool.name, Some(sample_input)).await {
                        Ok(result) => {
                            info!("   ✅ Second tool execution successful:");
                            for content_item in result {
                                if let Some(text) = content_item.as_text() {
                                    info!("      🎯 Result: {}", text);
                                }
                            }
                        }
                        Err(e) => warn!("   ⚠️  Failed to call second tool: {}", e),
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
