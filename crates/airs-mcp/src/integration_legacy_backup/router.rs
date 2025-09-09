//! Message Router
//!
//! This module provides message routing and handler registration capabilities
//! for processing incoming JSON-RPC messages.

use std::collections::HashMap;
use std::sync::Arc;

use crate::integration::{
    Handler, IntegrationError, IntegrationResult, NotificationHandler, RequestHandler,
};
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Configuration for route registration
#[derive(Debug, Clone)]
pub struct RouteConfig {
    /// Whether to enable detailed logging of message routing
    pub enable_logging: bool,

    /// Maximum number of handlers per method (0 = unlimited)
    pub max_handlers_per_method: usize,

    /// Whether to log unhandled messages
    pub log_unhandled: bool,

    /// Whether to return error for unhandled requests (notifications are always silent)
    pub error_on_unhandled_requests: bool,

    /// Error code to return for unhandled requests
    pub unhandled_error_code: i32,
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            max_handlers_per_method: 10,
            log_unhandled: true,
            error_on_unhandled_requests: true,
            unhandled_error_code: -32601, // Method not found
        }
    }
}

/// Message router for dispatching incoming JSON-RPC messages to registered handlers
pub struct MessageRouter {
    /// Registered notification handlers by method name
    notification_handlers: HashMap<String, Arc<dyn NotificationHandler>>,

    /// Registered request handlers by method name
    request_handlers: HashMap<String, Arc<dyn RequestHandler>>,

    /// Router configuration
    config: RouteConfig,
}

impl MessageRouter {
    /// Create a new message router with default configuration
    pub fn new(config: RouteConfig) -> Self {
        Self {
            notification_handlers: HashMap::new(),
            request_handlers: HashMap::new(),
            config,
        }
    }

    /// Register a handler for both notifications and requests
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler implementing both NotificationHandler and RequestHandler
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Handler registered successfully
    /// * `Err(IntegrationError)` - Registration failed
    pub fn register_handler<H>(&mut self, handler: Arc<H>) -> IntegrationResult<()>
    where
        H: Handler + 'static,
    {
        for method in handler.supported_methods() {
            self.register_notification_handler(method, handler.clone())?;
            self.register_request_handler(method, handler.clone())?;
        }
        Ok(())
    }

    /// Register a notification handler for a specific method
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `handler` - Handler for notifications
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Handler registered successfully
    /// * `Err(IntegrationError)` - Registration failed
    pub fn register_notification_handler<H>(
        &mut self,
        method: &str,
        handler: Arc<H>,
    ) -> IntegrationResult<()>
    where
        H: NotificationHandler + 'static,
    {
        if method.is_empty() {
            return Err(IntegrationError::handler_registration(
                "Method name cannot be empty",
            ));
        }

        if self.notification_handlers.contains_key(method) {
            return Err(IntegrationError::handler_registration(format!(
                "Notification handler for method '{method}' already registered"
            )));
        }

        self.notification_handlers
            .insert(method.to_string(), handler);
        Ok(())
    }

    /// Register a request handler for a specific method
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `handler` - Handler for requests
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Handler registered successfully
    /// * `Err(IntegrationError)` - Registration failed
    pub fn register_request_handler<H>(
        &mut self,
        method: &str,
        handler: Arc<H>,
    ) -> IntegrationResult<()>
    where
        H: RequestHandler + 'static,
    {
        if method.is_empty() {
            return Err(IntegrationError::handler_registration(
                "Method name cannot be empty",
            ));
        }

        if self.request_handlers.contains_key(method) {
            return Err(IntegrationError::handler_registration(format!(
                "Request handler for method '{method}' already registered"
            )));
        }

        self.request_handlers.insert(method.to_string(), handler);
        Ok(())
    }

    /// Route a notification to the appropriate handler
    ///
    /// # Arguments
    ///
    /// * `notification` - Notification to route
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification handled successfully
    /// * `Err(IntegrationError)` - Routing or handling failed
    pub async fn route_notification(
        &self,
        notification: &JsonRpcNotification,
    ) -> IntegrationResult<()> {
        if let Some(handler) = self.notification_handlers.get(&notification.method) {
            handler.handle_notification(notification).await
        } else {
            if self.config.log_unhandled {
                eprintln!("Unhandled notification: {}", notification.method);
            }
            Err(IntegrationError::routing(format!(
                "No handler registered for notification method '{}'",
                notification.method
            )))
        }
    }

    /// Route a request to the appropriate handler and generate response
    ///
    /// This method implements the core request routing logic with comprehensive
    /// error handling that always returns a valid JSON-RPC response when possible.
    ///
    /// # Routing Algorithm
    ///
    /// 1. **Handler Lookup**: O(1) HashMap lookup for registered method handlers
    /// 2. **Request Processing**: Delegate to handler with context preservation
    /// 3. **Error Translation**: Convert integration errors to JSON-RPC error responses
    /// 4. **Response Generation**: Always return valid JSON-RPC response format
    ///
    /// # Error Handling Strategy
    ///
    /// - **Handler Errors**: Converted to JSON-RPC error responses with code -32603
    /// - **Missing Handlers**: Configurable behavior (error response vs exception)
    /// - **ID Preservation**: Request ID always preserved in responses for correlation
    ///
    /// # Arguments
    ///
    /// * `request` - Request to route
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcResponse)` - Request handled successfully, returns response
    /// * `Err(IntegrationError)` - Routing or handling failed
    pub async fn route_request(
        &self,
        request: &JsonRpcRequest,
    ) -> IntegrationResult<JsonRpcResponse> {
        // Attempt O(1) handler lookup using method name as key
        if let Some(handler) = self.request_handlers.get(&request.method) {
            // Handler found - delegate processing and handle results
            match handler.handle_request(request).await {
                Ok(result) => {
                    // Success case: wrap result in JSON-RPC success response
                    Ok(JsonRpcResponse::success(result, request.id.clone()))
                }
                Err(e) => {
                    // Handler error: convert to JSON-RPC error response to maintain protocol compliance
                    // Using -32603 (Internal error) as per JSON-RPC 2.0 specification
                    let error_value = serde_json::json!({
                        "code": -32603, // Internal error
                        "message": e.to_string()
                    });
                    Ok(JsonRpcResponse::error(
                        error_value,
                        Some(request.id.clone()),
                    ))
                }
            }
        } else {
            // No handler registered for this method - handle according to configuration
            if self.config.log_unhandled {
                eprintln!("Unhandled request: {}", request.method);
            }

            if self.config.error_on_unhandled_requests {
                // Return JSON-RPC error response (maintains protocol compliance)
                let error_value = serde_json::json!({
                    "code": self.config.unhandled_error_code,
                    "message": format!("Method '{}' not found", request.method)
                });
                Ok(JsonRpcResponse::error(
                    error_value,
                    Some(request.id.clone()),
                ))
            } else {
                // Alternative: throw integration error (breaks protocol compliance)
                Err(IntegrationError::routing(format!(
                    "No handler registered for request method '{}'",
                    request.method
                )))
            }
        }
    }

    /// Get list of registered notification methods
    pub fn notification_methods(&self) -> Vec<String> {
        self.notification_handlers.keys().cloned().collect()
    }

    /// Get list of registered request methods
    pub fn request_methods(&self) -> Vec<String> {
        self.request_handlers.keys().cloned().collect()
    }

    /// Check if a notification handler is registered for a method
    pub fn has_notification_handler(&self, method: &str) -> bool {
        self.notification_handlers.contains_key(method)
    }

    /// Check if a request handler is registered for a method
    pub fn has_request_handler(&self, method: &str) -> bool {
        self.request_handlers.contains_key(method)
    }

    /// Remove a notification handler
    pub fn unregister_notification_handler(&mut self, method: &str) -> bool {
        self.notification_handlers.remove(method).is_some()
    }

    /// Remove a request handler
    pub fn unregister_request_handler(&mut self, method: &str) -> bool {
        self.request_handlers.remove(method).is_some()
    }

    /// Clear all handlers
    pub fn clear(&mut self) {
        self.notification_handlers.clear();
        self.request_handlers.clear();
    }
}

impl std::fmt::Debug for MessageRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageRouter")
            .field(
                "notification_handlers_count",
                &self.notification_handlers.len(),
            )
            .field("request_handlers_count", &self.request_handlers.len())
            .field("notification_methods", &self.notification_methods())
            .field("request_methods", &self.request_methods())
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::handler::EchoHandler;
    use crate::protocol::RequestId;
    use serde_json::json;

    #[tokio::test]
    async fn test_router_creation() {
        let config = RouteConfig::default();
        let router = MessageRouter::new(config);

        assert!(router.notification_methods().is_empty());
        assert!(router.request_methods().is_empty());
    }

    #[tokio::test]
    async fn test_register_handler() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router.register_handler(handler).unwrap();

        assert!(router.has_notification_handler("echo"));
        assert!(router.has_notification_handler("ping"));
        assert!(router.has_request_handler("echo"));
        assert!(router.has_request_handler("ping"));
    }

    #[tokio::test]
    async fn test_route_notification() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router.register_handler(handler).unwrap();

        let notification = JsonRpcNotification::new("echo", Some(json!({"test": "data"})));
        let result = router.route_notification(&notification).await;
        assert!(result.is_ok());

        // Test unhandled notification
        let unhandled = JsonRpcNotification::new("unknown", None);
        let result = router.route_notification(&unhandled).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_request() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router.register_handler(handler).unwrap();

        let request = JsonRpcRequest::new(
            "echo",
            Some(json!({"message": "hello"})),
            RequestId::new_string("test-1"),
        );

        let response = router.route_request(&request).await.unwrap();
        assert!(response.result.is_some());
        assert_eq!(response.result.unwrap(), json!({"message": "hello"}));
        assert_eq!(response.id, Some(RequestId::new_string("test-1")));

        // Test unhandled request
        let unhandled = JsonRpcRequest::new("unknown", None, RequestId::new_number(2));
        let response = router.route_request(&unhandled).await.unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.id, Some(RequestId::new_number(2)));
    }

    #[tokio::test]
    async fn test_duplicate_handler_registration() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router
            .register_notification_handler("test", handler.clone())
            .unwrap();

        // Duplicate registration should fail
        let result = router.register_notification_handler("test", handler);
        assert!(result.is_err());
    }

    #[test]
    fn test_handler_management() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router.register_handler(handler).unwrap();

        assert!(router.has_notification_handler("echo"));
        assert!(router.has_request_handler("echo"));

        // Unregister handlers
        assert!(router.unregister_notification_handler("echo"));
        assert!(router.unregister_request_handler("echo"));

        assert!(!router.has_notification_handler("echo"));
        assert!(!router.has_request_handler("echo"));

        // Unregistering non-existent handler should return false
        assert!(!router.unregister_notification_handler("nonexistent"));
    }

    #[test]
    fn test_clear_handlers() {
        let config = RouteConfig::default();
        let mut router = MessageRouter::new(config);
        let handler = Arc::new(EchoHandler);

        router.register_handler(handler).unwrap();
        assert!(!router.notification_methods().is_empty());
        assert!(!router.request_methods().is_empty());

        router.clear();
        assert!(router.notification_methods().is_empty());
        assert!(router.request_methods().is_empty());
    }
}
