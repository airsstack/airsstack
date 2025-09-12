//! Generic Axum MCP Request Handler
//!
//! This module provides a generic, zero-cost MCP request handler that directly processes
//! MCP requests without JSON-RPC intermediary layers. It uses associated types and generics
//! to provide type-safe provider injection while maintaining zero runtime overhead.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::Value;

// Layer 3: Internal module imports
use crate::integration::LoggingHandler;
use crate::protocol::{
    CallToolResult, GetPromptResult, InitializeRequest, 
    InitializeResponse, JsonRpcRequest, JsonRpcResponse, ListPromptsResult, ListResourcesResult, 
    ListToolsResult, LoggingCapabilities, PromptCapabilities, 
    ReadResourceResult, ResourceCapabilities, ServerCapabilities, ServerInfo, 
    ToolCapabilities,
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
/// let handler = DefaultAxumMcpRequestHandler::new();
///
/// // Handler with specific providers
/// let handler = AxumMcpRequestHandler::new(
///     Some(my_resource_provider),
///     Some(my_tool_provider),
///     None, // No prompt provider
///     None, // No logging handler
/// );
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
    async fn handle_initialize(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        // Parse InitializeRequest from request.params
        let _init_request: InitializeRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                HttpEngineError::Engine {
                    message: format!("Invalid initialization request: {e}"),
                }
            })?;

        // Create initialize response
        let capabilities_value = serde_json::to_value(&self.server_capabilities).map_err(|e| {
            HttpEngineError::Engine {
                message: format!("Failed to serialize server capabilities: {e}"),
            }
        })?;

        let response = InitializeResponse {
            protocol_version: crate::protocol::ProtocolVersion::current(),
            capabilities: capabilities_value,
            server_info: self.server_info.clone(),
        };

        serde_json::to_value(response).map_err(|e| HttpEngineError::Engine {
            message: format!("Failed to serialize initialize response: {e}"),
        })
    }

    /// Handle MCP list resources request
    async fn handle_list_resources(
        &self,
        _session_id: &str,
        _request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            let resources = provider
                .list_resources()
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Resource provider error: {e}"),
                })?;

            let result = ListResourcesResult::new(resources);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize resources: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No resource provider configured".to_string(),
            })
        }
    }

    /// Handle MCP read resource request
    async fn handle_read_resource(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.resource_provider {
            // Extract URI from params
            let params = request.params.unwrap_or_default();
            let uri = params
                .get("uri")
                .and_then(|v| v.as_str())
                .ok_or_else(|| HttpEngineError::Engine {
                    message: "Missing or invalid 'uri' parameter".to_string(),
                })?;

            let contents = provider
                .read_resource(uri)
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
    async fn handle_list_tools(
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

            let result = ListToolsResult::new(tools);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize tools: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No tool provider configured".to_string(),
            })
        }
    }

    /// Handle MCP call tool request
    async fn handle_call_tool(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.tool_provider {
            let params = request.params.unwrap_or_default();
            
            // Extract name from params
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| HttpEngineError::Engine {
                    message: "Missing or invalid 'name' parameter".to_string(),
                })?;
            
            // Extract arguments from params (default to empty object if not provided)
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

            let contents = provider
                .call_tool(name, arguments)
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Tool provider error: {e}"),
                })?;

            let result = CallToolResult::success(contents);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize tool result: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No tool provider configured".to_string(),
            })
        }
    }

    /// Handle MCP list prompts request
    async fn handle_list_prompts(
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

            let result = ListPromptsResult::new(prompts);
            serde_json::to_value(result).map_err(|e| HttpEngineError::Engine {
                message: format!("Failed to serialize prompts: {e}"),
            })
        } else {
            Err(HttpEngineError::Engine {
                message: "No prompt provider configured".to_string(),
            })
        }
    }

    /// Handle MCP get prompt request
    async fn handle_get_prompt(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref provider) = self.prompt_provider {
            let params = request.params.unwrap_or_default();
            
            // Extract name from params
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| HttpEngineError::Engine {
                    message: "Missing or invalid 'name' parameter".to_string(),
                })?;
            
            // Extract arguments from params (default to empty map if not provided)
            let arguments = params
                .get("arguments")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<String, String>>()
                })
                .unwrap_or_default();

            let (description, messages) = provider
                .get_prompt(name, arguments)
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

    /// Handle MCP set logging request
    async fn handle_set_logging(
        &self,
        _session_id: &str,
        request: JsonRpcRequest,
    ) -> Result<Value, HttpEngineError> {
        if let Some(ref handler) = self.logging_handler {
            let params = request.params.unwrap_or_default();
            
            // Extract level from params - this should be a LoggingConfig object
            let logging_config = serde_json::from_value(params).map_err(|e| {
                HttpEngineError::Engine {
                    message: format!("Invalid set logging request: {e}"),
                }
            })?;

            let result = handler
                .set_logging(logging_config)
                .await
                .map_err(|e| HttpEngineError::Engine {
                    message: format!("Logging handler error: {e}"),
                })?;

            // Return success response with the result
            Ok(serde_json::json!({ "success": result }))
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
        let request: JsonRpcRequest = serde_json::from_slice(&request_data).map_err(|e| {
            HttpEngineError::Engine {
                message: format!("Failed to parse JSON-RPC request: {e}"),
            }
        })?;

        // Clone the request ID before routing to handlers
        let request_id = request.id.clone();

        // Route to appropriate handler based on method
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&session_id, request).await?,
            "resources/list" => self.handle_list_resources(&session_id, request).await?,
            "resources/read" => self.handle_read_resource(&session_id, request).await?,
            "tools/list" => self.handle_list_tools(&session_id, request).await?,
            "tools/call" => self.handle_call_tool(&session_id, request).await?,
            "prompts/list" => self.handle_list_prompts(&session_id, request).await?,
            "prompts/get" => self.handle_get_prompt(&session_id, request).await?,
            "logging/setLevel" => self.handle_set_logging(&session_id, request).await?,
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
        let response_body = serde_json::to_vec(&response).map_err(|e| {
            HttpEngineError::Engine {
                message: format!("Failed to serialize response: {e}"),
            }
        })?;

        // Return appropriate response format
        match response_mode {
            ResponseMode::Json => Ok(HttpResponse::json(response_body)),
            ResponseMode::ServerSentEvents => Ok(HttpResponse::sse(response_body)),
            ResponseMode::Streaming => Ok(HttpResponse::json(response_body)), // Default to JSON for now
        }
    }
}

/// Type alias for the default handler with no providers
pub type DefaultAxumMcpRequestHandler = AxumMcpRequestHandler<
    NoResourceProvider,
    NoToolProvider,
    NoPromptProvider,
    NoLoggingHandler,
>;