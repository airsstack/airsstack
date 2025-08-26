// Standard library imports
use std::convert::Infallible;
use std::sync::Arc;

// Third-party crate imports
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{sse::Event, Sse},
    Json,
};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

// Internal module imports
use crate::base::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use crate::transport::http::sse::config::MigrationMode;
use crate::transport::http::sse::constants::{content_types, headers};
use crate::transport::http::sse::transport::HttpSseTransport;

/// SSE stream query parameters for client configuration
#[derive(Debug, Deserialize)]
pub struct SseQueryParams {
    /// Last event ID for resumption (SSE standard)
    #[serde(rename = "lastEventId")]
    pub last_event_id: Option<String>,

    /// Session ID for correlation with messages endpoint
    pub session_id: Option<String>,

    /// Heartbeat interval in seconds (client preference)
    pub heartbeat: Option<u64>,
}

/// JSON request format for the messages endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageRequest {
    /// JSON-RPC request payload
    pub request: JsonRpcRequest,

    /// Session ID for SSE correlation
    pub session_id: Option<String>,
}

/// JSON response format for the messages endpoint
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    /// JSON-RPC response payload
    pub response: JsonRpcResponse,

    /// Session ID that was used for correlation
    pub session_id: Option<String>,

    /// Deprecation warnings if enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_warning: Option<String>,

    /// Migration suggestion if enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration_available: Option<String>,
}

/// SSE endpoint handler for Server-Sent Events streaming
///
/// Provides unidirectional server-to-client messaging via SSE standard.
/// Clients connect to this endpoint to receive JSON-RPC messages, responses,
/// and status updates in real-time.
///
/// # Query Parameters
/// - `lastEventId`: Resume from specific event (SSE standard)
/// - `session_id`: Correlate with messages endpoint session
/// - `heartbeat`: Client-preferred heartbeat interval in seconds
///
/// # SSE Format
/// Events follow SSE specification with `event:` and `data:` fields:
/// ```
/// event: message
/// data: {"jsonrpc":"2.0","method":"ping","id":"123"}
///
/// event: heartbeat  
/// data: {"status":"connected","timestamp":"2025-08-26T..."}
/// ```
pub async fn sse_stream_handler(
    Query(params): Query<SseQueryParams>,
    State(transport): State<Arc<HttpSseTransport>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    // Subscribe to SSE broadcast channel
    let receiver = transport.broadcaster().subscribe();

    // Create SSE stream from broadcast receiver
    let stream = BroadcastStream::new(receiver).filter_map(|result| async move {
        match result {
            Ok(sse_event) => {
                // Convert SseEvent to SSE format
                let event_data = sse_event.to_sse_format();
                let event_type = sse_event.event_type();

                // Create SSE Event
                Some(Ok(Event::default().event(event_type).data(event_data)))
            }
            Err(_) => {
                // Handle any broadcast errors by ending the stream gracefully
                None
            }
        }
    });

    // Build SSE response with appropriate headers
    let mut sse_builder = Sse::new(stream);

    // Add keep-alive based on heartbeat preference
    if let Some(heartbeat_seconds) = params.heartbeat {
        let keep_alive = axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(heartbeat_seconds))
            .text("heartbeat");
        sse_builder = sse_builder.keep_alive(keep_alive);
    }

    Ok(sse_builder)
}

/// Messages endpoint handler for JSON-RPC request/response
///
/// Provides bidirectional JSON-RPC communication via HTTP POST.
/// Processes requests and returns responses, with optional session
/// correlation for SSE streaming.
///
/// # Request Format
/// ```json
/// {
///   "request": {
///     "jsonrpc": "2.0",
///     "method": "ping",
///     "id": "123"
///   },
///   "session_id": "session-456"
/// }
/// ```
///
/// # Response Format
/// ```json
/// {
///   "response": {
///     "jsonrpc": "2.0",
///     "result": "pong",
///     "id": "123"
///   },
///   "session_id": "session-456",
///   "deprecation_warning": "SSE transport is deprecated...",
///   "migration_available": "Consider migrating to HTTP Streamable"
/// }
/// ```
pub async fn messages_handler(
    State(transport): State<Arc<HttpSseTransport>>,
    Json(message_request): Json<MessageRequest>,
) -> Result<(HeaderMap, Json<MessageResponse>), (StatusCode, String)> {
    let config = transport.sse_config();

    // Process the JSON-RPC request
    // For now, we'll create a simple echo response
    // TODO: Integrate with actual MCP request processing
    let response = create_echo_response(&message_request.request);

    // Broadcast the response to SSE clients if session_id is provided
    if message_request.session_id.is_some() {
        transport.broadcast_response(response.clone()).await;
    }

    // Build response with deprecation headers if enabled
    let mut headers = HeaderMap::new();

    if config.deprecation.warnings_enabled {
        headers.insert(headers::TRANSPORT_DEPRECATED, "true".parse().unwrap());

        // Add migration assistance headers for Active mode
        if matches!(config.migration_mode, MigrationMode::Active) {
            headers.insert(
                headers::MIGRATION_AVAILABLE,
                "HTTP Streamable transport available".parse().unwrap(),
            );
        }
    }

    // Set content type
    headers.insert(header::CONTENT_TYPE, content_types::JSON.parse().unwrap());

    let message_response = MessageResponse {
        response,
        session_id: message_request.session_id,
        deprecation_warning: if config.deprecation.warnings_enabled {
            Some("SSE transport is deprecated. Consider migrating to HTTP Streamable transport for better performance.".to_string())
        } else {
            None
        },
        migration_available: if matches!(config.migration_mode, MigrationMode::Active) {
            Some("HTTP Streamable transport available with improved performance and bidirectional capabilities.".to_string())
        } else {
            None
        },
    };

    Ok((headers, Json(message_response)))
}

/// Create an echo response for testing purposes
///
/// TODO: Replace with actual MCP request processing integration
fn create_echo_response(request: &JsonRpcRequest) -> JsonRpcResponse {
    use serde_json::json;

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "echo": format!("Received method: {}", request.method),
            "original_params": request.params
        })),
        error: None,
        id: Some(request.id.clone()),
    }
}

/// Health check endpoint for SSE transport status
///
/// Provides transport status information including:
/// - Active SSE connections count
/// - Configuration status
/// - Deprecation timeline information
///
/// # Response Format
/// ```json
/// {
///   "status": "healthy",
///   "connections": 42,
///   "deprecated": true,
///   "migration_mode": "active"
/// }
/// ```
pub async fn health_handler(
    State(transport): State<Arc<HttpSseTransport>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use serde_json::json;

    let config = transport.sse_config();
    let connection_count = transport.broadcaster().connection_count();

    let health_info = json!({
        "status": "healthy",
        "transport": "sse",
        "connections": connection_count,
        "deprecated": config.deprecation.warnings_enabled,
        "migration_mode": config.migration_mode,
        "endpoints": {
            "sse": config.sse_endpoint.path,
            "messages": config.messages_endpoint
        }
    });

    Ok(Json(health_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::jsonrpc::RequestId;
    use crate::transport::http::config::HttpTransportConfig;
    use crate::transport::http::sse::config::HttpSseConfig;
    use serde_json::json;

    async fn create_test_transport() -> Arc<HttpSseTransport> {
        let http_config = HttpTransportConfig::default();
        let sse_config = HttpSseConfig::default();
        Arc::new(
            HttpSseTransport::new(http_config, sse_config)
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_echo_response_creation() {
        let request = JsonRpcRequest::new(
            "test_method",
            Some(json!({"param": "value"})),
            RequestId::new_string("test-123".to_string()),
        );

        let response = create_echo_response(&request);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert_eq!(response.id, Some(request.id.clone()));

        let result = response.result.unwrap();
        assert!(result.get("echo").is_some());
        assert!(result.get("original_params").is_some());
    }

    #[tokio::test]
    async fn test_message_request_deserialization() {
        let json_data = json!({
            "request": {
                "jsonrpc": "2.0",
                "method": "ping",
                "id": "123"
            },
            "session_id": "session-456"
        });

        let message_request: MessageRequest = serde_json::from_value(json_data).unwrap();

        assert_eq!(message_request.request.method, "ping");
        assert_eq!(message_request.session_id, Some("session-456".to_string()));
    }

    #[tokio::test]
    async fn test_message_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!("pong")),
            error: None,
            id: Some(RequestId::new_string("123".to_string())),
        };

        let message_response = MessageResponse {
            response,
            session_id: Some("session-456".to_string()),
            deprecation_warning: Some("Deprecated".to_string()),
            migration_available: Some("Available".to_string()),
        };

        let serialized = serde_json::to_value(message_response).unwrap();

        assert!(serialized.get("response").is_some());
        assert!(serialized.get("session_id").is_some());
        assert!(serialized.get("deprecation_warning").is_some());
        assert!(serialized.get("migration_available").is_some());
    }

    #[tokio::test]
    async fn test_sse_query_params_parsing() {
        // Test that our SseQueryParams struct can be deserialized correctly
        let params = SseQueryParams {
            last_event_id: Some("123".to_string()),
            session_id: Some("session-456".to_string()),
            heartbeat: Some(30),
        };

        assert_eq!(params.last_event_id, Some("123".to_string()));
        assert_eq!(params.session_id, Some("session-456".to_string()));
        assert_eq!(params.heartbeat, Some(30));
    }

    #[tokio::test]
    async fn test_health_handler_response() {
        let transport = create_test_transport().await;

        let result = health_handler(State(transport)).await;
        assert!(result.is_ok());

        let Json(health_info) = result.unwrap();
        assert_eq!(health_info["status"], "healthy");
        assert_eq!(health_info["transport"], "sse");
        assert!(health_info["connections"].is_number());
        assert!(health_info["endpoints"].is_object());
    }
}
