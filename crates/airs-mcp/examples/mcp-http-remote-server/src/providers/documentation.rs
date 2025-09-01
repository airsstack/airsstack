//! Documentation and code review prompt provider
//!
//! Provides structured prompts for code review, documentation, and development guidance 
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tracing::{debug, instrument, warn};

// Layer 3: Internal module imports
use airs_mcp::integration::mcp::{PromptProvider, McpError, McpResult};
use airs_mcp::shared::protocol::{Prompt, PromptArgument, PromptMessage, Content, MessageRole};

/// Documentation and code review prompt provider
#[derive(Debug, Clone)]
pub struct DocumentationPromptProvider {
    available_prompts: Vec<String>,
}

impl DocumentationPromptProvider {
    /// Create new documentation prompt provider
    pub fn new() -> Self {
        Self {
            available_prompts: vec![
                "code_review".to_string(),
                "documentation_guide".to_string(),
                "rust_best_practices".to_string(),
                "api_design".to_string(),
                "error_handling".to_string(),
                "testing_strategy".to_string(),
                "performance_review".to_string(),
            ],
        }
    }

    /// Generate prompt content based on type and context
    #[instrument(skip(self))]
    fn generate_prompt(&self, prompt_name: &str, context: &HashMap<String, String>) -> Result<(String, Vec<PromptMessage>), String> {
        match prompt_name {
            "code_review" => {
                let language = context.get("language")
                    .map(|s| s.as_str())
                    .unwrap_or("rust");
                let focus = context.get("focus")
                    .map(|s| s.as_str())
                    .unwrap_or("general");

                let description = "Comprehensive code review prompt".to_string();
                let messages = vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: Content::text(format!(
                            "You are a senior {} developer conducting a thorough code review. \
                            Focus on: {}. Analyze the provided code for:\n\
                            1. Code quality and readability\n\
                            2. Best practices adherence\n\
                            3. Performance considerations\n\
                            4. Security implications\n\
                            5. Error handling\n\
                            6. Testing coverage\n\
                            7. Documentation quality\n\
                            Provide constructive feedback with specific examples and suggestions.",
                            language, focus
                        )),
                    },
                    PromptMessage {
                        role: MessageRole::User,
                        content: Content::text("Please review the following code and provide detailed feedback:".to_string()),
                    },
                ];
                Ok((description, messages))
            }
            "documentation_guide" => {
                let project_type = context.get("project_type")
                    .map(|s| s.as_str())
                    .unwrap_or("library");

                let description = "Documentation writing guide".to_string();
                let messages = vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: Content::text(format!(
                            "You are a technical writing expert specializing in {} documentation. \
                            Help create comprehensive, clear, and maintainable documentation. \
                            Focus on:\n\
                            1. Clear API documentation\n\
                            2. Usage examples\n\
                            3. Architecture explanations\n\
                            4. Getting started guides\n\
                            5. Contributing guidelines\n\
                            6. Troubleshooting sections\n\
                            Ensure documentation is accessible to both beginners and experts.",
                            project_type
                        )),
                    },
                ];
                Ok((description, messages))
            }
            "rust_best_practices" => {
                let description = "Rust language best practices guide".to_string();
                let messages = vec![
                    PromptMessage {
                        role: MessageRole::System,
                        content: Content::text(
                            "You are a Rust expert providing guidance on best practices. \
                            Cover these key areas:\n\
                            1. Memory safety and ownership\n\
                            2. Error handling with Result and Option\n\
                            3. Async programming patterns\n\
                            4. Performance optimization\n\
                            5. API design principles\n\
                            6. Testing strategies\n\
                            7. Cargo and dependency management\n\
                            8. Documentation with rustdoc\n\
                            Provide practical examples and explain the reasoning behind each practice.".to_string()
                        ),
                    },
                ];
                Ok((description, messages))
            }
            _ => Err(format!("Unknown prompt: {}", prompt_name)),
        }
    }
}

#[async_trait]
impl PromptProvider for DocumentationPromptProvider {
    #[instrument(skip(self))]
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        debug!("Listing {} documentation prompts", self.available_prompts.len());
        
        let prompts = vec![
            Prompt {
                name: "code_review".to_string(),
                title: Some("Code Review Assistant".to_string()),
                description: Some("Generate structured code review prompts for various languages and focus areas".to_string()),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language (e.g., rust, python, javascript)")),
                    PromptArgument::required("focus", Some("Review focus area (general, security, performance, style)")),
                ],
            },
            Prompt {
                name: "documentation_guide".to_string(),
                title: Some("Documentation Writing Guide".to_string()),
                description: Some("Generate documentation writing guidance for various project types".to_string()),
                arguments: vec![
                    PromptArgument::optional("project_type", Some("Type of project (library, binary, web-service)")),
                ],
            },
            Prompt {
                name: "rust_best_practices".to_string(),
                title: Some("Rust Best Practices".to_string()),
                description: Some("Generate Rust-specific development best practices guidance".to_string()),
                arguments: vec![],
            },
        ];
        
        Ok(prompts)
    }

    #[instrument(skip(self))]
    async fn get_prompt(&self, name: &str, arguments: HashMap<String, String>) -> McpResult<(String, Vec<PromptMessage>)> {
        if !self.available_prompts.contains(&name.to_string()) {
            let error_msg = format!("Unknown prompt: {}", name);
            warn!("{}", error_msg);
            return Err(McpError::prompt_not_found(name));
        }

        match self.generate_prompt(name, &arguments) {
            Ok(response) => {
                debug!("Generated prompt '{}' successfully", name);
                Ok(response)
            }
            Err(error) => {
                warn!("Failed to generate prompt '{}': {}", name, error);
                Err(McpError::internal_error(error))
            }
        }
    }
}
