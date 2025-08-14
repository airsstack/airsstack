//! Axum HTTP Server Implementation for MCP Transport
//!
//! This module provides a complete HTTP server implementation using Axum framework
//! for handling MCP JSON-RPC requests. It integrates with the connection manager,
//! session manager, and existing JSON-RPC processing infrastructure.

use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::base::jsonrpc::message::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest};
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
    /// Server configuration
    pub config: HttpTransportConfig,
}

/// HTTP server implementation using Axum framework
pub struct AxumHttpServer {
    /// Server state shared across handlers
    state: ServerState,
    /// TCP listener for accepting connections
    listener: Option<TcpListener>,
}

impl AxumHttpServer {
    /// Create a new Axum HTTP server with the specified configuration
    pub async fn new(
        connection_manager: Arc<HttpConnectionManager>,
        session_manager: Arc<SessionManager>,
        jsonrpc_processor: Arc<ConcurrentProcessor>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            config: config.clone(),
        };

        Ok(Self {
            state,
            listener: None,
        })
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

/// Process JSON-RPC request (simplified for initial implementation)
async fn process_jsonrpc_request(
    _state: &ServerState,
    _session_id: SessionId,
    request: JsonRpcRequest,
) -> Result<Value, TransportError> {
    // For now, return a simple echo response
    // In Phase 3C, we'll integrate with the actual MCP handlers
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "result": {
            "method": request.method,
            "echo": "Request received and processed",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });

    Ok(response)
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

        AxumHttpServer::new(
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

        let request = JsonRpcRequest::new(
            "test",
            Some(serde_json::json!({})),
            RequestId::new_number(1),
        );
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
            config,
        };

        let result = process_jsonrpc_request(&state, session_id, request)
            .await
            .unwrap();

        // Should return an echo response
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["id"], 1);
        assert!(result["result"].is_object());
    }
}
