// Third-party crate imports
use tokio::sync::broadcast;

// Internal module imports - using available modules
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::sse::config::HttpSseConfig;
use crate::transport::adapters::http::sse::constants::events;

/// HTTP Server-Sent Events transport for legacy MCP client compatibility
///
/// Provides a dual-endpoint architecture:
/// - `/sse`: Unidirectional Server-Sent Events stream for server-to-client messages
/// - `/messages`: HTTP POST endpoint for client-to-server messages
///
/// ## Performance Characteristics
/// - Memory: O(n) where n = number of active connections
/// - Latency: ~1-2ms for message broadcasting (tokio broadcast channel)
/// - Throughput: Limited by HTTP connection limits and SSE browser constraints
///
/// ## Legacy Compatibility
/// Designed for MCP clients that cannot use modern WebSocket or HTTP Streamable transports.
/// Consider migrating to `HttpStreamableTransport` for better performance and bidirectional capabilities.
pub struct HttpSseTransport {
    /// Base HTTP transport configuration
    http_config: HttpTransportConfig,
    /// SSE-specific configuration options
    sse_config: HttpSseConfig,
    /// Broadcaster for SSE message distribution
    broadcaster: SseBroadcaster,
}

impl HttpSseTransport {
    /// Create a new HTTP SSE transport instance
    ///
    /// # Arguments
    /// * `http_config` - Base HTTP transport configuration
    /// * `sse_config` - SSE-specific configuration options
    ///
    /// # Returns
    /// Configured SSE transport ready for endpoint registration
    pub async fn new(
        http_config: HttpTransportConfig,
        sse_config: HttpSseConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let broadcaster = SseBroadcaster::new();

        Ok(Self {
            http_config,
            sse_config,
            broadcaster,
        })
    }

    /// Get reference to SSE configuration
    pub fn sse_config(&self) -> &HttpSseConfig {
        &self.sse_config
    }

    /// Get reference to HTTP configuration  
    pub fn http_config(&self) -> &HttpTransportConfig {
        &self.http_config
    }

    /// Get reference to SSE broadcaster for message distribution
    pub fn broadcaster(&self) -> &SseBroadcaster {
        &self.broadcaster
    }

    /// Send JSON-RPC request to all connected SSE clients
    ///
    /// # Arguments
    /// * `request` - JSON-RPC request to broadcast
    ///
    /// # Returns
    /// Number of clients that received the request
    pub async fn broadcast_request(&self, request: JsonRpcRequest) -> usize {
        self.broadcaster.broadcast_request(request).await
    }

    /// Send JSON-RPC response to all connected SSE clients
    ///
    /// # Arguments
    /// * `response` - JSON-RPC response to broadcast
    ///
    /// # Returns
    /// Number of clients that received the response
    pub async fn broadcast_response(&self, response: JsonRpcResponse) -> usize {
        self.broadcaster.broadcast_response(response).await
    }

    /// Send JSON-RPC notification to all connected SSE clients
    ///
    /// # Arguments
    /// * `notification` - JSON-RPC notification to broadcast
    ///
    /// # Returns
    /// Number of clients that received the notification
    pub async fn broadcast_notification(&self, notification: JsonRpcNotification) -> usize {
        self.broadcaster.broadcast_notification(notification).await
    }

    /// Send status update to all connected SSE clients
    ///
    /// # Arguments  
    /// * `status` - Status message to broadcast
    ///
    /// # Returns
    /// Number of clients that received the status update
    pub async fn broadcast_status(&self, status: String) -> usize {
        self.broadcaster.broadcast_status(status).await
    }
}

/// SSE message broadcaster for distributing messages to connected clients
///
/// Uses tokio broadcast channels for efficient fan-out messaging with automatic
/// cleanup of disconnected receivers.
#[derive(Debug, Clone)]
pub struct SseBroadcaster {
    /// Broadcast channel for SSE events
    sender: broadcast::Sender<SseEvent>,
}

impl SseBroadcaster {
    /// Create a new SSE broadcaster
    ///
    /// Uses a buffer size of 1000 messages to handle burst traffic.
    /// Clients that fall behind will miss messages (lagged receivers are dropped).
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self { sender }
    }

    /// Subscribe to SSE message stream
    ///
    /// # Returns
    /// Receiver for SSE events that can be used to generate Server-Sent Events
    pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
        self.sender.subscribe()
    }

    /// Broadcast JSON-RPC request to all connected clients
    ///
    /// # Arguments
    /// * `request` - JSON-RPC request to broadcast
    ///
    /// # Returns
    /// Number of active subscribers that received the request
    pub async fn broadcast_request(&self, request: JsonRpcRequest) -> usize {
        let event = SseEvent::Request(request);
        self.sender.send(event).unwrap_or(0)
    }

    /// Broadcast JSON-RPC response to all connected clients
    ///
    /// # Arguments
    /// * `response` - JSON-RPC response to broadcast
    ///
    /// # Returns
    /// Number of active subscribers that received the response
    pub async fn broadcast_response(&self, response: JsonRpcResponse) -> usize {
        let event = SseEvent::Response(response);
        self.sender.send(event).unwrap_or(0)
    }

    /// Broadcast JSON-RPC notification to all connected clients
    ///
    /// # Arguments
    /// * `notification` - JSON-RPC notification to broadcast
    ///
    /// # Returns
    /// Number of active subscribers that received the notification
    pub async fn broadcast_notification(&self, notification: JsonRpcNotification) -> usize {
        let event = SseEvent::Notification(notification);
        self.sender.send(event).unwrap_or(0)
    }

    /// Broadcast status update to all connected clients
    ///
    /// # Arguments
    /// * `status` - Status message to broadcast
    ///
    /// # Returns  
    /// Number of active subscribers that received the status
    pub async fn broadcast_status(&self, status: String) -> usize {
        let event = SseEvent::Status(status);
        self.sender.send(event).unwrap_or(0)
    }

    /// Get number of active SSE connections
    pub fn connection_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Server-Sent Event types for the SSE transport
#[derive(Debug, Clone)]
pub enum SseEvent {
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC response message
    Response(JsonRpcResponse),
    /// JSON-RPC notification message
    Notification(JsonRpcNotification),
    /// Connection status update  
    Status(String),
}

impl SseEvent {
    /// Convert SSE event to Server-Sent Events format
    ///
    /// # Returns
    /// Formatted SSE string ready for HTTP response
    pub fn to_sse_format(&self) -> String {
        match self {
            SseEvent::Request(request) => match serde_json::to_string(request) {
                Ok(json) => format!("event: {}\ndata: {}\n\n", events::MESSAGE, json),
                Err(_) => format!(
                    "event: {}\ndata: Failed to serialize request\n\n",
                    events::ERROR
                ),
            },
            SseEvent::Response(response) => match serde_json::to_string(response) {
                Ok(json) => format!("event: {}\ndata: {}\n\n", events::MESSAGE, json),
                Err(_) => format!(
                    "event: {}\ndata: Failed to serialize response\n\n",
                    events::ERROR
                ),
            },
            SseEvent::Notification(notification) => match serde_json::to_string(notification) {
                Ok(json) => format!("event: {}\ndata: {}\n\n", events::MESSAGE, json),
                Err(_) => format!(
                    "event: {}\ndata: Failed to serialize notification\n\n",
                    events::ERROR
                ),
            },
            SseEvent::Status(status) => {
                format!("event: {}\ndata: {}\n\n", events::HEARTBEAT, status)
            }
        }
    }

    /// Get event type string for SSE formatting
    pub fn event_type(&self) -> &'static str {
        match self {
            SseEvent::Request(_) | SseEvent::Response(_) | SseEvent::Notification(_) => {
                events::MESSAGE
            }
            SseEvent::Status(_) => events::HEARTBEAT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcRequest, RequestId};
    use serde_json::json;

    #[tokio::test]
    async fn test_transport_creation() {
        let http_config = HttpTransportConfig::default();
        let sse_config = HttpSseConfig::default();

        let transport = HttpSseTransport::new(http_config, sse_config)
            .await
            .unwrap();

        assert_eq!(transport.broadcaster().connection_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_request_distribution() {
        let broadcaster = SseBroadcaster::new();
        let mut receiver1 = broadcaster.subscribe();
        let mut receiver2 = broadcaster.subscribe();

        assert_eq!(broadcaster.connection_count(), 2);

        let test_request = JsonRpcRequest::new(
            "test",
            Some(json!({"key": "value"})),
            RequestId::new_string("test-123".to_string()),
        );

        let sent_count = broadcaster.broadcast_request(test_request.clone()).await;
        assert_eq!(sent_count, 2);

        // Verify both receivers get the request
        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();

        match (&event1, &event2) {
            (SseEvent::Request(req1), SseEvent::Request(req2)) => {
                assert_eq!(req1.id, test_request.id);
                assert_eq!(req2.id, test_request.id);
                assert_eq!(req1.method, test_request.method);
                assert_eq!(req2.method, test_request.method);
            }
            _ => panic!("Expected Request events"),
        }
    }

    #[tokio::test]
    async fn test_sse_event_formatting() {
        let test_request =
            JsonRpcRequest::new("test", None, RequestId::new_string("test-123".to_string()));

        let request_event = SseEvent::Request(test_request);
        let sse_format = request_event.to_sse_format();

        assert!(sse_format.starts_with("event: message\n"));
        assert!(sse_format.contains("data: "));
        assert!(sse_format.ends_with("\n\n"));
        assert!(sse_format.contains("test-123"));

        let status_event = SseEvent::Status("connected".to_string());
        let status_format = status_event.to_sse_format();

        assert!(status_format.starts_with("event: heartbeat\n"));
        assert!(status_format.contains("data: connected"));
        assert!(status_format.ends_with("\n\n"));
    }

    #[tokio::test]
    async fn test_broadcaster_connection_cleanup() {
        let broadcaster = SseBroadcaster::new();

        {
            let _receiver1 = broadcaster.subscribe();
            let _receiver2 = broadcaster.subscribe();
            assert_eq!(broadcaster.connection_count(), 2);
        } // receivers dropped here

        // Broadcast a request to trigger cleanup
        let test_request = JsonRpcRequest::new(
            "test",
            None,
            RequestId::new_string("cleanup-test".to_string()),
        );

        let sent_count = broadcaster.broadcast_request(test_request).await;
        assert_eq!(sent_count, 0); // No active receivers
        assert_eq!(broadcaster.connection_count(), 0);
    }

    #[tokio::test]
    async fn test_transport_request_broadcasting() {
        let http_config = HttpTransportConfig::default();
        let sse_config = HttpSseConfig::default();
        let transport = HttpSseTransport::new(http_config, sse_config)
            .await
            .unwrap();

        let mut receiver = transport.broadcaster().subscribe();

        let test_request = JsonRpcRequest::new(
            "transport_test",
            Some(json!({"param": "value"})),
            RequestId::new_string("transport-456".to_string()),
        );

        let sent_count = transport.broadcast_request(test_request.clone()).await;
        assert_eq!(sent_count, 1);

        let received_event = receiver.recv().await.unwrap();
        match received_event {
            SseEvent::Request(req) => {
                assert_eq!(req.id, test_request.id);
                assert_eq!(req.method, test_request.method);
            }
            _ => panic!("Expected Request event"),
        }
    }
}
