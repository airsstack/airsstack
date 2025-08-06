use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::integration::mcp::{McpError, PromptProvider, ResourceProvider, ToolProvider};
use airs_mcp::shared::protocol::{Content, MimeType, Prompt, PromptMessage, Resource, Tool, Uri};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio;

/// Simple file system resource provider
#[derive(Debug)]
struct SimpleResourceProvider;

#[async_trait]
impl ResourceProvider for SimpleResourceProvider {
    async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        Ok(vec![
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
        ])
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<Content>, McpError> {
        let content_text = match uri {
            "file:///tmp/example.txt" => "Hello from the MCP server!\nThis is example content.",
            "file:///tmp/config.json" => r#"{"app_name": "Simple MCP Server", "version": "1.0.0"}"#,
            _ => return Err(McpError::resource_not_found(uri)),
        };

        Ok(vec![Content::text(content_text)])
    }
}

/// Simple calculator tool provider
#[derive(Debug)]
struct SimpleToolProvider;

#[async_trait]
impl ToolProvider for SimpleToolProvider {
    async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        Ok(vec![
            Tool {
                name: "add".to_string(),
                display_name: "Add Numbers".to_string(),
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
                display_name: "Greet User".to_string(),
                description: Some("Generate a greeting message".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Name to greet"}
                    },
                    "required": ["name"]
                }),
            },
        ])
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<Content>, McpError> {
        let result =
            match name {
                "add" => {
                    let a = arguments.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
                        McpError::invalid_request("Missing or invalid parameter 'a'")
                    })?;

                    let b = arguments.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
                        McpError::invalid_request("Missing or invalid parameter 'b'")
                    })?;

                    json!({
                        "result": a + b,
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
                display_name: "Code Review".to_string(),
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
                display_name: "Explain Concept".to_string(),
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
    // Initialize logging
    env_logger::init();

    eprintln!("🚀 Starting Simple MCP Server...");

    // Create STDIO transport
    let transport = airs_mcp::transport::StdioTransport::new().await?;

    // Create the MCP server with all providers
    let server = McpServerBuilder::new()
        .with_resource_provider(SimpleResourceProvider)
        .with_tool_provider(SimpleToolProvider)
        .with_prompt_provider(SimplePromptProvider)
        .build(transport)
        .await?;

    eprintln!("✅ MCP Server initialized successfully!");
    eprintln!("📋 Available capabilities:");
    eprintln!("   - Resources: file system examples");
    eprintln!("   - Tools: add, greet");
    eprintln!("   - Prompts: code_review, explain_concept");
    eprintln!("🔗 Server ready for MCP client connections via STDIO");

    // Run the server (this will handle STDIO communication)
    server.run().await?;

    Ok(())
}
