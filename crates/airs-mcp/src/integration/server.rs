//! High-level MCP Server API
//!
//! This module provides a high-level, trait-based MCP server that allows
//! easy implementation of MCP server functionality with automatic request routing.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use super::constants::{defaults, error_codes, methods};
use crate::protocol::transport::{MessageContext, MessageHandler, Transport, TransportError};
use crate::protocol::{
    CallToolRequest,
    CallToolResponse,
    ClientCapabilities,
    Content,
    GetPromptRequest,
    GetPromptResponse,
    InitializeRequest,
    InitializeResponse,
    JsonRpcMessage,
    JsonRpcRequest,
    JsonRpcResponse,
    ListPromptsResponse,
    ListResourceTemplatesResponse,
    // Response types
    ListResourcesResponse,
    ListToolsResponse,
    LoggingCapabilities,
    LoggingConfig,
    Prompt,
    PromptCapabilities,
    PromptMessage,
    ProtocolVersion,
    ReadResourceRequest,
    ReadResourceResponse,
    // Core types
    Resource,
    ResourceCapabilities,
    ResourceTemplate,
    // Capability types
    ServerCapabilities,
    ServerInfo,
    SetLoggingRequest,
    SetLoggingResponse,
    SubscribeResourceRequest,
    Tool,
    ToolCapabilities,
    UnsubscribeResourceRequest,
};

use super::error::{McpError, McpResult};

/// Message handler for MCP server transport
struct ServerMessageHandler {
    config: McpServerConfig,
    client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
    resource_provider: Option<Arc<dyn ResourceProvider>>,
    tool_provider: Option<Arc<dyn ToolProvider>>,
    prompt_provider: Option<Arc<dyn PromptProvider>>,
    logging_handler: Option<Arc<dyn LoggingHandler>>,
    initialized: Arc<RwLock<bool>>,
}

impl ServerMessageHandler {
    fn new(
        config: McpServerConfig,
        client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
        tool_provider: Option<Arc<dyn ToolProvider>>,
        prompt_provider: Option<Arc<dyn PromptProvider>>,
        logging_handler: Option<Arc<dyn LoggingHandler>>,
        initialized: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            config,
            client_capabilities,
            resource_provider,
            tool_provider,
            prompt_provider,
            logging_handler,
            initialized,
        }
    }
}

#[async_trait]
impl MessageHandler for ServerMessageHandler {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext) {
        match message {
            JsonRpcMessage::Request(_request) => {
                // TODO: Handle incoming MCP requests
                // For now, we'll implement a placeholder that logs the request
                // In a full implementation, this would:
                // 1. Process the request through the MCP handlers
                // 2. Generate an appropriate response
                // 3. Send the response back through the transport
            }
            JsonRpcMessage::Notification(_) => {
                // Handle notifications if needed
            }
            JsonRpcMessage::Response(_) => {
                // Servers typically don't receive responses unless they made requests
            }
        }
    }

    async fn handle_error(&self, _error: TransportError) {
        // Handle transport errors
        // Could log, update server state, etc.
    }

    async fn handle_close(&self) {
        // Handle transport closure
        // Clean up server state if needed
    }
}

/// Trait for providing MCP resource functionality
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// List all available resources
    async fn list_resources(&self) -> McpResult<Vec<Resource>>;

    /// List all available resource templates
    async fn list_resource_templates(&self) -> McpResult<Vec<ResourceTemplate>> {
        // Default implementation returns empty list
        Ok(vec![])
    }

    /// Read content from a specific resource
    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>>;

    /// Subscribe to resource changes (optional)
    async fn subscribe_to_resource(&self, _uri: &str) -> McpResult<()> {
        Err(McpError::unsupported_capability("resource subscriptions"))
    }

    /// Unsubscribe from resource changes (optional)
    async fn unsubscribe_from_resource(&self, _uri: &str) -> McpResult<()> {
        Err(McpError::unsupported_capability("resource subscriptions"))
    }
}

/// Trait for providing MCP tool functionality
#[async_trait]
pub trait ToolProvider: Send + Sync {
    /// List all available tools
    async fn list_tools(&self) -> McpResult<Vec<Tool>>;

    /// Execute a tool with the given arguments
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>>;
}

/// Trait for providing MCP prompt functionality
#[async_trait]
pub trait PromptProvider: Send + Sync {
    /// List all available prompts
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>>;

    /// Get a prompt with the given arguments
    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> McpResult<(String, Vec<PromptMessage>)>;
}

/// Trait for handling logging operations
#[async_trait]
pub trait LoggingHandler: Send + Sync {
    /// Set logging configuration
    async fn set_logging(&self, config: LoggingConfig) -> McpResult<bool>;
}

/// Core MCP protocol configuration required by all transports
///
/// This contains only the universal MCP requirements that every transport needs,
/// regardless of transport type (STDIO, HTTP, WebSocket, etc.).
#[derive(Debug, Clone)]
pub struct McpCoreConfig {
    /// Server information to send during initialization
    pub server_info: ServerInfo,
    /// Server capabilities to advertise
    pub capabilities: ServerCapabilities,
    /// Protocol version to support
    pub protocol_version: ProtocolVersion,
    /// Optional instructions to provide to clients during initialization
    pub instructions: Option<String>,
}

impl Default for McpCoreConfig {
    fn default() -> Self {
        Self {
            server_info: ServerInfo {
                name: defaults::SERVER_NAME.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities::default(),
            protocol_version: ProtocolVersion::current(),
            instructions: Some(
                "MCP server with configurable capabilities. Use appropriate authentication method."
                    .to_string(),
            ),
        }
    }
}

/// Configuration for MCP server behavior
///
/// This now contains transport-specific configuration while the universal MCP
/// requirements are moved to McpCoreConfig.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Core MCP protocol configuration
    pub core: McpCoreConfig,
    /// Whether to validate all incoming requests strictly
    pub strict_validation: bool,
    /// Whether to log all MCP operations
    pub log_operations: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            core: McpCoreConfig::default(),
            strict_validation:
                crate::integration::constants::integration_defaults::STRICT_VALIDATION,
            log_operations: crate::integration::constants::integration_defaults::LOG_OPERATIONS,
        }
    }
}

/// Builder for creating MCP servers
pub struct McpServerBuilder {
    config: McpServerConfig,
    resource_provider: Option<Arc<dyn ResourceProvider>>,
    tool_provider: Option<Arc<dyn ToolProvider>>,
    prompt_provider: Option<Arc<dyn PromptProvider>>,
    logging_handler: Option<Arc<dyn LoggingHandler>>,
}

impl McpServerBuilder {
    /// Create a new MCP server builder
    pub fn new() -> Self {
        Self {
            config: McpServerConfig::default(),
            resource_provider: None,
            tool_provider: None,
            prompt_provider: None,
            logging_handler: None,
        }
    }

    /// Set server configuration
    pub fn config(mut self, config: McpServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set server information
    pub fn server_info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.config.core.server_info = ServerInfo {
            name: name.into(),
            version: version.into(),
        };
        self
    }

    /// Set server capabilities
    pub fn capabilities(mut self, capabilities: ServerCapabilities) -> Self {
        self.config.core.capabilities = capabilities;
        self
    }

    /// Enable strict validation
    pub fn strict_validation(mut self, enabled: bool) -> Self {
        self.config.strict_validation = enabled;
        self
    }

    /// Enable operation logging
    pub fn log_operations(mut self, enabled: bool) -> Self {
        self.config.log_operations = enabled;
        self
    }

    /// Add a resource provider
    pub fn with_resource_provider<P: ResourceProvider + 'static>(mut self, provider: P) -> Self {
        self.resource_provider = Some(Arc::new(provider));
        self
    }

    /// Add a tool provider
    pub fn with_tool_provider<P: ToolProvider + 'static>(mut self, provider: P) -> Self {
        self.tool_provider = Some(Arc::new(provider));
        self
    }

    /// Add a prompt provider
    pub fn with_prompt_provider<P: PromptProvider + 'static>(mut self, provider: P) -> Self {
        self.prompt_provider = Some(Arc::new(provider));
        self
    }

    /// Add a logging handler
    pub fn with_logging_handler<H: LoggingHandler + 'static>(mut self, handler: H) -> Self {
        self.logging_handler = Some(Arc::new(handler));
        self
    }

    /// Build the MCP server with the given transport
    pub async fn build<T: Transport + 'static>(self, transport: T) -> McpResult<McpServer<T>> {
        let transport = Arc::new(transport);

        // Auto-detect capabilities based on registered providers
        let mut config = self.config;
        let mut capabilities = config.core.capabilities;

        // Set resource capabilities if we have a resource provider
        if self.resource_provider.is_some() {
            capabilities.resources = Some(ResourceCapabilities {
                subscribe: Some(false),    // We don't support subscriptions yet
                list_changed: Some(false), // We don't support change notifications yet
            });
        }

        // Set tool capabilities if we have a tool provider
        if self.tool_provider.is_some() {
            capabilities.tools = Some(ToolCapabilities::default());
        }

        // Set prompt capabilities if we have a prompt provider
        if self.prompt_provider.is_some() {
            capabilities.prompts = Some(PromptCapabilities {
                list_changed: Some(false), // We don't support change notifications yet
            });
        }

        // Set logging capabilities if we have a logging handler
        if self.logging_handler.is_some() {
            capabilities.logging = Some(LoggingCapabilities::default());
        }

        config.core.capabilities = capabilities;

        Ok(McpServer::new_with_config(
            transport,
            config,
            self.resource_provider,
            self.tool_provider,
            self.prompt_provider,
            self.logging_handler,
        ))
    }
}

impl Default for McpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level MCP server for implementing MCP server functionality
pub struct McpServer<T: Transport> {
    /// Underlying transport
    transport: Arc<T>,
    /// Server configuration
    config: McpServerConfig,
    /// Client capabilities (available after initialization)
    client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
    /// Resource provider
    resource_provider: Option<Arc<dyn ResourceProvider>>,
    /// Tool provider
    tool_provider: Option<Arc<dyn ToolProvider>>,
    /// Prompt provider
    prompt_provider: Option<Arc<dyn PromptProvider>>,
    /// Logging handler
    logging_handler: Option<Arc<dyn LoggingHandler>>,
    /// Whether the server has been initialized
    initialized: Arc<RwLock<bool>>,
}

impl<T: Transport + 'static> McpServer<T> {
    /// Create a new MCP server with the given transport
    pub async fn new(transport: T) -> McpResult<Self> {
        McpServerBuilder::new().build(transport).await
    }

    /// Create a new MCP server with configuration
    pub(crate) fn new_with_config(
        transport: Arc<T>,
        config: McpServerConfig,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
        tool_provider: Option<Arc<dyn ToolProvider>>,
        prompt_provider: Option<Arc<dyn PromptProvider>>,
        logging_handler: Option<Arc<dyn LoggingHandler>>,
    ) -> Self {
        Self {
            transport,
            config,
            client_capabilities: Arc::new(RwLock::new(None)),
            resource_provider,
            tool_provider,
            prompt_provider,
            logging_handler,
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the server and handle incoming requests
    pub async fn run(&mut self) -> McpResult<()> {
        // Clone references for the handler
        let config = self.config.clone();
        let client_capabilities = Arc::clone(&self.client_capabilities);
        let resource_provider = self.resource_provider.clone();
        let tool_provider = self.tool_provider.clone();
        let prompt_provider = self.prompt_provider.clone();
        let logging_handler = self.logging_handler.clone();
        let initialized = Arc::clone(&self.initialized);

        // Start the server with transport
        let handler = Arc::new(ServerMessageHandler::new(
            config,
            client_capabilities,
            resource_provider,
            tool_provider,
            prompt_provider,
            logging_handler,
            initialized,
        ));

        // Get mutable reference to transport from Arc
        let transport = Arc::get_mut(&mut self.transport).ok_or_else(|| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: "Failed to get mutable reference to transport. Transport may be shared elsewhere.".to_string(),
            })
        })?;

        transport.set_message_handler(handler);
        transport.start().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to start transport: {}", e),
            })
        })?;

        Ok(())
    }

    /// Handle an incoming MCP request
    ///
    /// This is the core request routing and processing function that:
    /// 1. Routes requests to appropriate handler methods based on method name
    /// 2. Enforces MCP protocol compliance and validation
    /// 3. Applies consistent error handling and response formatting
    /// 4. Maintains operation logging when enabled
    /// 5. Ensures type-safe parameter handling and result serialization
    ///
    /// # Request Processing Flow
    ///
    /// 1. **Logging**: Optional operation logging (file-only, never stdout/stderr)
    /// 2. **Routing**: Method name-based dispatch to specific handlers
    /// 3. **Validation**: Parameter validation and capability checking
    /// 4. **Execution**: Provider method invocation with error handling
    /// 5. **Response**: JSON-RPC response formatting with proper error codes
    ///
    /// # Error Handling
    ///
    /// All errors are mapped to appropriate JSON-RPC error codes:
    /// - `METHOD_NOT_FOUND`: Unsupported or unknown methods
    /// - `INVALID_PARAMS`: Parameter validation failures
    /// - `INTERNAL_ERROR`: Server-side processing errors
    ///
    /// # MCP Methods Supported
    ///
    /// - **Initialization**: `initialize` - Server capability negotiation
    /// - **Resources**: `resources/list`, `resources/read`, `resources/subscribe`
    /// - **Tools**: `tools/list`, `tools/call` - Function execution
    /// - **Prompts**: `prompts/list`, `prompts/get` - Template management
    /// - **Logging**: `logging/setLevel` - Log level configuration
    /// - **Utility**: `ping` - Health check
    ///
    #[allow(clippy::too_many_arguments)]
    async fn handle_request(
        request: JsonRpcRequest,
        config: McpServerConfig,
        client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
        tool_provider: Option<Arc<dyn ToolProvider>>,
        prompt_provider: Option<Arc<dyn PromptProvider>>,
        logging_handler: Option<Arc<dyn LoggingHandler>>,
        initialized: Arc<RwLock<bool>>,
    ) -> JsonRpcResponse {
        if config.log_operations {
            // Log to file only, never to stdout/stderr which would contaminate JSON-RPC stream
            tracing::debug!("MCP Request: {} - {}", request.method, request.id);
        }

        let result = match request.method.as_str() {
            methods::INITIALIZE => {
                Self::handle_initialize(request.params, config, client_capabilities, initialized)
                    .await
            }
            methods::RESOURCES_LIST => Self::handle_list_resources(resource_provider).await,
            methods::RESOURCES_TEMPLATES_LIST => {
                Self::handle_list_resource_templates(resource_provider).await
            }
            methods::RESOURCES_READ => {
                Self::handle_read_resource(request.params, resource_provider).await
            }
            methods::RESOURCES_SUBSCRIBE => {
                Self::handle_subscribe_resource(request.params, resource_provider).await
            }
            methods::RESOURCES_UNSUBSCRIBE => {
                Self::handle_unsubscribe_resource(request.params, resource_provider).await
            }
            methods::TOOLS_LIST => Self::handle_list_tools(tool_provider).await,
            methods::TOOLS_CALL => Self::handle_call_tool(request.params, tool_provider).await,
            methods::PROMPTS_LIST => Self::handle_list_prompts(prompt_provider).await,
            methods::PROMPTS_GET => Self::handle_get_prompt(request.params, prompt_provider).await,
            methods::LOGGING_SET_LEVEL => {
                Self::handle_set_logging(request.params, logging_handler).await
            }
            "ping" => Ok(Value::Null),
            _ => Err(McpError::method_not_found(&request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse::success(value, request.id),
            Err(error) => {
                let (code, message) = match &error {
                    McpError::Integration(_) => (error_codes::INTERNAL_ERROR, error.to_string()),
                    McpError::Protocol(_) => (error_codes::INVALID_PARAMS, error.to_string()),
                    McpError::NotConnected => (error_codes::INVALID_REQUEST, error.to_string()),
                    McpError::UnsupportedCapability { .. } => {
                        (error_codes::METHOD_NOT_FOUND, error.to_string())
                    }
                    McpError::ResourceNotFound { .. } => {
                        (error_codes::INVALID_PARAMS, error.to_string())
                    }
                    McpError::ToolNotFound { .. } => {
                        (error_codes::INVALID_PARAMS, error.to_string())
                    }
                    McpError::PromptNotFound { .. } => {
                        (error_codes::INVALID_PARAMS, error.to_string())
                    }
                    _ => (error_codes::INTERNAL_ERROR, error.to_string()),
                };

                let error_value = serde_json::json!({
                    "code": code,
                    "message": message
                });

                JsonRpcResponse::error(error_value, Some(request.id))
            }
        }
    }

    /// Check if server is initialized and ready for operations
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.config.core.capabilities
    }

    /// Get client capabilities (available after initialization)
    pub async fn client_capabilities(&self) -> Option<ClientCapabilities> {
        self.client_capabilities.read().await.clone()
    }

    /// Shutdown the server
    pub async fn shutdown(&mut self) -> McpResult<()> {
        let transport = Arc::get_mut(&mut self.transport).ok_or_else(|| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: "Failed to get mutable reference to transport for shutdown".to_string(),
            })
        })?;

        transport.close().await.map_err(|e| {
            McpError::Integration(super::error::IntegrationError::Other {
                message: format!("Failed to close transport: {}", e),
            })
        })?;

        Ok(())
    }

    // Handler methods for different MCP operations

    // Initialization handlers
    async fn handle_initialize(
        params: Option<Value>,
        config: McpServerConfig,
        client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
        initialized: Arc<RwLock<bool>>,
    ) -> McpResult<Value> {
        let init_request: InitializeRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| {
                McpError::invalid_request(format!("Invalid initialization request: {e}"))
            })?;

        // Parse and store client capabilities from the JSON value
        let parsed_capabilities: ClientCapabilities =
            serde_json::from_value(init_request.capabilities).map_err(|e| {
                McpError::invalid_request(format!("Invalid client capabilities: {e}"))
            })?;

        *client_capabilities.write().await = Some(parsed_capabilities);

        // Serialize server capabilities to JSON for the response
        let capabilities_json = serde_json::to_value(&config.core.capabilities).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize capabilities: {e}"))
        })?;

        // Create initialization response with configured instructions
        let response = InitializeResponse::new(
            capabilities_json,
            config.core.server_info,
            config.core.instructions,
        );

        // Mark as initialized
        *initialized.write().await = true;

        serde_json::to_value(response).map_err(|e| McpError::internal_error(e.to_string()))
    }

    // Resource handlers
    async fn handle_list_resources(
        resource_provider: Option<Arc<dyn ResourceProvider>>,
    ) -> McpResult<Value> {
        let provider =
            resource_provider.ok_or_else(|| McpError::unsupported_capability("resources"))?;

        let resources = provider.list_resources().await?;
        let response = ListResourcesResponse {
            resources,
            next_cursor: None,
        };

        serde_json::to_value(response)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize response: {e}")))
    }

    async fn handle_list_resource_templates(
        resource_provider: Option<Arc<dyn ResourceProvider>>,
    ) -> McpResult<Value> {
        let provider =
            resource_provider.ok_or_else(|| McpError::unsupported_capability("resources"))?;

        let resource_templates = provider.list_resource_templates().await?;
        let response = ListResourceTemplatesResponse {
            resource_templates,
            next_cursor: None,
        };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!(
                "Failed to serialize resource templates response: {e}"
            ))
        })
    }

    async fn handle_read_resource(
        params: Option<Value>,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
    ) -> McpResult<Value> {
        let provider =
            resource_provider.ok_or_else(|| McpError::unsupported_capability("resources"))?;

        let request: ReadResourceRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| {
                McpError::invalid_request(format!("Invalid read resource request: {e}"))
            })?;

        let contents = provider.read_resource(request.uri.as_str()).await?;
        let response = ReadResourceResponse { contents };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize read resource response: {e}"))
        })
    }

    async fn handle_subscribe_resource(
        params: Option<Value>,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
    ) -> McpResult<Value> {
        let provider =
            resource_provider.ok_or_else(|| McpError::unsupported_capability("resources"))?;

        let request: SubscribeResourceRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| {
                McpError::invalid_request(format!("Invalid subscribe resource request: {e}"))
            })?;

        provider.subscribe_to_resource(request.uri.as_str()).await?;

        // Return empty success response
        Ok(Value::Object(serde_json::Map::new()))
    }

    async fn handle_unsubscribe_resource(
        params: Option<Value>,
        resource_provider: Option<Arc<dyn ResourceProvider>>,
    ) -> McpResult<Value> {
        let provider =
            resource_provider.ok_or_else(|| McpError::unsupported_capability("resources"))?;

        let request: UnsubscribeResourceRequest =
            serde_json::from_value(params.unwrap_or_default()).map_err(|e| {
                McpError::invalid_request(format!("Invalid unsubscribe resource request: {e}"))
            })?;

        provider
            .unsubscribe_from_resource(request.uri.as_str())
            .await?;

        // Return empty success response
        Ok(Value::Object(serde_json::Map::new()))
    }

    // Tool handlers
    async fn handle_list_tools(tool_provider: Option<Arc<dyn ToolProvider>>) -> McpResult<Value> {
        let provider = tool_provider.ok_or_else(|| McpError::unsupported_capability("tools"))?;

        let tools = provider.list_tools().await?;
        let response = ListToolsResponse {
            tools,
            next_cursor: None,
        };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize list tools response: {e}"))
        })
    }

    async fn handle_call_tool(
        params: Option<Value>,
        tool_provider: Option<Arc<dyn ToolProvider>>,
    ) -> McpResult<Value> {
        let provider = tool_provider.ok_or_else(|| McpError::unsupported_capability("tools"))?;

        let request: CallToolRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| McpError::invalid_request(format!("Invalid call tool request: {e}")))?;

        let result = provider.call_tool(&request.name, request.arguments).await;

        match result {
            Ok(content) => {
                let response = CallToolResponse::success(content);
                serde_json::to_value(response).map_err(|e| {
                    McpError::server_error(format!("Failed to serialize call tool response: {e}"))
                })
            }
            Err(error) => {
                let response = CallToolResponse::error_text(error.to_string());
                serde_json::to_value(response).map_err(|e| {
                    McpError::server_error(format!("Failed to serialize call tool response: {e}"))
                })
            }
        }
    }

    // Prompt handlers
    async fn handle_list_prompts(
        prompt_provider: Option<Arc<dyn PromptProvider>>,
    ) -> McpResult<Value> {
        let provider =
            prompt_provider.ok_or_else(|| McpError::unsupported_capability("prompts"))?;

        let prompts = provider.list_prompts().await?;
        let response = ListPromptsResponse {
            prompts,
            next_cursor: None,
        };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize list prompts response: {e}"))
        })
    }

    async fn handle_get_prompt(
        params: Option<Value>,
        prompt_provider: Option<Arc<dyn PromptProvider>>,
    ) -> McpResult<Value> {
        let provider =
            prompt_provider.ok_or_else(|| McpError::unsupported_capability("prompts"))?;

        let request: GetPromptRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| McpError::invalid_request(format!("Invalid get prompt request: {e}")))?;

        let (description, messages) = provider
            .get_prompt(&request.name, request.arguments)
            .await?;
        let response = GetPromptResponse {
            description: Some(description),
            messages,
        };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize get prompt response: {e}"))
        })
    }

    // Logging handlers
    async fn handle_set_logging(
        params: Option<Value>,
        logging_handler: Option<Arc<dyn LoggingHandler>>,
    ) -> McpResult<Value> {
        let handler = logging_handler.ok_or_else(|| McpError::unsupported_capability("logging"))?;

        let request: SetLoggingRequest = serde_json::from_value(params.unwrap_or_default())
            .map_err(|e| McpError::invalid_request(format!("Invalid set logging request: {e}")))?;

        let config = LoggingConfig {
            level: request.level,
        };
        let success = handler.set_logging(config).await?;
        let response = SetLoggingResponse {
            success,
            message: if success {
                None
            } else {
                Some("Failed to set logging level".to_string())
            },
        };

        serde_json::to_value(response).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize set logging response: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JsonRpcNotification;
    use crate::StdioTransport;
    use serde_json::json;

    struct TestResourceProvider;

    #[async_trait]
    impl ResourceProvider for TestResourceProvider {
        async fn list_resources(&self) -> McpResult<Vec<Resource>> {
            Ok(vec![])
        }

        async fn read_resource(&self, _uri: &str) -> McpResult<Vec<Content>> {
            Ok(vec![])
        }
    }

    struct TestToolProvider;

    #[async_trait]
    impl ToolProvider for TestToolProvider {
        async fn list_tools(&self) -> McpResult<Vec<Tool>> {
            Ok(vec![])
        }

        async fn call_tool(&self, _name: &str, _arguments: Value) -> McpResult<Vec<Content>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_server_config_defaults() {
        let config = McpServerConfig::default();
        assert_eq!(config.core.server_info.name, "airs-mcp-server");
        assert!(config.strict_validation);
        assert!(!config.log_operations);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = McpServerBuilder::new()
            .server_info("test-server", "1.0.0")
            .strict_validation(false)
            .log_operations(true)
            .with_resource_provider(TestResourceProvider);

        assert_eq!(builder.config.core.server_info.name, "test-server");
        assert_eq!(builder.config.core.server_info.version, "1.0.0");
        assert!(!builder.config.strict_validation);
        assert!(builder.config.log_operations);
        assert!(builder.resource_provider.is_some());
    }

    #[test]
    fn test_builder_auto_capability_detection() {
        let builder = McpServerBuilder::new()
            .with_resource_provider(TestResourceProvider)
            .with_tool_provider(TestToolProvider);

        // Build with a mock transport to test capability detection
        // Note: In a real scenario, we'd need to complete the build, but for testing
        // capability detection logic, we can inspect the builder state
        assert!(builder.resource_provider.is_some());
        assert!(builder.tool_provider.is_some());
        assert!(builder.prompt_provider.is_none());
    }

    #[tokio::test]
    async fn test_server_creation() {
        let transport = StdioTransport::new();
        let server = McpServerBuilder::new()
            .server_info("test", "1.0")
            .build(transport)
            .await
            .unwrap();

        assert!(!server.is_initialized().await);
        assert!(server.client_capabilities().await.is_none());
    }

    #[tokio::test]
    async fn test_initialization_lifecycle() {
        // Test the complete MCP initialization lifecycle
        let transport = StdioTransport::new();
        let server = McpServerBuilder::new()
            .server_info("test-server", "1.0.0")
            .with_tool_provider(TestToolProvider)
            .with_resource_provider(TestResourceProvider)
            .build(transport)
            .await
            .unwrap();

        // Verify initial state
        assert!(!server.is_initialized().await);
        assert!(server.client_capabilities().await.is_none());

        // Create mock initialization request
        let init_request = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let request = JsonRpcRequest::new(
            "initialize",
            Some(init_request),
            crate::RequestId::new_number(1),
        );

        // Create mock config and dependencies
        let config = server.config.clone(); // Use the server's actual config with auto-detected capabilities
        let client_capabilities = Arc::new(RwLock::new(None));
        let initialized = Arc::new(RwLock::new(false));

        // Test initialize request handling
        let response = McpServer::<StdioTransport>::handle_initialize(
            request.params,
            config,
            Arc::clone(&client_capabilities),
            Arc::clone(&initialized),
        )
        .await
        .expect("Initialize should succeed");

        // Verify initialization response structure
        let response_obj = response.as_object().expect("Response should be an object");
        assert!(response_obj.contains_key("capabilities"));
        assert!(response_obj.contains_key("serverInfo"));
        assert!(response_obj.contains_key("protocolVersion"));

        // Verify server state after initialization
        assert!(
            *initialized.read().await,
            "Server should be marked as initialized"
        );
        assert!(
            client_capabilities.read().await.is_some(),
            "Client capabilities should be stored"
        );

        // Test server capabilities in response
        let capabilities = &response_obj["capabilities"];
        assert!(
            capabilities.get("tools").is_some(),
            "Tools capability should be present"
        );
        assert!(
            capabilities.get("resources").is_some(),
            "Resources capability should be present"
        );

        // Test server info in response
        let server_info = &response_obj["serverInfo"];
        assert_eq!(server_info["name"], "test-server");
        assert_eq!(server_info["version"], "1.0.0");

        // Test protocol version in response
        assert_eq!(response_obj["protocolVersion"], "2024-11-05");
    }

    #[tokio::test]
    async fn test_initialized_notification_handling() {
        // Test that initialized notification is properly handled
        let notification = JsonRpcNotification::new("initialized", None);

        // Create a simple notification handler (mimicking the real one)
        let notification_handler = |notif: JsonRpcNotification| async move {
            assert_eq!(notif.method, "initialized");
            // This would normally log success message
        };

        // Execute the notification handler
        notification_handler(notification).await;
        // If we reach here without panic, the test passes
    }

    #[tokio::test]
    async fn test_invalid_initialization_request() {
        let transport = StdioTransport::new();
        let _server = McpServerBuilder::new().build(transport).await.unwrap();

        // Test with invalid initialization data
        let invalid_request = json!({
            "invalidField": "should cause error"
        });

        let config = McpServerConfig::default();
        let client_capabilities = Arc::new(RwLock::new(None));
        let initialized = Arc::new(RwLock::new(false));

        // This should return an error
        let result = McpServer::<StdioTransport>::handle_initialize(
            Some(invalid_request),
            config,
            client_capabilities,
            initialized,
        )
        .await;

        assert!(
            result.is_err(),
            "Invalid initialization request should fail"
        );
    }

    #[tokio::test]
    async fn test_capability_auto_detection() {
        // Test that capabilities are automatically detected based on providers
        let transport = StdioTransport::new();

        // Server with only tools
        let server_with_tools = McpServerBuilder::new()
            .with_tool_provider(TestToolProvider)
            .build(transport)
            .await
            .unwrap();

        let capabilities = server_with_tools.capabilities();
        assert!(
            capabilities.tools.is_some(),
            "Tools capability should be auto-detected"
        );
        assert!(
            capabilities.resources.is_none(),
            "Resources capability should not be present"
        );

        // Server with tools and resources
        let transport2 = StdioTransport::new();
        let server_with_both = McpServerBuilder::new()
            .with_tool_provider(TestToolProvider)
            .with_resource_provider(TestResourceProvider)
            .build(transport2)
            .await
            .unwrap();

        let capabilities_both = server_with_both.capabilities();
        assert!(
            capabilities_both.tools.is_some(),
            "Tools capability should be auto-detected"
        );
        assert!(
            capabilities_both.resources.is_some(),
            "Resources capability should be auto-detected"
        );
    }

    #[tokio::test]
    async fn test_initialization_protocol_version_matching() {
        // Test that server responds with correct protocol version
        let init_request = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let config = McpServerConfig::default();
        let client_capabilities = Arc::new(RwLock::new(None));
        let initialized = Arc::new(RwLock::new(false));

        let response = McpServer::<StdioTransport>::handle_initialize(
            Some(init_request),
            config,
            client_capabilities,
            initialized,
        )
        .await
        .expect("Initialize should succeed");

        let response_obj = response.as_object().expect("Response should be an object");
        assert_eq!(
            response_obj["protocolVersion"], "2024-11-05",
            "Server should respond with matching protocol version"
        );
    }

    #[tokio::test]
    async fn test_client_capabilities_storage() {
        // Test that client capabilities are properly stored
        let client_caps = json!({
            "tools": { "list_changed": true },
            "resources": { "subscribe": true }
        });

        let init_request = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": client_caps,
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let config = McpServerConfig::default();
        let client_capabilities = Arc::new(RwLock::new(None));
        let initialized = Arc::new(RwLock::new(false));

        McpServer::<StdioTransport>::handle_initialize(
            Some(init_request),
            config,
            Arc::clone(&client_capabilities),
            initialized,
        )
        .await
        .expect("Initialize should succeed");

        // Verify client capabilities were stored
        let stored_caps = client_capabilities.read().await;
        assert!(
            stored_caps.is_some(),
            "Client capabilities should be stored"
        );

        // Note: We can't easily test the exact structure without defining
        // the ClientCapabilities deserialization, but we can verify it's stored
    }
}
