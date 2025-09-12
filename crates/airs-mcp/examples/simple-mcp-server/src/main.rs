// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::{json, Value};

// Layer 3: Internal module imports
use airs_mcp::integration::{McpError, McpServer};
use airs_mcp::protocol::types::{
    Content, MimeType, Prompt, PromptArgument, PromptMessage, Resource, Tool, Uri,
};
use airs_mcp::protocol::{
    JsonRpcMessage, JsonRpcMessageTrait, JsonRpcRequest, JsonRpcResponse, MessageContext,
    MessageHandler, TransportBuilder, TransportError,
};
use airs_mcp::providers::{PromptProvider, ResourceProvider, ToolProvider};
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;

// Professional logging imports
use tracing::{error, info, instrument, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::layer, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// Initialize internal logging with graceful degradation
/// - First tries file-based logging for debugging and operations
/// - Falls back to no-op logging if file system access is denied
/// - Never outputs to stdout/stderr to avoid JSON-RPC contamination
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Try file-based logging first
    let file_layer_result = std::panic::catch_unwind(|| {
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            "/tmp/simple-mcp-server",
            "simple-mcp-server.log",
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
            info!("🚀 AIRS MCP Server starting with file-based logging");
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
/// Simple file system resource provider
#[derive(Debug)]
struct SimpleResourceProvider;

#[async_trait]
impl ResourceProvider for SimpleResourceProvider {
    #[instrument(level = "debug")]
    async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        info!("Listing available resources");

        let resources = vec![
            Resource {
                uri: Uri::new("file:///tmp/example.txt").unwrap(),
                name: "Example File".to_string(),
                description: Some("A simple example file".to_string()),
                mime_type: Some(MimeType::new("text/plain").unwrap()),
            },
            Resource {
                uri: Uri::new("file:///tmp/config.json").unwrap(),
                name: "Config File".to_string(),
                description: Some("Application configuration".to_string()),
                mime_type: Some(MimeType::new("application/json").unwrap()),
            },
        ];

        info!(
            resource_count = resources.len(),
            "Resources listed successfully"
        );
        Ok(resources)
    }

    #[instrument(level = "debug", fields(uri = %uri))]
    async fn read_resource(&self, uri: &str) -> Result<Vec<Content>, McpError> {
        info!(uri = %uri, "Reading resource");

        let content_text = match uri {
            "file:///tmp/example.txt" => "Hello from the MCP server!\nThis is example content.",
            "file:///tmp/config.json" => r#"{"app_name": "Simple MCP Server", "version": "1.0.0"}"#,
            _ => {
                warn!(uri = %uri, "Resource not found");
                return Err(McpError::resource_not_found(uri));
            }
        };

        // Use text_with_uri to provide proper URI for resource responses
        let content = vec![Content::text_with_uri(content_text, uri)
            .map_err(|e| McpError::internal_error(format!("Failed to create content: {e}")))?];
        info!(uri = %uri, content_size = content_text.len(), "Resource read successfully");
        Ok(content)
    }
}

/// Simple calculator tool provider
#[derive(Debug)]
struct SimpleToolProvider;

#[async_trait]
impl ToolProvider for SimpleToolProvider {
    #[instrument(level = "debug")]
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        info!("Listing available tools");

        let tools = vec![
            Tool {
                name: "add".to_string(),
                description: Some("Add two numbers together".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }),
            },
            Tool {
                name: "greet".to_string(),
                description: Some("Generate a greeting message".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Name to greet"}
                    },
                    "required": ["name"]
                }),
            },
        ];

        info!(tool_count = tools.len(), "Tools listed successfully");
        Ok(tools)
    }

    #[instrument(level = "debug", fields(tool_name = %name))]
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<Content>, McpError> {
        info!(tool_name = %name, arguments = %arguments, "Executing tool");

        let result = match name {
            "add" => {
                let a = arguments.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'a'");
                    McpError::invalid_request("Missing or invalid parameter 'a'")
                })?;

                let b = arguments.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
                    warn!(tool_name = %name, "Missing or invalid parameter 'b'");
                    McpError::invalid_request("Missing or invalid parameter 'b'")
                })?;

                let sum = a + b;
                info!(tool_name = %name, a = %a, b = %b, result = %sum, "Addition completed");

                json!({
                    "result": sum,
                    "operation": "addition"
                })
            }
            "greet" => {
                let name_param =
                    arguments
                        .get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            McpError::invalid_request("Missing or invalid parameter 'name'")
                        })?;

                json!({
                    "greeting": format!("Hello, {}! Welcome to the MCP server!", name_param)
                })
            }
            _ => return Err(McpError::tool_not_found(name)),
        };

        Ok(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap(),
        )])
    }
}

/// Simple prompt template provider
#[derive(Debug)]
struct SimplePromptProvider;

#[async_trait]
impl PromptProvider for SimplePromptProvider {
    async fn list_prompts(&self) -> Result<Vec<Prompt>, McpError> {
        Ok(vec![
            Prompt {
                name: "code_review".to_string(),
                title: Some("Code Review".to_string()),
                description: Some("Generate a code review prompt".to_string()),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to review")),
                ],
            },
            Prompt {
                name: "explain_concept".to_string(),
                title: Some("Explain Concept".to_string()),
                description: Some("Explain a technical concept".to_string()),
                arguments: vec![
                    PromptArgument::required("concept", Some("Concept to explain")),
                    PromptArgument::optional(
                        "level",
                        Some("Difficulty level (beginner, intermediate, advanced)"),
                    ),
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

                let prompt_text = format!(
                    "Please review the following {language} code and provide feedback:\n\n```{language}\n{code}\n```\n\nFocus on:\n- Code quality and best practices\n- Potential bugs or issues\n- Performance considerations\n- Readability and maintainability"
                );

                (
                    "Code review prompt template".to_string(),
                    vec![PromptMessage::user(Content::text(prompt_text))],
                )
            }
            "explain_concept" => {
                let concept = arguments
                    .get("concept")
                    .cloned()
                    .unwrap_or_else(|| "unknown concept".to_string());
                let level = arguments
                    .get("level")
                    .cloned()
                    .unwrap_or_else(|| "intermediate".to_string());

                let prompt_text = format!(
                    "Please explain the concept of '{concept}' at a {level} level. Include:\n- Clear definition\n- Key principles\n- Practical examples\n- Common use cases"
                );

                (
                    "Technical concept explanation template".to_string(),
                    vec![PromptMessage::user(Content::text(prompt_text))],
                )
            }
            _ => return Err(McpError::prompt_not_found(name)),
        };

        Ok((description, messages))
    }
}

/// Simple MCP Handler - Wraps existing providers for new MessageHandler architecture
///
/// This handler preserves all existing business logic while adapting to the modern
/// Generic MessageHandler<()> pattern. All provider implementations remain unchanged.
#[derive(Debug)]
struct SimpleMcpHandler {
    resource_provider: SimpleResourceProvider,
    tool_provider: SimpleToolProvider,
    prompt_provider: SimplePromptProvider,
}

impl SimpleMcpHandler {
    /// Create a new handler with the existing providers
    pub fn new(
        resource_provider: SimpleResourceProvider,
        tool_provider: SimpleToolProvider,
        prompt_provider: SimplePromptProvider,
    ) -> Self {
        Self {
            resource_provider,
            tool_provider,
            prompt_provider,
        }
    }

    /// Handle MCP protocol requests using existing provider logic
    async fn handle_mcp_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => {
                info!("Handling initialize request");
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
                        "name": "simple-mcp-server",
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

#[async_trait]
impl MessageHandler<()> for SimpleMcpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.handle_mcp_request(request).await;
                // For STDIO transport, we need to write the response to stdout
                if let Ok(response_json) = response.to_json() {
                    println!("{response_json}");
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
        error!(error = %error, "Transport error occurred");
    }

    async fn handle_close(&self) {
        info!("Transport connection closed");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize internal logging with graceful degradation
    let _ = init_logging();

    // Log startup (to file only, not stdout/stderr)
    info!("🚀 Starting Simple MCP Server with internal logging...");
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Server initialization starting"
    );

    // Create MCP handler with all providers (preserving existing business logic)
    info!("Creating MCP handler with providers...");
    let handler = Arc::new(SimpleMcpHandler::new(
        SimpleResourceProvider,
        SimpleToolProvider,
        SimplePromptProvider,
    ));
    info!("✅ MCP handler created successfully");

    // Create STDIO transport with pre-configured handler
    info!("Building STDIO transport with handler...");
    let transport = StdioTransportBuilder::new()
        .with_message_handler(handler)
        .build()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to build STDIO transport");
            e
        })?;
    info!("✅ STDIO transport built successfully");

    // Create the MCP server (simple lifecycle wrapper)
    info!("Creating MCP server...");
    let server = McpServer::new(transport);
    info!("✅ MCP Server created successfully!");
    info!("📋 Available capabilities:");
    info!("   - Resources: file system examples");
    info!("   - Tools: add, greet");
    info!("   - Prompts: code_review, explain_concept");
    info!("🔗 Server ready for MCP client connections via STDIO");

    // Start the server with error handling
    info!("Starting MCP server...");
    if let Err(e) = server.start().await {
        error!(error = %e, "Server error occurred");
        return Err(e.into());
    }

    // Keep the server running - wait for Ctrl+C
    info!("MCP server is running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C signal, shutting down...");
        }
        Err(err) => {
            error!(error = %err, "Unable to listen for shutdown signal");
        }
    }

    // Graceful shutdown
    info!("Shutting down MCP server...");
    if let Err(e) = server.shutdown().await {
        error!(error = %e, "Error during server shutdown");
    }

    info!("🛑 MCP Server shutdown completed");
    Ok(())
}
