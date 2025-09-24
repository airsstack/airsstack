//! HTTP MCP Client Implementation
//!
//! This module provides a high-level HTTP MCP client that uses the airs-mcp
//! HttpTransportClient for communication with HTTP MCP servers.

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use serde_json::Value;
use tokio::time::timeout;
use tracing::{error, info, warn};

// Layer 3: Internal module imports
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::transport::adapters::http::{AuthMethod, HttpTransportClientBuilder};

use crate::config::{AuthenticationMethod, ClientConfig};

/// HTTP MCP client with API key authentication support
pub struct HttpMcpClient {
    client:
        airs_mcp::integration::McpClient<airs_mcp::transport::adapters::http::HttpTransportClient>,
    config: ClientConfig,
}

impl HttpMcpClient {
    /// Create a new HTTP MCP client with the given configuration
    pub async fn new(
        config: &ClientConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| format!("Configuration error: {e}"))?;

        info!(
            "Creating HTTP transport client for endpoint: {}",
            config.server_url
        );

        // Convert our AuthenticationMethod to airs-mcp AuthMethod
        let auth_method = match config.auth_method {
            AuthenticationMethod::XApiKey => AuthMethod::ApiKey {
                key: config.api_key.clone(),
                header: "X-API-Key".to_string(),
            },
            AuthenticationMethod::Bearer => AuthMethod::Bearer {
                token: config.api_key.clone(),
            },
            AuthenticationMethod::QueryParameter => {
                // For query parameter auth, we'll modify the URL instead
                // This is a limitation of the current HttpTransportClient design
                warn!("Query parameter authentication may require URL modification");
                AuthMethod::ApiKey {
                    key: config.api_key.clone(),
                    header: "X-API-Key".to_string(),
                }
            }
        };

        // Determine the server URL (with query params if needed)
        let server_url = match config.auth_method {
            AuthenticationMethod::QueryParameter => config.server_url_with_auth(),
            _ => config.server_url.clone(),
        };

        // Build the HTTP transport client
        let mut builder = HttpTransportClientBuilder::new()
            .endpoint(&server_url)?
            .timeout(config.timeout)
            .user_agent("airs-mcp-http-client/0.1.0");

        // Add authentication only for non-query-parameter methods
        if !matches!(config.auth_method, AuthenticationMethod::QueryParameter) {
            builder = builder.auth(auth_method);
        }

        // Development mode configurations
        if config.dev_mode {
            builder = builder.accept_invalid_certs(true);
        }

        let transport = builder.build().await?;

        // Create the MCP client using the builder
        let client = McpClientBuilder::new().build(transport);

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Initialize the MCP session
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing MCP session");

        let init_timeout = Duration::from_secs(10);
        match timeout(init_timeout, self.client.initialize()).await {
            Ok(Ok(_capabilities)) => {
                info!("✓ MCP session initialized successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                error!("✗ MCP initialization failed: {}", e);
                Err(e.into())
            }
            Err(_) => {
                let err = format!("MCP initialization timed out after {init_timeout:?}");
                error!("✗ {}", err);
                Err(err.into())
            }
        }
    }

    /// List available tools
    pub async fn list_tools(
        &mut self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Listing available tools");

        match timeout(self.config.timeout, self.client.list_tools()).await {
            Ok(Ok(tools)) => {
                let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
                info!("✓ Found {} tools", tool_names.len());
                Ok(tool_names)
            }
            Ok(Err(e)) => {
                error!("✗ Failed to list tools: {}", e);
                Err(e.into())
            }
            Err(_) => {
                let err = format!("Tool listing timed out after {:?}", self.config.timeout);
                error!("✗ {}", err);
                Err(err.into())
            }
        }
    }

    /// Call a tool with the given name and arguments
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Calling tool: {}", tool_name);

        match timeout(
            self.config.timeout,
            self.client.call_tool(tool_name, arguments),
        )
        .await
        {
            Ok(Ok(result)) => {
                info!("✓ Tool '{}' executed successfully", tool_name);
                Ok(format!("Tool '{tool_name}' result: {result:?}"))
            }
            Ok(Err(e)) => {
                error!("✗ Tool '{}' execution failed: {}", tool_name, e);
                Err(e.into())
            }
            Err(_) => {
                let err = format!(
                    "Tool '{}' execution timed out after {:?}",
                    tool_name, self.config.timeout
                );
                error!("✗ {}", err);
                Err(err.into())
            }
        }
    }

    /// List available resources
    pub async fn list_resources(
        &mut self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Listing available resources");

        match timeout(self.config.timeout, self.client.list_resources()).await {
            Ok(Ok(resources)) => {
                let resource_uris: Vec<String> =
                    resources.iter().map(|r| r.uri.to_string()).collect();
                info!("✓ Found {} resources", resource_uris.len());
                Ok(resource_uris)
            }
            Ok(Err(e)) => {
                error!("✗ Failed to list resources: {}", e);
                Err(e.into())
            }
            Err(_) => {
                let err = format!("Resource listing timed out after {:?}", self.config.timeout);
                error!("✗ {}", err);
                Err(err.into())
            }
        }
    }

    /// Read a resource with the given URI
    pub async fn read_resource(
        &mut self,
        uri: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        info!("Reading resource: {}", uri);

        match timeout(self.config.timeout, self.client.read_resource(uri)).await {
            Ok(Ok(content)) => {
                info!("✓ Resource '{}' read successfully", uri);
                Ok(format!("Resource '{uri}' content: {content:?}"))
            }
            Ok(Err(e)) => {
                error!("✗ Failed to read resource '{}': {}", uri, e);
                Err(e.into())
            }
            Err(_) => {
                let err = format!(
                    "Resource '{}' read timed out after {:?}",
                    uri, self.config.timeout
                );
                error!("✗ {}", err);
                Err(err.into())
            }
        }
    }

    /// Perform a complete demo sequence
    pub async fn run_demo(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("=== HTTP MCP Client Demo ===\n");

        // Show configuration
        println!("📋 Configuration:");
        println!("   Server URL: {}", self.config.server_url);
        println!("   Auth Method: {:?}", self.config.auth_method);
        println!("   Mock Mode: {}", self.config.mock_mode);
        println!("   Timeout: {:?}", self.config.timeout);
        println!();

        // Initialize
        println!("1️⃣  Initializing MCP session...");
        match self.initialize().await {
            Ok(()) => {
                println!("   ✓ Initialization successful\n");
            }
            Err(e) => {
                println!("   ✗ Initialization failed: {e}\n");
                return Err(e);
            }
        }

        // List tools
        println!("2️⃣  Listing available tools...");
        match self.list_tools().await {
            Ok(tools) => {
                println!("   ✓ Found {} tools:", tools.len());
                for tool in &tools {
                    println!("     • {tool}");
                }
                println!();

                // Call a tool if available
                if !tools.is_empty() {
                    println!("3️⃣  Calling tool '{}'...", tools[0]);
                    match self.call_tool(&tools[0], None).await {
                        Ok(result) => {
                            println!("   ✓ {result}\n");
                        }
                        Err(e) => {
                            println!("   ✗ Tool call failed: {e}\n");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ✗ Failed to list tools: {e}\n");
            }
        }

        // List resources
        println!("4️⃣  Listing available resources...");
        match self.list_resources().await {
            Ok(resources) => {
                println!("   ✓ Found {} resources:", resources.len());
                for resource in &resources {
                    println!("     • {resource}");
                }
                println!();

                // Read a resource if available
                if !resources.is_empty() {
                    println!("5️⃣  Reading resource '{}'...", resources[0]);
                    match self.read_resource(&resources[0]).await {
                        Ok(content) => {
                            println!("   ✓ {content}\n");
                        }
                        Err(e) => {
                            println!("   ✗ Resource read failed: {e}\n");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ✗ Failed to list resources: {e}\n");
            }
        }

        println!("🎉 Demo completed successfully!");
        Ok(())
    }

    /// Test connection health
    #[allow(dead_code)]
    pub async fn test_connection(
        &mut self,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!("Testing connection health");

        match self.initialize().await {
            Ok(()) => {
                info!("✓ Connection test successful");
                Ok(true)
            }
            Err(e) => {
                warn!("✗ Connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get client configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}
