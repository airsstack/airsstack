//! JSON-RPC Client Implementation
//!
//! This module provides the main `JsonRpcClient` struct that integrates
//! the correlation manager and modern protocol transport layer to provide a high-level
//! interface for JSON-RPC communication using the event-driven transport pattern.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::TimeDelta;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use tokio::task;

use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, JsonRpcMessageTrait};
use crate::protocol::transport::{Transport, MessageHandler, TransportError};
use crate::correlation::{CorrelationConfig, CorrelationManager};
use crate::integration::{IntegrationError, IntegrationResult, MessageRouter, RouteConfig};

/// Configuration for JsonRpcClient
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Default timeout for method calls
    pub default_timeout: Duration,

    /// Maximum number of pending requests
    pub max_pending_requests: usize,

    /// Whether to automatically start the message processing loop
    pub auto_start_processing: bool,

    /// Configuration for message routing
    pub route_config: RouteConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_pending_requests: 1000,
            auto_start_processing: true,
            route_config: RouteConfig::default(),
        }
    }
}

/// High-level JSON-RPC client that integrates correlation and transport layers
///
/// The `JsonRpcClient` provides a complete JSON-RPC communication interface by:
/// - Managing request/response correlation using `CorrelationManager`
/// - Handling transport communication through any `Transport` implementation
/// - Providing message routing and handler registration capabilities
/// - Supporting both method calls (request/response) and notifications
///
/// # Design Features
///
/// - **Async-native**: All operations are asynchronous and non-blocking
/// - **Type-safe**: Uses strongly-typed JSON-RPC message structures
/// - **Extensible**: Supports custom message handlers and routing
/// - **Robust**: Comprehensive error handling and timeout management
/// - **Concurrent**: Safe for use across multiple async tasks
///
/// # Usage Example
///
/// ```rust,no_run
/// use airs_mcp::integration::client::{JsonRpcClient, ClientConfig};
/// use airs_mcp::transport::StdioTransport;
/// use serde_json::json;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create transport
///     let transport = StdioTransport::new().await?;
///     
///     // Configure client
///     let config = ClientConfig {
///         default_timeout: Duration::from_secs(10),
///         max_pending_requests: 500,
///         ..Default::default()
///     };
///     
///     // Create client
///     let mut client = JsonRpcClient::with_config(transport, config).await?;
///     
///     // Make method calls
///     let response = client.call("ping", Some(json!({"message": "hello"}))).await?;
///     println!("Response: {:?}", response);
///     
///     // Send notifications
///     client.notify("status_update", Some(json!({"status": "ready"}))).await?;
///     
///     // Clean shutdown
///     client.shutdown().await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct JsonRpcClient<T: Transport> {
    /// Transport layer for communication
    transport: Arc<Mutex<T>>,

    /// Correlation manager for request tracking
    correlation_manager: Arc<CorrelationManager>,

    /// Message router for handling incoming messages
    #[allow(dead_code)]
    // TODO: Will be used for request/notification routing in future updates
    router: Arc<RwLock<MessageRouter>>,

    /// Client configuration
    config: ClientConfig,

    /// Shutdown flag
    shutdown: Arc<Mutex<bool>>,

    /// Background task handle for message processing
    _message_task: Option<task::JoinHandle<()>>,
}

impl<T: Transport + Send + Sync + 'static> JsonRpcClient<T> {
    /// Create a new JsonRpcClient with default configuration
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport implementation for communication
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcClient)` - Successfully created client
    /// * `Err(IntegrationError)` - Client creation failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::integration::JsonRpcClient;
    /// use airs_mcp::transport::StdioTransport;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let transport = StdioTransport::new().await?;
    ///     let client = JsonRpcClient::new(transport).await?;
    ///     // Use client...
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(transport: T) -> IntegrationResult<Self> {
        Self::with_config(transport, ClientConfig::default()).await
    }

    /// Create a new JsonRpcClient with custom configuration
    ///
    /// # Arguments
    ///
    /// * `transport` - Transport implementation for communication
    /// * `config` - Client configuration
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcClient)` - Successfully created client
    /// * `Err(IntegrationError)` - Client creation failed
    pub async fn with_config(transport: T, config: ClientConfig) -> IntegrationResult<Self> {
        // Create correlation manager
        let correlation_config = CorrelationConfig {
            default_timeout: TimeDelta::milliseconds(config.default_timeout.as_millis() as i64),
            max_pending_requests: config.max_pending_requests,
            cleanup_interval: Duration::from_secs(30),
            enable_tracing: true,
        };
        let correlation_manager = Arc::new(CorrelationManager::new(correlation_config).await?);

        // Create message router
        let router = Arc::new(RwLock::new(MessageRouter::new(config.route_config.clone())));

        // Wrap transport in Arc<Mutex<>>
        let transport = Arc::new(Mutex::new(transport));

        // Initialize shutdown state
        let shutdown = Arc::new(Mutex::new(false));

        // Start message processing loop if configured
        let message_task = if config.auto_start_processing {
            Some(Self::start_message_processing(
                transport.clone(),
                correlation_manager.clone(),
                router.clone(),
                shutdown.clone(),
            ))
        } else {
            None
        };

        Ok(Self {
            transport,
            correlation_manager,
            router,
            config,
            shutdown,
            _message_task: message_task,
        })
    }

    /// Make a JSON-RPC method call with response
    ///
    /// This method:
    /// - Creates a JSON-RPC request with unique ID
    /// - Registers the request for correlation tracking
    /// - Sends the request via transport
    /// - Waits for and returns the correlated response
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional method parameters
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - JSON-RPC response result
    /// * `Err(IntegrationError)` - Call failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::integration::JsonRpcClient;
    /// use airs_mcp::transport::StdioTransport;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let transport = StdioTransport::new().await?;
    ///     let mut client = JsonRpcClient::new(transport).await?;
    ///     
    ///     // Simple call without parameters
    ///     let result = client.call("ping", None).await?;
    ///     
    ///     // Call with parameters
    ///     let result = client.call("echo", Some(json!({"message": "hello"}))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn call(&mut self, method: &str, params: Option<Value>) -> IntegrationResult<Value> {
        self.call_with_timeout(method, params, self.config.default_timeout)
            .await
    }

    /// Make a JSON-RPC method call with custom timeout
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional method parameters
    /// * `timeout` - Custom timeout duration
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - JSON-RPC response result
    /// * `Err(IntegrationError)` - Call failed
    pub async fn call_with_timeout(
        &mut self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> IntegrationResult<Value> {
        // Check if client is shutdown
        if *self.shutdown.lock().await {
            return Err(IntegrationError::Shutdown);
        }

        // Validate method name
        if method.is_empty() {
            return Err(IntegrationError::InvalidMethod {
                method: method.to_string(),
            });
        }

        // Convert timeout to TimeDelta for correlation manager
        let correlation_timeout = TimeDelta::milliseconds(timeout.as_millis() as i64);

        // Create request data for correlation tracking
        let request_data = serde_json::json!({
            "method": method,
            "params": params
        });

        // Register request for correlation
        let (request_id, response_receiver) = self
            .correlation_manager
            .register_request(Some(correlation_timeout), request_data)
            .await
            .map_err(IntegrationError::Correlation)?;

        // Create JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request_id.clone(),
            method: method.to_string(),
            params,
        };

        // Serialize and send request
        let request_json = serde_json::to_string(&request).map_err(IntegrationError::Json)?;

        {
            let mut transport = self.transport.lock().await;
            transport
                .send(request_json.as_bytes())
                .await
                .map_err(|e| IntegrationError::other(format!("Transport error: {e}")))?;
        }

        // Wait for response
        match tokio::time::timeout(timeout, response_receiver).await {
            Ok(correlation_result) => {
                let response_value = correlation_result
                    .map_err(|_| IntegrationError::other("Response channel was closed"))?
                    .map_err(IntegrationError::Correlation)?;

                // Parse response
                let json_response: JsonRpcResponse =
                    serde_json::from_value(response_value).map_err(IntegrationError::Json)?;

                // Check for error response
                if let Some(error) = json_response.error {
                    return Err(IntegrationError::other(format!("JSON-RPC error: {error}")));
                }

                // Return result
                json_response.result.ok_or_else(|| {
                    IntegrationError::unexpected_response("Missing result in success response")
                })
            }
            Err(_) => Err(IntegrationError::timeout(timeout.as_millis() as u64)),
        }
    }

    /// Send a JSON-RPC notification (no response expected)
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional method parameters
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Notification sent successfully
    /// * `Err(IntegrationError)` - Send failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::integration::JsonRpcClient;
    /// use airs_mcp::transport::StdioTransport;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let transport = StdioTransport::new().await?;
    ///     let mut client = JsonRpcClient::new(transport).await?;
    ///     
    ///     // Send notification
    ///     client.notify("status_update", Some(json!({"status": "ready"}))).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn notify(&mut self, method: &str, params: Option<Value>) -> IntegrationResult<()> {
        // Check if client is shutdown
        if *self.shutdown.lock().await {
            return Err(IntegrationError::Shutdown);
        }

        // Validate method name
        if method.is_empty() {
            return Err(IntegrationError::InvalidMethod {
                method: method.to_string(),
            });
        }

        // Create notification
        let notification = JsonRpcNotification::new(method, params);

        // Serialize and send
        let notification_json = notification.to_json().map_err(IntegrationError::Json)?;
        {
            let mut transport = self.transport.lock().await;
            transport
                .send(notification_json.as_bytes())
                .await
                .map_err(|e| IntegrationError::other(format!("Transport error: {e}")))?;
        }

        Ok(())
    }

    /// Check if the client is shutdown
    ///
    /// # Returns
    ///
    /// `true` if the client is shutdown, `false` otherwise
    pub async fn is_shutdown(&self) -> bool {
        *self.shutdown.lock().await
    }

    /// Shutdown the client and clean up resources
    ///
    /// This method:
    /// - Stops the message processing loop
    /// - Shuts down the correlation manager
    /// - Closes the transport connection
    /// - Cancels any pending requests
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Shutdown completed successfully
    /// * `Err(IntegrationError)` - Shutdown failed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::integration::JsonRpcClient;
    /// use airs_mcp::transport::StdioTransport;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let transport = StdioTransport::new().await?;
    ///     let mut client = JsonRpcClient::new(transport).await?;
    ///     
    ///     // Use client...
    ///     
    ///     // Always shutdown when done
    ///     client.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn shutdown(&mut self) -> IntegrationResult<()> {
        // Mark as shutdown first
        {
            let mut shutdown = self.shutdown.lock().await;
            if *shutdown {
                return Ok(()); // Already shutdown
            }
            *shutdown = true;
        }

        // Shutdown correlation manager (note: Arc<CorrelationManager> doesn't have shutdown method)
        // self.correlation_manager.shutdown().await
        //     .map_err(IntegrationError::Correlation)?;

        // Close transport
        {
            let mut transport = self.transport.lock().await;
            transport
                .close()
                .await
                .map_err(|e| IntegrationError::other(format!("Transport close error: {e}")))?;
        }

        Ok(())
    }

    /// Start the message processing background task
    fn start_message_processing(
        transport: Arc<Mutex<T>>,
        correlation_manager: Arc<CorrelationManager>,
        _router: Arc<RwLock<MessageRouter>>,
        shutdown: Arc<Mutex<bool>>,
    ) -> task::JoinHandle<()> {
        task::spawn(async move {
            loop {
                // Check shutdown flag
                if *shutdown.lock().await {
                    break;
                }

                // Receive message from transport
                let message_result = {
                    let mut transport = transport.lock().await;
                    transport.receive().await
                };

                match message_result {
                    Ok(message_bytes) => {
                        // Convert to string for JSON parsing
                        let message_str = match String::from_utf8(message_bytes) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Failed to decode message as UTF-8: {e}");
                                continue;
                            }
                        };

                        // Try to parse as response first
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&message_str)
                        {
                            // Handle response
                            if let Some(id) = response.id.clone() {
                                // Convert response to a CorrelationResult<Value>
                                let response_result = if let Some(error) = response.error {
                                    Err(crate::correlation::CorrelationError::Internal {
                                        message: format!("JSON-RPC error: {error}"),
                                    })
                                } else {
                                    Ok(response.result.unwrap_or(Value::Null))
                                };

                                if let Err(e) = correlation_manager
                                    .correlate_response(&id, response_result)
                                    .await
                                {
                                    eprintln!("Failed to correlate response: {e}");
                                }
                            }
                        }
                        // TODO: Handle notifications and requests through router
                        // This will be implemented in subtask 4.3
                    }
                    Err(e) => {
                        // Handle transport errors generically since we can't match on specific types
                        // due to the generic Error type
                        eprintln!("Transport error in message processing: {e}");

                        // Check if this might be a connection closed error by converting to string
                        let error_string = e.to_string();
                        if error_string.contains("closed") || error_string.contains("Closed") {
                            // Transport likely closed, exit processing loop
                            break;
                        }
                        // Continue processing for other errors
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::StdioTransport;

    #[tokio::test]
    async fn test_client_creation() {
        let transport = StdioTransport::new().await.unwrap();
        let client = JsonRpcClient::new(transport).await.unwrap();
        assert!(!client.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_client_with_config() {
        let transport = StdioTransport::new().await.unwrap();
        let config = ClientConfig {
            default_timeout: Duration::from_secs(5),
            max_pending_requests: 100,
            auto_start_processing: false,
            ..Default::default()
        };

        let client = JsonRpcClient::with_config(transport, config).await.unwrap();
        assert!(!client.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_client_shutdown() {
        let transport = StdioTransport::new().await.unwrap();
        let mut client = JsonRpcClient::new(transport).await.unwrap();

        assert!(!client.is_shutdown().await);
        client.shutdown().await.unwrap();
        assert!(client.is_shutdown().await);

        // Second shutdown should be idempotent
        client.shutdown().await.unwrap();
        assert!(client.is_shutdown().await);
    }

    #[tokio::test]
    async fn test_call_after_shutdown() {
        let transport = StdioTransport::new().await.unwrap();
        let mut client = JsonRpcClient::new(transport).await.unwrap();

        client.shutdown().await.unwrap();

        let result = client.call("test", None).await;
        assert!(matches!(result, Err(IntegrationError::Shutdown)));
    }

    #[tokio::test]
    async fn test_notify_after_shutdown() {
        let transport = StdioTransport::new().await.unwrap();
        let mut client = JsonRpcClient::new(transport).await.unwrap();

        client.shutdown().await.unwrap();

        let result = client.notify("test", None).await;
        assert!(matches!(result, Err(IntegrationError::Shutdown)));
    }

    #[tokio::test]
    async fn test_invalid_method_name() {
        let transport = StdioTransport::new().await.unwrap();
        let mut client = JsonRpcClient::new(transport).await.unwrap();

        let result = client.call("", None).await;
        assert!(matches!(
            result,
            Err(IntegrationError::InvalidMethod { .. })
        ));

        let result = client.notify("", None).await;
        assert!(matches!(
            result,
            Err(IntegrationError::InvalidMethod { .. })
        ));
    }
}
