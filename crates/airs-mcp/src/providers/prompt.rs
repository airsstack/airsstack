//! Prompt Provider Trait and Production-ready Implementations
//!
//! This module provides the PromptProvider trait definition and comprehensive
//! prompt provider implementations for common use cases:
//! - Code review prompt templates
//! - Documentation generation prompts
//! - Analysis and research prompts
//!
//! # Architecture
//!
//! The PromptProvider trait defines the interface for providing MCP prompts,
//! while the concrete implementations handle specific prompt categories and templates.

use std::collections::HashMap;

use async_trait::async_trait;
use tracing::{info, instrument};

use crate::integration::{McpError, McpResult};
use crate::protocol::{Content, Prompt, PromptArgument, PromptMessage};

/// Trait for providing MCP prompt functionality
///
/// This trait defines the interface for providing prompts in an MCP server.
/// Prompts represent reusable message templates that can be invoked by MCP clients
/// to generate structured conversations or analysis requests.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::providers::PromptProvider;
/// use airs_mcp::protocol::{Prompt, PromptMessage};
/// use airs_mcp::integration::{McpResult, McpError};
/// use async_trait::async_trait;
/// use std::collections::HashMap;
///
/// struct MyPromptProvider;
///
/// #[async_trait]
/// impl PromptProvider for MyPromptProvider {
///     async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
///         // Return list of available prompts
///         Ok(vec![])
///     }
///
///     async fn get_prompt(
///         &self,
///         name: &str,
///         arguments: HashMap<String, String>,
///     ) -> McpResult<(String, Vec<PromptMessage>)> {
///         // Generate the requested prompt
///         Ok(("Generated prompt".to_string(), vec![]))
///     }
/// }
/// ```
#[async_trait]
pub trait PromptProvider: Send + Sync {
    /// List all available prompts
    ///
    /// Returns a list of all prompts that this provider can generate.
    /// This method is called when clients request the list of available prompts.
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>>;

    /// Get a prompt with the given arguments
    ///
    /// Generates a prompt with the specified name using the provided arguments.
    /// Returns both the prompt content and any associated messages.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the prompt to generate
    /// * `arguments` - Key-value pairs for prompt parameter substitution
    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> McpResult<(String, Vec<PromptMessage>)>;
}

/// Code review prompt provider
#[derive(Debug, Clone)]
pub struct CodeReviewPromptProvider {
    /// Include security review prompts
    include_security: bool,
    /// Include performance review prompts
    include_performance: bool,
    /// Include style review prompts
    include_style: bool,
}

impl CodeReviewPromptProvider {
    /// Create a new code review prompt provider
    pub fn new() -> Self {
        Self {
            include_security: true,
            include_performance: true,
            include_style: true,
        }
    }

    /// Configure which types of reviews to include
    pub fn with_review_types(mut self, security: bool, performance: bool, style: bool) -> Self {
        self.include_security = security;
        self.include_performance = performance;
        self.include_style = style;
        self
    }

    /// Generate code review prompt content
    fn generate_review_prompt(&self, language: &str, code: &str, review_type: &str) -> String {
        match review_type {
            "general" => format!(
                "Please review the following {language} code for overall quality, readability, and best practices:\n\n```{language}\n{code}\n```\n\nFocus on:\n- Code structure and organization\n- Logic and correctness\n- Error handling\n- Documentation and comments\n- Adherence to {language} conventions"
            ),
            "security" => format!(
                "Please perform a security review of the following {language} code:\n\n```{language}\n{code}\n```\n\nFocus on:\n- Input validation and sanitization\n- Authentication and authorization\n- Data exposure risks\n- Injection vulnerabilities\n- Cryptographic usage\n- Memory safety (if applicable)"
            ),
            "performance" => format!(
                "Please review the following {language} code for performance optimization opportunities:\n\n```{language}\n{code}\n```\n\nFocus on:\n- Algorithm efficiency and complexity\n- Memory usage patterns\n- I/O operations optimization\n- Caching opportunities\n- Parallel processing potential\n- Resource cleanup"
            ),
            "style" => format!(
                "Please review the following {language} code for style and formatting:\n\n```{language}\n{code}\n```\n\nFocus on:\n- Consistent formatting and indentation\n- Naming conventions\n- Code organization\n- Comment quality and placement\n- Language-specific style guidelines\n- Readability improvements"
            ),
            _ => format!(
                "Please review the following {language} code:\n\n```{language}\n{code}\n```"
            ),
        }
    }
}

#[async_trait]
impl PromptProvider for CodeReviewPromptProvider {
    #[instrument(level = "debug", skip(self))]
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        info!("Listing code review prompts");

        let mut prompts = vec![Prompt {
            name: "code_review_general".to_string(),
            title: Some("General Code Review".to_string()),
            description: Some(
                "Comprehensive code review focusing on quality and best practices".to_string(),
            ),
            arguments: vec![
                PromptArgument::required(
                    "language",
                    Some("Programming language (e.g., rust, python, javascript)"),
                ),
                PromptArgument::required("code", Some("Code to review")),
                PromptArgument::optional(
                    "context",
                    Some("Additional context about the code's purpose"),
                ),
            ],
        }];

        if self.include_security {
            prompts.push(Prompt {
                name: "code_review_security".to_string(),
                title: Some("Security Code Review".to_string()),
                description: Some(
                    "Security-focused code review checking for vulnerabilities".to_string(),
                ),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to review for security issues")),
                    PromptArgument::optional(
                        "threat_model",
                        Some("Specific threat model or security concerns"),
                    ),
                ],
            });
        }

        if self.include_performance {
            prompts.push(Prompt {
                name: "code_review_performance".to_string(),
                title: Some("Performance Code Review".to_string()),
                description: Some(
                    "Performance-focused code review for optimization opportunities".to_string(),
                ),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to review for performance")),
                    PromptArgument::optional(
                        "performance_goals",
                        Some("Specific performance requirements or constraints"),
                    ),
                ],
            });
        }

        if self.include_style {
            prompts.push(Prompt {
                name: "code_review_style".to_string(),
                title: Some("Style Code Review".to_string()),
                description: Some("Style and formatting code review".to_string()),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to review for style")),
                    PromptArgument::optional(
                        "style_guide",
                        Some("Specific style guide or conventions to follow"),
                    ),
                ],
            });
        }

        info!(prompt_count = prompts.len(), "Code review prompts listed");
        Ok(prompts)
    }

    #[instrument(level = "debug", skip(self), fields(prompt_name = %name))]
    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> McpResult<(String, Vec<PromptMessage>)> {
        info!(prompt_name = %name, "Generating code review prompt");

        let language = arguments
            .get("language")
            .ok_or_else(|| McpError::invalid_request("Missing required argument: language"))?;

        let code = arguments
            .get("code")
            .ok_or_else(|| McpError::invalid_request("Missing required argument: code"))?;

        let (description, prompt_content) = match name {
            "code_review_general" => {
                let context = arguments.get("context").map(|s| s.as_str()).unwrap_or("");
                let mut content = self.generate_review_prompt(language, code, "general");

                if !context.is_empty() {
                    content.push_str(&format!("\n\nAdditional context: {context}"));
                }

                ("General code review prompt", content)
            }
            "code_review_security" => {
                let threat_model = arguments
                    .get("threat_model")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let mut content = self.generate_review_prompt(language, code, "security");

                if !threat_model.is_empty() {
                    content.push_str(&format!("\n\nThreat model considerations: {threat_model}"));
                }

                ("Security-focused code review prompt", content)
            }
            "code_review_performance" => {
                let performance_goals = arguments
                    .get("performance_goals")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let mut content = self.generate_review_prompt(language, code, "performance");

                if !performance_goals.is_empty() {
                    content.push_str(&format!(
                        "\n\nPerformance requirements: {performance_goals}"
                    ));
                }

                ("Performance-focused code review prompt", content)
            }
            "code_review_style" => {
                let style_guide = arguments
                    .get("style_guide")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let mut content = self.generate_review_prompt(language, code, "style");

                if !style_guide.is_empty() {
                    content.push_str(&format!("\n\nStyle guide: {style_guide}"));
                }

                ("Style and formatting code review prompt", content)
            }
            _ => return Err(McpError::prompt_not_found(name)),
        };

        let messages = vec![PromptMessage::user(Content::text(prompt_content))];

        info!(
            prompt_name = %name,
            language = %language,
            code_length = code.len(),
            "Code review prompt generated successfully"
        );

        Ok((description.to_string(), messages))
    }
}

/// Documentation generation prompt provider
#[derive(Debug, Clone)]
pub struct DocumentationPromptProvider {
    /// Include API documentation prompts
    include_api: bool,
    /// Include tutorial prompts
    include_tutorials: bool,
    /// Include README prompts
    include_readme: bool,
}

impl DocumentationPromptProvider {
    /// Create a new documentation prompt provider
    pub fn new() -> Self {
        Self {
            include_api: true,
            include_tutorials: true,
            include_readme: true,
        }
    }

    /// Configure which types of documentation to include
    pub fn with_doc_types(mut self, api: bool, tutorials: bool, readme: bool) -> Self {
        self.include_api = api;
        self.include_tutorials = tutorials;
        self.include_readme = readme;
        self
    }
}

#[async_trait]
impl PromptProvider for DocumentationPromptProvider {
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        let mut prompts = Vec::new();

        if self.include_api {
            prompts.push(Prompt {
                name: "generate_api_docs".to_string(),
                title: Some("Generate API Documentation".to_string()),
                description: Some("Generate comprehensive API documentation from code".to_string()),
                arguments: vec![
                    PromptArgument::required("language", Some("Programming language")),
                    PromptArgument::required("code", Some("Code to document")),
                    PromptArgument::optional(
                        "format",
                        Some("Documentation format (markdown, rst, html)"),
                    ),
                    PromptArgument::optional(
                        "audience",
                        Some("Target audience (developers, users, maintainers)"),
                    ),
                ],
            });
        }

        if self.include_tutorials {
            prompts.push(Prompt {
                name: "generate_tutorial".to_string(),
                title: Some("Generate Tutorial".to_string()),
                description: Some("Generate step-by-step tutorial or guide".to_string()),
                arguments: vec![
                    PromptArgument::required("topic", Some("Tutorial topic or subject")),
                    PromptArgument::required(
                        "level",
                        Some("Difficulty level (beginner, intermediate, advanced)"),
                    ),
                    PromptArgument::optional(
                        "format",
                        Some("Tutorial format (markdown, video, interactive)"),
                    ),
                    PromptArgument::optional("duration", Some("Expected tutorial duration")),
                ],
            });
        }

        if self.include_readme {
            prompts.push(Prompt {
                name: "generate_readme".to_string(),
                title: Some("Generate README".to_string()),
                description: Some("Generate comprehensive README documentation".to_string()),
                arguments: vec![
                    PromptArgument::required("project_name", Some("Project name")),
                    PromptArgument::required("description", Some("Project description")),
                    PromptArgument::optional("features", Some("Key features or capabilities")),
                    PromptArgument::optional("installation", Some("Installation instructions")),
                    PromptArgument::optional("usage", Some("Usage examples")),
                ],
            });
        }

        Ok(prompts)
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> McpResult<(String, Vec<PromptMessage>)> {
        match name {
            "generate_api_docs" => {
                let language = arguments.get("language").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: language")
                })?;
                let code = arguments
                    .get("code")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: code"))?;
                let format = arguments
                    .get("format")
                    .map(|s| s.as_str())
                    .unwrap_or("markdown");
                let audience = arguments
                    .get("audience")
                    .map(|s| s.as_str())
                    .unwrap_or("developers");

                let prompt = format!(
                    "Generate comprehensive API documentation for the following {language} code in {format} format, targeted at {audience}:\n\n```{language}\n{code}\n```\n\nInclude:\n- Function/method signatures and parameters\n- Return values and types\n- Usage examples\n- Error conditions\n- Notes about performance or limitations"
                );

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("API documentation generation prompt".to_string(), messages))
            }
            "generate_tutorial" => {
                let topic = arguments
                    .get("topic")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: topic"))?;
                let level = arguments
                    .get("level")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: level"))?;
                let format = arguments
                    .get("format")
                    .map(|s| s.as_str())
                    .unwrap_or("markdown");
                let duration = arguments
                    .get("duration")
                    .map(|s| s.as_str())
                    .unwrap_or("not specified");

                let prompt = format!(
                    "Create a comprehensive {level} level tutorial about '{topic}' in {format} format.\n\nTutorial requirements:\n- Target duration: {duration}\n- Include clear learning objectives\n- Provide step-by-step instructions\n- Include practical examples and exercises\n- Add troubleshooting section\n- Conclude with next steps or further resources"
                );

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("Tutorial generation prompt".to_string(), messages))
            }
            "generate_readme" => {
                let project_name = arguments.get("project_name").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: project_name")
                })?;
                let description = arguments.get("description").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: description")
                })?;
                let features = arguments.get("features").map(|s| s.as_str()).unwrap_or("");
                let installation = arguments
                    .get("installation")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let usage = arguments.get("usage").map(|s| s.as_str()).unwrap_or("");

                let mut prompt = format!(
                    "Generate a comprehensive README.md file for the project '{project_name}' with the description: '{description}'\n\n"
                );

                prompt.push_str("Include the following sections:\n");
                prompt.push_str("- Project title and description\n");
                prompt.push_str("- Table of contents\n");

                if !features.is_empty() {
                    prompt.push_str(&format!("- Features: {features}\n"));
                } else {
                    prompt.push_str("- Key features and capabilities\n");
                }

                if !installation.is_empty() {
                    prompt.push_str(&format!("- Installation: {installation}\n"));
                } else {
                    prompt.push_str("- Installation instructions\n");
                }

                if !usage.is_empty() {
                    prompt.push_str(&format!("- Usage examples: {usage}\n"));
                } else {
                    prompt.push_str("- Usage examples and quick start\n");
                }

                prompt.push_str("- Contributing guidelines\n");
                prompt.push_str("- License information\n");
                prompt.push_str("- Contact/support information\n");

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("README generation prompt".to_string(), messages))
            }
            _ => Err(McpError::prompt_not_found(name)),
        }
    }
}

/// Analysis and research prompt provider
#[derive(Debug, Clone)]
pub struct AnalysisPromptProvider {
    /// Include data analysis prompts
    include_data_analysis: bool,
    /// Include research prompts
    include_research: bool,
    /// Include comparison prompts
    include_comparison: bool,
}

impl AnalysisPromptProvider {
    /// Create a new analysis prompt provider
    pub fn new() -> Self {
        Self {
            include_data_analysis: true,
            include_research: true,
            include_comparison: true,
        }
    }

    /// Configure which types of analysis to include
    pub fn with_analysis_types(mut self, data: bool, research: bool, comparison: bool) -> Self {
        self.include_data_analysis = data;
        self.include_research = research;
        self.include_comparison = comparison;
        self
    }
}

#[async_trait]
impl PromptProvider for AnalysisPromptProvider {
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        let mut prompts = Vec::new();

        if self.include_data_analysis {
            prompts.push(Prompt {
                name: "analyze_data".to_string(),
                title: Some("Data Analysis".to_string()),
                description: Some("Analyze data and provide insights".to_string()),
                arguments: vec![
                    PromptArgument::required(
                        "data",
                        Some("Data to analyze (JSON, CSV, or description)"),
                    ),
                    PromptArgument::required(
                        "analysis_type",
                        Some(
                            "Type of analysis (descriptive, diagnostic, predictive, prescriptive)",
                        ),
                    ),
                    PromptArgument::optional(
                        "metrics",
                        Some("Specific metrics or KPIs to focus on"),
                    ),
                    PromptArgument::optional("context", Some("Business or domain context")),
                ],
            });
        }

        if self.include_research {
            prompts.push(Prompt {
                name: "research_topic".to_string(),
                title: Some("Topic Research".to_string()),
                description: Some("Conduct comprehensive research on a topic".to_string()),
                arguments: vec![
                    PromptArgument::required("topic", Some("Research topic or question")),
                    PromptArgument::required(
                        "scope",
                        Some("Research scope (broad, focused, specific)"),
                    ),
                    PromptArgument::optional("sources", Some("Preferred types of sources")),
                    PromptArgument::optional(
                        "depth",
                        Some("Research depth (overview, detailed, comprehensive)"),
                    ),
                ],
            });
        }

        if self.include_comparison {
            prompts.push(Prompt {
                name: "compare_options".to_string(),
                title: Some("Option Comparison".to_string()),
                description: Some("Compare multiple options or alternatives".to_string()),
                arguments: vec![
                    PromptArgument::required(
                        "options",
                        Some("Options to compare (comma-separated)"),
                    ),
                    PromptArgument::required("criteria", Some("Comparison criteria")),
                    PromptArgument::optional("weights", Some("Criteria weights or priorities")),
                    PromptArgument::optional("context", Some("Decision context or constraints")),
                ],
            });
        }

        Ok(prompts)
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> McpResult<(String, Vec<PromptMessage>)> {
        match name {
            "analyze_data" => {
                let data = arguments
                    .get("data")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: data"))?;
                let analysis_type = arguments.get("analysis_type").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: analysis_type")
                })?;
                let metrics = arguments.get("metrics").map(|s| s.as_str()).unwrap_or("");
                let context = arguments.get("context").map(|s| s.as_str()).unwrap_or("");

                let mut prompt = format!(
                    "Perform a {analysis_type} analysis of the following data:\n\n{data}\n\n"
                );

                prompt.push_str("Please provide:\n");
                prompt.push_str("- Key findings and insights\n");
                prompt.push_str("- Statistical summary (if applicable)\n");
                prompt.push_str("- Trends and patterns\n");
                prompt.push_str("- Anomalies or outliers\n");
                prompt.push_str("- Recommendations based on findings\n");

                if !metrics.is_empty() {
                    prompt.push_str(&format!("\nFocus on these specific metrics: {metrics}\n"));
                }

                if !context.is_empty() {
                    prompt.push_str(&format!("\nContext: {context}\n"));
                }

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("Data analysis prompt".to_string(), messages))
            }
            "research_topic" => {
                let topic = arguments
                    .get("topic")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: topic"))?;
                let scope = arguments
                    .get("scope")
                    .ok_or_else(|| McpError::invalid_request("Missing required argument: scope"))?;
                let sources = arguments
                    .get("sources")
                    .map(|s| s.as_str())
                    .unwrap_or("academic and industry sources");
                let depth = arguments
                    .get("depth")
                    .map(|s| s.as_str())
                    .unwrap_or("detailed");

                let prompt = format!(
                    "Conduct {depth} research on the topic: '{topic}' with {scope} scope.\n\nResearch guidelines:\n- Use {sources} as primary sources\n- Provide {depth} analysis\n- Include current trends and developments\n- Cite key findings and statistics\n- Identify knowledge gaps or controversies\n- Suggest areas for further investigation\n\nStructure your research with clear sections and conclusions."
                );

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("Topic research prompt".to_string(), messages))
            }
            "compare_options" => {
                let options = arguments.get("options").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: options")
                })?;
                let criteria = arguments.get("criteria").ok_or_else(|| {
                    McpError::invalid_request("Missing required argument: criteria")
                })?;
                let weights = arguments.get("weights").map(|s| s.as_str()).unwrap_or("");
                let context = arguments.get("context").map(|s| s.as_str()).unwrap_or("");

                let mut prompt = format!(
                    "Compare the following options: {options}\n\nUsing these criteria: {criteria}\n\n"
                );

                prompt.push_str("Provide a comprehensive comparison including:\n");
                prompt.push_str("- Detailed evaluation against each criterion\n");
                prompt.push_str("- Pros and cons for each option\n");
                prompt.push_str("- Scoring or ranking\n");
                prompt.push_str("- Risk assessment\n");
                prompt.push_str("- Final recommendation with rationale\n");

                if !weights.is_empty() {
                    prompt.push_str(&format!("\nCriteria weights: {weights}\n"));
                }

                if !context.is_empty() {
                    prompt.push_str(&format!("\nDecision context: {context}\n"));
                }

                let messages = vec![PromptMessage::user(Content::text(prompt))];
                Ok(("Option comparison prompt".to_string(), messages))
            }
            _ => Err(McpError::prompt_not_found(name)),
        }
    }
}

impl Default for CodeReviewPromptProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DocumentationPromptProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnalysisPromptProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_review_provider() {
        let provider = CodeReviewPromptProvider::new();
        let prompts = provider.list_prompts().await.unwrap();
        assert!(!prompts.is_empty());

        let mut args = HashMap::new();
        args.insert("language".to_string(), "rust".to_string());
        args.insert(
            "code".to_string(),
            "fn main() { println!(\"Hello\"); }".to_string(),
        );

        let (desc, messages) = provider
            .get_prompt("code_review_general", args)
            .await
            .unwrap();
        assert!(!desc.is_empty());
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_documentation_provider() {
        let provider = DocumentationPromptProvider::new();
        let prompts = provider.list_prompts().await.unwrap();
        assert!(!prompts.is_empty());

        let mut args = HashMap::new();
        args.insert("project_name".to_string(), "Test Project".to_string());
        args.insert("description".to_string(), "A test project".to_string());

        let (desc, messages) = provider.get_prompt("generate_readme", args).await.unwrap();
        assert!(!desc.is_empty());
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_analysis_provider() {
        let provider = AnalysisPromptProvider::new();
        let prompts = provider.list_prompts().await.unwrap();
        assert!(!prompts.is_empty());

        let mut args = HashMap::new();
        args.insert("data".to_string(), "1,2,3,4,5".to_string());
        args.insert("analysis_type".to_string(), "descriptive".to_string());

        let (desc, messages) = provider.get_prompt("analyze_data", args).await.unwrap();
        assert!(!desc.is_empty());
        assert_eq!(messages.len(), 1);
    }
}
