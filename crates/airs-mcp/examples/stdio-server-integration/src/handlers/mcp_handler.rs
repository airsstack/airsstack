//! MCP Message Handler Implementation
//!
//! This module provides the main MCP message handler that implements
//! the `MessageHandler<()>` trait for transport integration.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::json;
use tokio::io::{AsyncWriteExt, stdout};
use tracing::{error, info};

// Layer 3: Internal module imports
use airs_mcp::protocol::{
    constants::methods as mcp_methods, CallToolRequest, GetPromptRequest, InitializeResponse,
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, LoggingCapabilities, MessageContext,
    MessageHandler, PromptCapabilities, ProtocolVersion, ReadResourceRequest,
    ResourceCapabilities, ServerCapabilities, ServerInfo, ToolCapabilities, TransportError,
};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider, PromptProvider,
    ResourceProvider, StructuredLoggingHandler, ToolProvider,
};

/// MCP Message Handler for STDIO Transport
///
/// Handles MCP protocol messages with proper transport integration.
/// Implements the `MessageHandler<()>` trait for event-driven message processing.
#[derive(Debug)]
pub struct McpHandler {
    resource_provider: Arc<FileSystemResourceProvider>,
    tool_provider: Arc<MathToolProvider>,
    prompt_provider: Arc<CodeReviewPromptProvider>,
    #[allow(dead_code)] // TODO: Implement logging handler integration
    logging_handler: Arc<StructuredLoggingHandler>,
}

impl McpHandler {
    /// Create new MCP handler with all providers
    pub fn new(
        resource_provider: FileSystemResourceProvider,
        tool_provider: MathToolProvider,
        prompt_provider: CodeReviewPromptProvider,
        logging_handler: StructuredLoggingHandler,
    ) -> Self {
        Self {
            resource_provider: Arc::new(resource_provider),
            tool_provider: Arc::new(tool_provider),
            prompt_provider: Arc::new(prompt_provider),
            logging_handler: Arc::new(logging_handler),
        }
    }

    /// Process MCP request and return response
    pub async fn process_mcp_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            // Initialization and lifecycle methods
            mcp_methods::INITIALIZE => self.handle_initialize(request).await,
            mcp_methods::INITIALIZED => self.handle_initialized(request).await,

            // Resource management methods
            mcp_methods::RESOURCES_LIST => self.handle_resources_list(request).await,
            mcp_methods::RESOURCES_READ => self.handle_resources_read(request).await,
            mcp_methods::RESOURCES_TEMPLATES_LIST => {
                self.handle_resources_templates_list(request).await
            }
            mcp_methods::RESOURCES_SUBSCRIBE => self.handle_resources_subscribe(request).await,
            mcp_methods::RESOURCES_UNSUBSCRIBE => self.handle_resources_unsubscribe(request).await,

            // Tool management methods
            mcp_methods::TOOLS_LIST => self.handle_tools_list(request).await,
            mcp_methods::TOOLS_CALL => self.handle_tools_call(request).await,

            // Prompt management methods
            mcp_methods::PROMPTS_LIST => self.handle_prompts_list(request).await,
            mcp_methods::PROMPTS_GET => self.handle_prompts_get(request).await,

            // Logging methods
            mcp_methods::LOGGING_SET_LEVEL => self.handle_logging_set_level(request).await,

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

        let capabilities = ServerCapabilities {
            experimental: Some(json!({})),
            logging: Some(LoggingCapabilities {}),
            resources: Some(ResourceCapabilities {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            tools: Some(ToolCapabilities {
                list_changed: Some(false),
            }),
            prompts: Some(PromptCapabilities {
                list_changed: Some(false),
            }),
        };

        let response = InitializeResponse {
            protocol_version: ProtocolVersion::new("2024-11-05").expect("Valid protocol version"),
            capabilities: serde_json::to_value(capabilities).unwrap_or(json!({})),
            server_info: ServerInfo {
                name: "airs-mcp-stdio-server".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap_or(json!({}))),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle initialized notification
    async fn handle_initialized(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Client initialization completed");

        // Initialized is a notification, so we don't send a response
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/list request");

        match self.resource_provider.list_resources().await {
            Ok(resources) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "resources": resources })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {e}"))
            }
        }
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/read request");

        match serde_json::from_value::<ReadResourceRequest>(
            request.params.clone().unwrap_or(json!({})),
        ) {
            Ok(read_request) => {
                match self
                    .resource_provider
                    .read_resource(&read_request.uri.to_string())
                    .await
                {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!({ "contents": result })),
                        error: None,
                        id: Some(request.id.clone()),
                    },
                    Err(e) => self.create_error_response(
                        request,
                        -32603,
                        &format!("Internal error: {e}"),
                    ),
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {e}"))
            }
        }
    }

    /// Handle resources/templates/list request
    async fn handle_resources_templates_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/templates/list request");

        // Resource templates not implemented yet
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({ "resourceTemplates": [] })),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle resources/subscribe request
    async fn handle_resources_subscribe(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        self.create_method_not_found_response(request, "Resource subscriptions not supported")
    }

    /// Handle resources/unsubscribe request
    async fn handle_resources_unsubscribe(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        self.create_method_not_found_response(request, "Resource subscriptions not supported")
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/list request");

        match self.tool_provider.list_tools().await {
            Ok(tools) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "tools": tools })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {e}"))
            }
        }
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/call request");

        match serde_json::from_value::<CallToolRequest>(request.params.clone().unwrap_or(json!({})))
        {
            Ok(call_request) => {
                let arguments = call_request.arguments;
                match self
                    .tool_provider
                    .call_tool(&call_request.name, arguments)
                    .await
                {
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
                            self.create_error_response(
                                request,
                                -32602,
                                &format!("Invalid params: {e}"),
                            )
                        } else {
                            // Other tool errors are internal errors (execution failures)
                            self.create_error_response(
                                request,
                                -32603,
                                &format!("Internal error: {e}"),
                            )
                        }
                    }
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {e}"))
            }
        }
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling prompts/list request");

        match self.prompt_provider.list_prompts().await {
            Ok(prompts) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "prompts": prompts })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {e}"))
            }
        }
    }

    /// Handle prompts/get request
    async fn handle_prompts_get(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling prompts/get request");

        match serde_json::from_value::<GetPromptRequest>(
            request.params.clone().unwrap_or(json!({})),
        ) {
            Ok(get_request) => {
                match self
                    .prompt_provider
                    .get_prompt(&get_request.name, get_request.arguments)
                    .await
                {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
                        error: None,
                        id: Some(request.id.clone()),
                    },
                    Err(e) => self.create_error_response(
                        request,
                        -32603,
                        &format!("Internal error: {e}"),
                    ),
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {e}"))
            }
        }
    }

    /// Handle logging/setLevel request
    async fn handle_logging_set_level(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling logging/setLevel request");

        // Logging level setting handled by the logging handler
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({})),
            error: None,
            id: Some(request.id.clone()),
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

    /// Create method not found response
    fn create_method_not_found_response(
        &self,
        request: &JsonRpcRequest,
        message: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": "Method not found",
                "data": message
            })),
            id: Some(request.id.clone()),
        }
    }

    /// Create error response
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
}

#[async_trait]
impl MessageHandler<()> for McpHandler {
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
            }
            JsonRpcMessage::Response(_) | JsonRpcMessage::Notification(_) => {
                // STDIO servers typically only handle requests
                info!("Received non-request message, ignoring");
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