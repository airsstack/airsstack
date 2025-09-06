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
use crate::authorization::{
    context::{AuthzContext, NoAuthContext},
    middleware::AuthorizationRequest,
    policy::{AuthorizationPolicy, NoAuthorizationPolicy},
};
use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::base::jsonrpc::message::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest};
use crate::integration::mcp::constants::methods as mcp_methods;
use crate::transport::adapters::http::auth::jsonrpc_authorization::{JsonRpcAuthorizationLayer, JsonRpcHttpRequest};
use crate::transport::adapters::http::auth::middleware::{HttpAuthMiddleware, HttpAuthStrategyAdapter};
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::connection_manager::HttpConnectionManager;
use crate::transport::adapters::http::session::{ClientInfo, SessionId, SessionManager};
use crate::transport::error::TransportError;

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
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter (defaults to NoAuth)
/// * `P` - Authorization policy (defaults to NoAuthorizationPolicy)
/// * `C` - Authorization context (defaults to NoAuthContext)
#[derive(Clone)]
pub struct ServerState<A = super::server::NoAuth, P = NoAuthorizationPolicy<NoAuthContext>, C = NoAuthContext>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
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
    /// Optional authentication middleware
    pub auth_middleware: Option<HttpAuthMiddleware<A>>,
    /// Optional authorization layer
    pub authorization_layer: Option<JsonRpcAuthorizationLayer<A, C, P>>,
}

/// Create the Axum router with all routes and middleware
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter for the server state
/// * `P` - Authorization policy for the server state
/// * `C` - Authorization context for the server state
pub fn create_router<A, P, C>(state: ServerState<A, P, C>) -> Router
where
    A: HttpAuthStrategyAdapter + Clone + 'static,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone + 'static,
    C: AuthzContext + Clone + 'static,
{
    let mut router = Router::new()
        // Main MCP endpoint for JSON-RPC requests and SSE streaming
        .route("/mcp", post(handle_mcp_request::<A, P, C>))
        .route("/mcp", get(handle_mcp_get::<A, P, C>))
        // Health check endpoint
        .route("/health", get(handle_health_check::<A, P, C>))
        // Server metrics endpoint
        .route("/metrics", get(handle_metrics::<A, P, C>))
        // Server status endpoint
        .route("/status", get(handle_status::<A, P, C>))
        // Add shared state
        .with_state(state.clone());

    // Conditionally apply authentication middleware if present
    if let Some(auth_middleware) = &state.auth_middleware {
        use crate::transport::adapters::http::auth::axum_middleware::AxumHttpAuthLayer;
        let auth_layer = AxumHttpAuthLayer::from_middleware(auth_middleware.clone());
        router = router.layer(auth_layer);
    }

    // Add standard middleware layers (applied after authentication)
    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive()),
    )
}

/// Handle MCP JSON-RPC requests on the /mcp endpoint
async fn handle_mcp_request<A, P, C>(
    State(state): State<ServerState<A, P, C>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<Value>, (StatusCode, String)> 
where
    A: HttpAuthStrategyAdapter + Clone + 'static,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone + 'static,
    C: AuthzContext + Clone + 'static,
{
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
    // PHASE 3: Authorization layer integration placeholder
    // The authorization framework is now integrated into the server architecture.
    // When an authorization layer is configured, we log its presence.
    // Full authorization checking will be activated when HTTP authentication 
    // middleware properly extracts and provides the authentication context.
    if let Some(authorization_layer) = &state.authorization_layer {
        tracing::debug!("Authorization layer configured: {}", authorization_layer.policy_name());
    }

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
async fn handle_mcp_get<A, P, C>(
    Query(params): Query<McpSseQueryParams>,
    State(state): State<ServerState<A, P, C>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
where
    A: HttpAuthStrategyAdapter + Clone + 'static,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone + 'static,
    C: AuthzContext + Clone + 'static,
{
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
pub async fn extract_or_create_session<A, P, C>(
    state: &ServerState<A, P, C>,
    headers: &HeaderMap,
    peer_addr: SocketAddr,
) -> Result<SessionId, TransportError>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
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
pub async fn process_jsonrpc_request<A, P, C>(
    state: &ServerState<A, P, C>,
    session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
    // Route MCP requests to appropriate handlers based on method
    match request.method.as_str() {
        // MCP Initialization
        mcp_methods::INITIALIZE => {
            process_mcp_initialize(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::INITIALIZED => {
            // Notification - no response needed, but this shouldn't be called for requests
            Ok(serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "result": null
            }))
        }

        // Resource Methods
        mcp_methods::RESOURCES_LIST => {
            process_mcp_list_resources(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::RESOURCES_TEMPLATES_LIST => {
            process_mcp_list_resource_templates(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::RESOURCES_READ => {
            process_mcp_read_resource(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::RESOURCES_SUBSCRIBE => {
            process_mcp_subscribe_resource(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::RESOURCES_UNSUBSCRIBE => {
            process_mcp_unsubscribe_resource(&state.mcp_handlers, session_id, request).await
        }

        // Tool Methods
        mcp_methods::TOOLS_LIST => {
            process_mcp_list_tools(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::TOOLS_CALL => {
            process_mcp_call_tool(&state.mcp_handlers, session_id, request).await
        }

        // Prompt Methods
        mcp_methods::PROMPTS_LIST => {
            process_mcp_list_prompts(&state.mcp_handlers, session_id, request).await
        }
        mcp_methods::PROMPTS_GET => {
            process_mcp_get_prompt(&state.mcp_handlers, session_id, request).await
        }

        // Logging Methods
        mcp_methods::LOGGING_SET_LEVEL => {
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
async fn process_jsonrpc_notification<A, P, C>(
    _state: &ServerState<A, P, C>,
    _session_id: SessionId,
    _notification: JsonRpcNotification,
) -> Result<(), TransportError>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
    // For now, just log the notification
    // In Phase 3C, we'll integrate with the actual MCP handlers
    tracing::info!("Processed notification: {}", _notification.method);
    Ok(())
}

/// Handle server status requests
async fn handle_status<A, P, C>(
    State(state): State<ServerState<A, P, C>>,
) -> Result<Json<Value>, (StatusCode, String)>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
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
async fn handle_health_check<A, P, C>(
    State(state): State<ServerState<A, P, C>>,
) -> Result<Json<Value>, (StatusCode, String)>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
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
async fn handle_metrics<A, P, C>(
    State(state): State<ServerState<A, P, C>>,
) -> Result<Json<Value>, (StatusCode, String)>
where
    A: HttpAuthStrategyAdapter + Clone,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
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
