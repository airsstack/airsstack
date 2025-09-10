//! MCP Protocol Operations
//!
//! This module contains all MCP protocol-specific request processing operations.
//! Each operation is responsible for handling a specific MCP method with proper
//! error handling and response formatting.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use serde_json::Value;

// Layer 3: Internal module imports
use crate::protocol::{
    CallToolRequest, GetPromptRequest, InitializeRequest, InitializeResponse, JsonRpcRequest,
    LoggingConfig, ReadResourceRequest, ServerCapabilities, ServerInfo, SetLoggingRequest,
    SubscribeResourceRequest, UnsubscribeResourceRequest,
};
use crate::transport::adapters::http::session::SessionId;
use crate::transport::error::TransportError;

use super::mcp_handlers::McpHandlers;

/// Process MCP initialize request
pub async fn process_mcp_initialize(
    _mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    // Parse InitializeRequest from request.params
    let _init_request: InitializeRequest =
        serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
            TransportError::parse_error(format!("Invalid initialization request: {e}"))
        })?;

    // In a full implementation, we would store client capabilities and validate protocol version
    // For now, return initialize response with protocol negotiation using proper MCP protocol layer

    // Use proper MCP protocol layer instead of manual JSON construction
    // TODO(DEBT): Need to pass server config separately or use a default
    let default_capabilities = ServerCapabilities::default();
    let capabilities_json = serde_json::to_value(&default_capabilities).map_err(|e| {
        TransportError::serialization_error(format!("Failed to serialize capabilities: {e}"))
    })?;

    let default_server_info = ServerInfo {
        name: "airs-mcp-server".to_string(),
        version: "0.1.1".to_string(),
    };

    let mcp_response = InitializeResponse::new(
        capabilities_json,
        default_server_info,
        None, // No instructions for now
    );

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "result": serde_json::to_value(mcp_response).map_err(|e| {
            TransportError::serialization_error(format!("Failed to serialize MCP response: {e}"))
        })?
    });

    Ok(response)
}

/// Process MCP list resources request
pub async fn process_mcp_list_resources(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.resource_provider {
        match provider.list_resources().await {
            Ok(resources) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "resources": resources
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Resource provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No resource provider configured"
            }
        }))
    }
}

/// Process MCP list resource templates request
pub async fn process_mcp_list_resource_templates(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.resource_provider {
        match provider.list_resource_templates().await {
            Ok(templates) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "resourceTemplates": templates
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Resource provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No resource provider configured"
            }
        }))
    }
}

/// Process MCP read resource request
pub async fn process_mcp_read_resource(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.resource_provider {
        // Parse ReadResourceRequest from request.params
        let read_request: ReadResourceRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid read resource request: {e}"))
            })?;

        match provider.read_resource(read_request.uri.as_str()).await {
            Ok(contents) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "contents": contents
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Resource provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No resource provider configured"
            }
        }))
    }
}

/// Process MCP subscribe resource request
pub async fn process_mcp_subscribe_resource(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.resource_provider {
        // Parse SubscribeResourceRequest from request.params
        let subscribe_request: SubscribeResourceRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid subscribe resource request: {e}"))
            })?;

        match provider
            .subscribe_to_resource(subscribe_request.uri.as_str())
            .await
        {
            Ok(()) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {}
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Resource provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No resource provider configured"
            }
        }))
    }
}

/// Process MCP unsubscribe resource request
pub async fn process_mcp_unsubscribe_resource(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.resource_provider {
        // Parse UnsubscribeResourceRequest from request.params
        let unsubscribe_request: UnsubscribeResourceRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid unsubscribe resource request: {e}"))
            })?;

        match provider
            .unsubscribe_from_resource(unsubscribe_request.uri.as_str())
            .await
        {
            Ok(()) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {}
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Resource provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No resource provider configured"
            }
        }))
    }
}

/// Process MCP list tools request
pub async fn process_mcp_list_tools(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.tool_provider {
        match provider.list_tools().await {
            Ok(tools) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "tools": tools
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Tool provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No tool provider configured"
            }
        }))
    }
}

/// Process MCP call tool request
pub async fn process_mcp_call_tool(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.tool_provider {
        // Parse CallToolRequest from request.params
        let call_request: CallToolRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid call tool request: {e}"))
            })?;

        match provider
            .call_tool(&call_request.name, call_request.arguments)
            .await
        {
            Ok(content) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "content": content,
                    "isError": false
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "content": [],
                    "isError": true,
                    "errorMessage": e.to_string()
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No tool provider configured"
            }
        }))
    }
}

/// Process MCP list prompts request
pub async fn process_mcp_list_prompts(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.prompt_provider {
        match provider.list_prompts().await {
            Ok(prompts) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "prompts": prompts
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Prompt provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No prompt provider configured"
            }
        }))
    }
}

/// Process MCP get prompt request
pub async fn process_mcp_get_prompt(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &mcp_handlers.prompt_provider {
        // Parse GetPromptRequest from request.params
        let prompt_request: GetPromptRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid get prompt request: {e}"))
            })?;

        match provider
            .get_prompt(&prompt_request.name, prompt_request.arguments)
            .await
        {
            Ok((description, messages)) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "description": description,
                    "messages": messages
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Prompt provider error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No prompt provider configured"
            }
        }))
    }
}

/// Process MCP set logging request
pub async fn process_mcp_set_logging(
    mcp_handlers: &Arc<McpHandlers>,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(handler) = &mcp_handlers.logging_handler {
        // Parse SetLoggingRequest from request.params
        let logging_request: SetLoggingRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid set logging request: {e}"))
            })?;

        match handler
            .set_logging(LoggingConfig {
                level: logging_request.level,
            })
            .await
        {
            Ok(success) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": {
                    "success": success,
                    "message": if success {
                        "Logging configuration updated"
                    } else {
                        "Failed to update logging configuration"
                    }
                }
            })),
            Err(e) => Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32000,
                    "message": "Internal error",
                    "data": format!("Logging handler error: {}", e)
                }
            })),
        }
    } else {
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "No logging handler configured"
            }
        }))
    }
}
