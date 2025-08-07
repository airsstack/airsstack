use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::integration::mcp::{McpError, ToolProvider};
use airs_mcp::shared::protocol::{Content, Tool};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Minimal tool provider for testing
#[derive(Debug)]
struct MinimalToolProvider;

#[async_trait]
impl ToolProvider for MinimalToolProvider {
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        Ok(vec![Tool {
            name: "test".to_string(),
            title: Some("Test Tool".to_string()),
            description: Some("A simple test tool".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }])
    }

    async fn call_tool(&self, _name: &str, _arguments: Value) -> Result<Vec<Content>, McpError> {
        Ok(vec![Content::text("Test result")])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Minimal server - no logging interference
    let transport = airs_mcp::transport::StdioTransport::new().await?;
    let server = McpServerBuilder::new()
        .with_tool_provider(MinimalToolProvider)
        .build(transport)
        .await?;

    // Start immediately without any stderr output
    server.run().await?;
    Ok(())
}
