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
use tokio::time::{sleep, Instant};

use crate::integration::constants::{defaults, methods};
use crate::integration::{IntegrationError, McpError};
use crate::protocol::transport::{
    MessageContext, MessageHandler, Transport, TransportBuilder, TransportError,
};
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
    /// Initial retry delay (doubles with each retry for exponential backoff)
    pub initial_retry_delay: Duration,
    /// Maximum retry delay (caps exponential backoff)
    pub max_retry_delay: Duration,
    /// Whether to automatically reconnect on connection loss
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts before giving up
    pub max_reconnect_attempts: u32,
    /// Initial reconnection delay
    pub initial_reconnect_delay: Duration,
    /// Maximum reconnection delay
    pub max_reconnect_delay: Duration,
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
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(30),
            auto_reconnect: false,
            max_reconnect_attempts: 5,
            initial_reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(60),
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

    /// Configure retry timing (exponential backoff)
    pub fn retry_timing(mut self, initial_delay: Duration, max_delay: Duration) -> Self {
        self.config.initial_retry_delay = initial_delay;
        self.config.max_retry_delay = max_delay;
        self
    }

    /// Enable automatic reconnection
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Configure reconnection behavior
    pub fn reconnection_config(
        mut self,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        self.config.max_reconnect_attempts = max_attempts;
        self.config.initial_reconnect_delay = initial_delay;
        self.config.max_reconnect_delay = max_delay;
        self
    }

    /// Build the MCP client with the given transport builder (pre-configured pattern)
    ///
    /// This is the only supported way to create an MCP client. The transport builder
    /// must be pre-configured with a message handler before calling this method.
    /// This ensures proper message correlation for request-response operations.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let client = McpClientBuilder::new()
    ///     .client_info("my-client", "1.0.0")
    ///     .build(StdioTransportBuilder::new())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build<TB: TransportBuilder + 'static>(
        self,
        transport_builder: TB,
    ) -> McpResult<McpClient<TB::Transport>>
    where
        TB::Transport: 'static,
        TB::Error: 'static,
    {
        // Create pending requests map
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        // Create client message handler with proper context
        let handler = Arc::new(ClientMessageHandler {
            pending_requests: pending_requests.clone(),
        });

        // Build transport with handler pre-configured (CRITICAL: This fixes the broken pattern!)
        let mut transport = transport_builder
            .with_message_handler(handler)
            .build()
            .await
            .map_err(|e| McpError::custom(format!("Failed to build transport: {e}")))?;

        // Start the transport
        transport
            .start()
            .await
            .map_err(|e| McpError::custom(format!("Failed to start transport: {e}")))?;

        Ok(McpClient {
            transport: Arc::new(RwLock::new(transport)),
            config: self.config,
            mcp_session: Arc::new(RwLock::new(McpSessionState::NotInitialized)),
            server_capabilities: Arc::new(RwLock::new(None)),
            resource_cache: Arc::new(RwLock::new(HashMap::new())),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            prompt_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_requests,
            reconnection_state: Arc::new(RwLock::new(ReconnectionState::default())),
        })
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

/// Reconnection state tracking
#[derive(Debug, Clone)]
struct ReconnectionState {
    /// Current reconnection attempt count
    attempt_count: u32,
    /// Last reconnection attempt time
    last_attempt: Option<Instant>,
    /// Whether client is currently attempting to reconnect
    is_reconnecting: bool,
}

impl Default for ReconnectionState {
    fn default() -> Self {
        Self {
            attempt_count: 0,
            last_attempt: None,
            is_reconnecting: false,
        }
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
    /// Reconnection state tracking
    reconnection_state: Arc<RwLock<ReconnectionState>>,
}

impl<T: Transport + 'static> McpClient<T> {
    /// Check if an error is retryable
    fn is_retryable_error(error: &McpError) -> bool {
        match error {
            // Network/transport errors are usually retryable
            McpError::Integration(IntegrationError::Transport(_)) => true,
            McpError::Integration(IntegrationError::Timeout { .. }) => true,
            // Server errors might be temporary
            McpError::ServerError { .. } => true,
            McpError::Timeout { .. } => true,
            // Connection errors are retryable
            McpError::NotConnected => true,
            // Protocol errors are usually not retryable
            McpError::Protocol(_) => false,
            // Resource/tool not found are not retryable
            McpError::ResourceNotFound { .. } => false,
            McpError::ToolNotFound { .. } => false,
            McpError::PromptNotFound { .. } => false,
            // Invalid arguments are not retryable
            McpError::InvalidPromptArguments { .. } => false,
            McpError::InvalidResponse { .. } => false,
            // Capability errors are not retryable
            McpError::UnsupportedCapability { .. } => false,
            McpError::CapabilityNegotiationFailed { .. } => false,
            // Tool execution failures might be retryable (server-dependent)
            McpError::ToolExecutionFailed { .. } => true,
            // Subscription failures might be retryable
            McpError::SubscriptionFailed { .. } => true,
            // JSON errors are usually not retryable
            McpError::Integration(IntegrationError::Json(_)) => false,
            // Already connected is not retryable
            McpError::AlreadyConnected => false,
            // Invalid state errors are not retryable
            McpError::InvalidState { .. } => false,
            // Custom errors are not retryable by default (conservative)
            McpError::Custom { .. } => false,
            // Other integration errors might be retryable
            McpError::Integration(_) => true,
        }
    }

    /// Check if an error indicates connection loss (should trigger reconnection)
    fn is_connection_error(error: &McpError) -> bool {
        match error {
            McpError::NotConnected => true,
            McpError::Integration(IntegrationError::Transport(_)) => true,
            McpError::Integration(IntegrationError::Timeout { .. }) => true,
            _ => false,
        }
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let delay = self.config.initial_retry_delay * 2_u32.pow(attempt);
        std::cmp::min(delay, self.config.max_retry_delay)
    }

    /// Calculate reconnection delay with exponential backoff
    fn calculate_reconnection_delay(&self, attempt: u32) -> Duration {
        let delay = self.config.initial_reconnect_delay * 2_u32.pow(attempt);
        std::cmp::min(delay, self.config.max_reconnect_delay)
    }

    /// Execute an operation with retry logic
    async fn execute_with_retry<F, Fut, R>(&self, operation: F) -> McpResult<R>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = McpResult<R>>,
    {
        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Check if we should retry
                    if !self.config.auto_retry
                        || attempt >= self.config.max_retries
                        || !Self::is_retryable_error(&error)
                    {
                        return Err(error);
                    }

                    // Check if we need to reconnect
                    if Self::is_connection_error(&error) && self.config.auto_reconnect {
                        if let Err(_) = self.attempt_reconnection().await {
                            // If reconnection fails, return the original error
                            return Err(error);
                        }
                    }

                    attempt += 1;
                    let delay = self.calculate_retry_delay(attempt - 1);

                    // Log retry attempt (we would add proper logging here)
                    eprintln!(
                        "Retrying operation (attempt {}/{}) after {}ms delay due to: {}",
                        attempt,
                        self.config.max_retries,
                        delay.as_millis(),
                        error
                    );

                    sleep(delay).await;
                }
            }
        }
    }

    /// Attempt to reconnect the transport
    async fn attempt_reconnection(&self) -> McpResult<()> {
        let mut reconnection_state = self.reconnection_state.write().await;

        // Check if already reconnecting
        if reconnection_state.is_reconnecting {
            return Err(McpError::custom("Reconnection already in progress"));
        }

        // Check reconnection attempts limit
        if reconnection_state.attempt_count >= self.config.max_reconnect_attempts {
            return Err(McpError::custom("Maximum reconnection attempts exceeded"));
        }

        reconnection_state.is_reconnecting = true;
        reconnection_state.attempt_count += 1;
        reconnection_state.last_attempt = Some(Instant::now());

        drop(reconnection_state); // Release lock before async operations

        let result = async {
            // Calculate delay for this attempt
            let delay = self.calculate_reconnection_delay(
                self.reconnection_state.read().await.attempt_count - 1,
            );

            eprintln!(
                "Attempting reconnection (attempt {}/{}) after {}s delay",
                self.reconnection_state.read().await.attempt_count,
                self.config.max_reconnect_attempts,
                delay.as_secs()
            );

            sleep(delay).await;

            // Reset MCP session state
            *self.mcp_session.write().await = McpSessionState::NotInitialized;

            // Try to restart transport (this depends on transport implementation)
            // For now, we'll just check if it's connected
            if !self.transport_connected().await {
                return Err(McpError::custom(
                    "Transport still not connected after reconnection attempt",
                ));
            }

            // Try to re-initialize MCP session (use non-retrying version to avoid recursion)
            self.initialize_without_retry().await?;

            Ok(())
        }
        .await;

        // Update reconnection state
        let mut reconnection_state = self.reconnection_state.write().await;
        reconnection_state.is_reconnecting = false;

        match &result {
            Ok(_) => {
                // Reset attempt count on successful reconnection
                reconnection_state.attempt_count = 0;
                eprintln!("Reconnection successful");
            }
            Err(error) => {
                eprintln!("Reconnection attempt failed: {}", error);
            }
        }

        result
    }

    /// Initialize connection with the MCP server (with retry logic)
    pub async fn initialize(&self) -> McpResult<ServerCapabilities> {
        self.execute_with_retry(|| self.initialize_without_retry())
            .await
    }

    /// Initialize connection with the MCP server (without retry logic, used during reconnection)
    async fn initialize_without_retry(&self) -> McpResult<ServerCapabilities> {
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

        // Use send_request_once to avoid retry recursion during reconnection
        let response = self.send_request_once(&request_msg).await?;

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

    /// Get current reconnection status
    pub async fn reconnection_status(&self) -> (u32, bool) {
        let state = self.reconnection_state.read().await;
        (state.attempt_count, state.is_reconnecting)
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

        // Reset reconnection state
        *self.reconnection_state.write().await = ReconnectionState::default();

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
        self.execute_with_retry(|| self.send_request_once(request))
            .await
    }

    /// Send a single request without retry logic
    async fn send_request_once(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
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
    use crate::transport::adapters::stdio::StdioTransportBuilder;

    // Mock transport for testing static methods
    struct MockTransport;

    #[async_trait]
    impl Transport for MockTransport {
        type Error = std::io::Error;

        async fn send(&mut self, _message: &JsonRpcMessage) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn is_connected(&self) -> bool {
            false
        }

        fn session_id(&self) -> Option<String> {
            None
        }

        fn set_session_context(&mut self, _session_id: Option<String>) {
            // Mock implementation - no-op
        }

        fn transport_type(&self) -> &'static str {
            "mock"
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
        // Test the new pre-configured transport builder pattern
        let client = McpClientBuilder::new()
            .client_info("test", "1.0")
            .build(StdioTransportBuilder::new())
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
        // Test with the new pre-configured pattern
        let client = McpClientBuilder::new()
            .build(StdioTransportBuilder::new())
            .await
            .unwrap();

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
        // Test with the new pre-configured pattern
        let client = McpClientBuilder::new()
            .build(StdioTransportBuilder::new())
            .await
            .unwrap();

        // Should return false when no capabilities are set
        let supports_resources = client
            .supports_capability(|caps| caps.resources.is_some())
            .await;
        assert!(!supports_resources);
    }

    #[tokio::test]
    async fn test_retry_configuration() {
        // Test retry configuration through builder
        let client = McpClientBuilder::new()
            .auto_retry(true, 5)
            .retry_timing(Duration::from_millis(50), Duration::from_secs(10))
            .build(StdioTransportBuilder::new())
            .await
            .unwrap();

        assert_eq!(client.config.max_retries, 5);
        assert_eq!(client.config.initial_retry_delay, Duration::from_millis(50));
        assert_eq!(client.config.max_retry_delay, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_reconnection_configuration() {
        // Test reconnection configuration through builder
        let client = McpClientBuilder::new()
            .auto_reconnect(true)
            .reconnection_config(10, Duration::from_secs(2), Duration::from_secs(120))
            .build(StdioTransportBuilder::new())
            .await
            .unwrap();

        assert!(client.config.auto_reconnect);
        assert_eq!(client.config.max_reconnect_attempts, 10);
        assert_eq!(
            client.config.initial_reconnect_delay,
            Duration::from_secs(2)
        );
        assert_eq!(client.config.max_reconnect_delay, Duration::from_secs(120));

        // Check initial reconnection status
        let (attempt_count, is_reconnecting) = client.reconnection_status().await;
        assert_eq!(attempt_count, 0);
        assert!(!is_reconnecting);
    }

    #[test]
    fn test_error_classification() {
        // Test retryable errors
        assert!(McpClient::<MockTransport>::is_retryable_error(
            &McpError::NotConnected
        ));
        assert!(McpClient::<MockTransport>::is_retryable_error(
            &McpError::ServerError {
                message: "Temporary error".to_string()
            }
        ));
        assert!(McpClient::<MockTransport>::is_retryable_error(
            &McpError::Timeout { seconds: 30 }
        ));
        assert!(McpClient::<MockTransport>::is_retryable_error(
            &McpError::ToolExecutionFailed {
                name: "test".to_string(),
                reason: "timeout".to_string()
            }
        ));

        // Test non-retryable errors
        assert!(!McpClient::<MockTransport>::is_retryable_error(
            &McpError::AlreadyConnected
        ));
        assert!(!McpClient::<MockTransport>::is_retryable_error(
            &McpError::UnsupportedCapability {
                capability: "test".to_string()
            }
        ));
        assert!(!McpClient::<MockTransport>::is_retryable_error(
            &McpError::ResourceNotFound {
                uri: "test://resource".to_string()
            }
        ));
        assert!(!McpClient::<MockTransport>::is_retryable_error(
            &McpError::Custom {
                message: "Custom error".to_string()
            }
        ));
    }

    #[test]
    fn test_connection_error_classification() {
        // Test connection errors (should trigger reconnection)
        assert!(McpClient::<MockTransport>::is_connection_error(
            &McpError::NotConnected
        ));
        assert!(McpClient::<MockTransport>::is_connection_error(
            &McpError::Integration(IntegrationError::Transport(
                crate::transport::TransportError::Closed
            ))
        ));
        assert!(McpClient::<MockTransport>::is_connection_error(
            &McpError::Integration(IntegrationError::Timeout { timeout_ms: 5000 })
        ));

        // Test non-connection errors
        assert!(!McpClient::<MockTransport>::is_connection_error(
            &McpError::ServerError {
                message: "Server error".to_string()
            }
        ));
        assert!(!McpClient::<MockTransport>::is_connection_error(
            &McpError::ToolNotFound {
                name: "test".to_string()
            }
        ));
    }
}
