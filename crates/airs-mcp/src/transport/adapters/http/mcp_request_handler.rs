//! Generic Axum MCP Request Handler
//!
//! This module provides a generic, zero-cost MCP request handler that directly processes
//! MCP requests without JSON-RPC intermediary layers. It uses associated types and generics
//! to provide type-safe provider injection while maintaining zero runtime overhead.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::Value;

// Layer 3: Internal module imports
use crate::integration::LoggingHandler;
use crate::protocol::{
    constants::methods, CallToolRequest, GetPromptRequest, GetPromptResult, InitializeRequest,
    InitializeResponse, JsonRpcRequest, JsonRpcResponse, LoggingCapabilities, LoggingConfig,
    PromptCapabilities, ReadResourceRequest, ReadResourceResult, ResourceCapabilities,
    ServerCapabilities, ServerInfo, SetLoggingRequest, SubscribeResourceRequest, ToolCapabilities,
    UnsubscribeResourceRequest,
};
use crate::providers::{PromptProvider, ResourceProvider, ToolProvider};
use crate::transport::adapters::http::engine::{
    AuthenticationContext, HttpEngineError, HttpResponse, McpRequestHandler, ResponseMode,
};

use super::defaults::{NoLoggingHandler, NoPromptProvider, NoResourceProvider, NoToolProvider};

/// Generic MCP request handler with zero-cost provider abstractions
///
/// This handler processes MCP requests directly without JSON-RPC intermediary layers,
/// using generic type parameters to provide compile-time type safety and zero runtime overhead.
///
/// # Type Parameters
///
/// * `R` - Resource provider type implementing ResourceProvider
/// * `T` - Tool provider type implementing ToolProvider  
/// * `P` - Prompt provider type implementing PromptProvider
/// * `L` - Logging handler type implementing LoggingHandler
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::defaults::DefaultAxumMcpRequestHandler;
///
/// // Default handler with no providers
/// let handler = DefaultAxumMcpRequestHandler::new(None, None, None, None);
/// ```
pub struct AxumMcpRequestHandler<R, T, P, L>
where
    R: ResourceProvider + Send + Sync + 'static,
    T: ToolProvider + Send + Sync + 'static,
    P: PromptProvider + Send + Sync + 'static,
    L: LoggingHandler + Send + Sync + 'static,
{
    /// Resource provider for handling resource-related MCP requests
    resource_provider: Option<R>,
    /// Tool provider for handling tool-related MCP requests
    tool_provider: Option<T>,
    /// Prompt provider for handling prompt-related MCP requests
    prompt_provider: Option<P>,
    /// Logging handler for MCP logging operations
    logging_handler: Option<L>,
    /// Server configuration and capabilities
    server_capabilities: ServerCapabilities,
    /// Server information
    server_info: ServerInfo,
}

impl<R, T, P, L> AxumMcpRequestHandler<R, T, P, L>
where
    R: ResourceProvider + Send + Sync + 'static,
    T: ToolProvider + Send + Sync + 'static,
    P: PromptProvider + Send + Sync + 'static,
    L: LoggingHandler + Send + Sync + 'static,
{
    /// Create a new generic MCP request handler
    ///
    /// # Arguments
    ///
    /// * `resource_provider` - Optional resource provider implementation
    /// * `tool_provider` - Optional tool provider implementation
    /// * `prompt_provider` - Optional prompt provider implementation
    /// * `logging_handler` - Optional logging handler implementation
    pub fn new(
        resource_provider: Option<R>,
        tool_provider: Option<T>,
        prompt_provider: Option<P>,
        logging_handler: Option<L>,
    ) -> Self {
        let server_capabilities = ServerCapabilities {
            resources: if resource_provider.is_some() {
                Some(ResourceCapabilities::default())
            } else {
                None
            },
            tools: if tool_provider.is_some() {
                Some(ToolCapabilities::default())
            } else {
                None
            },
            prompts: if prompt_provider.is_some() {
                Some(PromptCapabilities::default())
            } else {
                None
            },
            logging: if logging_handler.is_some() {
                Some(LoggingCapabilities::default())
            } else {
                None
            },
            experimental: None,
        };

        let server_info = ServerInfo {
            name: "airs-mcp-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Self {
            resource_provider,
            tool_provider,
            prompt_provider,
            logging_handler,
            server_capabilities,
            server_info,
        }
    }

    /// Handle MCP initialize request
    pub async fn handle_initialize(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        // Parse InitializeRequest from request.params
        let init_request: InitializeRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                HttpEngineError::Engine {
                    message: format!("Invalid initialization request: {e}"),
                }
            })?;

        // Validate protocol version compatibility
        let current_version = crate::protocol::ProtocolVersion::current();
        if !current_version.is_compatible_with(&init_request.protocol_version) {
            return Err(HttpEngineError::Engine {
                message: format!(
                    "Protocol version mismatch: client version '{}', server version '{}'",
                    init_request.protocol_version.as_str(),
                    current_version.as_str()
                ),
            });
        }

        // TODO(ENHANCEMENT): Store client capabilities for later use in session
        // For now, we acknowledge the client capabilities but don't store them
        // In a full implementation, we would:
        // 1. Store init_request.capabilities in session context
        // 2. Use client capabilities to optimize responses
        // 3. Validate that client supports required features

        // Return server capabilities based on configured providers
        let capabilities_json = serde_json::to_value(&self.server_capabilities).map_err(|e| {
            HttpEngineError::Engine {
                message: format!("Failed to serialize server capabilities: {e}"),
            }
        })?;

        let response = InitializeResponse {
            protocol_version: current_version,
            capabilities: capabilities_json,
            server_info: self.server_info.clone(),
        };

        serde_json::to_value(response).map_err(|e| HttpEngineError::Engine {
            message: format!("Failed to serialize initialize response: {e}"),
        })
    }

    /// Handle MCP list resources request
    pub async fn handle_list_resources(
        &self,
        _session_id: &str,
        _request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            let resources =
                provider
                    .list_resources()
                    .await
                    .map_err(|e| HttpEngineError::Engine {
                        message: format!("Resource provider error: {e}"),
                    })?;

            // Return direct result structure to match original implementation
            Ok(serde_json::json!({
                "resources": resources
            }))
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP read resource request
    pub async fn handle_read_resource(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            // Parse ReadResourceRequest from request.params
            let read_request: ReadResourceRequest =
                serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                    HttpEngineError::Engine {
                        message: format!("Invalid read resource request: {e}"),
                    }
                })?;

            let contents = provider
                .read_resource(read_request.uri.as_str())
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Resource provider error: {e}"),
                })?;

            let result = ReadResourceResult::new(contents);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize resource content: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP list tools request
    pub async fn handle_list_tools(
        &self,
        _session_id: &str,
        _request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.tool_provider {
            let tools = provider
                .list_tools()
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Tool provider error: {e}"),
                })?;

            // Return direct result structure to match original implementation
            Ok(serde_json::json!({
                "tools": tools
            }))
        } else {
            Err(HttpEngineError::Engine {
                message: "No tool provider configured".to_string(),
            })
        }
    }

    /// Handle MCP call tool request
    pub async fn handle_call_tool(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.tool_provider {
            // Parse CallToolRequest from request.params
            let call_request: CallToolRequest =
                serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                    HttpEngineError::Engine {
                        message: format!("Invalid call tool request: {e}"),
                    }
                })?;

            match provider
                .call_tool(&call_request.name, call_request.arguments)
                .await
            {
                Ok(content) => {
                    // Return success result with content and isError: false
                    Ok(serde_json::json!({
                        "content": content,
                        "isError": false
                    }))
                }
                Err(e) => {
                    // Tool provider errors are returned as successful results with isError flag
                    Ok(serde_json::json!({
                        "content": [],
                        "isError": true,
                        "errorMessage": e.to_string()
                    }))
                }
            }
        } else {
            Err(HttpEngineError::Engine {
                message: "No tool provider configured".to_string(),
            })
        }
    }

    /// Handle MCP list prompts request
    pub async fn handle_list_prompts(
        &self,
        _session_id: &str,
        _request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.prompt_provider {
            let prompts = provider
                .list_prompts()
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Prompt provider error: {e}"),
                })?;

            // Return direct result structure to match original implementation
            Ok(serde_json::json!({
                "prompts": prompts
            }))
        } else {
            Err(HttpEngineError::Engine {
                message: "No prompt provider configured".to_string(),
            })
        }
    }

    /// Handle MCP get prompt request
    pub async fn handle_get_prompt(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.prompt_provider {
            // Parse GetPromptRequest from request.params
            let prompt_request: GetPromptRequest =
                serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                    HttpEngineError::Engine {
                        message: format!("Invalid get prompt request: {e}"),
                    }
                })?;

            let (description, messages) = provider
                .get_prompt(&prompt_request.name, prompt_request.arguments)
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Prompt provider error: {e}"),
                })?;

            let result = GetPromptResult::new(Some(description), messages);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize prompt result: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No prompt provider configured".to_string(),
            })
        }
    }

    /// Handle MCP list resource templates request
    pub async fn handle_list_resource_templates(
        &self,
        _session_id: &str,
        _request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            let resource_templates =
                provider
                    .list_resource_templates()
                    .await
                    .map_err(|e| HttpEngineError::Engine {
                        message: format!("Resource provider error: {e}"),
                    })?;

            // Return direct result structure to match original implementation (camelCase field name)
            Ok(serde_json::json!({
                "resourceTemplates": resource_templates
            }))
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP subscribe resource request
    pub async fn handle_subscribe_resource(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            let params = request.params.unwrap_or_default();
            let subscribe_request: SubscribeResourceRequest = serde_json::from_value(params)
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Invalid subscribe resource request: {e}"),
                })?;

            provider
                .subscribe_to_resource(subscribe_request.uri.as_str())
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Resource provider error: {e}"),
                })?;

            // Return empty result for subscribe operations (matches original implementation)
            Ok(serde_json::json!({}))
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP unsubscribe resource request
    pub async fn handle_unsubscribe_resource(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            let params = request.params.unwrap_or_default();
            let unsubscribe_request: UnsubscribeResourceRequest = serde_json::from_value(params)
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Invalid unsubscribe resource request: {e}"),
                })?;

            provider
                .unsubscribe_from_resource(unsubscribe_request.uri.as_str())
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Resource provider error: {e}"),
                })?;

            // Return empty result for unsubscribe operations (matches original implementation)
            Ok(serde_json::json!({}))
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP set logging request
    pub async fn handle_set_logging(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref handler) = self.logging_handler {
            // Parse SetLoggingRequest from request.params
            let logging_request: SetLoggingRequest =
                serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                    HttpEngineError::Engine {
                        message: format!("Invalid set logging request: {e}"),
                    }
                })?;

            match handler
                .set_logging(LoggingConfig {
                    level: logging_request.level,
                })
                .await
            {
                Ok(success) => {
                    // Return success response with proper JSON-RPC structure
                    Ok(serde_json::json!({
                        "success": success,
                        "message": if success {
                            "Logging configuration updated"
                        } else {
                            "Failed to update logging configuration"
                        }
                    }))
                }
                Err(e) => Err(HttpEngineError::Engine {
                    message: format!("Logging handler error: {e}"),
                }),
            }
        } else {
            Err(HttpEngineError::Engine {
                message: "No logging handler configured".to_string(),
            })
        }
    }
}

#[async_trait]
impl<R, T, P, L> McpRequestHandler for AxumMcpRequestHandler<R, T, P, L>
where
    R: ResourceProvider + Send + Sync + 'static,
    T: ToolProvider + Send + Sync + 'static,
    P: PromptProvider + Send + Sync + 'static,
    L: LoggingHandler + Send + Sync + 'static,
{
    async fn handle_mcp_request(
        &self,
        session_id: String,
        request_data: Vec<u8>,
        response_mode: ResponseMode,
        _auth_context: Option<AuthenticationContext>,
    ) -> Result<HttpResponse, HttpEngineError> {
        // Parse JSON-RPC request
        let request: JsonRpcRequest =
            serde_json::from_slice(&request_data).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to parse JSON-RPC request: {e}"),
            })?;

        // Clone the request ID before routing to handlers
        let request_id = request.id.clone();

        // Route to appropriate handler based on method
        let result = match request.method.as_str() {
            methods::INITIALIZE => self.handle_initialize(&session_id, request).await?,
            methods::RESOURCES_LIST => self.handle_list_resources(&session_id, request).await?,
            methods::RESOURCES_TEMPLATES_LIST => {
                self.handle_list_resource_templates(&session_id, request)
                    .await?
            }
            methods::RESOURCES_READ => self.handle_read_resource(&session_id, request).await?,
            methods::RESOURCES_SUBSCRIBE => {
                self.handle_subscribe_resource(&session_id, request).await?
            }
            methods::RESOURCES_UNSUBSCRIBE => {
                self.handle_unsubscribe_resource(&session_id, request)
                    .await?
            }
            methods::TOOLS_LIST => self.handle_list_tools(&session_id, request).await?,
            methods::TOOLS_CALL => self.handle_call_tool(&session_id, request).await?,
            methods::PROMPTS_LIST => self.handle_list_prompts(&session_id, request).await?,
            methods::PROMPTS_GET => self.handle_get_prompt(&session_id, request).await?,
            methods::LOGGING_SET_LEVEL => self.handle_set_logging(&session_id, request).await?,
            _ => {
                return Err(HttpEngineError::Engine {
                    message: format!("Unknown method: {}", request.method),
                });
            }
        };

        // Create JSON-RPC response
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(request_id),
            result: Some(result),
            error: None,
        };

        // Serialize response
        let response_body = serde_json::to_vec(&response).map_err(|e| HttpEngineError::Engine {
            message: format!("Failed to serialize response: {e}"),
        })?;

        // Return appropriate response format
        match response_mode {
            ResponseMode::Json => Ok(HttpResponse::json(response_body)),
            ResponseMode::ServerSentEvents => Ok(HttpResponse::sse(response_body)),
            ResponseMode::Streaming => Ok(HttpResponse::streaming(response_body)),
        }
    }
}

/// Type alias for the default handler with no providers
pub type DefaultAxumMcpRequestHandler =
    AxumMcpRequestHandler<NoResourceProvider, NoToolProvider, NoPromptProvider, NoLoggingHandler>;
