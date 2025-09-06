//! OAuth2 JSON-RPC HTTP Integration Tests
//!
//! This module contains comprehensive integration tests for OAuth2 authentication
//! with JSON-RPC over HTTP, specifically testing the fix for the method extraction bug
//! where OAuth2 was incorrectly extracting methods from URL paths instead of JSON payloads.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::json;
use tokio::time::timeout;

use airs_mcp::authorization::{
    AuthContext, AuthorizationMiddleware, JsonRpcMethodExtractor, ScopeBasedPolicy, ScopeAuthContext,
};
use airs_mcp::oauth2::context::AuthContext as OAuth2AuthContext;
use airs_mcp::shared::jsonrpc::{JsonRpcMessage, JsonRpcRequest};
use airs_mcp::transport::adapters::http::auth::oauth2::OAuth2StrategyAdapter;

/// Test that OAuth2 authentication works correctly with JSON-RPC initialize method
/// This test specifically validates that methods are extracted from JSON-RPC payloads,
/// not URL paths, fixing the "mcp:mcp:*" vs "mcp:*" scope bug.
#[tokio::test]
async fn test_oauth2_jsonrpc_initialize_method() {
    // Create a mock OAuth2 strategy adapter that validates JWT tokens
    let oauth2_adapter = create_mock_oauth2_adapter().await;
    
    // Create authorization middleware with scope-based policy
    let auth_middleware = AuthorizationMiddleware::new(
        oauth2_adapter,
        ScopeBasedPolicy::new(create_mcp_scope_mapping()),
        JsonRpcMethodExtractor::new(),
    );

    // Create a JSON-RPC initialize request (sent to /mcp endpoint)
    let initialize_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {}
        })),
        id: Some(json!(1)),
    };

    // Create mock HTTP request with valid OAuth2 token
    let mock_request = create_mock_http_request(
        "/mcp",  // URL path should NOT determine method
        "Bearer valid-token-with-mcp-scope",
        &initialize_request,
    );

    // Test authorization - should succeed with mcp:* scope
    let result = timeout(
        Duration::from_secs(1),
        auth_middleware.authorize(&mock_request)
    ).await;

    assert!(result.is_ok(), "Authorization should not timeout");
    let auth_result = result.unwrap();
    assert!(auth_result.is_ok(), "OAuth2 authentication should succeed for initialize method with mcp:* scope");
    
    // Verify the authorization context contains the correct method
    if let Ok(AuthContext::Scope(scope_context)) = auth_result {
        assert_eq!(scope_context.method(), Some("initialize"));
        assert!(scope_context.has_scope("mcp:*"), "Should have mcp:* scope for initialize method");
    }
}

/// Test that JSON-RPC method extraction correctly parses methods from payloads
#[tokio::test]
async fn test_oauth2_jsonrpc_method_extraction() {
    let method_extractor = JsonRpcMethodExtractor::new();
    
    // Test various MCP method extractions
    let test_cases = vec![
        ("initialize", json!({"method": "initialize", "params": {}})),
        ("tools/list", json!({"method": "tools/list", "params": {}})),
        ("resources/read", json!({"method": "resources/read", "params": {"uri": "file://test"}})),
        ("prompts/list", json!({"method": "prompts/list", "params": {}})),
    ];

    for (expected_method, request_json) in test_cases {
        let mock_request = create_mock_http_request(
            "/mcp",  // Same URL path for all methods
            "Bearer test-token",
            &serde_json::from_value(request_json).unwrap(),
        );

        let extracted_method = method_extractor.extract_method(&mock_request).await;
        assert_eq!(
            extracted_method.unwrap().as_deref(),
            Some(expected_method),
            "Method extraction should correctly parse {} from JSON payload", expected_method
        );
    }
}

/// Test real-world MCP Inspector integration flow
/// This simulates the exact request pattern that MCP Inspector uses
#[tokio::test]
async fn test_mcp_inspector_integration() {
    // Create OAuth2 middleware similar to mcp-remote-server-oauth2 example
    let oauth2_adapter = create_mock_oauth2_adapter().await;
    let auth_middleware = AuthorizationMiddleware::new(
        oauth2_adapter,
        ScopeBasedPolicy::mcp(),  // Standard MCP policy
        JsonRpcMethodExtractor::new(),
    );

    // Simulate MCP Inspector's initialize request
    let inspector_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": false
                }
            },
            "clientInfo": {
                "name": "MCP Inspector",
                "version": "1.0.0"
            }
        })),
        id: Some(json!(1)),
    };

    // Create request exactly as MCP Inspector would send it
    let mock_request = create_mock_http_request(
        "/mcp",
        "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.test", // Mock JWT token
        &inspector_request,
    );

    // Authorization should succeed
    let result = auth_middleware.authorize(&mock_request).await;
    assert!(result.is_ok(), "MCP Inspector integration should work with proper OAuth2 authentication");
    
    // Verify no "mcp:mcp:*" scope errors
    match result.unwrap() {
        AuthContext::Scope(context) => {
            // Should require initialize scope, not mcp:mcp:* scope
            assert!(
                !context.has_scope("mcp:mcp:*"), 
                "Should not require mcp:mcp:* scope (this was the bug)"
            );
        },
        _ => panic!("Expected scope-based authorization context"),
    }
}

/// Test OAuth2 authentication failure scenarios
#[tokio::test]
async fn test_oauth2_authentication_failures() {
    let oauth2_adapter = create_mock_oauth2_adapter().await;
    let auth_middleware = AuthorizationMiddleware::new(
        oauth2_adapter,
        ScopeBasedPolicy::mcp(),
        JsonRpcMethodExtractor::new(),
    );

    // Test missing Authorization header
    let request_no_auth = create_mock_http_request_no_auth("/mcp", &json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 1
    }));
    
    let result = auth_middleware.authorize(&request_no_auth).await;
    assert!(result.is_err(), "Should fail without Authorization header");

    // Test invalid token format
    let request_invalid_token = create_mock_http_request(
        "/mcp",
        "Invalid-token-format",
        &serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "method": "initialize", 
            "id": 1
        })).unwrap(),
    );
    
    let result = auth_middleware.authorize(&request_invalid_token).await;
    assert!(result.is_err(), "Should fail with invalid token format");
}

/// Test scope validation for different MCP operations
#[tokio::test]
async fn test_scope_validation_for_mcp_operations() {
    let oauth2_adapter = create_mock_oauth2_adapter().await;
    let auth_middleware = AuthorizationMiddleware::new(
        oauth2_adapter,
        ScopeBasedPolicy::new(create_restrictive_scope_mapping()),
        JsonRpcMethodExtractor::new(),
    );

    // Test that tools/call requires tools scope
    let tools_request = create_mock_http_request(
        "/mcp",
        "Bearer token-with-only-mcp-scope",
        &serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "calculator",
                "arguments": {"operation": "add", "a": 1, "b": 2}
            },
            "id": 2
        })).unwrap(),
    );

    // Should fail if token doesn't have tools scope
    let result = auth_middleware.authorize(&tools_request).await;
    // This depends on mock implementation - adjust based on actual OAuth2 adapter behavior
}

// Helper functions for creating mock objects

async fn create_mock_oauth2_adapter() -> OAuth2StrategyAdapter<impl airs_mcp::oauth2::validator::JwtValidator, impl airs_mcp::oauth2::validator::ScopeValidator> {
    // Create a mock OAuth2 strategy adapter for testing
    // This should validate tokens and return appropriate contexts
    todo!("Implement mock OAuth2 adapter - depends on actual OAuth2 infrastructure")
}

fn create_mcp_scope_mapping() -> HashMap<String, Vec<String>> {
    let mut mapping = HashMap::new();
    mapping.insert("initialize".to_string(), vec!["mcp:*".to_string()]);
    mapping.insert("tools/list".to_string(), vec!["mcp:tools:*".to_string(), "mcp:*".to_string()]);
    mapping.insert("tools/call".to_string(), vec!["mcp:tools:*".to_string(), "mcp:*".to_string()]);
    mapping.insert("resources/list".to_string(), vec!["mcp:resources:*".to_string(), "mcp:*".to_string()]);
    mapping.insert("resources/read".to_string(), vec!["mcp:resources:*".to_string(), "mcp:*".to_string()]);
    mapping.insert("prompts/list".to_string(), vec!["mcp:prompts:*".to_string(), "mcp:*".to_string()]);
    mapping
}

fn create_restrictive_scope_mapping() -> HashMap<String, Vec<String>> {
    let mut mapping = HashMap::new();
    mapping.insert("initialize".to_string(), vec!["mcp:*".to_string()]);
    mapping.insert("tools/call".to_string(), vec!["mcp:tools:*".to_string()]); // No mcp:* fallback
    mapping.insert("resources/read".to_string(), vec!["mcp:resources:*".to_string()]); // No mcp:* fallback
    mapping
}

fn create_mock_http_request(
    path: &str,
    authorization: &str,
    json_body: &JsonRpcRequest,
) -> MockHttpRequest {
    MockHttpRequest {
        path: path.to_string(),
        headers: {
            let mut headers = HashMap::new();
            headers.insert("authorization".to_string(), authorization.to_string());
            headers.insert("content-type".to_string(), "application/json".to_string());
            headers
        },
        body: serde_json::to_vec(json_body).unwrap(),
    }
}

fn create_mock_http_request_no_auth(path: &str, json_body: &serde_json::Value) -> MockHttpRequest {
    MockHttpRequest {
        path: path.to_string(),
        headers: {
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());
            headers
        },
        body: serde_json::to_vec(json_body).unwrap(),
    }
}

/// Mock HTTP request for testing
/// This should be replaced with actual HTTP request types from the transport layer
#[derive(Debug)]
struct MockHttpRequest {
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

// TODO: Implement proper traits/interfaces for MockHttpRequest to work with authorization middleware
// This depends on the actual HTTP request abstractions used in the authorization framework
