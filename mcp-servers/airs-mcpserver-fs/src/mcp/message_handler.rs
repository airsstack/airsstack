//! MessageHandler implementation for airs-mcp-fs
//!
//! This module provides the MessageHandler<()> implementation that integrates
//! the existing ToolProvider business logic with the new airs-mcp architecture.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::json;
use tokio::io::{stdout, AsyncWriteExt};
use tracing::{error, info};

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::protocol::{
    constants::methods as mcp_methods, InitializeResponse, JsonRpcMessage, JsonRpcNotification,
    JsonRpcRequest, JsonRpcResponse, MessageContext, MessageHandler, ProtocolVersion, ServerInfo,
    TransportError,
};
use airs_mcp::providers::ToolProvider;

// Layer 3b: Local crate modules
use crate::mcp::handlers::{DirectoryOperations, FileOperations};
use crate::mcp::server::FilesystemMcpServer;

/// MCP Message Handler for airs-mcp-fs STDIO Transport
///
/// This handler wraps the existing FilesystemMcpServer (ToolProvider) and provides
/// the MessageHandler<()> interface required by the new airs-mcp architecture.
/// It preserves all existing business logic while enabling proper transport integration.
#[derive(Debug)]
pub struct FilesystemMessageHandler<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
    server: Arc<FilesystemMcpServer<F, D>>,
}

impl<F, D> FilesystemMessageHandler<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
    /// Create a new message handler wrapping the existing server
    pub fn new(server: Arc<FilesystemMcpServer<F, D>>) -> Self {
        Self { server }
    }

    /// Process MCP JSON-RPC notifications
    async fn process_mcp_notification(&self, notification: &JsonRpcNotification) {
        info!("Processing MCP notification: {}", notification.method);

        match notification.method.as_str() {
            // Protocol initialization complete
            mcp_methods::INITIALIZED => self.handle_initialized_notification(notification).await,

            // Unknown notifications
            _ => {
                info!("Unknown notification method: {}", notification.method);
            }
        }
    }

    /// Process MCP JSON-RPC requests using existing ToolProvider logic
    async fn process_mcp_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Processing MCP request: {}", request.method);

        match request.method.as_str() {
            // Protocol initialization
            mcp_methods::INITIALIZE => self.handle_initialize(request).await,

            // Tool management methods (delegated to existing ToolProvider)
            mcp_methods::TOOLS_LIST => self.handle_tools_list(request).await,
            mcp_methods::TOOLS_CALL => self.handle_tools_call(request).await,

            // Ping/pong for connectivity testing
            mcp_methods::PING => self.handle_ping(request).await,

            // Unknown methods
            _ => self.create_method_not_found_response(
                request,
                &format!("Unknown method: {}", request.method),
            ),
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling initialize request");

        // Manually construct capabilities JSON - only tools and experimental
        let capabilities_json = json!({
            "experimental": {},
            "tools": {
                "list_changed": false
            }
            // Only filesystem tools - no resources, prompts, or logging
        });

        // Use the current supported protocol version instead of hardcoded old version
        let protocol_version = ProtocolVersion::current();

        let response = InitializeResponse {
            protocol_version,
            capabilities: capabilities_json,
            server_info: ServerInfo {
                name: "airs-mcp-fs".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap_or(json!({}))),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle initialized notification (completes the MCP handshake)
    async fn handle_initialized_notification(&self, _notification: &JsonRpcNotification) {
        info!("MCP initialization handshake completed successfully");
        info!("Server is now ready to handle tool calls");
        // No response needed for notifications
    }

    /// Handle tools/list request - delegates to existing ToolProvider
    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/list request");

        match self.server.list_tools().await {
            Ok(tools) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "tools": tools })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => self.create_error_response(request, -32603, &format!("Internal error: {e}")),
        }
    }

    /// Handle tools/call request - delegates to existing ToolProvider  
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/call request");

        // Parse call_tool request parameters
        let params = request.params.clone().unwrap_or(json!({}));

        // Extract tool name and arguments according to MCP spec
        let tool_name = match params.get("name") {
            Some(name) => name.as_str().unwrap_or(""),
            None => {
                return self.create_error_response(
                    request,
                    -32602,
                    "Invalid params: missing 'name' field",
                );
            }
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        // Delegate to existing ToolProvider implementation
        match self.server.call_tool(tool_name, arguments).await {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "content": result })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                // Map tool provider errors to appropriate JSON-RPC error codes
                let error_message = e.to_string();
                if error_message.contains("Tool not found")
                    || error_message.contains("Unknown tool")
                {
                    // Tool not found should be treated as invalid params (tool name is invalid)
                    self.create_error_response(request, -32602, &format!("Invalid params: {e}"))
                } else {
                    // Other tool errors are internal errors (execution failures)
                    self.create_error_response(request, -32603, &format!("Internal error: {e}"))
                }
            }
        }
    }

    /// Handle ping request
    async fn handle_ping(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling ping request");

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!("pong")),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Create standardized error response
    fn create_error_response(
        &self,
        request: &JsonRpcRequest,
        code: i32,
        message: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(json!({
                "code": code,
                "message": message
            })),
            id: Some(request.id.clone()),
        }
    }

    /// Create method not found response
    fn create_method_not_found_response(
        &self,
        request: &JsonRpcRequest,
        message: &str,
    ) -> JsonRpcResponse {
        self.create_error_response(request, -32601, message)
    }
}

#[async_trait]
impl<F, D> MessageHandler<()> for FilesystemMessageHandler<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
    /// Handle incoming JSON-RPC messages from the transport layer
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
        match message {
            JsonRpcMessage::Request(request) => {
                info!("Processing MCP request: {}", request.method);
                let response = self.process_mcp_request(&request).await;

                // Send response via stdout (STDIO transport integration)
                let response_json = serde_json::to_string(&JsonRpcMessage::Response(response))
                    .unwrap_or_else(|e| {
                        error!("Failed to serialize response: {}", e);
                        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Serialization error"},"id":null}"#.to_string()
                    });

                info!("Sending JSON response: {}", response_json);

                // Write to stdout
                if let Err(e) = stdout().write_all(response_json.as_bytes()).await {
                    error!("Failed to write response to stdout: {}", e);
                }
                if let Err(e) = stdout().write_all(b"\n").await {
                    error!("Failed to write newline to stdout: {}", e);
                }
                if let Err(e) = stdout().flush().await {
                    error!("Failed to flush stdout: {}", e);
                }
                
                info!("Response sent successfully");
                
                // Ensure we don't immediately close the connection after sending response
                // The transport should stay alive for subsequent messages (like 'initialized' notification)
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            JsonRpcMessage::Notification(notification) => {
                info!("Processing MCP notification: {}", notification.method);
                self.process_mcp_notification(&notification).await;
            }
            JsonRpcMessage::Response(_) => {
                // STDIO servers typically don't receive responses from clients
                info!("Received response message, ignoring");
            }
        }
    }

    /// Handle transport-level errors
    async fn handle_error(&self, error: TransportError) {
        error!("Transport error: {}", error);
    }

    /// Handle transport close events
    async fn handle_close(&self) {
        info!("Transport connection closed");
    }
}
