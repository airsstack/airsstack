use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::integration::mcp::{McpError, PromptProvider, ResourceProvider, ToolProvider};
use airs_mcp::shared::protocol::{Content, MimeType, Prompt, PromptMessage, Resource, Tool, Uri};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio;

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

        let content = vec![Content::text(content_text)];
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
                title: Some("Add Numbers".to_string()),
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
                title: Some("Greet User".to_string()),
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
                arguments: json!({
                    "type": "object",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "Programming language"
                        },
                        "code": {
                            "type": "string",
                            "description": "Code to review"
                        }
                    },
                    "required": ["language", "code"]
                }),
            },
            Prompt {
                name: "explain_concept".to_string(),
                title: Some("Explain Concept".to_string()),
                description: Some("Explain a technical concept".to_string()),
                arguments: json!({
                    "type": "object",
                    "properties": {
                        "concept": {
                            "type": "string",
                            "description": "Concept to explain"
                        },
                        "level": {
                            "type": "string",
                            "description": "Difficulty level (beginner, intermediate, advanced)"
                        }
                    },
                    "required": ["concept"]
                }),
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
                    "Please review the following {} code and provide feedback:\n\n```{}\n{}\n```\n\nFocus on:\n- Code quality and best practices\n- Potential bugs or issues\n- Performance considerations\n- Readability and maintainability",
                    language, language, code
                );

                (
                    "Code review prompt template".to_string(),
                    vec![PromptMessage::user(prompt_text)],
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
                    "Please explain the concept of '{}' at a {} level. Include:\n- Clear definition\n- Key principles\n- Practical examples\n- Common use cases",
                    concept, level
                );

                (
                    "Technical concept explanation template".to_string(),
                    vec![PromptMessage::user(prompt_text)],
                )
            }
            _ => return Err(McpError::prompt_not_found(name)),
        };

        Ok((description, messages))
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

    // Create STDIO transport
    info!("Creating STDIO transport...");
    let transport = airs_mcp::transport::StdioTransport::new()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to create STDIO transport");
            e
        })?;
    info!("✅ STDIO transport created successfully");

    // Create the MCP server with all providers
    info!("Building MCP server with providers...");
    let server = McpServerBuilder::new()
        .with_resource_provider(SimpleResourceProvider)
        .with_tool_provider(SimpleToolProvider)
        .with_prompt_provider(SimplePromptProvider)
        .build(transport)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to build MCP server");
            e
        })?;

    info!("✅ MCP Server initialized successfully!");
    info!("📋 Available capabilities:");
    info!("   - Resources: file system examples");
    info!("   - Tools: add, greet");
    info!("   - Prompts: code_review, explain_concept");
    info!("🔗 Server ready for MCP client connections via STDIO");

    // Start the server with error handling
    info!("Starting MCP server event loop...");
    if let Err(e) = server.run().await {
        error!(error = %e, "Server error occurred");
        return Err(e.into());
    }

    info!("🛑 MCP Server shutdown completed");
    Ok(())
}
