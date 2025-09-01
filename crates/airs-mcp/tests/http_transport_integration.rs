//! Integration tests for HTTP Client Transport
//!
//! This test suite validates the HTTP client transport implementation in isolation
//! and integration scenarios.

use airs_mcp::transport::adapters::http::{HttpClientTransport, HttpTransportConfig};
use airs_mcp::transport::Transport;
use reqwest::Url;
use std::time::Duration;

#[tokio::test]
async fn test_http_client_transport_creation_and_configuration() {
    // Test transport creation with various configurations
    let config = HttpTransportConfig::new()
        .bind_address("127.0.0.1:9000".parse().unwrap())
        .max_connections(2000)
        .request_timeout(Duration::from_secs(10))
        .enable_buffer_pool();

    let mut transport = HttpClientTransport::new(config);

    // Verify configuration
    assert_eq!(
        transport.config().bind_address.to_string(),
        "127.0.0.1:9000"
    );
    assert_eq!(transport.config().max_connections, 2000);
    assert_eq!(transport.config().request_timeout, Duration::from_secs(10));
    assert!(transport.buffer_stats().is_some());

    // Test target URL setting
    let target_url = Url::parse("http://localhost:8080/mcp").unwrap();
    transport.set_target(target_url.clone());

    // Test session ID setting
    transport.set_session_id("test-session-123".to_string());
}

#[tokio::test]
async fn test_http_client_transport_send_without_target_fails() {
    let config = HttpTransportConfig::new();
    let mut transport = HttpClientTransport::new(config);

    // Attempt to send without setting target URL should fail
    let test_message = b"{'jsonrpc': '2.0', 'method': 'test'}";
    let result = transport.send(test_message).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Target URL not set"));
}

#[tokio::test]
async fn test_http_client_transport_message_size_validation() {
    let config = HttpTransportConfig::new().max_message_size(100); // Small message size for testing

    let mut transport = HttpClientTransport::new(config);

    // Set a target URL
    let target_url = Url::parse("http://httpbin.org/post").unwrap();
    transport.set_target(target_url);

    // Create a message that exceeds the limit
    let large_message = vec![b'x'; 150];
    let result = transport.send(&large_message).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Message too large"));
}

#[tokio::test]
async fn test_http_client_transport_receive_without_messages() {
    let config = HttpTransportConfig::new();
    let mut transport = HttpClientTransport::new(config);

    // Attempt to receive without any queued messages
    let result = transport.receive().await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No response available"));
}

#[tokio::test]
async fn test_http_client_transport_close_cleanup() {
    let config = HttpTransportConfig::new();
    let mut transport = HttpClientTransport::new(config);

    // Set session ID
    transport.set_session_id("test-session".to_string());

    // Close transport
    let result = transport.close().await;
    assert!(result.is_ok());

    // Verify cleanup - this is internal state so we can't directly test it,
    // but the operation should complete successfully
}

#[tokio::test]
async fn test_http_client_transport_builder_pattern() {
    // Test the full builder pattern configuration
    let config = HttpTransportConfig::new()
        .bind_address("0.0.0.0:8080".parse().unwrap())
        .max_connections(5000)
        .max_concurrent_requests(50)
        .session_timeout(Duration::from_secs(600))
        .keep_alive_timeout(Duration::from_secs(120))
        .request_timeout(Duration::from_secs(30))
        .buffer_pool_size(500)
        .max_message_size(64 * 1024 * 1024);

    let transport = HttpClientTransport::new(config);

    // Verify all configuration values
    assert_eq!(transport.config().bind_address.to_string(), "0.0.0.0:8080");
    assert_eq!(transport.config().max_connections, 5000);
    assert_eq!(transport.config().max_concurrent_requests, 50);
    assert_eq!(transport.config().session_timeout, Duration::from_secs(600));
    assert_eq!(
        transport.config().keep_alive_timeout,
        Duration::from_secs(120)
    );
    assert_eq!(transport.config().request_timeout, Duration::from_secs(30));
    assert_eq!(transport.config().parser.max_message_size, 64 * 1024 * 1024);

    // Verify buffer pool is configured
    assert!(transport.buffer_stats().is_some());
}

// Note: Real HTTP server integration tests would require a test server
// For now, we focus on the transport configuration and error handling
// In a production environment, you might use tools like wiremock or
// spin up actual test servers for full integration testing
