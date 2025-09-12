//! Simple ApiKey-based MCP Server Example
//!
//! Demonstrates a production-ready MCP server with ApiKey authentication - much simpler than OAuth2
//! while still providing solid security for many use cases. Perfect for:
//! - Internal services and APIs
//! - Machine-to-machine communication
//! - Microservices authentication
//! - Development and testing environments
//!
//! This example showcases the modern HttpTransportBuilder with API key authentication,
//! demonstrating the Generic MessageHandler<HttpContext> pattern from TASK-028.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use serde_json::{json, Value};

// Layer 3: Internal module imports
use airs_mcp::{
    authentication::{
        AuthContext, AuthMethod,
        strategies::apikey::{ApiKeyAuthData, ApiKeySource, InMemoryApiKeyValidator},
    },
    integration::McpError,
    protocol::types::{
        Content, MimeType, Prompt, PromptArgument, PromptMessage, Resource, Tool, Uri,
    },
    protocol::{
        JsonRpcMessage, JsonRpcMessageTrait, JsonRpcRequest, JsonRpcResponse, MessageContext,
        MessageHandler, TransportError,
    },
    providers::{PromptProvider, ResourceProvider, ToolProvider},
    transport::adapters::http::{HttpContext, HttpTransportBuilder},
};

// Professional logging imports
use tracing::{error, info, instrument, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// Simple file system resource provider (API key server version)
#[derive(Debug)]
struct ApiKeyResourceProvider {
    temp_dir: std::path::PathBuf,
}

impl ApiKeyResourceProvider {
    fn new(temp_dir: std::path::PathBuf) -> Self {
        Self { temp_dir }
    }
}

#[async_trait::async_trait]
impl ResourceProvider for ApiKeyResourceProvider {
    #[instrument(level = "debug")]
    async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        info!("Listing available resources for API key server");

        let resources = vec![
            Resource {
                uri: Uri::new("file:///welcome.txt").unwrap(),
                name: "Welcome File".to_string(),
                description: Some("Server introduction and capabilities".to_string()),
                mime_type: Some(MimeType::new("text/plain").unwrap()),
            },
            Resource {
                uri: Uri::new("file:///config.json").unwrap(),
                name: "Server Configuration".to_string(),
                description: Some("API key server configuration".to_string()),
                mime_type: Some(MimeType::new("application/json").unwrap()),
            },
            Resource {
                uri: Uri::new("file:///sample.md").unwrap(),
                name: "Documentation".to_string(),
                description: Some("Server documentation and usage examples".to_string()),
                mime_type: Some(MimeType::new("text/markdown").unwrap()),
            },
            Resource {
                uri: Uri::new("file:///api-keys.yaml").unwrap(),
                name: "API Keys Info".to_string(),
                description: Some("API key usage and configuration information".to_string()),
                mime_type: Some(MimeType::new("application/yaml").unwrap()),
            },
        ];

        info!(
            resource_count = resources.len(),
            "Resources listed successfully for API key server"
        );
        Ok(resources)
    }

    #[instrument(level = "debug", fields(uri = %uri))]
    async fn read_resource(&self, uri: &str) -> Result<Vec<Content>, McpError> {
        info!(uri = %uri, "Reading resource from API key server");

        let file_path = match uri {
            "file:///welcome.txt" => self.temp_dir.join("welcome.txt"),
            "file:///config.json" => self.temp_dir.join("config.json"),
            "file:///sample.md" => self.temp_dir.join("sample.md"),
            "file:///api-keys.yaml" => self.temp_dir.join("api-keys.yaml"),
            _ => {
                warn!(uri = %uri, "Resource not found");
                return Err(McpError::resource_not_found(uri));
            }
        };

        let content_text = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| {
                error!(uri = %uri, error = %e, "Failed to read file");
                McpError::internal_error(format!("Failed to read file: {e}"))
            })?;

        let content = vec![Content::text_with_uri(&content_text, uri)
            .map_err(|e| McpError::internal_error(format!("Failed to create content: {e}")))?];
        
        info!(uri = %uri, content_size = content_text.len(), "Resource read successfully");
        Ok(content)
    }
}

/// Mathematical tool provider (API key server version)
#[derive(Debug)]
struct ApiKeyMathToolProvider;

#[async_trait::async_trait]
impl ToolProvider for ApiKeyMathToolProvider {
    #[instrument(level = "debug")]
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        info!("Listing available tools for API key server");

        let tools = vec![
            Tool {
                name: "calculate".to_string(),
                description: Some("Perform mathematical calculations".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "expression": {"type": "string", "description": "Mathematical expression to evaluate"},
                        "precision": {"type": "integer", "description": "Number of decimal places", "default": 2}
                    },
                    "required": ["expression"]
                }),
            },
            Tool {
                name: "convert_units".to_string(),
                description: Some("Convert between different units".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "value": {"type": "number", "description": "Value to convert"},
                        "from_unit": {"type": "string", "description": "Source unit"},
                        "to_unit": {"type": "string", "description": "Target unit"}
                    },
                    "required": ["value", "from_unit", "to_unit"]
                }),
            },
        ];

        info!(tool_count = tools.len(), "Tools listed successfully for API key server");
        Ok(tools)
    }

    #[instrument(level = "debug", fields(tool_name = %name))]
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<Content>, McpError> {
        info!(tool_name = %name, arguments = %arguments, "Executing tool in API key server");

        let result = match name {
            "calculate" => {
                let expression = arguments.get("expression").and_then(|v| v.as_str()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'expression'");
                    McpError::invalid_request("Missing or invalid parameter 'expression'")
                })?;

                let precision = arguments.get("precision").and_then(|v| v.as_u64()).unwrap_or(2);

                // Simple expression evaluation (for demo purposes)
                let result_value = match expression {
                    expr if expr.contains('+') => {
                        let parts: Vec<&str> = expr.split('+').collect();
                        if parts.len() == 2 {
                            let a: f64 = parts[0].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            let b: f64 = parts[1].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            a + b
                        } else {
                            return Err(McpError::invalid_request("Complex expressions not supported in demo"));
                        }
                    },
                    expr if expr.contains('-') => {
                        let parts: Vec<&str> = expr.split('-').collect();
                        if parts.len() == 2 {
                            let a: f64 = parts[0].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            let b: f64 = parts[1].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            a - b
                        } else {
                            return Err(McpError::invalid_request("Complex expressions not supported in demo"));
                        }
                    },
                    expr if expr.contains('*') => {
                        let parts: Vec<&str> = expr.split('*').collect();
                        if parts.len() == 2 {
                            let a: f64 = parts[0].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            let b: f64 = parts[1].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            a * b
                        } else {
                            return Err(McpError::invalid_request("Complex expressions not supported in demo"));
                        }
                    },
                    expr if expr.contains('/') => {
                        let parts: Vec<&str> = expr.split('/').collect();
                        if parts.len() == 2 {
                            let a: f64 = parts[0].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            let b: f64 = parts[1].trim().parse().map_err(|_| McpError::invalid_request("Invalid number in expression"))?;
                            if b == 0.0 {
                                return Err(McpError::invalid_request("Division by zero"));
                            }
                            a / b
                        } else {
                            return Err(McpError::invalid_request("Complex expressions not supported in demo"));
                        }
                    },
                    _ => return Err(McpError::invalid_request("Unsupported expression format")),
                };

                info!(tool_name = %name, expression = %expression, result = %result_value, "Calculation completed");

                json!({
                    "result": format!("{:.prec$}", result_value, prec = precision as usize),
                    "expression": expression,
                    "precision": precision
                })
            }
            "convert_units" => {
                let value = arguments.get("value").and_then(|v| v.as_f64()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'value'");
                    McpError::invalid_request("Missing or invalid parameter 'value'")
                })?;

                let from_unit = arguments.get("from_unit").and_then(|v| v.as_str()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'from_unit'");
                    McpError::invalid_request("Missing or invalid parameter 'from_unit'")
                })?;

                let to_unit = arguments.get("to_unit").and_then(|v| v.as_str()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'to_unit'");
                    McpError::invalid_request("Missing or invalid parameter 'to_unit'")
                })?;

                // Simple unit conversions (for demo purposes)
                let converted_value = match (from_unit, to_unit) {
                    ("m", "ft") => value * 3.28084,
                    ("ft", "m") => value / 3.28084,
                    ("kg", "lb") => value * 2.20462,
                    ("lb", "kg") => value / 2.20462,
                    ("c", "f") => (value * 9.0 / 5.0) + 32.0,
                    ("f", "c") => (value - 32.0) * 5.0 / 9.0,
                    _ => return Err(McpError::invalid_request("Unsupported unit conversion")),
                };

                info!(tool_name = %name, value = %value, from_unit = %from_unit, to_unit = %to_unit, result = %converted_value, "Unit conversion completed");

                json!({
                    "converted_value": converted_value,
                    "original_value": value,
                    "from_unit": from_unit,
                    "to_unit": to_unit
                })
            }
            _ => return Err(McpError::tool_not_found(name)),
        };

        Ok(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap(),
        )])
    }
}

/// Code review prompt provider (API key server version)
#[derive(Debug)]
struct ApiKeyPromptProvider;

#[async_trait::async_trait]
impl PromptProvider for ApiKeyPromptProvider {
    async fn list_prompts(&self) -> Result<Vec<Prompt>, McpError> {
        Ok(vec![
            Prompt {
                name: "code_review".to_string(),
                title: Some("Code Review".to_string()),
                description: Some("Generate a comprehensive code review prompt".to_string()),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to review")),
                    PromptArgument::optional("focus", Some("Review focus area")),
                ],
            },
            Prompt {
                name: "api_documentation".to_string(),
                title: Some("API Documentation".to_string()),
                description: Some("Generate API documentation prompt".to_string()),
                arguments: vec![
                    PromptArgument::required("endpoint", Some("API endpoint to document")),
                    PromptArgument::optional("format", Some("Documentation format (markdown, openapi)")),
                ],
            },
        ])
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<(String, Vec<PromptMessage>), McpError> {
        let (description, messages) = match name {
            "code_review" => {
                let language = arguments
                    .get("language")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                let code = arguments
                    .get("code")
                    .cloned()
                    .unwrap_or_else(|| "".to_string());
                let focus = arguments
                    .get("focus")
                    .cloned()
                    .unwrap_or_else(|| "general review".to_string());

                let prompt_text = format!(
                    "Please review the following {language} code with focus on {focus}:\n\n```{language}\n{code}\n```\n\nProvide feedback on:\n- Code quality and best practices\n- Security considerations\n- Performance optimization\n- API design patterns\n- Error handling\n- Documentation quality"
                );

                (
                    "API Key server code review template".to_string(),
                    vec![PromptMessage::user(Content::text(prompt_text))],
                )
            }
            "api_documentation" => {
                let endpoint = arguments
                    .get("endpoint")
                    .cloned()
                    .unwrap_or_else(|| "/api/unknown".to_string());
                let format = arguments
                    .get("format")
                    .cloned()
                    .unwrap_or_else(|| "markdown".to_string());

                let prompt_text = format!(
                    "Please generate comprehensive API documentation for the '{endpoint}' endpoint in {format} format. Include:\n- Endpoint description and purpose\n- Request/response schemas\n- Authentication requirements\n- Error codes and responses\n- Usage examples\n- Rate limiting information"
                );

                (
                    "API documentation generation template".to_string(),
                    vec![PromptMessage::user(Content::text(prompt_text))],
                )
            }
            _ => return Err(McpError::prompt_not_found(name)),
        };

        Ok((description, messages))
    }
}

/// Initialize internal logging with graceful degradation
/// - First tries file-based logging for debugging and operations
/// - Falls back to no-op logging if file system access is denied
/// - Never outputs to stdout/stderr to avoid JSON-RPC contamination
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Try file-based logging first
    let file_layer_result = std::panic::catch_unwind(|| {
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            "/tmp/apikey-mcp-server",
            "apikey-mcp-server.log",
        );

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = layer()
            .with_writer(non_blocking)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .json();

        tracing_subscriber::registry()
            .with(file_layer.with_filter(EnvFilter::new("debug")))
            .init();

        // Intentionally leak the guard to keep logging alive for the process lifetime
        std::mem::forget(_guard);
    });

    match file_layer_result {
        Ok(_) => {
            // File logging successful
            info!("🚀 AIRS MCP API Key Server starting with file-based logging");
            Ok(())
        }
        Err(_) => {
            // File logging failed - use no-op logging but continue operation
            tracing_subscriber::registry()
                .with(EnvFilter::new("off"))
                .init();

            // Cannot log the failure since we have no logging, but continue silently
            Ok(())
        }
    }
}

/// API Key MCP Handler - Implements MessageHandler<HttpContext> for HTTP transport
///
/// This handler provides API key authentication for HTTP MCP requests while preserving
/// all business logic in the provider implementations. It demonstrates the modern
/// Generic MessageHandler<HttpContext> pattern from TASK-028.
#[derive(Debug)]
struct ApiKeyMcpHandler {
    resource_provider: ApiKeyResourceProvider,
    tool_provider: ApiKeyMathToolProvider,
    prompt_provider: ApiKeyPromptProvider,
    api_key_validator: InMemoryApiKeyValidator,
}

impl ApiKeyMcpHandler {
    /// Create a new handler with providers and API key validation
    pub fn new(
        resource_provider: ApiKeyResourceProvider,
        tool_provider: ApiKeyMathToolProvider,
        prompt_provider: ApiKeyPromptProvider,
        api_key_validator: InMemoryApiKeyValidator,
    ) -> Self {
        Self {
            resource_provider,
            tool_provider,
            prompt_provider,
            api_key_validator,
        }
    }

    /// Validate API key from HTTP context
    fn validate_api_key(&self, context: &HttpContext) -> Result<AuthContext, McpError> {
        // Check X-API-Key header
        if let Some(api_key) = context.headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
            if let Some(auth_context) = self.api_key_validator.validate_key(api_key) {
                info!(key_source = "x-api-key", "API key validated successfully");
                return Ok(auth_context);
            }
        }

        // Check Authorization Bearer header
        if let Some(auth_header) = context.headers.get("authorization").and_then(|h| h.to_str().ok()) {
            if let Some(bearer_token) = auth_header.strip_prefix("Bearer ") {
                if let Some(auth_context) = self.api_key_validator.validate_key(bearer_token) {
                    info!(key_source = "authorization_bearer", "API key validated successfully");
                    return Ok(auth_context);
                }
            }
        }

        warn!("API key validation failed - no valid key found");
        Err(McpError::unauthorized("Valid API key required"))
    }

    /// Handle MCP protocol requests with API key authentication
    async fn handle_mcp_request(&self, request: JsonRpcRequest, context: &HttpContext) -> JsonRpcResponse {
        // Validate API key for all MCP requests
        if let Err(auth_error) = self.validate_api_key(context) {
            error!(error = %auth_error, "Authentication failed for MCP request");
            let error_data = json!({
                "code": -32001,
                "message": format!("Authentication failed: {}", auth_error)
            });
            return JsonRpcResponse::error(error_data, Some(request.id));
        }

        match request.method.as_str() {
            "initialize" => {
                info!("Handling initialize request for API key server");
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "resources": {
                            "subscribe": false,
                            "listChanged": false
                        },
                        "tools": {
                            "listChanged": false
                        },
                        "prompts": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "apikey-mcp-server",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                JsonRpcResponse::success(result, request.id)
            }
            "resources/list" => {
                info!("Handling resources/list request");
                match self.resource_provider.list_resources().await {
                    Ok(resources) => {
                        let result = json!({ "resources": resources });
                        JsonRpcResponse::success(result, request.id)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to list resources");
                        let error_data = json!({
                            "code": -32603,
                            "message": format!("Failed to list resources: {}", e)
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                }
            }
            "resources/read" => {
                info!("Handling resources/read request");
                if let Some(params) = request.params {
                    if let Some(uri) = params.get("uri").and_then(|u| u.as_str()) {
                        match self.resource_provider.read_resource(uri).await {
                            Ok(contents) => {
                                let result = json!({ "contents": contents });
                                JsonRpcResponse::success(result, request.id)
                            }
                            Err(e) => {
                                error!(error = %e, uri = %uri, "Failed to read resource");
                                let error_data = json!({
                                    "code": -32603,
                                    "message": format!("Failed to read resource: {}", e)
                                });
                                JsonRpcResponse::error(error_data, Some(request.id))
                            }
                        }
                    } else {
                        let error_data = json!({
                            "code": -32602,
                            "message": "Missing required parameter: uri"
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                } else {
                    let error_data = json!({
                        "code": -32602,
                        "message": "Missing parameters"
                    });
                    JsonRpcResponse::error(error_data, Some(request.id))
                }
            }
            "tools/list" => {
                info!("Handling tools/list request");
                match self.tool_provider.list_tools().await {
                    Ok(tools) => {
                        let result = json!({ "tools": tools });
                        JsonRpcResponse::success(result, request.id)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to list tools");
                        let error_data = json!({
                            "code": -32603,
                            "message": format!("Failed to list tools: {}", e)
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                }
            }
            "tools/call" => {
                info!("Handling tools/call request");
                if let Some(params) = request.params {
                    if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                        let arguments = params
                            .get("arguments")
                            .cloned()
                            .unwrap_or_else(|| json!({}));
                        match self.tool_provider.call_tool(name, arguments).await {
                            Ok(result) => {
                                let result_json = json!({ "content": result });
                                JsonRpcResponse::success(result_json, request.id)
                            }
                            Err(e) => {
                                error!(error = %e, tool = %name, "Failed to call tool");
                                let error_data = json!({
                                    "code": -32603,
                                    "message": format!("Failed to call tool: {}", e)
                                });
                                JsonRpcResponse::error(error_data, Some(request.id))
                            }
                        }
                    } else {
                        let error_data = json!({
                            "code": -32602,
                            "message": "Missing required parameter: name"
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                } else {
                    let error_data = json!({
                        "code": -32602,
                        "message": "Missing parameters"
                    });
                    JsonRpcResponse::error(error_data, Some(request.id))
                }
            }
            "prompts/list" => {
                info!("Handling prompts/list request");
                match self.prompt_provider.list_prompts().await {
                    Ok(prompts) => {
                        let result = json!({ "prompts": prompts });
                        JsonRpcResponse::success(result, request.id)
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to list prompts");
                        let error_data = json!({
                            "code": -32603,
                            "message": format!("Failed to list prompts: {}", e)
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                }
            }
            "prompts/get" => {
                info!("Handling prompts/get request");
                if let Some(params) = request.params {
                    if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                        let arguments = params
                            .get("arguments")
                            .and_then(|a| a.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect::<HashMap<String, String>>()
                            })
                            .unwrap_or_default();
                        match self.prompt_provider.get_prompt(name, arguments).await {
                            Ok((description, messages)) => {
                                let result = json!({
                                    "description": description,
                                    "messages": messages
                                });
                                JsonRpcResponse::success(result, request.id)
                            }
                            Err(e) => {
                                error!(error = %e, prompt = %name, "Failed to get prompt");
                                let error_data = json!({
                                    "code": -32603,
                                    "message": format!("Failed to get prompt: {}", e)
                                });
                                JsonRpcResponse::error(error_data, Some(request.id))
                            }
                        }
                    } else {
                        let error_data = json!({
                            "code": -32602,
                            "message": "Missing required parameter: name"
                        });
                        JsonRpcResponse::error(error_data, Some(request.id))
                    }
                } else {
                    let error_data = json!({
                        "code": -32602,
                        "message": "Missing parameters"
                    });
                    JsonRpcResponse::error(error_data, Some(request.id))
                }
            }
            _ => {
                warn!(method = %request.method, "Unknown method");
                let error_data = json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", request.method)
                });
                JsonRpcResponse::error(error_data, Some(request.id))
            }
            }
        }
    }
}

#[async_trait::async_trait]
impl MessageHandler<HttpContext> for ApiKeyMcpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.handle_mcp_request(request, &context.transport_context).await;
                // For HTTP transport, the response is handled by the transport layer
                // We just need to send it back through the context
                if let Ok(response_json) = response.to_json() {
                    info!(response = %response_json, "Sending HTTP MCP response");
                    // The HTTP transport will handle sending the response
                }
            }
            JsonRpcMessage::Notification(notification) => {
                info!(method = %notification.method, "Received notification");
                // Handle notifications as needed
            }
            JsonRpcMessage::Response(response) => {
                info!(id = ?response.id, "Received response");
                // Handle responses as needed
            }
        }
    }

    async fn handle_error(&self, error: TransportError) {
        error!(error = %error, "HTTP transport error occurred");
    }

    async fn handle_close(&self) {
        info!("HTTP transport connection closed");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (graceful degradation)
    init_logging()?;
    
    info!("🚀 Starting AIRS MCP API Key Server");
    
    // Create API key validator with valid keys for demonstration
    let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
    
    // Add valid API keys with their authentication context
    let api_keys = vec![
        "mcp_dev_key_12345",
        "mcp_prod_key_67890", 
        "mcp_test_key_abcdef",
    ];
    
    for (idx, key) in api_keys.iter().enumerate() {
        let auth_context = AuthContext::new(
            AuthMethod::new("api_key"),
            ApiKeyAuthData {
                key_id: format!("user_{}", idx + 1),
                source: ApiKeySource::Header("X-API-Key".to_string()),
            },
        );
        validator.add_key(key.to_string(), auth_context);
    }
    
    info!(key_count = api_keys.len(), "API key validator initialized");

    // Create temporary directory with sample files for the resource provider
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
    let temp_path = temp_dir.path();
    tokio::fs::write(temp_path.join("welcome.txt"), 
        "Welcome to the MCP API Key Server!\n\nThis server provides:\n- Filesystem resources\n- Mathematical tools\n- Code review prompts\n\nAuthenticate with X-API-Key or Authorization Bearer token.").await
        .map_err(|e| format!("Failed to create welcome.txt: {}", e))?;
        
    tokio::fs::write(temp_path.join("config.json"), 
        serde_json::to_string_pretty(&serde_json::json!({
            "server": {
                "name": "ApiKey MCP Server",
                "version": "1.0.0",
                "authentication": "api_key"
            },
            "capabilities": {
                "resources": true,
                "tools": true,
                "prompts": true
            }
        }))?).await
        .map_err(|e| format!("Failed to create config.json: {}", e))?;
        
    tokio::fs::write(temp_path.join("sample.md"), 
        "# MCP Server Resources\n\n## Available Resources\n\n- **welcome.txt**: Server introduction\n- **config.json**: Server configuration\n- **sample.md**: This markdown file\n- **api-keys.yaml**: API key information\n\n## Authentication\n\nUse one of these API keys:\n- mcp_dev_key_12345\n- mcp_prod_key_67890\n- mcp_test_key_abcdef\n").await
        .map_err(|e| format!("Failed to create sample.md: {}", e))?;
        
    tokio::fs::write(temp_path.join("api-keys.yaml"), 
        "# API Keys Configuration\napi_keys:\n  - key: mcp_dev_key_12345\n    name: Development Key\n    scope: full\n    environment: development\n  - key: mcp_prod_key_67890\n    name: Production Key\n    scope: full\n    environment: production\n  - key: mcp_test_key_abcdef\n    name: Test Key\n    scope: full\n    environment: testing\n\nusage:\n  header_methods:\n    - \"X-API-Key: <api_key>\"\n    - \"Authorization: Bearer <api_key>\"\n").await
        .map_err(|e| format!("Failed to create api-keys.yaml: {}", e))?;

    // Create providers using the new simplified pattern
    let resource_provider = ApiKeyResourceProvider::new(temp_path.to_path_buf());
    let tool_provider = ApiKeyMathToolProvider;
    let prompt_provider = ApiKeyPromptProvider;
    
    info!(temp_dir = %temp_path.display(), "Sample files created for resource provider");
    
    // Create the API key MCP handler with the Generic MessageHandler<HttpContext> pattern
    let mcp_handler = ApiKeyMcpHandler::new(
        resource_provider,
        tool_provider,
        prompt_provider,
        validator,
    );
    
    info!("MCP handler created with API key authentication");
    
    // Create HTTP transport using the modern HttpTransportBuilder pattern
    let transport = HttpTransportBuilder::new()
        .bind_address("127.0.0.1:3001")
        .with_message_handler(mcp_handler)
        .build()
        .await
        .map_err(|e| format!("Failed to create HTTP transport: {}", e))?;
    
    info!("HTTP transport created and configured");
    
    // Display server information
    info!("🔗 ENDPOINTS:");
    info!("   • MCP JSON-RPC endpoint: http://127.0.0.1:3001/mcp");
    info!("🔑 AUTHENTICATION:");
    info!("   • X-API-Key: mcp_dev_key_12345");
    info!("   • Authorization: Bearer mcp_dev_key_12345");
    info!("");
    info!("🚀 Server ready! Connect with MCP Inspector or any MCP client.");
    
    // Start the transport (this will run indefinitely)
    transport.start().await
        .map_err(|e| format!("HTTP transport error: {}", e))?;
    
    Ok(())
}
        .with_message_handler(mcp_handler)
        .build()
        .await
        .map_err(|e| format!("Failed to create HTTP transport: {}", e))?;
    
    info!("HTTP transport created and configured");
    
    // Display server information
    info!("🔗 ENDPOINTS:");
    info!("   • MCP JSON-RPC endpoint: http://127.0.0.1:3001/mcp");
    info!("🔑 AUTHENTICATION:");
    info!("   • X-API-Key: mcp_dev_key_12345");
    info!("   • Authorization: Bearer mcp_dev_key_12345");
    info!("");
    info!("🚀 Server ready! Connect with MCP Inspector or any MCP client.");
    
    // Start the transport (this will run indefinitely)
    transport.start().await
        .map_err(|e| format!("HTTP transport error: {}", e))?;
    
    Ok(())
}



/// Create utility routes for health checks, key management, and debugging
fn create_utility_routes(app_state: AppState) -> Router {
    use axum::routing::options;
    
    Router::new()
        .route("/health", get(health_handler).options(cors_preflight_handler))
        .route("/keys", get(keys_handler).post(create_key_handler).options(cors_preflight_handler))
        .route("/auth/info", get(auth_info_handler).options(cors_preflight_handler))
        .route("/server/info", get(server_info_handler).options(cors_preflight_handler))
        // Catch-all OPTIONS handler
        .fallback(options(cors_preflight_handler))
        .with_state(app_state)
}

/// Health check endpoint
async fn health_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    info!(target: "handler", "Health check endpoint called");
    let uptime = state.start_time.elapsed();
    
    let response = json!({
        "status": "healthy",
        "server_id": state.server_id,
        "uptime_seconds": uptime.as_secs(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    });
    
    debug!(target: "handler", response = %response, "Health check response");
    Ok(Json(response))
}

/// API keys management endpoint (for demo purposes)
async fn keys_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    warn!(target: "handler", "API keys endpoint accessed - development use only");
    info!(target: "handler", "Returning API keys information");
    
    Ok(Json(json!({
        "valid_keys": {
            "mcp_dev_key_12345": {
                "name": "Development Key",
                "scope": "full",
                "environment": "development"
            },
            "mcp_prod_key_67890": {
                "name": "Production Key",
                "scope": "full",
                "environment": "production"
            },
            "mcp_test_key_abcdef": {
                "name": "Test Key",
                "scope": "full",
                "environment": "testing"
            }
        },
        "usage": {
            "header_name": "X-API-Key",
            "alternative_header": "Authorization: Bearer <api_key>",
            "example": "curl -H \"X-API-Key: mcp_dev_key_12345\" http://127.0.0.1:3001/mcp"
        },
        "configured_keys_count": state.valid_api_keys.len(),
        "note": "This endpoint is for development only - remove in production"
    })))
}

/// Create new API key endpoint (for demo purposes)
async fn create_key_handler() -> Result<Json<Value>, StatusCode> {
    let new_key = format!("mcp_gen_key_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..12].to_string());
    
    warn!(generated_key = %new_key, "Generated new API key - development use only");
    
    Ok(Json(json!({
        "api_key": new_key,
        "type": "generated",
        "expires": "never",
        "scope": "full",
        "note": "Add this key to your server configuration to use it",
        "warning": "This endpoint is for development only - implement proper key management in production"
    })))
}

/// Authentication information endpoint
async fn auth_info_handler() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "auth_method": "api_key",
        "authorization_type": "key_based",
        "supported_headers": ["X-API-Key", "Authorization"],
        "key_formats": {
            "x_api_key": "X-API-Key: <your_api_key>",
            "bearer_format": "Authorization: Bearer <your_api_key>"
        },
        "examples": {
            "curl_x_api_key": "curl -H \"X-API-Key: mcp_dev_key_12345\" http://127.0.0.1:3001/mcp",
            "curl_bearer": "curl -H \"Authorization: Bearer mcp_dev_key_12345\" http://127.0.0.1:3001/mcp"
        },
        "endpoints": {
            "mcp": "/mcp",
            "health": "/health",
            "keys": "/keys"
        }
    })))
}

/// Server information endpoint
async fn server_info_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "server_id": state.server_id,
        "name": "ApiKey MCP Server",
        "version": "1.0.0",
        "description": "Simple ApiKey-based MCP Server Example",
        "capabilities": {
            "tools": ["math/calculate"],
            "resources": ["filesystem"],
            "prompts": ["code_review"]
        },
        "transport": "http",
        "authentication": "api_key",
        "authorization": "key_based",
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "api_keys_count": state.valid_api_keys.len()
    })))
}
