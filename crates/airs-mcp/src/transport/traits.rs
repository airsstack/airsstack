//! Transport Trait Definitions
//!
//! This module defines the core `Transport` trait that all transport
//! implementations must satisfy.

use std::future::Future;

/// Core transport abstraction for JSON-RPC communication.
///
/// The `Transport` trait defines the fundamental operations required for
/// bidirectional communication in the AIRS MCP system. All transport
/// implementations (STDIO, HTTP, WebSocket, etc.) must implement this trait.
///
/// # Design Philosophy
///
/// - **Async-native**: All operations are asynchronous to integrate seamlessly with Tokio
/// - **Generic messages**: Uses raw bytes (`&[u8]`/`Vec<u8>`) for maximum flexibility
/// - **Error transparency**: Associated `Error` type allows transport-specific error handling
/// - **Resource safety**: Explicit `close()` method ensures proper cleanup
/// - **Thread safety**: All implementations must be `Send + Sync`
///
/// # Usage Example
///
/// ```rust
/// use airs_mcp::transport::Transport;
///
/// // Example with a mock transport (for demonstration)
/// async fn communicate() -> Result<(), Box<dyn std::error::Error>> {
///     // This example shows the trait usage pattern
///     // Actual transport implementations will be available in subtask 3.2
///     println!("Transport trait defines send/receive/close operations");
///     Ok(())
/// }
/// ```
///
/// # Implementation Requirements
///
/// Implementations must ensure:
/// - **Message boundaries**: Properly frame messages (e.g., newline-delimited for STDIO)
/// - **Buffering**: Efficient memory usage and prevent unbounded growth
/// - **Error recovery**: Handle connection failures gracefully
/// - **Concurrency**: Safe operation under concurrent access
///
/// # Error Handling
///
/// The associated `Error` type should provide meaningful context for:
/// - Connection failures
/// - I/O errors
/// - Protocol violations
/// - Resource exhaustion
pub trait Transport: Send + Sync {
    /// Transport-specific error type.
    ///
    /// Must implement `std::error::Error + Send + Sync + 'static` to ensure
    /// proper error propagation and thread safety.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send a message through the transport.
    ///
    /// This method should handle message framing and ensure the complete
    /// message is transmitted before returning. The implementation should:
    ///
    /// - Frame the message according to transport protocol (e.g., add newlines for STDIO)
    /// - Handle partial writes and retry logic
    /// - Provide backpressure if the transport is congested
    /// - Return an error if the transport is closed or fails
    ///
    /// # Arguments
    ///
    /// * `message` - Raw message bytes to transmit
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(Self::Error)` - Transport failure occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::transport::{Transport, StdioTransport};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut transport = StdioTransport::new().await?;
    ///
    /// // Send a JSON-RPC notification
    /// let notification = br#"{"jsonrpc":"2.0","method":"initialized"}"#;
    /// transport.send(notification).await?;
    /// # Ok(())
    /// # }
    /// ```
    fn send(&mut self, message: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Receive a message from the transport.
    ///
    /// This method should:
    ///
    /// - Block until a complete message is available
    /// - Handle message deframing (e.g., split on newlines for STDIO)
    /// - Manage internal buffering efficiently
    /// - Return exactly one complete message per call
    /// - Handle EOF and connection closure gracefully
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Complete message received
    /// * `Err(Self::Error)` - Transport failure or connection closed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::transport::{Transport, StdioTransport};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut transport = StdioTransport::new().await?;
    ///
    /// // Receive and parse a message
    /// let raw_message = transport.receive().await?;
    /// let message_str = String::from_utf8(raw_message)?;
    /// println!("Received: {}", message_str);
    /// # Ok(())
    /// # }
    /// ```
    fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send;

    /// Close the transport and clean up resources.
    ///
    /// This method should:
    ///
    /// - Flush any pending writes
    /// - Close underlying connections/handles
    /// - Release allocated resources
    /// - Make the transport unusable for further operations
    /// - Be idempotent (safe to call multiple times)
    ///
    /// After calling `close()`, subsequent calls to `send()` or `receive()`
    /// should return appropriate errors.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transport closed successfully
    /// * `Err(Self::Error)` - Error during closure (resources may still be cleaned up)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::transport::{Transport, StdioTransport};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut transport = StdioTransport::new().await?;
    ///
    /// // Use transport...
    ///
    /// // Always close when done
    /// transport.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Mock transport for testing purposes
    struct MockTransport {
        messages: Arc<Mutex<Vec<Vec<u8>>>>,
        responses: Arc<Mutex<Vec<Vec<u8>>>>,
        closed: Arc<Mutex<bool>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                responses: Arc::new(Mutex::new(Vec::new())),
                closed: Arc::new(Mutex::new(false)),
            }
        }

        async fn add_response(&self, response: Vec<u8>) {
            let mut responses = self.responses.lock().await;
            responses.push(response);
        }

        async fn get_sent_messages(&self) -> Vec<Vec<u8>> {
            let messages = self.messages.lock().await;
            messages.clone()
        }
    }

    #[derive(Debug, thiserror::Error)]
    enum MockError {
        #[error("Transport is closed")]
        Closed,
        #[error("No response available")]
        NoResponse,
    }

    impl Transport for MockTransport {
        type Error = MockError;

        async fn send(&mut self, message: &[u8]) -> Result<(), Self::Error> {
            let closed = *self.closed.lock().await;
            if closed {
                return Err(MockError::Closed);
            }

            let mut messages = self.messages.lock().await;
            messages.push(message.to_vec());
            Ok(())
        }

        async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
            let closed = *self.closed.lock().await;
            if closed {
                return Err(MockError::Closed);
            }

            let mut responses = self.responses.lock().await;
            if responses.is_empty() {
                return Err(MockError::NoResponse);
            }

            Ok(responses.remove(0))
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            let mut closed = self.closed.lock().await;
            *closed = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_transport_trait_basic_operations() {
        let mut transport = MockTransport::new();

        // Add a response for receive
        transport.add_response(b"test response".to_vec()).await;

        // Test send
        let message = b"test message";
        transport.send(message).await.unwrap();

        // Verify message was sent
        let sent = transport.get_sent_messages().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], message);

        // Test receive
        let received = transport.receive().await.unwrap();
        assert_eq!(received, b"test response");

        // Test close
        transport.close().await.unwrap();

        // Verify transport is closed
        assert!(transport.send(b"should fail").await.is_err());
        assert!(transport.receive().await.is_err());
    }

    #[tokio::test]
    async fn test_transport_trait_error_handling() {
        let mut transport = MockTransport::new();

        // Test receive with no responses
        assert!(transport.receive().await.is_err());

        // Close and test operations fail
        transport.close().await.unwrap();
        assert!(transport.send(b"fail").await.is_err());
        assert!(transport.receive().await.is_err());

        // Test idempotent close
        transport.close().await.unwrap(); // Should not panic
    }

    #[tokio::test]
    async fn test_transport_trait_concurrency() {
        use tokio::task;

        let transport = Arc::new(Mutex::new(MockTransport::new()));

        // Add responses for concurrent receives
        {
            let transport_clone = transport.clone();
            let mock = transport_clone.lock().await;
            mock.add_response(b"response1".to_vec()).await;
            mock.add_response(b"response2".to_vec()).await;
        }

        // Test concurrent sends
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let transport_clone = transport.clone();
                task::spawn(async move {
                    let mut transport = transport_clone.lock().await;
                    let message = format!("message{i}");
                    transport.send(message.as_bytes()).await
                })
            })
            .collect();

        // Wait for all sends to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Verify all messages were sent
        let transport_guard = transport.lock().await;
        let sent = transport_guard.get_sent_messages().await;
        assert_eq!(sent.len(), 5);
    }
}
