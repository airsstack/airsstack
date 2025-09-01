//! HTTP Client Transport Implementation
//!
//! This module provides the client-side HTTP transport for MCP communication.
//! It handles sending requests to remote MCP servers and receiving responses.

use crate::transport::{error::TransportError, Transport};
use reqwest::{Client, Url};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;

use super::{HttpTransportConfig, RequestParser};

/// HTTP Client Transport implementation
///
/// This transport implements the client side of HTTP communication, where it
/// sends requests to a remote server and receives responses. It properly models
/// the HTTP request-response pattern within the Transport trait semantics.
///
/// # Usage
///
/// ```rust
/// use airs_mcp::transport::adapters::http::{HttpTransportConfig, HttpClientTransport};
/// use reqwest::Url;
///
/// let config = HttpTransportConfig::new();
/// let mut client = HttpClientTransport::new(config);
/// client.set_target("http://localhost:3000/mcp".parse().unwrap());
/// ```
pub struct HttpClientTransport {
    config: HttpTransportConfig,
    request_parser: RequestParser,
    // HTTP client for sending requests to remote server
    client: Client,
    // Target server URL where requests are sent
    target_url: Option<Url>,
    // Queue for responses received from the server
    message_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    // Session ID for correlation with server
    session_id: Option<String>,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport with the given configuration
    ///
    /// This creates a new HTTP client transport instance ready to send requests
    /// to a remote MCP server and receive responses. The transport properly models
    /// the client side of HTTP request-response communication.
    pub fn new(config: HttpTransportConfig) -> Self {
        let request_parser = RequestParser::new(config.parser.clone());

        // Create HTTP client with timeout configuration
        let client = Client::builder()
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            request_parser,
            client,
            target_url: None,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            session_id: None,
        }
    }

    /// Set the target URL for HTTP requests
    ///
    /// This must be called before sending any messages. The URL should point
    /// to the MCP server endpoint (typically `/mcp`).
    pub fn set_target(&mut self, url: Url) {
        self.target_url = Some(url);
    }

    /// Set the session ID for this transport
    ///
    /// Session IDs are used for correlation and connection recovery in HTTP
    /// scenarios where maintaining state across requests is important.
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Get the transport configuration
    pub fn config(&self) -> &HttpTransportConfig {
        &self.config
    }

    /// Get the request parser
    pub fn parser(&self) -> &RequestParser {
        &self.request_parser
    }

    /// Get buffer pool statistics (if using buffer pool)
    pub fn buffer_stats(&self) -> Option<super::BufferPoolStats> {
        self.request_parser.buffer_stats()
    }
}

// Implementation of Transport trait for HTTP Client Transport
impl Transport for HttpClientTransport {
    type Error = TransportError;

    async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
        // Validate that target URL is set
        let target_url = self
            .target_url
            .as_ref()
            .ok_or_else(|| TransportError::Other {
                details: "Target URL not set. Call set_target() before sending messages."
                    .to_string(),
            })?;

        // Validate message size
        if message.len() > self.config.parser.max_message_size {
            return Err(TransportError::MessageTooLarge {
                size: message.len(),
                max_size: self.config.parser.max_message_size,
            });
        }

        // Build HTTP request
        let mut request_builder = self
            .client
            .post(target_url.clone())
            .header("Content-Type", "application/json")
            .body(message.to_vec());

        // Add session ID header if available
        if let Some(session_id) = &self.session_id {
            request_builder = request_builder.header("Mcp-Session-Id", session_id);
        }

        // Send request
        let response = request_builder
            .send()
            .await
            .map_err(|e| TransportError::Other {
                details: format!("HTTP request failed: {e}"),
            })?;

        // Check response status
        if !response.status().is_success() {
            return Err(TransportError::Other {
                details: format!("HTTP request failed with status: {}", response.status()),
            });
        }

        // Read response body
        let response_bytes = response.bytes().await.map_err(|e| TransportError::Other {
            details: format!("Failed to read response body: {e}"),
        })?;

        // Queue response for receive() method
        {
            let mut queue = self.message_queue.lock().await;
            queue.push_back(response_bytes.to_vec());
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        // Try to get message from queue first
        {
            let mut queue = self.message_queue.lock().await;
            if let Some(message) = queue.pop_front() {
                return Ok(message);
            }
        }

        // If no messages in queue, return an error indicating no data available
        // In HTTP client context, receive() returns responses to previous send() calls
        Err(TransportError::Other {
            details: "No response available. Call send() first to generate a response to receive."
                .to_string(),
        })
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Clear message queue
        {
            let mut queue = self.message_queue.lock().await;
            queue.clear();
        }

        // Reset session state
        self.session_id = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_transport_creation() {
        let config = HttpTransportConfig::new()
            .bind_address("127.0.0.1:8080".parse().unwrap())
            .max_connections(1000)
            .enable_buffer_pool();

        let transport = HttpClientTransport::new(config);

        assert_eq!(
            transport.config().bind_address.to_string(),
            "127.0.0.1:8080"
        );
        assert_eq!(transport.config().max_connections, 1000);
        assert!(transport.buffer_stats().is_some());
    }

    #[test]
    fn test_client_transport_creation_variations() {
        let config = HttpTransportConfig::new();

        // Test that HttpClientTransport works correctly
        let transport = HttpClientTransport::new(config);

        // Should have the same functionality as expected
        assert!(transport
            .config()
            .bind_address
            .to_string()
            .contains("127.0.0.1"));
    }

    #[test]
    fn test_client_specific_functionality() {
        let config = HttpTransportConfig::new();
        let mut transport = HttpClientTransport::new(config);

        // Test target URL setting
        let url = "http://localhost:8080/mcp".parse().unwrap();
        transport.set_target(url);

        // Test session ID setting
        transport.set_session_id("test-session-123".to_string());

        // Verify parser access
        assert!(transport.parser().buffer_stats().is_none()); // No buffer pool by default
    }
}
