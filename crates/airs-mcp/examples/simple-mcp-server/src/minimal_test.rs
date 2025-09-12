use airs_mcp::integration::{McpError, McpServer};
use airs_mcp::protocol::types::{Content, Tool};
use airs_mcp::protocol::{
    JsonRpcMessage, JsonRpcMessageTrait, MessageContext, MessageHandler, TransportBuilder,
    TransportError,
};
use airs_mcp::providers::ToolProvider;
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Minimal tool provider for testing
#[derive(Debug)]
struct MinimalToolProvider;

#[async_trait]
impl ToolProvider for MinimalToolProvider {
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        Ok(vec![Tool {
            name: "test".to_string(),
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

/// Minimal MCP Handler for testing
#[derive(Debug)]
struct MinimalMcpHandler {
    tool_provider: MinimalToolProvider,
}

impl MinimalMcpHandler {
    pub fn new(tool_provider: MinimalToolProvider) -> Self {
        Self { tool_provider }
    }

    async fn handle_mcp_request(
        &self,
        request: airs_mcp::protocol::JsonRpcRequest,
    ) -> airs_mcp::protocol::JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "minimal-test", "version": "0.1.0" }
                });
                airs_mcp::protocol::JsonRpcResponse::success(result, request.id)
            }
            "tools/list" => match self.tool_provider.list_tools().await {
                Ok(tools) => {
                    let result = json!({ "tools": tools });
                    airs_mcp::protocol::JsonRpcResponse::success(result, request.id)
                }
                Err(_) => {
                    let error_data = json!({ "code": -32603, "message": "Failed to list tools" });
                    airs_mcp::protocol::JsonRpcResponse::error(error_data, Some(request.id))
                }
            },
            _ => {
                let error_data = json!({ "code": -32601, "message": "Method not found" });
                airs_mcp::protocol::JsonRpcResponse::error(error_data, Some(request.id))
            }
        }
    }
}

#[async_trait]
impl MessageHandler<()> for MinimalMcpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
        if let JsonRpcMessage::Request(request) = message {
            let response = self.handle_mcp_request(request).await;
            if let Ok(response_json) = response.to_json() {
                println!("{response_json}");
            }
        }
    }

    async fn handle_error(&self, _error: TransportError) {}
    async fn handle_close(&self) {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Minimal server - no logging interference
    let handler = Arc::new(MinimalMcpHandler::new(MinimalToolProvider));
    let transport = StdioTransportBuilder::new()
        .with_message_handler(handler)
        .build()
        .await?;
    let server = McpServer::new(transport);

    // Start immediately without any stderr output
    server.start().await?;
    Ok(())
}
