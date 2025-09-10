use axum::{routing::get, Router};

use airs_mcp::transport::adapters::http::sse::{
    handlers::{health_handler, messages_handler, sse_stream_handler, MessageRequest},
};
use airs_mcp::protocol::{JsonRpcRequest, RequestId};

/// Test that handlers can be used in routes (basic compilation test)
#[tokio::test]
async fn test_handlers_compilation() -> Result<(), Box<dyn std::error::Error>> {
    // This test verifies that our handlers can be used in Axum routers
    // without requiring actual transport state
    
    // Test route creation doesn't panic
    let _app = Router::new()
        .route("/sse", get(sse_stream_handler))
        .route("/messages", axum::routing::post(messages_handler))
        .route("/health", get(health_handler));
    
    Ok(())
}

/// Test MessageRequest serialization
#[tokio::test]
async fn test_message_request_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let message_request = MessageRequest {
        request: JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ping".to_string(),
            params: None,
            id: RequestId::new_string("test_123".to_string()),
        },
        session_id: Some("test_session".to_string()),
    };
    
    // Test that MessageRequest can be serialized
    let json_string = serde_json::to_string(&message_request)?;
    assert!(json_string.contains("ping"));
    assert!(json_string.contains("test_session"));
    
    // Test that it can be deserialized back
    let parsed: MessageRequest = serde_json::from_str(&json_string)?;
    assert_eq!(parsed.request.method, "ping");
    assert_eq!(parsed.session_id, Some("test_session".to_string()));
    
    Ok(())
}

/// Test that handlers have correct function signatures for Axum
#[tokio::test] 
async fn test_handler_signatures() -> Result<(), Box<dyn std::error::Error>> {
    // This is a compilation test to ensure our handlers have the right signatures
    // for Axum routing
    
    // Create routes - if this compiles, the signatures are correct
    let _sse_route = get(sse_stream_handler);
    let _messages_route = axum::routing::post(messages_handler);
    let _health_route = get(health_handler);
    
    Ok(())
}
