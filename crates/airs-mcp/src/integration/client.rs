//! High-level MCP Client API
//!
//! This module provides a high-level, type-safe MCP client that simplifies
//! interaction with MCP servers through intuitive method calls.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::{oneshot, Mutex, RwLock};

use crate::integration::constants::{defaults, methods};
use crate::integration::McpError;
use crate::protocol::transport::{MessageContext, MessageHandler, Transport, TransportError};
use crate::protocol::RequestId;
use crate::protocol::{
    CallToolRequest,
    CallToolResponse,
    ClientCapabilities,
    ClientInfo,
    Content,
    GetPromptRequest,
    GetPromptResponse,
    InitializeRequest,
    InitializeResponse,
    ListPromptsRequest,
    ListPromptsResponse,
    // Request types for outgoing requests
    ListResourcesRequest,
    ListResourcesResponse,
    ListToolsRequest,
    ListToolsResponse,
    LoggingConfig,
    Prompt,
    PromptMessage,
    ProtocolVersion,
    ReadResourceRequest,
    ReadResourceResponse,
    // Core types needed by client
    Resource,
    // Capability types
    ServerCapabilities,
    SetLoggingRequest,
    // Response types that clients receive
    SetLoggingResponse,
    SubscribeResourceRequest,
    Tool,
};
use crate::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};

/// Type alias for MCP client results
pub type McpResult<T> = Result<T, McpError>;

/// MCP Protocol Session State (separate from transport connectivity)
#[derive(Debug, Clone, PartialEq)]
pub enum McpSessionState {
    /// Haven't done MCP handshake yet
    NotInitialized,
    /// MCP initialize request sent, waiting for response
    Initializing,
    /// MCP handshake complete, server capabilities received
    Ready,
    /// MCP protocol failed (handshake failed, incompatible version, etc.)
    Failed,
}

/// Configuration for MCP client behavior
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    /// Client information to send during initialization
    pub client_info: ClientInfo,
    /// Client capabilities to advertise
    pub capabilities: ClientCapabilities,
    /// Protocol version to use
    pub protocol_version: ProtocolVersion,
    /// Default timeout for operations
    pub default_timeout: Duration,
    /// Whether to automatically retry failed operations
    pub auto_retry: bool,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Whether to automatically reconnect on connection loss
    pub auto_reconnect: bool,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            client_info: ClientInfo {
                name: defaults::CLIENT_NAME.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ClientCapabilities::default(),
            protocol_version: ProtocolVersion::current(),
            default_timeout: Duration::from_secs(defaults::TIMEOUT_SECONDS),
            auto_retry: true,
            max_retries: defaults::MAX_RETRIES,
            auto_reconnect: false,
        }
    }
}

/// Builder for creating MCP clients
pub struct McpClientBuilder {
    config: McpClientConfig,
}

impl McpClientBuilder {
    /// Create a new MCP client builder
    pub fn new() -> Self {
        Self {
            config: McpClientConfig::default(),
        }
    }

    /// Set client information
    pub fn client_info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.config.client_info = ClientInfo {
            name: name.into(),
            version: version.into(),
        };
        self
    }

    /// Set client capabilities
    pub fn capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.config.capabilities = capabilities;
        self
    }

    /// Set protocol version
    pub fn protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.config.protocol_version = version;
        self
    }

    /// Set default timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// Enable automatic retry on failures
    pub fn auto_retry(mut self, enabled: bool, max_retries: u32) -> Self {
        self.config.auto_retry = enabled;
        self.config.max_retries = max_retries;
        self
    }

    /// Enable automatic reconnection
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Build the MCP client with the given transport
    pub async fn build<T: Transport + 'static>(self, transport: T) -> McpResult<McpClient<T>>
    where
        T::Error: 'static,
    {
        McpClient::new_with_config(transport, self.config).await
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Message handler for MCP client responses
#[derive(Clone)]
struct ClientMessageHandler {
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>>,
}

#[async_trait]
impl MessageHandler for ClientMessageHandler {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
        match message {
            JsonRpcMessage::Response(response) => {
                // Handle response by completing the pending request
                if let Some(id) = &response.id {
                    let id_str = id.to_string();
                    let mut pending = self.pending_requests.lock().await;
                    if let Some(sender) = pending.remove(&id_str) {
                        let _ = sender.send(response); // Ignore send errors (receiver might be dropped)
                    }
                }
            }
            JsonRpcMessage::Notification(_) => {
                // Handle notifications (could be subscription updates, etc.)
                // For now, we'll ignore them
            }
            JsonRpcMessage::Request(_) => {
                // Client shouldn't receive requests, but we'll ignore them
            }
        }
    }

    async fn handle_error(&self, _error: TransportError) {
        // Handle transport errors
        // For now, we'll just log or ignore them
    }

    async fn handle_close(&self) {
        // Handle connection close
        // Clear all pending requests
        let mut pending = self.pending_requests.lock().await;
        pending.clear();
    }
}

/// High-level MCP client for interacting with MCP servers
pub struct McpClient<T: Transport> {
    /// Transport layer for communication
    transport: Arc<RwLock<T>>,
    /// Client configuration
    config: McpClientConfig,
    /// Current MCP session state (separate from transport connectivity)
    mcp_session: Arc<RwLock<McpSessionState>>,
    /// Server capabilities (available after initialization)
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Cached resources for efficient access
    resource_cache: Arc<RwLock<HashMap<String, Resource>>>,
    /// Cached tools for efficient access
    tool_cache: Arc<RwLock<HashMap<String, Tool>>>,
    /// Cached prompts for efficient access
    prompt_cache: Arc<RwLock<HashMap<String, Prompt>>>,
    /// Pending requests for correlation
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>>,
}

impl<T: Transport + 'static> McpClient<T> {
    /// Create a new MCP client with the given transport
    pub async fn new(transport: T) -> McpResult<Self> {
        McpClientBuilder::new().build(transport).await
    }

    /// Create a new MCP client with configuration
    pub(crate) async fn new_with_config(
        mut transport: T,
        config: McpClientConfig,
    ) -> McpResult<Self>
    where
        T::Error: 'static,
    {
        // Create pending requests map
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        // TODO(DEBT): Client should use pre-configured transport pattern
        // For now, we'll need to add set_message_handler back to Transport trait
        // or implement a ClientTransportBuilder pattern
        // Create and set message handler
        let _handler = Arc::new(ClientMessageHandler {
            pending_requests: pending_requests.clone(),
        });
        // transport.set_message_handler(handler); // TODO: Fix this pattern

        // Start the transport
        transport
            .start()
            .await
            .map_err(|e| McpError::custom(format!("Failed to start transport: {e}")))?;

        Ok(Self {
            transport: Arc::new(RwLock::new(transport)),
            config,
            mcp_session: Arc::new(RwLock::new(McpSessionState::NotInitialized)),
            server_capabilities: Arc::new(RwLock::new(None)),
            resource_cache: Arc::new(RwLock::new(HashMap::new())),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            prompt_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_requests,
        })
    }

    /// Initialize connection with the MCP server
    pub async fn initialize(&self) -> McpResult<ServerCapabilities> {
        // 1. Ensure transport is connected
        if !self.transport_connected().await {
            return Err(McpError::custom("Transport not connected"));
        }

        // 2. Check current MCP session state
        {
            let session_state = self.mcp_session.read().await;
            match *session_state {
                McpSessionState::Ready => return Err(McpError::already_connected()),
                McpSessionState::Initializing => {
                    return Err(McpError::custom("Initialization already in progress"))
                }
                McpSessionState::Failed => return Err(McpError::custom("MCP session failed")),
                McpSessionState::NotInitialized => {}
            }
        }

        // 3. Update MCP session state to initializing
        *self.mcp_session.write().await = McpSessionState::Initializing;

        // Create initialization request
        let request = InitializeRequest::with_version(
            self.config.protocol_version.clone(),
            serde_json::to_value(&self.config.capabilities).map_err(|e| {
                McpError::invalid_request(format!("Failed to serialize capabilities: {e}"))
            })?,
            self.config.client_info.clone(),
        );

        // Send initialization request
        let request_params = serde_json::to_value(&request).map_err(|e| {
            McpError::invalid_request(format!("Failed to serialize initialize request: {e}"))
        })?;

        let request_msg = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::INITIALIZE.to_string(),
            params: Some(request_params),
            id: RequestId::new_string("init"),
        };

        let response = self.send_request(&request_msg).await?;

        // Parse initialization response
        let init_response: InitializeResponse =
            serde_json::from_value(response.result.ok_or_else(|| {
                McpError::invalid_response("Missing result in initialization response")
            })?)
            .map_err(|e| {
                McpError::invalid_response(format!("Invalid initialization response: {e}"))
            })?;

        // Store server capabilities
        let server_caps: ServerCapabilities = serde_json::from_value(init_response.capabilities)
            .map_err(|e| McpError::invalid_response(format!("Invalid server capabilities: {e}")))?;
        *self.server_capabilities.write().await = Some(server_caps.clone());

        // 4. Update MCP session state to ready
        *self.mcp_session.write().await = McpSessionState::Ready;

        Ok(server_caps)
    }

    /// Check if transport is connected
    pub async fn transport_connected(&self) -> bool {
        self.transport.read().await.is_connected()
    }

    /// Get current MCP session state
    pub async fn session_state(&self) -> McpSessionState {
        self.mcp_session.read().await.clone()
    }

    /// Check if client is ready for MCP operations
    pub async fn is_ready(&self) -> bool {
        self.transport_connected().await
            && matches!(self.session_state().await, McpSessionState::Ready)
    }

    /// Get current connection state (deprecated - use session_state() instead)
    #[deprecated(
        since = "0.2.0",
        note = "Use session_state() instead for MCP protocol state"
    )]
    pub async fn state(&self) -> McpSessionState {
        self.session_state().await
    }

    /// Check if client is initialized and ready for operations (deprecated - use is_ready() instead)
    #[deprecated(since = "0.2.0", note = "Use is_ready() instead")]
    pub async fn is_initialized(&self) -> bool {
        self.is_ready().await
    }

    /// Get server capabilities (available after initialization)
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    /// Ensure client is initialized, returning an error if not
    async fn ensure_initialized(&self) -> McpResult<()> {
        if !self.is_ready().await {
            return Err(McpError::NotConnected);
        }
        Ok(())
    }

    /// Check if server supports a specific capability
    pub async fn supports_capability(&self, check: impl Fn(&ServerCapabilities) -> bool) -> bool {
        if let Some(caps) = self.server_capabilities().await {
            check(&caps)
        } else {
            false
        }
    }

    // Resource Operations

    /// List available resources from the server
    pub async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        self.ensure_initialized().await?;

        // Check if server supports resources
        if !self
            .supports_capability(|caps| caps.resources.is_some())
            .await
        {
            return Err(McpError::unsupported_capability("resources"));
        }

        let request = ListResourcesRequest::new();
        let response = self.call_mcp(methods::RESOURCES_LIST, &request).await?;

        let list_response: ListResourcesResponse =
            serde_json::from_value(response).map_err(|e| {
                McpError::invalid_response(format!("Invalid list resources response: {e}"))
            })?;

        // Update cache
        {
            let mut cache = self.resource_cache.write().await;
            for resource in &list_response.resources {
                cache.insert(resource.uri.to_string(), resource.clone());
            }
        }

        Ok(list_response.resources)
    }

    /// Read content from a specific resource
    pub async fn read_resource(&self, uri: impl Into<String>) -> McpResult<Vec<Content>> {
        self.ensure_initialized().await?;
        let uri = uri.into();

        let request = ReadResourceRequest::new(uri.clone())
            .map_err(|e| McpError::invalid_request(e.to_string()))?;
        let response = self.call_mcp(methods::RESOURCES_READ, &request).await?;

        let read_response: ReadResourceResponse =
            serde_json::from_value(response).map_err(|e| {
                McpError::invalid_response(format!("Invalid read resource response: {e}"))
            })?;

        Ok(read_response.contents)
    }

    /// Subscribe to changes for a specific resource
    pub async fn subscribe_to_resource(&self, uri: impl Into<String>) -> McpResult<()> {
        self.ensure_initialized().await?;
        let uri = uri.into();

        // Check if server supports subscriptions
        if !self
            .supports_capability(|caps| {
                caps.resources
                    .as_ref()
                    .is_some_and(|r| r.subscribe.unwrap_or(false))
            })
            .await
        {
            return Err(McpError::unsupported_capability("resource subscriptions"));
        }

        let request = SubscribeResourceRequest::new(uri.clone())
            .map_err(|e| McpError::invalid_request(e.to_string()))?;
        let _response = self
            .call_mcp(methods::RESOURCES_SUBSCRIBE, &request)
            .await?;

        Ok(())
    }

    // Tool Operations

    /// List available tools from the server
    pub async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        self.ensure_initialized().await?;

        // Check if server supports tools
        if !self.supports_capability(|caps| caps.tools.is_some()).await {
            return Err(McpError::unsupported_capability("tools"));
        }

        let request = ListToolsRequest::new();
        let response = self.call_mcp(methods::TOOLS_LIST, &request).await?;

        let list_response: ListToolsResponse = serde_json::from_value(response)
            .map_err(|e| McpError::invalid_response(format!("Invalid list tools response: {e}")))?;

        // Update cache
        {
            let mut cache = self.tool_cache.write().await;
            for tool in &list_response.tools {
                cache.insert(tool.name.clone(), tool.clone());
            }
        }

        Ok(list_response.tools)
    }

    /// Execute a tool with the given arguments
    pub async fn call_tool(
        &self,
        name: impl Into<String>,
        arguments: Option<Value>,
    ) -> McpResult<Vec<Content>> {
        self.ensure_initialized().await?;
        let name = name.into();

        let request = CallToolRequest::new(name.clone(), arguments.unwrap_or(Value::Null));
        let response = self.call_mcp(methods::TOOLS_CALL, &request).await?;

        let call_response: CallToolResponse = serde_json::from_value(response)
            .map_err(|e| McpError::invalid_response(format!("Invalid call tool response: {e}")))?;

        if call_response.is_error.unwrap_or(false) {
            if let Some(error_content) = call_response.content.first() {
                if let Some(text) = error_content.as_text() {
                    return Err(McpError::tool_execution_failed(name, text));
                }
            }
            return Err(McpError::tool_execution_failed(
                name,
                "Tool execution failed",
            ));
        }

        Ok(call_response.content)
    }

    // Prompt Operations

    /// List available prompts from the server
    pub async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        self.ensure_initialized().await?;

        // Check if server supports prompts
        if !self
            .supports_capability(|caps| caps.prompts.is_some())
            .await
        {
            return Err(McpError::unsupported_capability("prompts"));
        }

        let request = ListPromptsRequest::new();
        let response = self.call_mcp(methods::PROMPTS_LIST, &request).await?;

        let list_response: ListPromptsResponse = serde_json::from_value(response).map_err(|e| {
            McpError::invalid_response(format!("Invalid list prompts response: {e}"))
        })?;

        // Update cache
        {
            let mut cache = self.prompt_cache.write().await;
            for prompt in &list_response.prompts {
                cache.insert(prompt.name.clone(), prompt.clone());
            }
        }

        Ok(list_response.prompts)
    }

    /// Get a prompt with the given arguments
    pub async fn get_prompt(
        &self,
        name: impl Into<String>,
        arguments: HashMap<String, String>,
    ) -> McpResult<Vec<PromptMessage>> {
        self.ensure_initialized().await?;
        let name = name.into();

        let request = GetPromptRequest::new(name.clone(), arguments);
        let response = self.call_mcp(methods::PROMPTS_GET, &request).await?;

        let prompt_response: GetPromptResponse = serde_json::from_value(response)
            .map_err(|e| McpError::invalid_response(format!("Invalid get prompt response: {e}")))?;

        Ok(prompt_response.messages)
    }

    // Logging Operations

    /// Set logging configuration
    pub async fn set_logging_config(&self, config: LoggingConfig) -> McpResult<()> {
        self.ensure_initialized().await?;

        // Check if server supports logging
        if !self
            .supports_capability(|caps| caps.logging.is_some())
            .await
        {
            return Err(McpError::unsupported_capability("logging"));
        }

        let request = SetLoggingRequest::new(config.level);
        let response = self.call_mcp(methods::LOGGING_SET_LEVEL, &request).await?;

        let log_response: SetLoggingResponse = serde_json::from_value(response).map_err(|e| {
            McpError::invalid_response(format!("Invalid set logging response: {e}"))
        })?;

        if !log_response.success {
            return Err(McpError::server_error(
                log_response
                    .message
                    .unwrap_or_else(|| "Logging configuration failed".to_string()),
            ));
        }

        Ok(())
    }

    // Utility Operations

    /// Close the connection to the server
    pub async fn close(&self) -> McpResult<()> {
        // Reset MCP session state
        *self.mcp_session.write().await = McpSessionState::NotInitialized;

        // Close transport
        let mut transport = self.transport.write().await;
        transport
            .close()
            .await
            .map_err(|e| McpError::custom(e.to_string()))?;
        Ok(())
    }

    /// Internal helper to send a JSON-RPC request and get response
    async fn send_request(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // Create a oneshot channel for the response
        let (sender, receiver) = oneshot::channel();

        // Register the pending request
        let id_str = request.id.to_string();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id_str.clone(), sender);
        }

        // Send the request through the transport
        let mut transport = self.transport.write().await;
        let message = JsonRpcMessage::Request(request.clone());
        transport
            .send(&message)
            .await
            .map_err(|e| McpError::custom(format!("Failed to send request: {e}")))?;

        // Release the transport lock
        drop(transport);

        // Wait for the response with timeout
        let response = tokio::time::timeout(self.config.default_timeout, receiver)
            .await
            .map_err(|_| McpError::custom("Request timeout"))?
            .map_err(|_| McpError::custom("Request cancelled"))?;

        Ok(response)
    }

    /// Internal helper to make MCP method calls
    async fn call_mcp<P: serde::Serialize>(&self, method: &str, params: &P) -> McpResult<Value> {
        let params_value = serde_json::to_value(params)
            .map_err(|e| McpError::invalid_response(format!("Failed to serialize request: {e}")))?;

        let request = crate::protocol::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params_value),
            id: RequestId::new_string(method),
        };

        let response = self.send_request(&request).await?;

        if let Some(error) = response.error {
            return Err(McpError::server_error(format!("Server error: {error}")));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

// Implement Drop to ensure clean shutdown
impl<T: Transport> Drop for McpClient<T> {
    fn drop(&mut self) {
        // Note: We can't call async methods in Drop, but the underlying transport
        // should handle cleanup automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcMessage, MessageHandler, TransportBuilder};
    use crate::transport::adapters::stdio::{StdioMessageContext, StdioTransportBuilder};

    // Simple test message handler for integration tests
    #[derive(Debug)]
    struct TestMessageHandler;

    #[async_trait]
    impl MessageHandler<()> for TestMessageHandler {
        async fn handle_message(&self, _message: JsonRpcMessage, _context: StdioMessageContext) {
            // Simple test handler - just ignore messages
        }

        async fn handle_error(&self, _error: TransportError) {
            // Simple test handler - just ignore errors
        }

        async fn handle_close(&self) {
            // Simple test handler - no cleanup needed
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = McpClientConfig::default();
        assert_eq!(config.client_info.name, "airs-mcp-client");
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert!(config.auto_retry);
        assert_eq!(config.max_retries, 3);
        assert!(!config.auto_reconnect);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = McpClientBuilder::new()
            .client_info("test-client", "1.0.0")
            .timeout(Duration::from_secs(60))
            .auto_retry(false, 0)
            .auto_reconnect(true);

        assert_eq!(builder.config.client_info.name, "test-client");
        assert_eq!(builder.config.client_info.version, "1.0.0");
        assert_eq!(builder.config.default_timeout, Duration::from_secs(60));
        assert!(!builder.config.auto_retry);
        assert_eq!(builder.config.max_retries, 0);
        assert!(builder.config.auto_reconnect);
    }

    #[tokio::test]
    async fn test_client_creation() {
        // Note: This test requires a mock transport for full testing
        // For now, we just test the creation logic using the new builder pattern
        let handler = Arc::new(TestMessageHandler);
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .build()
            .await
            .unwrap();

        let client = McpClientBuilder::new()
            .client_info("test", "1.0")
            .build(transport)
            .await
            .unwrap();

        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );
        assert!(!client.is_ready().await);
        assert!(client.server_capabilities().await.is_none());
    }

    #[tokio::test]
    async fn test_state_management() {
        let handler = Arc::new(TestMessageHandler);
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .build()
            .await
            .unwrap();

        let client = McpClient::new(transport).await.unwrap();

        // Initial state should be not initialized
        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );
        assert!(!client.is_ready().await);

        // Operations should fail when not initialized
        let result = client.list_resources().await;
        assert!(matches!(result.unwrap_err(), McpError::NotConnected));
    }

    #[tokio::test]
    async fn test_capability_checking() {
        let handler = Arc::new(TestMessageHandler);
        let transport = StdioTransportBuilder::new()
            .with_message_handler(handler)
            .build()
            .await
            .unwrap();

        let client = McpClient::new(transport).await.unwrap();

        // Should return false when no capabilities are set
        let supports_resources = client
            .supports_capability(|caps| caps.resources.is_some())
            .await;
        assert!(!supports_resources);
    }
}
