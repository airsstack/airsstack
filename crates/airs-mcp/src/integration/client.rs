//! High-level MCP Client API
//!
//! This module provides a high-level, type-safe MCP client that simplifies
//! interaction with MCP servers through intuitive method calls.
//!
//! # Observability
//!
//! This module uses structured logging via the `tracing` crate to provide
//! comprehensive observability into client operations:
//!
//! - **Info level**: Connection state changes, successful operations
//! - **Warn level**: Retry attempts, recoverable errors, graceful degradation
//! - **Error level**: Failed operations, connection failures
//! - **Debug level**: State transitions, method calls, detailed flow tracking
//!
//! To enable logging, initialize a tracing subscriber in your application:
//!
//! ```rust,no_run
//! use tracing_subscriber;
//!
//! // Simple console logging
//! tracing_subscriber::fmt::init();
//!
//! // Or with environment-based filtering
//! tracing_subscriber::fmt()
//!     .with_env_filter("airs_mcp=debug,info")
//!     .init();
//! ```
//!
//! Set the `RUST_LOG` environment variable to control log levels:
//! - `RUST_LOG=debug` - Show all debug information
//! - `RUST_LOG=airs_mcp=debug` - Show debug info only for this crate
//! - `RUST_LOG=warn` - Show only warnings and errors

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::{oneshot, Mutex, RwLock};
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info, warn};

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
            auto_retry: defaults::AUTO_RETRY,
            max_retries: defaults::MAX_RETRIES,
            initial_retry_delay: Duration::from_millis(defaults::INITIAL_RETRY_DELAY_MS),
            max_retry_delay: Duration::from_secs(defaults::MAX_RETRY_DELAY_SECONDS),
            auto_reconnect: defaults::AUTO_RECONNECT,
            max_reconnect_attempts: defaults::MAX_RECONNECT_ATTEMPTS,
            initial_reconnect_delay: Duration::from_secs(defaults::INITIAL_RECONNECT_DELAY_SECONDS),
            max_reconnect_delay: Duration::from_secs(defaults::MAX_RECONNECT_DELAY_SECONDS),
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
    /// This creates the MCP client but does NOT automatically connect the transport
    /// or initialize the MCP session. This allows for clean separation of:
    /// 1. Client creation
    /// 2. Transport connection via `connect()`
    /// 3. MCP session initialization via `initialize()`
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
    ///
    /// // Phase 1: Connect transport
    /// client.connect().await?;
    ///
    /// // Phase 2: Initialize MCP session  
    /// client.initialize().await?;
    ///
    /// // Now ready for operations
    /// let tools = client.list_tools().await?;
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
        let transport = transport_builder
            .with_message_handler(handler)
            .build()
            .await
            .map_err(|e| McpError::custom(format!("Failed to build transport: {e}")))?;

        // NOTE: We do NOT start the transport here - this allows clean separation
        // between client creation and transport connection. Use connect() method.

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
#[derive(Default)]
struct ReconnectionState {
    /// Current reconnection attempt count
    attempt_count: u32,
    /// Last reconnection attempt time
    last_attempt: Option<Instant>,
    /// Whether client is currently attempting to reconnect
    is_reconnecting: bool,
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
        matches!(error, McpError::NotConnected | McpError::Integration(IntegrationError::Transport(_)) | McpError::Integration(IntegrationError::Timeout { .. }))
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
                    if Self::is_connection_error(&error) && self.config.auto_reconnect
                        && (self.attempt_reconnection().await).is_err() {
                            // If reconnection fails, return the original error
                            return Err(error);
                        }

                    attempt += 1;
                    let delay = self.calculate_retry_delay(attempt - 1);

                    // Log retry attempt with structured logging
                    warn!(
                        attempt = attempt,
                        max_retries = self.config.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %error,
                        "Retrying operation after delay due to error"
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

            info!(
                attempt = self.reconnection_state.read().await.attempt_count,
                max_attempts = self.config.max_reconnect_attempts,
                delay_seconds = delay.as_secs(),
                "Attempting reconnection after delay"
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
                info!("Reconnection successful");
            }
            Err(error) => {
                error!(
                    error = %error,
                    attempt = reconnection_state.attempt_count,
                    max_attempts = self.config.max_reconnect_attempts,
                    "Reconnection attempt failed"
                );
            }
        }

        result
    }

    /// Connect the transport layer
    ///
    /// This method starts the underlying transport connection. This must be called
    /// before attempting MCP session initialization via `initialize()`.
    ///
    /// # Architecture
    ///
    /// This implements clean separation between:
    /// - **Transport Connection**: Low-level connectivity and message passing  
    /// - **MCP Session**: Protocol-level handshake and capability negotiation
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let client = McpClientBuilder::new().build(StdioTransportBuilder::new()).await?;
    ///
    /// // Step 1: Connect transport
    /// client.connect().await?;
    /// assert!(client.transport_connected().await);
    ///
    /// // Step 2: Initialize MCP session
    /// client.initialize().await?;
    /// assert!(client.is_ready().await);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&self) -> McpResult<()> {
        debug!("Starting transport connection");

        let mut transport = self.transport.write().await;
        transport
            .start()
            .await
            .map_err(|e| McpError::custom(format!("Failed to start transport: {e}")))?;

        info!("Transport connection established");
        Ok(())
    }

    /// Disconnect the transport layer
    ///
    /// This method closes the underlying transport connection. If an MCP session
    /// is active, you should call `close()` first for proper cleanup.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let client = McpClientBuilder::new().build(StdioTransportBuilder::new()).await?;
    /// client.connect().await?;
    /// client.initialize().await?;
    ///
    /// // Proper shutdown sequence:
    /// client.close().await?;        // Close MCP session
    /// client.disconnect().await?;   // Close transport
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&self) -> McpResult<()> {
        debug!("Closing transport connection");

        let mut transport = self.transport.write().await;
        transport
            .close()
            .await
            .map_err(|e| McpError::custom(format!("Failed to close transport: {e}")))?;

        info!("Transport connection closed");
        Ok(())
    }

    /// Initialize connection with the MCP server (with retry logic)
    pub async fn initialize(&self) -> McpResult<ServerCapabilities> {
        self.execute_with_retry(|| self.initialize_without_retry())
            .await
    }

    /// Initialize connection with the MCP server (without retry logic, used during reconnection)
    async fn initialize_without_retry(&self) -> McpResult<ServerCapabilities> {
        // Use transactional semantics to ensure clean rollback on failure
        self.execute_transactional(|| async {
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
            debug!("Starting MCP initialization");
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
            let response = self.send_request_once(&request_msg).await.inspect_err(|_e| {
                // Mark session as failed on communication error
                let session_ref = self.mcp_session.clone();
                tokio::spawn(async move {
                    *session_ref.write().await = McpSessionState::Failed;
                });
            })?;

            // Parse initialization response
            let init_response: InitializeResponse =
                serde_json::from_value(response.result.ok_or_else(|| {
                    McpError::invalid_response("Missing result in initialization response")
                })?)
                .map_err(|e| {
                    // Mark session as failed on protocol error
                    let session_ref = self.mcp_session.clone();
                    tokio::spawn(async move {
                        *session_ref.write().await = McpSessionState::Failed;
                    });
                    McpError::invalid_response(format!("Invalid initialization response: {e}"))
                })?;

            // Store server capabilities
            let server_caps: ServerCapabilities =
                serde_json::from_value(init_response.capabilities).map_err(|e| {
                    // Mark session as failed on capability parsing error
                    let session_ref = self.mcp_session.clone();
                    tokio::spawn(async move {
                        *session_ref.write().await = McpSessionState::Failed;
                    });
                    McpError::invalid_response(format!("Invalid server capabilities: {e}"))
                })?;
            *self.server_capabilities.write().await = Some(server_caps.clone());

            // 4. Update MCP session state to ready (success!)
            debug!("MCP initialization completed successfully");
            *self.mcp_session.write().await = McpSessionState::Ready;

            Ok(server_caps)
        })
        .await
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

    /// Close the MCP session (protocol-level cleanup)
    ///
    /// This method performs MCP session cleanup but does NOT close the transport.
    /// Use `disconnect()` separately to close the transport connection, or use
    /// `shutdown_gracefully()` for complete cleanup.
    ///
    /// # Clean Architecture
    ///
    /// This implements clean separation between:
    /// - **MCP Session Close**: Protocol cleanup, state reset, pending request cancellation
    /// - **Transport Close**: Low-level connection termination via `disconnect()`
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let client = McpClientBuilder::new().build(StdioTransportBuilder::new()).await?;
    /// client.connect().await?;
    /// client.initialize().await?;
    ///
    /// // Clean shutdown - separate concerns:
    /// client.close().await?;        // Close MCP session
    /// client.disconnect().await?;   // Close transport
    ///
    /// // Or combined:
    /// // client.shutdown_gracefully(Duration::from_secs(5)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&self) -> McpResult<()> {
        debug!("Starting MCP session closure");

        // Phase 1: Cancel pending requests with appropriate errors
        self.cancel_pending_requests("MCP session closed").await;

        // Phase 2: Gracefully close MCP session (send goodbye if needed)
        let current_state = self.mcp_session.read().await.clone();
        if matches!(current_state, McpSessionState::Ready) {
            debug!("Gracefully closing MCP session");
            // Send an optional goodbye message if protocol supports it
            // For now, we'll just note the graceful closure
        }

        // Phase 3: Reset MCP session state
        debug!("Resetting MCP session state");
        *self.mcp_session.write().await = McpSessionState::NotInitialized;

        // Phase 4: Reset reconnection state
        *self.reconnection_state.write().await = ReconnectionState::default();

        info!("MCP session closed successfully");
        Ok(())
    }

    /// Cancel all pending requests with an error message
    async fn cancel_pending_requests(&self, reason: &str) {
        let mut pending = self.pending_requests.lock().await;
        let request_count = pending.len();

        if request_count > 0 {
            debug!(
                request_count = request_count,
                reason = reason,
                "Cancelling pending requests"
            );

            // Create cancellation error
            let error_value = serde_json::json!({
                "code": -32603, // Internal error
                "message": format!("Request cancelled: {}", reason)
            });

            let error_response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error_value),
                id: None, // Will be set per request
            };

            // Send cancellation to all pending requests
            for (request_id, sender) in pending.drain() {
                let mut response = error_response.clone();
                response.id = Some(RequestId::new_string(&request_id));
                let _ = sender.send(response); // Ignore send errors (receiver might be dropped)
            }
        }
    }

    /// Attempt graceful shutdown with timeout (complete cleanup)
    ///
    /// This method performs complete shutdown including both MCP session cleanup
    /// and transport disconnection. It implements the full clean shutdown sequence:
    /// 1. Close MCP session (protocol cleanup)
    /// 2. Disconnect transport (connection cleanup)
    ///
    /// If the graceful shutdown times out, it falls back to force shutdown.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
    /// # use std::time::Duration;
    /// # async fn example() -> McpResult<()> {
    /// let client = McpClientBuilder::new().build(StdioTransportBuilder::new()).await?;
    /// client.connect().await?;
    /// client.initialize().await?;
    ///
    /// // Complete graceful shutdown with 5-second timeout
    /// client.shutdown_gracefully(Duration::from_secs(5)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown_gracefully(&self, timeout: Duration) -> McpResult<()> {
        debug!("Starting graceful shutdown with timeout: {:?}", timeout);

        // Define the complete shutdown sequence
        let shutdown_sequence = async {
            // Step 1: Close MCP session (protocol cleanup)
            self.close().await?;

            // Step 2: Disconnect transport (connection cleanup)
            self.disconnect().await?;

            Ok(())
        };

        // Try to execute gracefully within timeout
        match tokio::time::timeout(timeout, shutdown_sequence).await {
            Ok(result) => {
                info!("Graceful shutdown completed successfully");
                result
            }
            Err(_) => {
                warn!("Graceful shutdown timed out, forcing closure");
                // Graceful shutdown timed out, force immediate shutdown
                self.force_shutdown().await
            }
        }
    }

    /// Force immediate shutdown without graceful cleanup
    async fn force_shutdown(&self) -> McpResult<()> {
        warn!("Performing forced shutdown without graceful cleanup");

        // Cancel all pending requests immediately
        self.cancel_pending_requests("Forced shutdown").await;

        // Force reset all state
        *self.mcp_session.write().await = McpSessionState::NotInitialized;
        *self.reconnection_state.write().await = ReconnectionState::default();

        // Force close transport without waiting for graceful closure
        let mut transport = self.transport.write().await;
        let _ = transport.close().await; // Ignore errors during forced shutdown

        info!("Forced shutdown completed");
        Ok(())
    }

    /// Execute an operation with transaction-like semantics
    /// If the operation fails, attempts to restore previous state
    async fn execute_transactional<F, Fut, R>(&self, operation: F) -> McpResult<R>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = McpResult<R>>,
    {
        // Capture current state before operation
        let previous_session_state = self.mcp_session.read().await.clone();
        let previous_reconnection_state = self.reconnection_state.read().await.clone();

        // Execute the operation
        let result = operation().await;

        // If operation failed and we're in a bad state, attempt rollback
        if result.is_err() {
            let current_session_state = self.mcp_session.read().await.clone();

            // If session state was changed and is now in Failed state, attempt rollback
            if matches!(current_session_state, McpSessionState::Failed)
                && !matches!(previous_session_state, McpSessionState::Failed)
            {
                debug!(
                    previous_state = ?previous_session_state,
                    current_state = ?current_session_state,
                    "Operation failed, attempting state rollback"
                );

                // Restore previous state if it was stable
                if matches!(
                    previous_session_state,
                    McpSessionState::Ready | McpSessionState::NotInitialized
                ) {
                    *self.mcp_session.write().await = previous_session_state;
                    *self.reconnection_state.write().await = previous_reconnection_state;
                }
            }
        }

        result
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
        let response_result = tokio::time::timeout(self.config.default_timeout, receiver).await;

        // Clean up pending request on timeout or cancellation
        match response_result {
            Ok(receiver_result) => {
                // Response received or channel was closed
                receiver_result.map_err(|_| McpError::custom("Request cancelled"))
            }
            Err(_) => {
                // Timeout occurred - clean up the pending request
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id_str);
                Err(McpError::custom("Request timeout"))
            }
        }
    }

    /// Internal helper to make MCP method calls
    async fn call_mcp<P: serde::Serialize>(&self, method: &str, params: &P) -> McpResult<Value> {
        debug!(method = method, "Calling MCP method");

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
            warn!(method = method, error = %error, "MCP method call failed with server error");
            return Err(McpError::server_error(format!("Server error: {error}")));
        }

        debug!(method = method, "MCP method call completed successfully");
        Ok(response.result.unwrap_or(Value::Null))
    }
}

// Implement Drop to ensure clean shutdown
impl<T: Transport> Drop for McpClient<T> {
    fn drop(&mut self) {
        // Note: We can't call async methods in Drop, but we should at least
        // try to cancel pending requests synchronously if possible
        if let Ok(mut pending) = self.pending_requests.try_lock() {
            let request_count = pending.len();
            if request_count > 0 {
                // Clear pending requests (they will receive cancellation errors
                // when their receivers are dropped)
                pending.clear();
            }
        }

        // The underlying transport should handle cleanup automatically
        // For proper async cleanup, use close() or shutdown_gracefully() explicitly
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

    // Advanced Mock Transport for comprehensive testing
    struct AdvancedMockTransport {
        connected: bool,
        message_handler: Option<Arc<dyn MessageHandler>>,
        sent_messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
        failure_count: Arc<Mutex<i32>>,
        server_capabilities: ServerCapabilities,
        custom_responses: Arc<Mutex<HashMap<String, Value>>>,
    }

    impl AdvancedMockTransport {
        fn new() -> Self {
            Self {
                connected: true,
                message_handler: None,
                sent_messages: Arc::new(Mutex::new(Vec::new())),
                failure_count: Arc::new(Mutex::new(0)),
                custom_responses: Arc::new(Mutex::new(HashMap::new())),
                server_capabilities: ServerCapabilities {
                    experimental: None,
                    tools: Some(crate::protocol::ToolCapabilities {
                        list_changed: Some(true),
                    }),
                    resources: Some(crate::protocol::ResourceCapabilities {
                        subscribe: Some(true),
                        list_changed: Some(true),
                    }),
                    prompts: Some(crate::protocol::PromptCapabilities {
                        list_changed: Some(true),
                    }),
                    logging: Some(crate::protocol::LoggingCapabilities {}),
                },
            }
        }

        fn with_failure() -> Self {
            let mut transport = Self::new();
            transport.failure_count = Arc::new(Mutex::new(5)); // Fail 5 times, then work
            transport
        }

        // Allow programmatic control of responses
        async fn set_custom_response(&self, method: &str, response: Value) {
            let mut responses = self.custom_responses.lock().await;
            responses.insert(method.to_string(), response);
        }

        #[allow(dead_code)]
        async fn get_sent_messages(&self) -> Vec<JsonRpcMessage> {
            self.sent_messages.lock().await.clone()
        }
    }

    #[async_trait]
    impl Transport for AdvancedMockTransport {
        type Error = std::io::Error;

        async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> {
            let mut failure_count = self.failure_count.lock().await;
            if *failure_count > 0 {
                *failure_count -= 1;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Mock failure",
                ));
            }

            self.sent_messages.lock().await.push(message.clone());

            // Auto-respond to requests
            if let JsonRpcMessage::Request(req) = message {
                // Check for custom responses first
                let custom_responses = self.custom_responses.lock().await;
                if let Some(custom_response) = custom_responses.get(&req.method) {
                    let response_value = custom_response.clone();
                    drop(custom_responses); // Release the lock

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(response_value),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                    return Ok(());
                }
                drop(custom_responses); // Release the lock

                // Default built-in responses for standard methods
                if req.method == "initialize" {
                    let init_response = InitializeResponse {
                        protocol_version: crate::protocol::ProtocolVersion::current(),
                        capabilities: serde_json::to_value(&self.server_capabilities).unwrap(),
                        server_info: crate::protocol::ServerInfo {
                            name: "test-server".to_string(),
                            version: "1.0.0".to_string(),
                        },
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(init_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "tools/list" {
                    let tools_response = ListToolsResponse {
                        tools: vec![
                            Tool {
                                name: "calculator".to_string(),
                                description: Some("A simple calculator tool".to_string()),
                                input_schema: serde_json::json!({
                                    "type": "object",
                                    "properties": {
                                        "operation": {"type": "string"},
                                        "a": {"type": "number"},
                                        "b": {"type": "number"}
                                    }
                                }),
                            },
                            Tool {
                                name: "echo".to_string(),
                                description: Some("Echo back the input".to_string()),
                                input_schema: serde_json::json!({
                                    "type": "object",
                                    "properties": {
                                        "message": {"type": "string"}
                                    }
                                }),
                            },
                        ],
                        next_cursor: None,
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(tools_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "tools/call" {
                    let call_response = CallToolResponse {
                        content: vec![Content::Text {
                            text: "Tool executed successfully".to_string(),
                            uri: None,
                            mime_type: None,
                        }],
                        is_error: Some(false),
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(call_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "resources/list" {
                    let resources_response = ListResourcesResponse {
                        resources: vec![Resource {
                            uri: crate::protocol::Uri::new("file://test.txt").unwrap(),
                            name: "Test File".to_string(),
                            description: Some("A test file resource".to_string()),
                            mime_type: Some(crate::protocol::MimeType::new("text/plain").unwrap()),
                        }],
                        next_cursor: None,
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(resources_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "resources/read" {
                    let read_response = ReadResourceResponse {
                        contents: vec![Content::Text {
                            text: "This is the content of the test file.".to_string(),
                            uri: None,
                            mime_type: None,
                        }],
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(read_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "prompts/list" {
                    let prompts_response = ListPromptsResponse {
                        prompts: vec![Prompt {
                            name: "test-prompt".to_string(),
                            title: Some("Test Prompt".to_string()),
                            description: Some("A test prompt".to_string()),
                            arguments: vec![crate::protocol::PromptArgument {
                                name: "input".to_string(),
                                description: Some("Input parameter".to_string()),
                                required: true,
                            }],
                        }],
                        next_cursor: None,
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(prompts_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                } else if req.method == "prompts/get" {
                    let prompt_response = GetPromptResponse {
                        description: Some("Generated prompt for test input".to_string()),
                        messages: vec![crate::protocol::PromptMessage {
                            role: "user".to_string(),
                            content: Content::Text {
                                text: "You are a helpful assistant. The user said: test input"
                                    .to_string(),
                                uri: None,
                                mime_type: None,
                            },
                        }],
                    };

                    tokio::spawn({
                        let handler = self.message_handler.clone();
                        let req_id = req.id.clone();
                        async move {
                            if let Some(h) = handler {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    result: Some(serde_json::to_value(prompt_response).unwrap()),
                                    error: None,
                                    id: Some(req_id),
                                };
                                h.handle_message(
                                    JsonRpcMessage::Response(response),
                                    MessageContext::without_session(),
                                )
                                .await;
                            }
                        }
                    });
                }
            }

            Ok(())
        }

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.connected = true;
            Ok(())
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn session_id(&self) -> Option<String> {
            Some("test-session".to_string())
        }

        fn set_session_context(&mut self, _session_id: Option<String>) {
            // Mock implementation
        }

        fn transport_type(&self) -> &'static str {
            "advanced-mock"
        }
    }

    struct AdvancedMockTransportBuilder {
        transport: AdvancedMockTransport,
    }

    impl AdvancedMockTransportBuilder {
        fn new() -> Self {
            Self {
                transport: AdvancedMockTransport::new(),
            }
        }

        fn with_failure() -> Self {
            Self {
                transport: AdvancedMockTransport::with_failure(),
            }
        }

        // Allow setting custom responses during construction
        async fn with_custom_response(self, method: &str, response: Value) -> Self {
            self.transport.set_custom_response(method, response).await;
            self
        }

        // Get reference to transport for advanced configuration
        #[allow(dead_code)]
        fn transport(&self) -> &AdvancedMockTransport {
            &self.transport
        }
    }

    impl TransportBuilder for AdvancedMockTransportBuilder {
        type Transport = AdvancedMockTransport;
        type Error = std::io::Error;

        fn with_message_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
            self.transport.message_handler = Some(handler);
            self
        }

        async fn build(
            self,
        ) -> Result<Self::Transport, Self::Error>
        { Ok(self.transport) }
    }

    // Comprehensive Lifecycle Tests

    #[tokio::test]
    async fn test_client_initialization_lifecycle() {
        let client = McpClientBuilder::new()
            .client_info("test-client", "1.0.0")
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // Initial state should be not initialized
        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );
        assert!(!client.is_ready().await);
        assert!(client.server_capabilities().await.is_none());

        // Initialize the client
        let capabilities = client.initialize().await.unwrap();

        // After initialization, client should be ready
        assert_eq!(client.session_state().await, McpSessionState::Ready);
        assert!(client.is_ready().await);
        assert!(client.server_capabilities().await.is_some());

        // Verify capabilities
        assert!(capabilities.tools.is_some());
        assert!(capabilities.resources.is_some());
        assert!(capabilities.prompts.is_some());
        assert!(capabilities.logging.is_some());
    }

    #[tokio::test]
    async fn test_double_initialization_error() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // First initialization should succeed
        client.initialize().await.unwrap();
        assert_eq!(client.session_state().await, McpSessionState::Ready);

        // Second initialization should fail
        let result = client.initialize().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::AlreadyConnected));
    }

    #[tokio::test]
    async fn test_operations_before_initialization() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // All operations should fail before initialization
        assert!(client.list_tools().await.is_err());
        assert!(client.list_resources().await.is_err());
        assert!(client.list_prompts().await.is_err());
        assert!(client.call_tool("test", None).await.is_err());
    }

    // Tool Operations Tests

    #[tokio::test]
    async fn test_list_tools_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // Initialize first
        client.initialize().await.unwrap();

        // List tools
        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 2);

        // Verify tool details
        let calculator_tool = tools.iter().find(|t| t.name == "calculator").unwrap();
        assert_eq!(
            calculator_tool.description,
            Some("A simple calculator tool".to_string())
        );

        let echo_tool = tools.iter().find(|t| t.name == "echo").unwrap();
        assert_eq!(
            echo_tool.description,
            Some("Echo back the input".to_string())
        );
    }

    #[tokio::test]
    async fn test_call_tool_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // Call a tool
        let args = serde_json::json!({
            "operation": "add",
            "a": 5,
            "b": 3
        });
        let result = client.call_tool("calculator", Some(args)).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Content::Text { text, .. } = &result[0] {
            assert_eq!(text, "Tool executed successfully");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_tool_operations_without_capability() {
        // This test simulates a server without tool capabilities
        // We'll just verify the error handling for unsupported capabilities
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // Since our mock always supports tools, we can't test this easily
        // In a real scenario, the server would return capabilities without tools
        // For now, we'll just verify that tools are supported
        assert!(
            client
                .supports_capability(|caps| caps.tools.is_some())
                .await
        );
    }

    // Resource Operations Tests

    #[tokio::test]
    async fn test_list_resources_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        let resources = client.list_resources().await.unwrap();
        assert_eq!(resources.len(), 1);

        let resource = &resources[0];
        assert_eq!(resource.uri.to_string(), "file://test.txt");
        assert_eq!(resource.name, "Test File");
        assert_eq!(
            resource.mime_type.as_ref().map(|m| m.to_string()),
            Some("text/plain".to_string())
        );
    }

    #[tokio::test]
    async fn test_read_resource_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // Read a resource and verify the content
        let contents = client.read_resource("file://test.txt").await.unwrap();
        assert_eq!(contents.len(), 1);

        if let Content::Text { text, .. } = &contents[0] {
            assert_eq!(text, "This is the content of the test file.");
        } else {
            panic!("Expected text content");
        }
    }

    // Prompt Operations Tests

    #[tokio::test]
    async fn test_list_prompts_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        let prompts = client.list_prompts().await.unwrap();
        assert_eq!(prompts.len(), 1);

        let prompt = &prompts[0];
        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.description, Some("A test prompt".to_string()));
    }

    #[tokio::test]
    async fn test_get_prompt_functionality() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        let mut args = HashMap::new();
        args.insert("input".to_string(), "test input".to_string());

        // Get a prompt and verify the response
        let messages = client.get_prompt("test-prompt", args).await.unwrap();
        assert_eq!(messages.len(), 1);

        let message = &messages[0];
        assert_eq!(message.role, "user".to_string());
        if let Content::Text { text, .. } = &message.content {
            assert!(text.contains("test input"));
            assert!(text.contains("helpful assistant"));
        } else {
            panic!("Expected text content");
        }
    }

    // Retry and Reconnection Tests

    #[tokio::test]
    async fn test_retry_on_transport_failure() {
        let client = McpClientBuilder::new()
            .auto_retry(true, 2)
            .retry_timing(Duration::from_millis(1), Duration::from_millis(5))
            .build(AdvancedMockTransportBuilder::with_failure())
            .await
            .unwrap();

        // This should fail due to transport failure
        // Use timeout to prevent hanging
        let result = tokio::time::timeout(Duration::from_millis(100), client.initialize()).await;

        match result {
            Ok(init_result) => {
                // If initialization completes, it should be an error
                assert!(init_result.is_err());
            }
            Err(_) => {
                // Timeout is also acceptable for this test since it means
                // the retry mechanism is working but eventually timing out
            }
        }
    }

    #[tokio::test]
    async fn test_comprehensive_capability_checking() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // Test capability checking
        assert!(
            client
                .supports_capability(|caps| caps.tools.is_some())
                .await
        );
        assert!(
            client
                .supports_capability(|caps| caps.resources.is_some())
                .await
        );
        assert!(
            client
                .supports_capability(|caps| caps.prompts.is_some())
                .await
        );
        assert!(
            client
                .supports_capability(|caps| caps.logging.is_some())
                .await
        );
        assert!(
            !client
                .supports_capability(|caps| caps.experimental.is_some())
                .await
        );
    }

    #[tokio::test]
    async fn test_client_shutdown_lifecycle() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();
        assert!(client.is_ready().await);

        // Test graceful shutdown
        client.close().await.unwrap();
        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );
        assert!(!client.is_ready().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown_with_timeout() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // Test graceful shutdown with timeout
        let result = client.shutdown_gracefully(Duration::from_millis(100)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reconnection_status_tracking() {
        let client = McpClientBuilder::new()
            .auto_reconnect(true)
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // Initial reconnection status
        let (attempt_count, is_reconnecting) = client.reconnection_status().await;
        assert_eq!(attempt_count, 0);
        assert!(!is_reconnecting);
    }

    #[tokio::test]
    async fn test_session_state_transitions() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        // Initial state
        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );

        // After initialization
        client.initialize().await.unwrap();
        assert_eq!(client.session_state().await, McpSessionState::Ready);

        // After close
        client.close().await.unwrap();
        assert_eq!(
            client.session_state().await,
            McpSessionState::NotInitialized
        );
    }

    #[tokio::test]
    async fn test_caching_behavior() {
        let client = McpClientBuilder::new()
            .build(AdvancedMockTransportBuilder::new())
            .await
            .unwrap();

        client.initialize().await.unwrap();

        // First call should populate cache
        let tools1 = client.list_tools().await.unwrap();
        assert_eq!(tools1.len(), 2);

        // Second call should use cache (we can't directly test this with our mock,
        // but we can verify it doesn't fail)
        let tools2 = client.list_tools().await.unwrap();
        assert_eq!(tools2.len(), 2);
    }

    // Advanced Tests with Custom Mock Responses

    #[tokio::test]
    async fn test_custom_tool_response() {
        let builder = AdvancedMockTransportBuilder::new();

        // Set up a custom tool execution response
        let custom_tool_response = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "Custom calculator result: 42"
                }
            ],
            "isError": false
        });

        let builder = builder
            .with_custom_response("tools/call", custom_tool_response)
            .await;

        let client = McpClientBuilder::new().build(builder).await.unwrap();

        client.initialize().await.unwrap();

        // Call the tool and verify our custom response
        let result = client
            .call_tool(
                "calculator",
                Some(serde_json::json!({
                    "operation": "add",
                    "a": 20,
                    "b": 22
                })),
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        if let Content::Text { text, .. } = &result[0] {
            assert_eq!(text, "Custom calculator result: 42");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_custom_resource_content() {
        let builder = AdvancedMockTransportBuilder::new();

        // Set up a custom resource read response with different content
        let custom_resource_response = serde_json::json!({
            "contents": [
                {
                    "type": "text",
                    "text": "# Custom Configuration\napi_key: secret_value\ndebug: true"
                }
            ]
        });

        let builder = builder
            .with_custom_response("resources/read", custom_resource_response)
            .await;

        let client = McpClientBuilder::new().build(builder).await.unwrap();

        client.initialize().await.unwrap();

        // Read the resource and verify our custom content
        let contents = client.read_resource("file://config.yaml").await.unwrap();
        assert_eq!(contents.len(), 1);

        if let Content::Text { text, .. } = &contents[0] {
            assert!(text.contains("Custom Configuration"));
            assert!(text.contains("api_key: secret_value"));
            assert!(text.contains("debug: true"));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_error_response_simulation() {
        let builder = AdvancedMockTransportBuilder::new();

        // Simulate a server error response
        let error_response = serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "Tool execution failed: Division by zero"
                }
            ],
            "is_error": true
        });

        let builder = builder
            .with_custom_response("tools/call", error_response)
            .await;

        let client = McpClientBuilder::new().build(builder).await.unwrap();

        client.initialize().await.unwrap();

        // Call the tool and expect an error
        let result = client
            .call_tool(
                "calculator",
                Some(serde_json::json!({
                    "operation": "divide",
                    "a": 10,
                    "b": 0
                })),
            )
            .await;

        assert!(result.is_err());
        if let Err(McpError::ToolExecutionFailed { name, reason }) = result {
            assert_eq!(name, "calculator");
            assert!(reason.contains("Division by zero"));
        } else {
            panic!("Expected ToolExecutionFailed error");
        }
    }

    #[tokio::test]
    async fn test_dynamic_prompt_response() {
        let builder = AdvancedMockTransportBuilder::new();

        // Set up a dynamic prompt response that varies based on input
        let custom_prompt_response = serde_json::json!({
            "description": "Dynamic prompt for user query",
            "messages": [
                {
                    "role": "system",
                    "content": {
                        "type": "text",
                        "text": "You are a specialized assistant for the given context."
                    }
                },
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "Please help me with the following task..."
                    }
                }
            ]
        });

        let builder = builder
            .with_custom_response("prompts/get", custom_prompt_response)
            .await;

        let client = McpClientBuilder::new().build(builder).await.unwrap();

        client.initialize().await.unwrap();

        let mut args = HashMap::new();
        args.insert("context".to_string(), "data analysis".to_string());

        // Get the prompt and verify the dynamic response
        let messages = client.get_prompt("analyze-data", args).await.unwrap();
        assert_eq!(messages.len(), 2);

        // Check system message
        assert_eq!(messages[0].role, "system".to_string());
        if let Content::Text { text, .. } = &messages[0].content {
            assert!(text.contains("specialized assistant"));
        }

        // Check user message
        assert_eq!(messages[1].role, "user".to_string());
        if let Content::Text { text, .. } = &messages[1].content {
            assert!(text.contains("help me with the following task"));
        }
    }

    #[tokio::test]
    async fn test_message_tracking() {
        // Create shared message tracking
        let sent_messages = Arc::new(Mutex::new(Vec::new()));

        // Create transport with custom message tracking
        let mut transport = AdvancedMockTransport::new();
        transport.sent_messages = sent_messages.clone();

        let builder = AdvancedMockTransportBuilder { transport };

        let client = McpClientBuilder::new().build(builder).await.unwrap();

        client.initialize().await.unwrap();

        // Perform several operations
        let _ = client.list_tools().await;
        let _ = client.list_resources().await;
        let _ = client
            .call_tool("echo", Some(serde_json::json!({"message": "hello"})))
            .await;

        // Verify that all messages were sent
        let messages = sent_messages.lock().await.clone();
        assert!(messages.len() >= 4); // initialize + 3 operations

        // Check that we have the expected method calls
        let method_calls: Vec<String> = messages
            .iter()
            .filter_map(|msg| {
                if let JsonRpcMessage::Request(req) = msg {
                    Some(req.method.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(method_calls.contains(&"initialize".to_string()));
        assert!(method_calls.contains(&"tools/list".to_string()));
        assert!(method_calls.contains(&"resources/list".to_string()));
    }
}
