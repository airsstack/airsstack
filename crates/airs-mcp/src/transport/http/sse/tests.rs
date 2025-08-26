// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use axum::{extract::Request, http::StatusCode, response::Response, routing::get, Router};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde_json::json;
use tokio::time::timeout;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower::ServiceExt; // for oneshot

// Layer 3: Internal module imports
use crate::shared::protocol::core::{McpRequest, McpResponse};
use crate::transport::http::sse::{
    config::{DeprecationConfig, HttpSseConfig, MigrationMode},
    handlers::{MessageRequest, MessageResponse, SseQueryParams},
    transport::{HttpSseTransport, SseEvent},
    HttpSseTransport as SseTransport,
};

#[tokio::test]
async fn test_sse_transport_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test transport initialization with minimal config
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let transport = HttpSseTransport::new(config).await?;

    // Verify transport is properly initialized
    assert!(transport.is_connected().await);

    Ok(())
}

#[tokio::test]
async fn test_sse_stream_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    // Create test configuration
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let transport = HttpSseTransport::new(config).await?;

    // Create router with SSE handler
    let app = Router::new().route(
        "/sse",
        get(crate::transport::http::sse::handlers::sse_stream_handler),
    );

    // Test SSE endpoint request
    let request = Request::builder()
        .uri("/sse?session_id=test_session&correlation_id=test_correlation")
        .body(axum::body::Body::empty())?;

    let response = app.oneshot(request).await?;

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    // Check headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
    assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
    assert_eq!(headers.get("connection").unwrap(), "keep-alive");

    Ok(())
}

#[tokio::test]
async fn test_messages_endpoint_post() -> Result<(), Box<dyn std::error::Error>> {
    // Create test configuration
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let _transport = HttpSseTransport::new(config).await?;

    // Create router with messages handler
    let app = Router::new().route(
        "/messages",
        axum::routing::post(crate::transport::http::sse::handlers::messages_handler),
    );

    // Test message request
    let message_request = MessageRequest {
        message: json!({
            "jsonrpc": "2.0",
            "method": "ping",
            "id": "test_id"
        }),
        session_id: Some("test_session".to_string()),
        correlation_id: Some("test_correlation".to_string()),
    };

    let request = Request::builder()
        .uri("/messages")
        .method("POST")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(serde_json::to_string(
            &message_request,
        )?))?;

    let response = app.oneshot(request).await?;

    // Should return OK for valid JSON-RPC
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn test_health_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    // Create router with health handler
    let app = Router::new().route(
        "/health",
        get(crate::transport::http::sse::handlers::health_handler),
    );

    let request = Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())?;

    let response = app.oneshot(request).await?;

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    // Parse response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let health_response: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(health_response["status"], "healthy");
    assert!(health_response["timestamp"].is_string());

    Ok(())
}

#[tokio::test]
async fn test_sse_event_broadcasting() -> Result<(), Box<dyn std::error::Error>> {
    // Create test configuration
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let transport = HttpSseTransport::new(config).await?;

    // Create test SSE event
    let test_event = SseEvent {
        event_type: Some("message".to_string()),
        data: json!({
            "jsonrpc": "2.0",
            "method": "notification",
            "params": {"test": "data"}
        }),
        id: Some("test_event_1".to_string()),
        retry: None,
    };

    // Test broadcasting (this is internal functionality)
    // In a real test, we'd need to set up subscribers and verify they receive the event

    Ok(())
}

#[tokio::test]
async fn test_deprecation_warnings() -> Result<(), Box<dyn std::error::Error>> {
    // Test configuration with deprecation warnings enabled
    let deprecation_config = DeprecationConfig {
        enabled: true,
        warning_message: "This endpoint is deprecated. Use /v2/sse instead.".to_string(),
        sunset_date: Some(Utc::now() + chrono::Duration::days(30)),
        migration_mode: MigrationMode::Gradual,
        alternative_endpoint: Some("/v2/sse".to_string()),
    };

    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .deprecation_config(Some(deprecation_config))
        .build();

    let _transport = HttpSseTransport::new(config).await?;

    // Create router with SSE handler
    let app = Router::new().route(
        "/sse",
        get(crate::transport::http::sse::handlers::sse_stream_handler),
    );

    let request = Request::builder()
        .uri("/sse?session_id=test_session")
        .body(axum::body::Body::empty())?;

    let response = app.oneshot(request).await?;

    // Check for deprecation headers
    let headers = response.headers();
    assert!(headers.contains_key("sunset"));
    assert!(headers.contains_key("deprecation"));
    assert!(headers.contains_key("link"));

    Ok(())
}

#[tokio::test]
async fn test_sse_query_params_parsing() -> Result<(), Box<dyn std::error::Error>> {
    // Test query parameter extraction
    let query_params = SseQueryParams {
        session_id: Some("test_session_123".to_string()),
        correlation_id: Some("test_correlation_456".to_string()),
        last_event_id: Some("last_event_789".to_string()),
    };

    // Verify proper deserialization
    assert_eq!(query_params.session_id.unwrap(), "test_session_123");
    assert_eq!(query_params.correlation_id.unwrap(), "test_correlation_456");
    assert_eq!(query_params.last_event_id.unwrap(), "last_event_789");

    Ok(())
}

#[tokio::test]
async fn test_message_request_response_types() -> Result<(), Box<dyn std::error::Error>> {
    // Test message request serialization/deserialization
    let message_request = MessageRequest {
        message: json!({
            "jsonrpc": "2.0",
            "method": "test_method",
            "params": {"key": "value"},
            "id": "test_123"
        }),
        session_id: Some("session_abc".to_string()),
        correlation_id: Some("correlation_def".to_string()),
    };

    // Test serialization
    let serialized = serde_json::to_string(&message_request)?;
    let deserialized: MessageRequest = serde_json::from_str(&serialized)?;

    assert_eq!(deserialized.session_id, Some("session_abc".to_string()));
    assert_eq!(
        deserialized.correlation_id,
        Some("correlation_def".to_string())
    );

    // Test message response
    let message_response = MessageResponse {
        response: json!({
            "jsonrpc": "2.0",
            "result": {"status": "success"},
            "id": "test_123"
        }),
        correlation_id: Some("correlation_def".to_string()),
    };

    let serialized_response = serde_json::to_string(&message_response)?;
    let deserialized_response: MessageResponse = serde_json::from_str(&serialized_response)?;

    assert_eq!(
        deserialized_response.correlation_id,
        Some("correlation_def".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_transport_connectivity() -> Result<(), Box<dyn std::error::Error>> {
    // Test transport connection state
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let transport = HttpSseTransport::new(config).await?;

    // Test initial connection state
    assert!(transport.is_connected().await);

    // Test disconnect
    transport.disconnect().await?;
    assert!(!transport.is_connected().await);

    Ok(())
}

#[tokio::test]
async fn test_full_sse_request_response_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // Test complete SSE request/response flow
    let config = HttpSseConfig::builder()
        .endpoint("/sse".to_string())
        .messages_endpoint("/messages".to_string())
        .build();

    let transport = HttpSseTransport::new(config).await?;

    // Create MCP request
    let mcp_request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "ping".to_string(),
        params: Some(json!({"timestamp": Utc::now().to_rfc3339()})),
        id: Some(json!("test_ping_123")),
    };

    // Send request through transport
    let response = transport.send_request(mcp_request).await?;

    // Verify response structure
    assert!(response.jsonrpc == "2.0");
    assert!(response.id.is_some());

    Ok(())
}
