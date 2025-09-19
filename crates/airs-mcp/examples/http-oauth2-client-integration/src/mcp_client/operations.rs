//! MCP operations testing with OAuth2 authentication

// Layer 2: Third-party crate imports
use tracing::{info, warn};

// Layer 3: Internal module imports
use crate::mcp_client::McpSession;
use crate::oauth2::OAuth2IntegrationError;

/// MCP operations handler
pub struct McpOperations;

impl McpOperations {
    /// Test MCP operations using OAuth2 access token with airs-mcp HttpTransportClient
    pub async fn test_operations(mcp_server_url: &str, access_token: &str) -> Result<(), OAuth2IntegrationError> {
        // Create MCP session
        let mut mcp_session = McpSession::new(mcp_server_url, access_token).await?;

        // Run all test operations
        Self::test_list_tools(&mut mcp_session).await;
        Self::test_list_resources(&mut mcp_session).await;
        Self::test_tool_calling(&mut mcp_session).await;

        // Close the session
        mcp_session.close().await?;

        info!("🎯 MCP operations testing completed");
        Ok(())
    }

    /// Test listing available tools
    async fn test_list_tools(mcp_session: &mut McpSession) {
        info!("📋 Testing: List available tools");
        match mcp_session.client_mut().list_tools().await {
            Ok(tools) => {
                info!("✅ Tools list successful: {} tools found", tools.len());
                for tool in tools {
                    info!("  🔧 Tool: {} - {}", tool.name, tool.description.unwrap_or("No description".to_string()));
                }
            }
            Err(e) => {
                warn!("⚠️  Tools list failed: {}", e);
            }
        }
    }

    /// Test listing available resources
    async fn test_list_resources(mcp_session: &mut McpSession) {
        info!("📋 Testing: List available resources");
        match mcp_session.client_mut().list_resources().await {
            Ok(resources) => {
                info!("✅ Resources list successful: {} resources found", resources.len());
                for resource in resources {
                    let name = if resource.name.is_empty() { "No name" } else { &resource.name };
                    info!("  - {}: {}", resource.uri, name);
                }
            }
            Err(e) => {
                warn!("⚠️  Resources list failed: {}", e);
            }
        }
    }

    /// Test calling a tool if available
    async fn test_tool_calling(mcp_session: &mut McpSession) {
        info!("🔧 Testing: Tool calling");
        match mcp_session.client_mut().list_tools().await {
            Ok(tools) if !tools.is_empty() => {
                let tool_name = &tools[0].name;
                info!("🎯 Calling tool: {}", tool_name);
                
                match mcp_session.client_mut().call_tool(tool_name, None).await {
                    Ok(result) => {
                        info!("✅ Tool call successful: {:?}", result);
                    }
                    Err(e) => {
                        warn!("⚠️  Tool call failed: {}", e);
                    }
                }
            }
            Ok(_) => {
                info!("ℹ️  No tools available to test");
            }
            Err(e) => {
                warn!("⚠️  Failed to get tools for testing: {}", e);
            }
        }
    }
}