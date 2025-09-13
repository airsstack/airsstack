//! HTTP Streamable GET Handler
//!
//! These tests verify the complete HTTP Streamable GET handler functionality
//! including SSE streaming, session management, query parameter handling,
//! and broadcast channel integration.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::sync::broadcast;
use tokio::time::timeout;
use uuid::Uuid;

use airs_mcp::transport::adapters::http::axum::{
    create_router, McpSseQueryParams, ServerState, SseEvent,
};
use airs_mcp::transport::adapters::http::config::HttpTransportConfig;
use airs_mcp::transport::adapters::http::connection_manager::{
    HealthCheckConfig, HttpConnectionManager,
};
use airs_mcp::transport::adapters::http::defaults::DefaultAxumMcpRequestHandler;
use airs_mcp::transport::adapters::http::engine::McpRequestHandler;

/// Helper to create a test ServerState with all required components
async fn create_test_server_state() -> ServerState {
    let config = HttpTransportConfig::default();
    let health_config = HealthCheckConfig::default();
    let connection_manager = Arc::new(HttpConnectionManager::new(
        config.max_connections,
        health_config,
    ));

    let mcp_handler: Option<Arc<dyn McpRequestHandler>> = Some(Arc::new(
        DefaultAxumMcpRequestHandler::new(None, None, None, None),
    ));
    let (sse_broadcaster, _) = broadcast::channel::<SseEvent>(1024);

    ServerState {
        connection_manager,
        mcp_handler,
        config,
        sse_broadcaster,
        auth_middleware: None,
        authorization_layer: None,
    }
}

/// Helper to create test query parameters
fn create_test_query_params(
    last_event_id: Option<String>,
    session_id: Option<String>,
    heartbeat: Option<u64>,
) -> McpSseQueryParams {
    McpSseQueryParams {
        last_event_id,
        session_id,
        heartbeat,
    }
}

/// Test that ServerState can be created with all required components
#[tokio::test]
async fn test_server_state_creation() {
    let state = create_test_server_state().await;

    // Verify all components are properly initialized
    // Instead of null pointer checks, just verify they exist by using Arc::strong_count
    assert!(Arc::strong_count(&state.connection_manager) > 0);
    assert!(state.mcp_handler.is_some());
    if let Some(ref handler) = state.mcp_handler {
        assert!(Arc::strong_count(handler) > 0);
    }
    assert_eq!(state.config.bind_address.port(), 3000); // Default port
}

/// Test SSE query parameters parsing structure
#[tokio::test]
async fn test_sse_query_params_structure() {
    let query_params = create_test_query_params(
        Some("123".to_string()),
        Some(Uuid::new_v4().to_string()),
        Some(30),
    );

    assert_eq!(query_params.last_event_id, Some("123".to_string()));
    assert!(query_params.session_id.is_some());
    assert_eq!(query_params.heartbeat, Some(30));
}

/// Test SSE event broadcasting and reception
#[tokio::test]
async fn test_sse_event_broadcasting() {
    let state = create_test_server_state().await;
    let session_id = Uuid::new_v4();

    // Create test SSE event
    let test_event = SseEvent::new(
        "test_message".to_string(),
        json!({"message": "Hello SSE"}),
        session_id,
    );

    // Subscribe to broadcaster before sending
    let mut receiver = state.sse_broadcaster.subscribe();

    // Send event through broadcaster
    let send_result = state.sse_broadcaster.send(test_event.clone());
    assert!(send_result.is_ok(), "Should be able to send SSE event");

    // Receive event
    let received = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(received.is_ok(), "Should receive event within timeout");

    let received_event = received.unwrap().unwrap();
    assert_eq!(received_event.event_type, "test_message");
    assert_eq!(received_event.session_id, session_id);
}

/// Test SSE event format conversion
#[tokio::test]
async fn test_sse_event_format() {
    let session_id = Uuid::new_v4();
    let test_data = json!({"key": "value", "number": 42});

    let sse_event = SseEvent::new("message".to_string(), test_data.clone(), session_id);

    // Test SSE format string
    let sse_format = sse_event.to_sse_format();

    // Should contain required SSE fields
    assert!(sse_format.contains("id:"), "Should contain event ID");
    assert!(
        sse_format.contains("event: message"),
        "Should contain event type"
    );
    assert!(sse_format.contains("data:"), "Should contain data field");
    assert!(
        sse_format.contains("\"key\":\"value\""),
        "Should contain JSON data"
    );
    assert!(
        sse_format.ends_with("\n\n"),
        "Should end with double newline"
    );
}

/// Test router creation with HTTP Streamable GET endpoint
#[tokio::test]
async fn test_router_creation_with_get_endpoint() {
    let state = create_test_server_state().await;

    // Create router with GET endpoint
    let _router = create_router(state);

    // Router should be created successfully
    // Note: We can't easily test route registration in unit tests,
    // but we can verify the router was created without panics
}

/// Test connection manager configuration
#[tokio::test]
async fn test_connection_manager_configuration() {
    let _config = HttpTransportConfig::default();
    let health_config = HealthCheckConfig::default();

    // Should be able to create connection manager with various configurations
    let _connection_manager = HttpConnectionManager::new(100, health_config.clone());
    // Connection manager creation should succeed (no panic)

    let _zero_connection_manager = HttpConnectionManager::new(0, health_config);
    // Connection manager with 0 connections should still be creatable (no panic)
}

/// Test SSE event unique ID generation
#[tokio::test]
async fn test_sse_event_unique_ids() {
    let session_id = Uuid::new_v4();
    let test_data = json!({"test": "data"});

    let event1 = SseEvent::new("message".to_string(), test_data.clone(), session_id);
    let event2 = SseEvent::new("message".to_string(), test_data.clone(), session_id);

    // Event IDs should be unique
    assert_ne!(event1.id, event2.id, "Event IDs should be unique");
}

/// Test SSE event with different types
#[tokio::test]
async fn test_sse_event_types() {
    let session_id = Uuid::new_v4();

    let message_event = SseEvent::new("message".to_string(), json!({}), session_id);
    let error_event = SseEvent::new("error".to_string(), json!({}), session_id);
    let heartbeat_event = SseEvent::new("heartbeat".to_string(), json!({}), session_id);

    assert_eq!(message_event.event_type, "message");
    assert_eq!(error_event.event_type, "error");
    assert_eq!(heartbeat_event.event_type, "heartbeat");
}

/// Test configuration integration between components
#[tokio::test]
async fn test_configuration_integration() {
    let config = HttpTransportConfig::default();
    let health_config = HealthCheckConfig::default();

    // All configurations should have reasonable defaults
    assert!(config.max_connections > 0, "Should allow connections");
    assert!(
        health_config.check_interval.as_secs() > 0,
        "Should have health check interval"
    );
}

/// Test broadcast channel capacity and overflow behavior
#[tokio::test]
async fn test_broadcast_channel_behavior() {
    let (broadcaster, mut receiver) = broadcast::channel::<SseEvent>(2); // Small capacity
    let session_id = Uuid::new_v4();

    // Fill the channel beyond capacity
    for i in 0..5 {
        let event = SseEvent::new("test".to_string(), json!({"index": i}), session_id);
        let _ = broadcaster.send(event);
    }

    // Should still be able to receive events (old ones may be dropped)
    let received = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(received.is_ok(), "Should receive at least one event");
}
