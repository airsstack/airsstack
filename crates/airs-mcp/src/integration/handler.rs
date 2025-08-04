//! Message Handler Traits
//!
//! This module defines traits for handling incoming JSON-RPC messages,
//! including notifications and requests from remote peers.

use async_trait::async_trait;
use serde_json::Value;

use crate::base::jsonrpc::{JsonRpcNotification, JsonRpcRequest};
use crate::integration::IntegrationResult;

/// Trait for handling JSON-RPC notifications
///
/// Notifications are one-way messages that don't expect a response.
/// Implementations should process the notification and perform any
/// necessary side effects.
#[async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Handle an incoming JSON-RPC notification
    ///
    /// # Arguments
    ///
    /// * `notification` - The incoming notification to handle
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification handled successfully
    /// * `Err(IntegrationError)` - Handling failed
    async fn handle_notification(
        &self,
        notification: &JsonRpcNotification,
    ) -> IntegrationResult<()>;
}

/// Trait for handling JSON-RPC requests
///
/// Requests expect a response (either success or error).
/// Implementations should process the request and return an appropriate result.
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle an incoming JSON-RPC request
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request to handle
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - Request handled successfully, returns result value
    /// * `Err(IntegrationError)` - Handling failed, will be converted to error response
    async fn handle_request(&self, request: &JsonRpcRequest) -> IntegrationResult<Value>;
}

/// Combined trait for handlers that can process both notifications and requests
#[async_trait]
pub trait Handler: NotificationHandler + RequestHandler + Send + Sync {
    /// Get the method names this handler supports
    fn supported_methods(&self) -> Vec<&str>;

    /// Check if this handler supports a specific method
    fn supports_method(&self, method: &str) -> bool {
        self.supported_methods().contains(&method)
    }
}

/// Example echo handler for testing and demonstration
pub struct EchoHandler;

#[async_trait]
impl NotificationHandler for EchoHandler {
    async fn handle_notification(
        &self,
        notification: &JsonRpcNotification,
    ) -> IntegrationResult<()> {
        println!(
            "Echo notification: {} -> {:?}",
            notification.method, notification.params
        );
        Ok(())
    }
}

#[async_trait]
impl RequestHandler for EchoHandler {
    async fn handle_request(&self, request: &JsonRpcRequest) -> IntegrationResult<Value> {
        // Echo back the parameters as the result
        Ok(request.params.clone().unwrap_or(Value::Null))
    }
}

#[async_trait]
impl Handler for EchoHandler {
    fn supported_methods(&self) -> Vec<&str> {
        vec!["echo", "ping"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::jsonrpc::RequestId;
    use serde_json::json;

    #[tokio::test]
    async fn test_echo_handler_notification() {
        let handler = EchoHandler;
        let notification = JsonRpcNotification::new("echo", Some(json!({"test": "data"})));

        let result = handler.handle_notification(&notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_echo_handler_request() {
        let handler = EchoHandler;
        let request = JsonRpcRequest::new(
            "echo",
            Some(json!({"message": "hello"})),
            RequestId::new_string("test-1"),
        );

        let result = handler.handle_request(&request).await.unwrap();
        assert_eq!(result, json!({"message": "hello"}));
    }

    #[tokio::test]
    async fn test_echo_handler_request_no_params() {
        let handler = EchoHandler;
        let request = JsonRpcRequest::new("ping", None, RequestId::new_number(1));

        let result = handler.handle_request(&request).await.unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_handler_supported_methods() {
        let handler = EchoHandler;
        let methods = handler.supported_methods();
        assert!(methods.contains(&"echo"));
        assert!(methods.contains(&"ping"));

        assert!(handler.supports_method("echo"));
        assert!(handler.supports_method("ping"));
        assert!(!handler.supports_method("unknown"));
    }
}
