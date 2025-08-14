//! Axum HTTP Server Implementation for MCP Transport
//!
//! This module provides a complete HTTP server implementation using Axum framework
//! for handling MCP JSON-RPC requests. It integrates with the connection manager,
//! session manager, and MCP server infrastructure for full protocol support.

use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::base::jsonrpc::message::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest};
use crate::integration::mcp::server::McpServerConfig;
use crate::integration::mcp::{LoggingHandler, PromptProvider, ResourceProvider, ToolProvider};
use crate::shared::protocol::messages::{
    initialization::InitializeRequest,
    logging::SetLoggingRequest,
    prompts::GetPromptRequest,
    resources::{ReadResourceRequest, SubscribeResourceRequest, UnsubscribeResourceRequest},
    tools::CallToolRequest,
};
use crate::transport::error::TransportError;
use crate::transport::http::config::HttpTransportConfig;
use crate::transport::http::connection_manager::HttpConnectionManager;
use crate::transport::http::session::{ClientInfo, SessionId, SessionManager};

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

/// Shared application state for the Axum server
#[derive(Clone)]
pub struct ServerState {
    /// Connection manager for tracking HTTP connections
    pub connection_manager: Arc<HttpConnectionManager>,
    /// Session manager for handling user sessions
    pub session_manager: Arc<SessionManager>,
    /// JSON-RPC processor for handling requests
    pub jsonrpc_processor: Arc<ConcurrentProcessor>,
    /// MCP server for processing MCP protocol requests
    pub mcp_handlers: Arc<McpHandlers>,
    /// Server configuration
    pub config: HttpTransportConfig,
}

/// MCP handlers container for different provider types
pub struct McpHandlers {
    /// Resource provider for handling resource-related MCP requests
    pub resource_provider: Option<Arc<dyn ResourceProvider>>,
    /// Tool provider for handling tool-related MCP requests  
    pub tool_provider: Option<Arc<dyn ToolProvider>>,
    /// Prompt provider for handling prompt-related MCP requests
    pub prompt_provider: Option<Arc<dyn PromptProvider>>,
    /// Logging handler for MCP logging operations
    pub logging_handler: Option<Arc<dyn LoggingHandler>>,
    /// MCP server configuration
    pub config: McpServerConfig,
}

/// HTTP server implementation using Axum framework
pub struct AxumHttpServer {
    /// Server state shared across handlers
    state: ServerState,
    /// TCP listener for accepting connections
    listener: Option<TcpListener>,
}

impl AxumHttpServer {
    /// Create a new Axum HTTP server with the specified configuration and handlers
    pub async fn new(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        mcp_handlers: Arc<McpHandlers>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config: config.clone(),
        };

        Ok(Self {
            state,
            listener: None,
        })
    }

    /// Create a new Axum HTTP server with empty MCP handlers (for testing/development)
    pub async fn new_with_empty_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(McpHandlers {
            resource_provider: None,
            tool_provider: None,
            prompt_provider: None,
            logging_handler: None,
            config: McpServerConfig::default(),
        });

        Self::new(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config,
        )
        .await
    }

    /// Create a new Axum HTTP server using a handlers builder
    pub async fn with_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        handlers_builder: McpHandlersBuilder,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(handlers_builder.build());

        Self::new(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers,
            config,
        )
        .await
    }

    /// Bind the server to the specified address
    pub async fn bind(&mut self, addr: SocketAddr) -> Result<(), TransportError> {
        let listener = TcpListener::bind(addr).await.map_err(TransportError::Io)?;

        self.listener = Some(listener);
        Ok(())
    }

    /// Start the HTTP server and begin accepting connections
    pub async fn serve(self) -> Result<(), TransportError> {
        let app = self.create_router();

        let listener = self.listener.ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(TransportError::Io)?;

        Ok(())
    }

    /// Create the Axum router with all routes and middleware
    fn create_router(&self) -> Router {
        Router::new()
            // Main MCP endpoint for JSON-RPC requests
            .route("/mcp", post(handle_mcp_request))
            // Health check endpoint
            .route("/health", get(handle_health_check))
            // Server metrics endpoint
            .route("/metrics", get(handle_metrics))
            // Server status endpoint
            .route("/status", get(handle_status))
            // Add shared state
            .with_state(self.state.clone())
            // Add middleware layers
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            )
    }
}

/// Builder for MCP handlers to enable fluent configuration
pub struct McpHandlersBuilder {
    resource_provider: Option<Arc<dyn ResourceProvider>>,
    tool_provider: Option<Arc<dyn ToolProvider>>,
    prompt_provider: Option<Arc<dyn PromptProvider>>,
    logging_handler: Option<Arc<dyn LoggingHandler>>,
    config: McpServerConfig,
}

impl McpHandlersBuilder {
    /// Create a new MCP handlers builder with default configuration
    pub fn new() -> Self {
        Self {
            resource_provider: None,
            tool_provider: None,
            prompt_provider: None,
            logging_handler: None,
            config: McpServerConfig::default(),
        }
    }

    /// Set the resource provider
    pub fn with_resource_provider(mut self, provider: Arc<dyn ResourceProvider>) -> Self {
        self.resource_provider = Some(provider);
        self
    }

    /// Set the tool provider
    pub fn with_tool_provider(mut self, provider: Arc<dyn ToolProvider>) -> Self {
        self.tool_provider = Some(provider);
        self
    }

    /// Set the prompt provider
    pub fn with_prompt_provider(mut self, provider: Arc<dyn PromptProvider>) -> Self {
        self.prompt_provider = Some(provider);
        self
    }

    /// Set the logging handler
    pub fn with_logging_handler(mut self, handler: Arc<dyn LoggingHandler>) -> Self {
        self.logging_handler = Some(handler);
        self
    }

    /// Set the MCP server configuration
    pub fn with_config(mut self, config: McpServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the MCP handlers
    pub fn build(self) -> McpHandlers {
        McpHandlers {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
            config: self.config,
        }
    }
}

impl Default for McpHandlersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle MCP JSON-RPC requests on the /mcp endpoint
async fn handle_mcp_request(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Register connection with connection manager
    let connection_id = state
        .connection_manager
        .register_connection(addr)
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Connection limit exceeded: {e}"),
            )
        })?;

    // Update connection activity
    if let Err(e) = state
        .connection_manager
        .update_connection_activity(connection_id)
    {
        tracing::warn!("Failed to update connection activity: {}", e);
    }

    // Extract or create session
    let session_id = extract_or_create_session(&state, &headers, addr)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Session error: {e}")))?;

    // Parse JSON to determine message type
    let json_value: Value = serde_json::from_str(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")))?;

    // Check if it's a request (has "id") or notification (no "id")
    let response = if json_value.get("id").is_some() {
        // It's a request
        let request = JsonRpcRequest::from_json(&body).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON-RPC request: {e}"),
            )
        })?;

        process_jsonrpc_request(&state, session_id, request)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Processing error: {e}"),
                )
            })?
    } else {
        // It's a notification
        let notification = JsonRpcNotification::from_json(&body).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid JSON-RPC notification: {e}"),
            )
        })?;

        process_jsonrpc_notification(&state, session_id, notification)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Processing error: {e}"),
                )
            })?;

        // Return empty response for notifications
        serde_json::json!({"jsonrpc": "2.0"})
    };

    // Update session activity
    if let Err(e) = state.session_manager.update_session_activity(session_id) {
        tracing::warn!("Failed to update session activity: {}", e);
    }

    Ok(Json(response))
}

/// Extract session ID from headers or create a new session
async fn extract_or_create_session(
    state: &ServerState,
    headers: &HeaderMap,
    peer_addr: SocketAddr,
) -> Result<SessionId, TransportError> {
    // Try to extract existing session ID from headers
    if let Some(session_header) = headers.get("X-Session-ID") {
        if let Ok(session_str) = session_header.to_str() {
            if let Ok(session_id) = Uuid::parse_str(session_str) {
                // Validate existing session
                if state.session_manager.get_session(session_id).is_some() {
                    return Ok(session_id);
                }
            }
        }
    }

    // Create new session if none exists or invalid
    let client_info = ClientInfo {
        remote_addr: peer_addr,
        user_agent: headers
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        client_capabilities: None, // Will be populated during MCP negotiation
    };

    state.session_manager.create_session(client_info)
}

/// Process JSON-RPC request with MCP protocol support
async fn process_jsonrpc_request(
    state: &ServerState,
    session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    // Route MCP requests to appropriate handlers based on method
    match request.method.as_str() {
        // MCP Initialization
        "initialize" => process_mcp_initialize(state, session_id, request).await,
        "initialized" => {
            // Notification - no response needed, but this shouldn't be called for requests
            Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": null
            }))
        }

        // Resource Methods
        "resources/list" => process_mcp_list_resources(state, session_id, request).await,
        "resources/templates/list" => {
            process_mcp_list_resource_templates(state, session_id, request).await
        }
        "resources/read" => process_mcp_read_resource(state, session_id, request).await,
        "resources/subscribe" => process_mcp_subscribe_resource(state, session_id, request).await,
        "resources/unsubscribe" => {
            process_mcp_unsubscribe_resource(state, session_id, request).await
        }

        // Tool Methods
        "tools/list" => process_mcp_list_tools(state, session_id, request).await,
        "tools/call" => process_mcp_call_tool(state, session_id, request).await,

        // Prompt Methods
        "prompts/list" => process_mcp_list_prompts(state, session_id, request).await,
        "prompts/get" => process_mcp_get_prompt(state, session_id, request).await,

        // Logging Methods
        "logging/setLevel" => process_mcp_set_logging(state, session_id, request).await,

        // Unknown method - return method not found error
        _ => Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": format!("Unknown method: {}", request.method)
            }
        })),
    }
}

/// Process JSON-RPC notification (no response expected)
async fn process_jsonrpc_notification(
    _state: &ServerState,
    _session_id: SessionId,
    _notification: JsonRpcNotification,
) -> Result<(), TransportError> {
    // For now, just log the notification
    // In Phase 3C, we'll integrate with the actual MCP handlers
    tracing::info!("Processed notification: {}", _notification.method);
    Ok(())
}

/// Process MCP initialize request
async fn process_mcp_initialize(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    use crate::shared::protocol::types::common::ProtocolVersion;

    // Parse InitializeRequest from request.params
    let _init_request: InitializeRequest =
        serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
            TransportError::parse_error(format!("Invalid initialization request: {e}"))
        })?;

    // In a full implementation, we would store client capabilities and validate protocol version
    // For now, return initialize response with protocol negotiation
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "result": {
            "protocolVersion": ProtocolVersion::current(),
            "capabilities": state.mcp_handlers.config.capabilities,
            "serverInfo": state.mcp_handlers.config.server_info,
            "instructions": null
        }
    });

    Ok(response)
}

/// Process MCP list resources request
async fn process_mcp_list_resources(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.resource_provider {
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
async fn process_mcp_list_resource_templates(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.resource_provider {
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
async fn process_mcp_read_resource(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.resource_provider {
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
async fn process_mcp_subscribe_resource(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.resource_provider {
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
async fn process_mcp_unsubscribe_resource(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.resource_provider {
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
async fn process_mcp_list_tools(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.tool_provider {
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
async fn process_mcp_call_tool(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.tool_provider {
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
async fn process_mcp_list_prompts(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.prompt_provider {
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
async fn process_mcp_get_prompt(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(provider) = &state.mcp_handlers.prompt_provider {
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
async fn process_mcp_set_logging(
    state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    if let Some(handler) = &state.mcp_handlers.logging_handler {
        // Parse SetLoggingRequest from request.params
        let logging_request: SetLoggingRequest =
            serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                TransportError::parse_error(format!("Invalid set logging request: {e}"))
            })?;

        match handler.set_logging(logging_request.config).await {
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

/// Handle server status requests
async fn handle_status(
    State(state): State<ServerState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let status = serde_json::json!({
        "service": "airs-mcp-http-server",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "mcp",
        "transport": "http",
        "framework": "axum",
        "config": {
            "max_message_size": state.config.parser.max_message_size,
            "request_timeout": format!("{:?}", state.config.request_timeout),
            "optimization_strategy": format!("{:?}", state.config.parser.optimization_strategy),
            "max_connections": state.config.max_connections,
            "session_timeout": format!("{:?}", state.config.session_timeout),
        }
    });

    Ok(Json(status))
}

/// Handle health check requests
async fn handle_health_check(
    State(state): State<ServerState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let connection_stats = state.connection_manager.get_stats();
    let session_stats = state.session_manager.get_stats();

    let health_data = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "connections": {
            "active": connection_stats.currently_active,
            "total": connection_stats.total_created,
            "limit": connection_stats.max_connections,
        },
        "sessions": {
            "active": session_stats.currently_active,
            "total": session_stats.total_created,
        },
        "uptime": "TODO: implement uptime tracking"
    });

    Ok(Json(health_data))
}

/// Handle metrics requests
async fn handle_metrics(
    State(state): State<ServerState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let connection_stats = state.connection_manager.get_stats();
    let session_stats = state.session_manager.get_stats();
    let health_result = state.connection_manager.health_check();

    let metrics = serde_json::json!({
        "connections": {
            "total_created": connection_stats.total_created,
            "currently_active": connection_stats.currently_active,
            "total_requests": connection_stats.total_requests,
            "health_closures": connection_stats.health_closures,
            "limit_closures": connection_stats.limit_closures,
            "max_connections": connection_stats.max_connections,
            "health": {
                "healthy": health_result.healthy_connections,
                "degraded": health_result.degraded_connections,
                "unhealthy": health_result.unhealthy_connections,
                "closed": health_result.connections_closed,
            }
        },
        "sessions": {
            "total_created": session_stats.total_created,
            "currently_active": session_stats.currently_active,
            "total_requests": session_stats.total_requests,
            "timeout_cleanups": session_stats.timeout_cleanups,
            "manual_closures": session_stats.manual_closures,
        }
    });

    Ok(Json(metrics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
    use crate::correlation::manager::{CorrelationConfig, CorrelationManager};
    use crate::transport::http::connection_manager::HealthCheckConfig;
    use crate::transport::http::session::SessionConfig;

    async fn create_test_server() -> AxumHttpServer {
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        AxumHttpServer::new_with_empty_handlers(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            config,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_axum_server_creation() {
        let server = create_test_server().await;
        assert!(server.listener.is_none());
    }

    #[tokio::test]
    async fn test_axum_server_bind() {
        let mut server = create_test_server().await;
        let addr = "127.0.0.1:0".parse().unwrap();

        server.bind(addr).await.unwrap();
        assert!(server.listener.is_some());
    }

    #[tokio::test]
    async fn test_router_creation() {
        let server = create_test_server().await;
        let router = server.create_router();

        // Router should be created successfully
        // Note: Testing actual routes would require more complex setup
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_extract_session_from_headers() {
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers: Arc::new(McpHandlers {
                resource_provider: None,
                tool_provider: None,
                prompt_provider: None,
                logging_handler: None,
                config: McpServerConfig::default(),
            }),
            config,
        };

        let peer_addr = "127.0.0.1:8080".parse().unwrap();
        let mut headers = HeaderMap::new();

        // Test with no session header - should create new session
        let session_id = extract_or_create_session(&state, &headers, peer_addr)
            .await
            .unwrap();
        assert!(session_id != Uuid::nil());

        // Test with invalid session header - should create new session
        headers.insert("X-Session-ID", "invalid-uuid".parse().unwrap());
        let session_id2 = extract_or_create_session(&state, &headers, peer_addr)
            .await
            .unwrap();
        assert!(session_id2 != Uuid::nil());
    }

    #[tokio::test]
    async fn test_process_jsonrpc_request() {
        use crate::base::jsonrpc::message::RequestId;
        use crate::shared::protocol::types::common::ProtocolVersion;

        // Test initialize method with valid MCP InitializeRequest
        let init_params = serde_json::json!({
            "protocolVersion": ProtocolVersion::current(),
            "capabilities": {
                "roots": {"listChanged": false},
                "sampling": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let request =
            JsonRpcRequest::new("initialize", Some(init_params), RequestId::new_number(1));
        let session_id = Uuid::new_v4();

        // Create minimal state for testing
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers: Arc::new(McpHandlers {
                resource_provider: None,
                tool_provider: None,
                prompt_provider: None,
                logging_handler: None,
                config: McpServerConfig::default(),
            }),
            config,
        };

        let result = process_jsonrpc_request(&state, session_id, request)
            .await
            .unwrap();

        // Should return initialize response with MCP protocol info
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["id"], 1);
        assert!(result["result"].is_object());
        assert!(result["result"]["protocolVersion"].is_string());
        assert!(result["result"]["capabilities"].is_object());
        assert!(result["result"]["serverInfo"].is_object());
    }
}
