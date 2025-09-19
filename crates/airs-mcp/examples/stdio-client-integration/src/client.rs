// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use serde_json::json;
use tokio::time::timeout;
use tracing::{error, info};

// Layer 3: Internal module imports
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;

use crate::config::ClientConfig;

/// MCP client that uses StdioTransportClient for communication
pub struct StdioMcpClient {
    client: airs_mcp::integration::McpClient<
        airs_mcp::transport::adapters::stdio::StdioTransportClient,
    >,
}

impl StdioMcpClient {
    /// Create a new STDIO MCP client with the given configuration
    pub async fn new(
        config: &ClientConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Creating STDIO transport client with command: {}",
            config.server_command
        );

        // Build the STDIO transport client
        let mut builder = StdioTransportClientBuilder::new()
            .command(&config.server_command)
            .timeout(config.request_timeout);

        // Add arguments if provided
        for arg in &config.server_args {
            builder = builder.arg(arg);
        }

        let transport = builder.build().await?;

        // Create the MCP client using the builder
        let client = McpClientBuilder::new().build(transport);

        Ok(Self { client })
    }

    /// Initialize the MCP session
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing MCP session");

        let _capabilities = self.client.initialize().await?;
        info!("Initialization complete");
        Ok(())
    }

    /// List available tools
    pub async fn list_tools(
        &mut self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Listing available tools");

        let tools = self.client.list_tools().await?;
        let tool_names = tools.iter().map(|t| t.name.clone()).collect();
        Ok(tool_names)
    }

    /// Call a tool with the given name and arguments
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Calling tool: {}", tool_name);

        let result = self.client.call_tool(tool_name, arguments).await?;

        // Return the tool result as a formatted string
        Ok(format!("Tool '{tool_name}' result: {result:?}"))
    }

    /// Perform a complete demo sequence
    pub async fn run_demo(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("=== STDIO MCP Client Demo ===\n");

        // Initialize
        println!("1. Initializing MCP session...");
        match timeout(Duration::from_secs(10), self.initialize()).await {
            Ok(Ok(())) => {
                println!("✓ Initialization successful");
            }
            Ok(Err(e)) => {
                error!("✗ Initialization failed: {}", e);
                return Err(e);
            }
            Err(_) => {
                let err = "Initialization timed out";
                error!("✗ {}", err);
                return Err(err.into());
            }
        }

        println!();

        // List tools
        println!("2. Listing available tools...");
        match self.list_tools().await {
            Ok(tool_names) => {
                println!("✓ Tools listed successfully");
                for tool_name in &tool_names {
                    println!("   - {tool_name}");
                }
            }
            Err(e) => {
                error!("✗ Failed to list tools: {}", e);
                return Err(e);
            }
        }

        println!();

        // Test echo tool
        println!("3. Testing echo tool...");
        let echo_args = json!({"message": "Hello from STDIO client!"});
        match self.call_tool("echo", Some(echo_args)).await {
            Ok(response) => {
                println!("✓ Echo tool called successfully");
                println!("   Response: {response}");
            }
            Err(e) => {
                error!("✗ Echo tool failed: {}", e);
                return Err(e);
            }
        }

        println!();

        // Test health check
        println!("4. Testing health check...");
        match self.call_tool("health_check", Some(json!({}))).await {
            Ok(response) => {
                println!("✓ Health check successful");
                println!("   Status: {response}");
            }
            Err(e) => {
                error!("✗ Health check failed: {}", e);
                return Err(e);
            }
        }

        println!();

        // Test timestamp
        println!("5. Getting current timestamp...");
        match self.call_tool("get_timestamp", Some(json!({}))).await {
            Ok(response) => {
                println!("✓ Timestamp retrieved successfully");
                println!("   Timestamp: {response}");
            }
            Err(e) => {
                error!("✗ Timestamp failed: {}", e);
                return Err(e);
            }
        }

        println!("\n=== Demo Complete ===");
        Ok(())
    }
}
