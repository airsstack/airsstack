//! STDIO MCP Server Integration Example
//!
//! This example demonstrates a complete MCP server implementation using direct STDIO
//! communication (without the Transport layer) for one-shot request-response patterns.
//! This matches the typical STDIO MCP usage pattern where each invocation processes
//! a single request and exits.
//!
//! ## Features
//!
//! - **Direct STDIO**: Reads from stdin, writes to stdout, exits on completion
//! - **Standard Providers**: FileSystem resources, Math tools, Code review prompts, Structured logging
//! - **MCP Protocol**: Full MCP 2024-11-05 protocol implementation
//! - **One-Shot Pattern**: Process request → Send response → Exit cleanly
//!
//! ## Usage
//!
//! ```bash
//! # Run the server
//! cargo run --bin stdio-server
//!
//! # Test with MCP client
//! echo '{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}' | cargo run --bin stdio-server
//! ```

// Layer 1: Standard library imports
use std::fs;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use serde_json::json;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Layer 3: Internal module imports
use airs_mcp::protocol::{
    constants::methods as mcp_methods, CallToolRequest, GetPromptRequest, InitializeResponse,
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, LoggingCapabilities, PromptCapabilities,
    ProtocolVersion, ReadResourceRequest, ResourceCapabilities, ServerCapabilities, ServerInfo,
    ToolCapabilities,
};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider, PromptProvider,
    ResourceProvider, StructuredLoggingHandler, ToolProvider,
};

/// STDIO MCP Message Handler
///
/// Handles MCP protocol messages over direct STDIO communication.
/// Implements the full MCP 2024-11-05 protocol specification.
#[derive(Debug)]
pub struct StdioMcpHandler {
    resource_provider: Arc<FileSystemResourceProvider>,
    tool_provider: Arc<MathToolProvider>,
    prompt_provider: Arc<CodeReviewPromptProvider>,
    #[allow(dead_code)] // TODO: Implement logging handler integration
    logging_handler: Arc<StructuredLoggingHandler>,
}

impl StdioMcpHandler {
    /// Create new STDIO MCP handler with all providers
    pub fn new(
        resource_provider: FileSystemResourceProvider,
        tool_provider: MathToolProvider,
        prompt_provider: CodeReviewPromptProvider,
        logging_handler: StructuredLoggingHandler,
    ) -> Self {
        Self {
            resource_provider: Arc::new(resource_provider),
            tool_provider: Arc::new(tool_provider),
            prompt_provider: Arc::new(prompt_provider),
            logging_handler: Arc::new(logging_handler),
        }
    }

    /// Process MCP request and return response
    pub async fn process_mcp_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            // Initialization and lifecycle methods
            mcp_methods::INITIALIZE => self.handle_initialize(request).await,
            mcp_methods::INITIALIZED => self.handle_initialized(request).await,

            // Resource management methods
            mcp_methods::RESOURCES_LIST => self.handle_resources_list(request).await,
            mcp_methods::RESOURCES_READ => self.handle_resources_read(request).await,
            mcp_methods::RESOURCES_TEMPLATES_LIST => {
                self.handle_resources_templates_list(request).await
            }
            mcp_methods::RESOURCES_SUBSCRIBE => self.handle_resources_subscribe(request).await,
            mcp_methods::RESOURCES_UNSUBSCRIBE => self.handle_resources_unsubscribe(request).await,

            // Tool management methods
            mcp_methods::TOOLS_LIST => self.handle_tools_list(request).await,
            mcp_methods::TOOLS_CALL => self.handle_tools_call(request).await,

            // Prompt management methods
            mcp_methods::PROMPTS_LIST => self.handle_prompts_list(request).await,
            mcp_methods::PROMPTS_GET => self.handle_prompts_get(request).await,

            // Logging methods
            mcp_methods::LOGGING_SET_LEVEL => self.handle_logging_set_level(request).await,

            // Ping/pong for connectivity testing
            mcp_methods::PING => self.handle_ping(request).await,

            // Unknown methods
            _ => self.create_method_not_found_response(
                request,
                &format!("Unknown method: {}", request.method),
            ),
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling initialize request");

        let capabilities = ServerCapabilities {
            experimental: Some(json!({})),
            logging: Some(LoggingCapabilities {}),
            resources: Some(ResourceCapabilities {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            tools: Some(ToolCapabilities {
                list_changed: Some(false),
            }),
            prompts: Some(PromptCapabilities {
                list_changed: Some(false),
            }),
        };

        let response = InitializeResponse {
            protocol_version: ProtocolVersion::new("2024-11-05").expect("Valid protocol version"),
            capabilities: serde_json::to_value(capabilities).unwrap_or(json!({})),
            server_info: ServerInfo {
                name: "airs-mcp-stdio-server".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(response).unwrap_or(json!({}))),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle initialized notification
    async fn handle_initialized(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Client initialization completed");

        // Initialized is a notification, so we don't send a response
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/list request");

        match self.resource_provider.list_resources().await {
            Ok(resources) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "resources": resources })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {}", e))
            }
        }
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/read request");

        match serde_json::from_value::<ReadResourceRequest>(
            request.params.clone().unwrap_or(json!({})),
        ) {
            Ok(read_request) => {
                match self
                    .resource_provider
                    .read_resource(&read_request.uri.to_string())
                    .await
                {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!({ "contents": result })),
                        error: None,
                        id: Some(request.id.clone()),
                    },
                    Err(e) => self.create_error_response(
                        request,
                        -32603,
                        &format!("Internal error: {}", e),
                    ),
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {}", e))
            }
        }
    }

    /// Handle resources/templates/list request
    async fn handle_resources_templates_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling resources/templates/list request");

        // Resource templates not implemented yet
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({ "resourceTemplates": [] })),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle resources/subscribe request
    async fn handle_resources_subscribe(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        self.create_method_not_found_response(request, "Resource subscriptions not supported")
    }

    /// Handle resources/unsubscribe request
    async fn handle_resources_unsubscribe(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        self.create_method_not_found_response(request, "Resource subscriptions not supported")
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/list request");

        match self.tool_provider.list_tools().await {
            Ok(tools) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "tools": tools })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {}", e))
            }
        }
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling tools/call request");

        match serde_json::from_value::<CallToolRequest>(request.params.clone().unwrap_or(json!({})))
        {
            Ok(call_request) => {
                let arguments = call_request.arguments;
                match self
                    .tool_provider
                    .call_tool(&call_request.name, arguments)
                    .await
                {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!({ "content": result })),
                        error: None,
                        id: Some(request.id.clone()),
                    },
                    Err(e) => {
                        // Map tool provider errors to appropriate JSON-RPC error codes
                        let error_message = e.to_string();
                        if error_message.contains("Tool not found")
                            || error_message.contains("Unknown tool")
                        {
                            // Tool not found should be treated as invalid params (tool name is invalid)
                            self.create_error_response(
                                request,
                                -32602,
                                &format!("Invalid params: {}", e),
                            )
                        } else {
                            // Other tool errors are internal errors (execution failures)
                            self.create_error_response(
                                request,
                                -32603,
                                &format!("Internal error: {}", e),
                            )
                        }
                    }
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {}", e))
            }
        }
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling prompts/list request");

        match self.prompt_provider.list_prompts().await {
            Ok(prompts) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({ "prompts": prompts })),
                error: None,
                id: Some(request.id.clone()),
            },
            Err(e) => {
                self.create_error_response(request, -32603, &format!("Internal error: {}", e))
            }
        }
    }

    /// Handle prompts/get request
    async fn handle_prompts_get(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling prompts/get request");

        match serde_json::from_value::<GetPromptRequest>(
            request.params.clone().unwrap_or(json!({})),
        ) {
            Ok(get_request) => {
                match self
                    .prompt_provider
                    .get_prompt(&get_request.name, get_request.arguments)
                    .await
                {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
                        error: None,
                        id: Some(request.id.clone()),
                    },
                    Err(e) => self.create_error_response(
                        request,
                        -32603,
                        &format!("Internal error: {}", e),
                    ),
                }
            }
            Err(e) => {
                self.create_error_response(request, -32602, &format!("Invalid params: {}", e))
            }
        }
    }

    /// Handle logging/setLevel request
    async fn handle_logging_set_level(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling logging/setLevel request");

        // Logging level setting handled by the logging handler
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({})),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Handle ping request
    async fn handle_ping(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("Handling ping request");

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!("pong")),
            error: None,
            id: Some(request.id.clone()),
        }
    }

    /// Create method not found response
    fn create_method_not_found_response(
        &self,
        request: &JsonRpcRequest,
        message: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": "Method not found",
                "data": message
            })),
            id: Some(request.id.clone()),
        }
    }

    /// Create error response
    fn create_error_response(
        &self,
        request: &JsonRpcRequest,
        code: i32,
        message: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(json!({
                "code": code,
                "message": message
            })),
            id: Some(request.id.clone()),
        }
    }
}

/// Create test environment with filesystem resources for STDIO server
///
/// Creates a test directory with sample files and returns the configured
/// STDIO message handler with all providers enabled.
async fn create_test_environment(
) -> Result<(StdioMcpHandler, Option<TempDir>), Box<dyn std::error::Error>> {
    // Create persistent test directory instead of temporary one
    // This ensures files persist between server invocations (important for STDIO one-shot pattern)
    let test_path = std::env::current_dir()?.join("test_resources");

    // Create directory if it doesn't exist
    if !test_path.exists() {
        fs::create_dir_all(&test_path)?;
        info!(
            "📂 Created persistent test directory: {}",
            test_path.display()
        );
    } else {
        info!("📂 Using existing test directory: {}", test_path.display());
    }

    // Create test files if they don't exist (avoid overwriting)
    let text_file = test_path.join("stdio-test.txt");
    if !text_file.exists() {
        fs::write(
            &text_file,
            "Hello from STDIO MCP server! This file demonstrates resource access.",
        )?;
    }

    let config_file = test_path.join("config.json");
    if !config_file.exists() {
        fs::write(
            &config_file,
            r#"{"server": "stdio-mcp-server", "version": "1.0.0", "transport": "stdio"}"#,
        )?;
    }

    let readme_file = test_path.join("README.md");
    if !readme_file.exists() {
        fs::write(
            &readme_file,
            "# STDIO MCP Server\n\nThis file is accessible through the filesystem resources provider.",
        )?;
    }

    let yaml_file = test_path.join("sample.yaml");
    if !yaml_file.exists() {
        fs::write(
            &yaml_file,
            "# Sample YAML file\nname: stdio-test\nversion: 1.0\ntools:\n  - math\n  - system\n  - text",
        )?;
    }

    // Create subdirectory with more files
    let subdir = test_path.join("examples");
    if !subdir.exists() {
        fs::create_dir_all(&subdir)?;
    }

    let example_file = subdir.join("example.txt");
    if !example_file.exists() {
        fs::write(&example_file, "This is an example file in a subdirectory.")?;
    }

    info!("📂 Verified test files in directory");

    // Verify files exist
    let files: Vec<_> = fs::read_dir(&test_path)?.filter_map(|e| e.ok()).collect();
    info!("📂 Files in test directory: {} files", files.len());

    // Get canonical path for resource provider
    let canonical_path = test_path.canonicalize()?;
    info!("📂 Canonical path: {}", canonical_path.display());

    // Create all providers following OAuth2 server pattern
    let resource_provider = FileSystemResourceProvider::new(&canonical_path)
        .expect("Failed to create filesystem resource provider");
    let tool_provider = MathToolProvider::new();
    let prompt_provider = CodeReviewPromptProvider::new();
    let logging_handler = StructuredLoggingHandler::new();

    info!("📦 Created all MCP providers");

    // Create STDIO message handler
    let handler = StdioMcpHandler::new(
        resource_provider,
        tool_provider,
        prompt_provider,
        logging_handler,
    );

    // Return None for temp_dir since we're using a persistent directory
    Ok((handler, None))
}

/// Initialize tracing/logging with default configuration
fn init_logging() {
    // Set default log level from environment or use INFO
    let log_level = std::env::var("STDIO_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::builder()
            .with_default_directive(log_level.parse().unwrap_or(tracing::Level::INFO.into()))
            .from_env_lossy(),
    );

    if std::env::var("STDIO_LOG_STRUCTURED").is_ok() {
        subscriber
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    }
}

/// Main server entry point - Direct STDIO MCP Server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    info!("🚀 Starting STDIO MCP Server (Direct Mode)");

    // Create test environment and MCP handler
    let (handler, _temp_dir) = create_test_environment().await?;

    info!("📦 MCP providers initialized");
    info!("🌟 MCP server starting with direct STDIO processing");
    info!("📋 Available capabilities:");
    info!("   • Tools: Math operations and calculations");
    info!("   • Resources: Filesystem access to test directory");
    info!("   • Prompts: Code review templates");
    info!("   • Logging: Structured logging with configurable levels");
    info!("");
    info!("💡 Usage:");
    info!("   Send JSON-RPC requests to stdin, receive responses on stdout");
    info!("   Example: echo '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{{}}}}' | ./stdio-server");
    info!("");
    info!("🛠️  Environment variables:");
    info!("   • STDIO_LOG_LEVEL: Log level (trace, debug, info, warn, error)");
    info!("   • STDIO_LOG_STRUCTURED: Enable structured logging");

    // Process STDIO directly - one-shot pattern
    info!("✅ MCP server ready, processing stdin");

    // Read all available input from stdin
    let mut stdin = tokio::io::stdin();
    let mut buffer = Vec::new();

    // Read until EOF
    match stdin.read_to_end(&mut buffer).await {
        Ok(0) => {
            info!("📭 No input received on stdin");
            return Ok(());
        }
        Ok(bytes_read) => {
            info!("📨 Received {} bytes from stdin", bytes_read);
            let input = String::from_utf8_lossy(&buffer);

            // Process each line as a separate JSON-RPC message
            for line in input.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                info!("🔄 Processing request: {}", trimmed);

                // Parse and process the JSON-RPC message
                match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                    Ok(request) => {
                        let response = handler.process_mcp_request(&request).await;

                        // Send response to stdout
                        let response_json =
                            serde_json::to_string(&JsonRpcMessage::Response(response))?;
                        println!("{}", response_json);

                        // Flush to ensure immediate output
                        tokio::io::stdout().flush().await?;
                    }
                    Err(e) => {
                        error!("❌ Failed to parse JSON-RPC request: {}", e);

                        // Send error response for malformed JSON
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(json!({
                                "code": -32700,
                                "message": "Parse error",
                                "data": format!("Invalid JSON: {}", e)
                            })),
                            id: None,
                        };

                        let error_json =
                            serde_json::to_string(&JsonRpcMessage::Response(error_response))?;
                        println!("{}", error_json);
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to read from stdin: {}", e);
            return Err(e.into());
        }
    }

    info!("✅ MCP server completed processing");
    Ok(())
}
