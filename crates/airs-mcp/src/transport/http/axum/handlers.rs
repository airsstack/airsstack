//! HTTP Request Handlers
//!
//! This module contains HTTP endpoint handlers for health checks, metrics,
//! status monitoring, and JSON-RPC request processing. Each handler focuses
//! on a specific endpoint responsibility.

// Layer 1: Standard library imports
use std::net::SocketAddr;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{sse::Event, Json, Sse},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::base::jsonrpc::message::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest};
use crate::transport::error::TransportError;
use crate::transport::http::config::HttpTransportConfig;
use crate::transport::http::connection_manager::HttpConnectionManager;
use crate::transport::http::session::{ClientInfo, SessionId, SessionManager};

use super::mcp_handlers::McpHandlers;
use super::mcp_operations::*;

/// SSE stream query parameters for HTTP Streamable GET requests
#[derive(Debug, Deserialize)]
pub struct McpSseQueryParams {
    /// Last event ID for resumption (SSE standard)
    #[serde(rename = "lastEventId")]
    pub last_event_id: Option<String>,

    /// Session ID for correlation (optional, will create if missing)
    pub session_id: Option<String>,

    /// Heartbeat interval in seconds (client preference)
    pub heartbeat: Option<u64>,
}

/// SSE Event for HTTP Streamable transport
#[derive(Debug, Clone, Serialize)]
pub struct SseEvent {
    /// Event ID for resumption support
    pub id: String,
    /// Event type (message, notification, error, heartbeat)
    pub event_type: String,
    /// JSON data payload
    pub data: Value,
    /// Session ID for correlation
    pub session_id: SessionId,
    /// Timestamp for event ordering
    pub timestamp: DateTime<Utc>,
}

impl SseEvent {
    /// Create a new SSE event
    pub fn new(event_type: String, data: Value, session_id: SessionId) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            data,
            session_id,
            timestamp: Utc::now(),
        }
    }

    /// Convert to SSE format string
    pub fn to_sse_format(&self) -> String {
        format!(
            "id: {}\nevent: {}\ndata: {}\n\n",
            self.id,
            self.event_type,
            serde_json::to_string(&self.data).unwrap_or_default()
        )
    }
}

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
    /// Broadcast channel for SSE events
    pub sse_broadcaster: broadcast::Sender<SseEvent>,
}

/// Create the Axum router with all routes and middleware
pub fn create_router(state: ServerState) -> Router {
    Router::new()
        // Main MCP endpoint for JSON-RPC requests and SSE streaming
        .route("/mcp", post(handle_mcp_request))
        .route("/mcp", get(handle_mcp_get))
        // Health check endpoint
        .route("/health", get(handle_health_check))
        // Server metrics endpoint
        .route("/metrics", get(handle_metrics))
        // Server status endpoint
        .route("/status", get(handle_status))
        // Add shared state
        .with_state(state)
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
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

/// Handle MCP SSE streaming requests on the /mcp endpoint (GET method)
async fn handle_mcp_get(
    Query(params): Query<McpSseQueryParams>,
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
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
    let session_id = if let Some(provided_session_id) = params.session_id {
        // Try to parse provided session ID
        Uuid::parse_str(&provided_session_id).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid session ID format".to_string(),
            )
        })?
    } else {
        // Extract or create session using existing logic
        extract_or_create_session(&state, &headers, addr)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Session error: {e}")))?
    };

    // Log event resumption if last_event_id is provided
    if let Some(ref last_event_id) = params.last_event_id {
        tracing::info!("Resuming SSE stream from event ID: {}", last_event_id);
        // TODO(TASK025): Implement event replay from last_event_id
        // This will require maintaining event history for resumption
    }

    // Subscribe to SSE broadcast channel
    let receiver = state.sse_broadcaster.subscribe();

    // Create SSE stream from broadcast receiver, filtering by session
    let stream = BroadcastStream::new(receiver).filter_map(move |result| async move {
        match result {
            Ok(sse_event) => {
                // Current implementation: Broadcast to all sessions
                // This design allows for global events (system status, announcements)
                // and simplifies the initial implementation. Session-specific filtering
                // can be added later when per-session event routing is required.

                // Convert SseEvent to SSE Event
                let event_id = sse_event.id.clone();
                let event_type = sse_event.event_type.clone();
                let event_data = serde_json::to_string(&sse_event.data).unwrap_or_default();

                Some(Ok(Event::default()
                    .id(event_id)
                    .event(event_type)
                    .data(event_data)))
            }
            Err(_) => {
                // Handle broadcast errors by ending the stream gracefully
                None
            }
        }
    });

    // Build SSE response with appropriate headers and keep-alive
    let mut sse_builder = Sse::new(stream);

    // Add keep-alive based on heartbeat preference
    if let Some(heartbeat_seconds) = params.heartbeat {
        let keep_alive = axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(heartbeat_seconds))
            .text("heartbeat");
        sse_builder = sse_builder.keep_alive(keep_alive);
    } else {
        // Default 30-second heartbeat
        let keep_alive = axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("heartbeat");
        sse_builder = sse_builder.keep_alive(keep_alive);
    }

    // Update session activity
    if let Err(e) = state.session_manager.update_session_activity(session_id) {
        tracing::warn!("Failed to update session activity: {}", e);
    }

    Ok(sse_builder)
}

/// Extract session ID from headers or create a new session
pub async fn extract_or_create_session(
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
pub async fn process_jsonrpc_request(
    state: &ServerState,
    session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    // Route MCP requests to appropriate handlers based on method
    match request.method.as_str() {
        // MCP Initialization
        "initialize" => process_mcp_initialize(&state.mcp_handlers, session_id, request).await,
        "initialized" => {
            // Notification - no response needed, but this shouldn't be called for requests
            Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": null
            }))
        }

        // Resource Methods
        "resources/list" => {
            process_mcp_list_resources(&state.mcp_handlers, session_id, request).await
        }
        "resources/templates/list" => {
            process_mcp_list_resource_templates(&state.mcp_handlers, session_id, request).await
        }
        "resources/read" => {
            process_mcp_read_resource(&state.mcp_handlers, session_id, request).await
        }
        "resources/subscribe" => {
            process_mcp_subscribe_resource(&state.mcp_handlers, session_id, request).await
        }
        "resources/unsubscribe" => {
            process_mcp_unsubscribe_resource(&state.mcp_handlers, session_id, request).await
        }

        // Tool Methods
        "tools/list" => process_mcp_list_tools(&state.mcp_handlers, session_id, request).await,
        "tools/call" => process_mcp_call_tool(&state.mcp_handlers, session_id, request).await,

        // Prompt Methods
        "prompts/list" => process_mcp_list_prompts(&state.mcp_handlers, session_id, request).await,
        "prompts/get" => process_mcp_get_prompt(&state.mcp_handlers, session_id, request).await,

        // Logging Methods
        "logging/setLevel" => {
            process_mcp_set_logging(&state.mcp_handlers, session_id, request).await
        }

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
        "uptime": {
            "status": "operational",
            "message": "Uptime tracking will be implemented in future release"
        }
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
