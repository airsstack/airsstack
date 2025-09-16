//! High-level MCP Client API (TransportClient-based)
//!
//! This module provides a high-level, type-safe MCP client that simplifies
//! interaction with MCP servers through the clean TransportClient interface.
//!
//! This implementation replaces the previous Transport-based architecture with
//! a cleaner request-response pattern that eliminates the complexity of
//! MessageHandler correlation and provides better separation of concerns.
//!
//! # Architecture
//!
//! ```text
//! McpClient -> TransportClient -> MCP Server
//!     |             |                |
//!     |- call() ->  |- HTTP/STDIO -> |- JSON-RPC
//!     |<- response <-|               |- direct response
//! ```
//!
//! # Examples
//!
//! ## Using HTTP Transport
//!
//! ```rust,no_run
//! use airs_mcp::integration::{McpClientBuilder, McpResult};
//! use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> McpResult<()> {
//! // Create HTTP transport client
//! let transport = HttpTransportClientBuilder::new()
//!     .endpoint("https://api.example.com/mcp")?
//!     .auth(AuthMethod::ApiKey {
//!         key: "your-api-key".to_string(),
//!         header: "X-API-Key".to_string()
//!     })
//!     .timeout(Duration::from_secs(30))
//!     .build()
//!     .await?;
//!
//! // Create MCP client
//! let mut client = McpClientBuilder::new()
//!     .client_info("my-client", "1.0.0")
//!     .timeout(Duration::from_secs(60))
//!     .build(transport);
//!
//! // Initialize MCP session
//! let capabilities = client.initialize().await?;
//!
//! // Use the client
//! let tools = client.list_tools().await?;
//! let result = client.call_tool("calculator", None).await?;
//!
//! client.close().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using STDIO Transport
//!
//! ```rust,no_run
//! use airs_mcp::integration::{McpClientBuilder, McpResult};
//! use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> McpResult<()> {
//! // Create STDIO transport client for child process
//! let transport = StdioTransportClientBuilder::new()
//!     .command("python")
//!     .args(vec!["-m".to_string(), "my_mcp_server".to_string()])
//!     .timeout(Duration::from_secs(30))
//!     .build()
//!     .await?;
//!
//! // Create MCP client
//! let mut client = McpClientBuilder::new()
//!     .client_info("my-client", "1.0.0")
//!     .build(transport);
//!
//! // Initialize and use
//! client.initialize().await?;
//! let resources = client.list_resources().await?;
//! client.close().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Observability
//!
//! This module uses structured logging via the `tracing` crate:
//!
//! - **Info level**: Connection state changes, successful operations
//! - **Error level**: Failed operations, connection failures
//! - **Debug level**: Method calls, detailed flow tracking
//!
//! To enable logging:
//!
//! ```rust,no_run
//! tracing_subscriber::fmt()
//!     .with_env_filter("airs_mcp=debug")
//!     .init();
//! ```

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports
use serde_json::Value;
use tracing::{debug, error, info};

// Layer 3: Internal module imports
use crate::integration::constants::methods;
use crate::integration::McpError;
use crate::protocol::{
    CallToolRequest, CallToolResponse, ClientCapabilities, ClientInfo, Content, GetPromptRequest,
    GetPromptResponse, InitializeRequest, InitializeResponse, JsonRpcRequest, ListPromptsRequest,
    ListPromptsResponse, ListResourcesRequest, ListResourcesResponse, ListToolsRequest,
    ListToolsResponse, LoggingConfig, Prompt, PromptMessage, ProtocolVersion, ReadResourceRequest,
    ReadResourceResponse, RequestId, Resource, ServerCapabilities, SetLoggingRequest,
    SetLoggingResponse, SubscribeResourceRequest, Tool, TransportClient,
};

/// Type alias for MCP client results
pub type McpResult<T> = Result<T, McpError>;

/// MCP Protocol Session State
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
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            client_info: ClientInfo {
                name: "airs-mcp-client".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ClientCapabilities::default(),
            protocol_version: ProtocolVersion::current(),
            default_timeout: Duration::from_secs(30),
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

    /// Build the MCP client with a TransportClient
    ///
    /// This creates an MCP client that uses the clean TransportClient interface
    /// for direct request-response communication without the complexity of
    /// MessageHandler correlation patterns.
    ///
    /// # Architecture Benefits
    ///
    /// - **Clean separation**: Transport handles connectivity, client handles MCP protocol
    /// - **Simple flow**: Direct call() method instead of event-driven patterns
    /// - **Better errors**: Transport-specific errors with clear context
    /// - **Easy testing**: Mock TransportClient implementations are straightforward
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let transport = StdioTransportClientBuilder::new()
    ///     .command("python")
    ///     .arg("-m")
    ///     .arg("my_server")
    ///     .build()
    ///     .await?;
    ///
    /// let mut client = McpClientBuilder::new()
    ///     .client_info("my-client", "1.0.0")
    ///     .build(transport);
    ///
    /// client.initialize().await?;
    /// let tools = client.list_tools().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build<T: TransportClient + 'static>(self, transport: T) -> McpClient<T> {
        McpClient {
            transport,
            config: self.config,
            session_state: McpSessionState::NotInitialized,
            server_capabilities: None,
        }
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level MCP client using TransportClient for communication
pub struct McpClient<T: TransportClient> {
    /// Transport client for communication
    transport: T,
    /// Client configuration
    config: McpClientConfig,
    /// Current MCP session state
    session_state: McpSessionState,
    /// Server capabilities (available after initialization)
    server_capabilities: Option<ServerCapabilities>,
}

impl<T: TransportClient + 'static> McpClient<T> {
    /// Initialize connection with the MCP server
    pub async fn initialize(&mut self) -> McpResult<ServerCapabilities> {
        info!("Starting MCP client initialization");

        // Check if already initialized
        if matches!(self.session_state, McpSessionState::Ready) {
            return Err(McpError::AlreadyConnected);
        }

        // Check if transport is ready
        if !self.transport.is_ready() {
            return Err(McpError::NotConnected);
        }

        self.session_state = McpSessionState::Initializing;

        let result = self.perform_initialize().await;

        match &result {
            Ok(capabilities) => {
                self.session_state = McpSessionState::Ready;
                self.server_capabilities = Some(capabilities.clone());
                info!("MCP client initialization completed successfully");
            }
            Err(error) => {
                self.session_state = McpSessionState::Failed;
                error!(%error, "MCP client initialization failed");
            }
        }

        result
    }

    /// Perform the actual initialize request
    async fn perform_initialize(&mut self) -> McpResult<ServerCapabilities> {
        debug!("Sending initialize request");

        let request = InitializeRequest {
            protocol_version: self.config.protocol_version.clone(),
            capabilities: serde_json::to_value(&self.config.capabilities)
                .map_err(|e| McpError::custom(format!("Serialization error: {}", e)))?,
            client_info: self.config.client_info.clone(),
        };

        let json_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::INITIALIZE.to_string(),
            params: Some(
                serde_json::to_value(&request)
                    .map_err(|e| McpError::custom(format!("Serialization error: {}", e)))?,
            ),
            id: RequestId::new_number(1),
        };

        debug!("Calling transport with initialize request");
        let response = self
            .transport
            .call(json_request)
            .await
            .map_err(|e| McpError::custom(format!("Transport error: {}", e)))?;

        debug!("Received initialize response");

        // Check for JSON-RPC error
        if let Some(error) = response.error {
            return Err(McpError::custom(format!("JSON-RPC error: {:?}", error)));
        }

        // Parse the response
        let init_response: InitializeResponse =
            serde_json::from_value(response.result.ok_or_else(|| McpError::InvalidResponse {
                reason: "Missing result in initialize response".to_string(),
            })?)
            .map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid initialize response: {e}"),
            })?;

        debug!(
            protocol_version = %init_response.protocol_version,
            "Initialization successful"
        );

        // Parse server capabilities from JSON
        let server_capabilities: ServerCapabilities =
            serde_json::from_value(init_response.capabilities).map_err(|e| {
                McpError::InvalidResponse {
                    reason: format!("Invalid server capabilities: {e}"),
                }
            })?;

        Ok(server_capabilities)
    }

    /// Get current session state
    pub fn session_state(&self) -> McpSessionState {
        self.session_state.clone()
    }

    /// Check if client is ready for MCP operations
    pub fn is_ready(&self) -> bool {
        self.transport.is_ready() && matches!(self.session_state, McpSessionState::Ready)
    }

    /// Get server capabilities (available after initialization)
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Ensure client is initialized, returning an error if not
    fn ensure_initialized(&self) -> McpResult<()> {
        if !self.is_ready() {
            return Err(McpError::NotConnected);
        }
        Ok(())
    }

    /// Check if server supports a specific capability
    pub fn supports_capability(&self, check: impl Fn(&ServerCapabilities) -> bool) -> bool {
        if let Some(caps) = &self.server_capabilities {
            check(caps)
        } else {
            false
        }
    }

    // Resource Operations

    /// List available resources from the server
    pub async fn list_resources(&mut self) -> McpResult<Vec<Resource>> {
        self.ensure_initialized()?;

        // Check if server supports resources
        if !self.supports_capability(|caps| caps.resources.is_some()) {
            return Err(McpError::UnsupportedCapability {
                capability: "resources".to_string(),
            });
        }

        let request = ListResourcesRequest::new();
        let response = self.call_mcp(methods::RESOURCES_LIST, &request).await?;

        let list_response: ListResourcesResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid list resources response: {e}"),
            })?;

        Ok(list_response.resources)
    }

    /// Read content from a specific resource
    pub async fn read_resource(&mut self, uri: impl Into<String>) -> McpResult<Vec<Content>> {
        self.ensure_initialized()?;
        let uri = uri.into();

        let request =
            ReadResourceRequest::new(uri.clone()).map_err(|e| McpError::custom(e.to_string()))?;

        let response = self.call_mcp(methods::RESOURCES_READ, &request).await?;

        let read_response: ReadResourceResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid read resource response: {e}"),
            })?;

        Ok(read_response.contents)
    }

    /// Subscribe to changes for a specific resource
    pub async fn subscribe_to_resource(&mut self, uri: impl Into<String>) -> McpResult<()> {
        self.ensure_initialized()?;
        let uri = uri.into();

        // Check if server supports subscriptions
        if !self.supports_capability(|caps| {
            caps.resources
                .as_ref()
                .map(|r| r.subscribe.unwrap_or(false))
                .unwrap_or(false)
        }) {
            return Err(McpError::UnsupportedCapability {
                capability: "resource subscriptions".to_string(),
            });
        }

        let request = SubscribeResourceRequest::new(uri.clone())
            .map_err(|e| McpError::custom(e.to_string()))?;

        let _response = self
            .call_mcp(methods::RESOURCES_SUBSCRIBE, &request)
            .await?;

        Ok(())
    }

    // Tool Operations

    /// List available tools from the server
    pub async fn list_tools(&mut self) -> McpResult<Vec<Tool>> {
        self.ensure_initialized()?;

        // Check if server supports tools
        if !self.supports_capability(|caps| caps.tools.is_some()) {
            return Err(McpError::UnsupportedCapability {
                capability: "tools".to_string(),
            });
        }

        let request = ListToolsRequest::new();
        let response = self.call_mcp(methods::TOOLS_LIST, &request).await?;

        let list_response: ListToolsResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid list tools response: {e}"),
            })?;

        Ok(list_response.tools)
    }

    /// Execute a tool with the given arguments
    pub async fn call_tool(
        &mut self,
        name: impl Into<String>,
        arguments: Option<Value>,
    ) -> McpResult<Vec<Content>> {
        self.ensure_initialized()?;
        let name = name.into();

        let request = CallToolRequest::new(name.clone(), arguments.unwrap_or(Value::Null));
        let response = self.call_mcp(methods::TOOLS_CALL, &request).await?;

        let call_response: CallToolResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid call tool response: {e}"),
            })?;

        if call_response.is_error.unwrap_or(false) {
            use crate::protocol::errors::ProtocolError;
            return Err(McpError::Protocol(ProtocolError::invalid_message(format!(
                "Tool '{}' returned error: {}",
                name,
                call_response
                    .content
                    .first()
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "Unknown error".to_string())
            ))));
        }

        Ok(call_response.content)
    }

    // Prompt Operations

    /// List available prompts from the server
    pub async fn list_prompts(&mut self) -> McpResult<Vec<Prompt>> {
        self.ensure_initialized()?;

        // Check if server supports prompts
        if !self.supports_capability(|caps| caps.prompts.is_some()) {
            return Err(McpError::UnsupportedCapability {
                capability: "prompts".to_string(),
            });
        }

        let request = ListPromptsRequest::new();
        let response = self.call_mcp(methods::PROMPTS_LIST, &request).await?;

        let list_response: ListPromptsResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid list prompts response: {e}"),
            })?;

        Ok(list_response.prompts)
    }

    /// Get a prompt with the given arguments
    pub async fn get_prompt(
        &mut self,
        name: impl Into<String>,
        arguments: HashMap<String, String>,
    ) -> McpResult<Vec<PromptMessage>> {
        self.ensure_initialized()?;
        let name = name.into();

        let request = GetPromptRequest::new(name.clone(), arguments);
        let response = self.call_mcp(methods::PROMPTS_GET, &request).await?;

        let prompt_response: GetPromptResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid get prompt response: {e}"),
            })?;

        Ok(prompt_response.messages)
    }

    // Logging Operations

    /// Set logging configuration
    pub async fn set_logging_config(&mut self, config: LoggingConfig) -> McpResult<()> {
        self.ensure_initialized()?;

        // Check if server supports logging
        if !self.supports_capability(|caps| caps.logging.is_some()) {
            return Err(McpError::UnsupportedCapability {
                capability: "logging".to_string(),
            });
        }

        let request = SetLoggingRequest::new(config.level);
        let response = self.call_mcp(methods::LOGGING_SET_LEVEL, &request).await?;

        let log_response: SetLoggingResponse =
            serde_json::from_value(response).map_err(|e| McpError::InvalidResponse {
                reason: format!("Invalid set logging response: {e}"),
            })?;

        if !log_response.success {
            use crate::protocol::errors::ProtocolError;
            return Err(McpError::Protocol(ProtocolError::invalid_message(
                "Server rejected logging configuration".to_string(),
            )));
        }

        Ok(())
    }

    // Utility Operations

    /// Close the MCP session
    ///
    /// This method closes the underlying transport connection and resets the
    /// client state. After calling this method, the client must be reinitialized
    /// before it can be used again.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use airs_mcp::integration::{McpClientBuilder, McpResult};
    /// # use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
    /// # async fn example() -> McpResult<()> {
    /// let transport = StdioTransportClientBuilder::new()
    ///     .command("python")
    ///     .arg("-m")
    ///     .arg("my_server")
    ///     .build()
    ///     .await?;
    ///
    /// let mut client = McpClientBuilder::new().build(transport);
    /// client.initialize().await?;
    ///
    /// // Use the client...
    /// let tools = client.list_tools().await?;
    ///
    /// // Clean shutdown
    /// client.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> McpResult<()> {
        info!("Closing MCP client");

        // Close the transport
        self.transport
            .close()
            .await
            .map_err(|e| McpError::custom(format!("Transport close error: {}", e)))?;

        // Reset client state
        self.session_state = McpSessionState::NotInitialized;
        self.server_capabilities = None;

        info!("MCP client closed successfully");
        Ok(())
    }

    /// Internal helper to make MCP method calls
    async fn call_mcp<P: serde::Serialize>(
        &mut self,
        method: &str,
        params: &P,
    ) -> McpResult<Value> {
        debug!(method = method, "Calling MCP method");

        let params_value = serde_json::to_value(params)
            .map_err(|e| McpError::custom(format!("Serialization error: {}", e)))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params_value),
            id: RequestId::new_number(42), // Use a simple ID for now
        };

        let response = self
            .transport
            .call(request)
            .await
            .map_err(|e| McpError::custom(format!("Transport error: {}", e)))?;

        if let Some(error) = response.error {
            return Err(McpError::custom(format!("JSON-RPC error: {:?}", error)));
        }

        debug!(method = method, "MCP method call completed successfully");
        Ok(response.result.unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcResponse, TransportError};
    use async_trait::async_trait;

    // Mock transport for testing
    struct MockTransportClient {
        ready: bool,
        responses: HashMap<String, Value>,
    }

    impl MockTransportClient {
        fn new() -> Self {
            let mut responses = HashMap::new();

            // Mock initialize response
            responses.insert(
                "initialize".to_string(),
                serde_json::json!({
                    "protocolVersion": "1.0.0",
                    "capabilities": {
                        "tools": { "listChanged": true },
                        "resources": { "subscribe": true, "listChanged": true },
                        "prompts": { "listChanged": true },
                        "logging": {}
                    },
                    "serverInfo": {
                        "name": "mock-server",
                        "version": "1.0.0"
                    }
                }),
            );

            Self {
                ready: true,
                responses,
            }
        }
    }

    #[async_trait]
    impl TransportClient for MockTransportClient {
        type Error = TransportError;

        async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, Self::Error> {
            let result = self
                .responses
                .get(&request.method)
                .cloned()
                .unwrap_or(serde_json::json!({}));

            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: Some(request.id),
            })
        }

        fn is_ready(&self) -> bool {
            self.ready
        }

        fn transport_type(&self) -> &'static str {
            "mock"
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            self.ready = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        let transport = MockTransportClient::new();
        let client = McpClientBuilder::new()
            .client_info("test-client", "1.0.0")
            .build(transport);

        assert_eq!(client.session_state(), McpSessionState::NotInitialized);
        assert!(!client.is_ready());
    }

    #[tokio::test]
    async fn test_initialization() {
        let transport = MockTransportClient::new();
        let mut client = McpClientBuilder::new().build(transport);

        let capabilities = client.initialize().await.unwrap();

        assert_eq!(client.session_state(), McpSessionState::Ready);
        assert!(client.is_ready());
        assert!(capabilities.tools.is_some());
        assert!(capabilities.resources.is_some());
    }

    #[tokio::test]
    async fn test_double_initialization() {
        let transport = MockTransportClient::new();
        let mut client = McpClientBuilder::new().build(transport);

        // First initialization should succeed
        client.initialize().await.unwrap();

        // Second initialization should fail
        let result = client.initialize().await;
        assert!(matches!(result.unwrap_err(), McpError::AlreadyConnected));
    }

    #[tokio::test]
    async fn test_client_close() {
        let transport = MockTransportClient::new();
        let mut client = McpClientBuilder::new().build(transport);

        client.initialize().await.unwrap();
        assert!(client.is_ready());

        client.close().await.unwrap();
        assert_eq!(client.session_state(), McpSessionState::NotInitialized);
        assert!(!client.is_ready());
    }
}
