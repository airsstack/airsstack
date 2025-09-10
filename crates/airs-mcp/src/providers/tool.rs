//! Tool Provider Trait and Production-ready Implementations
//!
//! This module provides the ToolProvider trait definition and comprehensive
//! tool provider implementations for common use cases:
//! - Mathematical operations and calculations
//! - System operations and utilities
//! - Text processing and analysis tools
//!
//! # Architecture
//!
//! The ToolProvider trait defines the interface for providing MCP tools,
//! while the concrete implementations handle specific tool categories and capabilities.

use std::process::Stdio;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::{info, instrument};

use crate::integration::{McpError, McpResult};
use crate::protocol::{Content, Tool};

/// Trait for providing MCP tool functionality
///
/// This trait defines the interface for providing tools in an MCP server.
/// Tools represent executable functionality that can be invoked by MCP clients,
/// such as mathematical operations, system commands, or custom business logic.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::providers::ToolProvider;
/// use airs_mcp::protocol::{Tool, Content};
/// use airs_mcp::integration::{McpResult, McpError};
/// use async_trait::async_trait;
/// use serde_json::Value;
///
/// struct MyToolProvider;
///
/// #[async_trait]
/// impl ToolProvider for MyToolProvider {
///     async fn list_tools(&self) -> McpResult<Vec<Tool>> {
///         // Return list of available tools
///         Ok(vec![])
///     }
///
///     async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
///         // Execute the requested tool
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait ToolProvider: Send + Sync {
    /// List all available tools
    ///
    /// Returns a list of all tools that this provider can execute.
    /// This method is called when clients request the list of available tools.
    async fn list_tools(&self) -> McpResult<Vec<Tool>>;

    /// Execute a tool with the given arguments
    ///
    /// Executes the specified tool with the provided arguments and returns the result.
    /// The tool execution should be isolated and safe, with proper error handling.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to execute
    /// * `arguments` - JSON value containing the tool's input arguments
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>>;
}

/// Mathematical operations tool provider
#[derive(Debug, Clone)]
pub struct MathToolProvider {
    /// Precision for floating point operations
    precision: u32,
    /// Whether to enable advanced mathematical functions
    advanced_functions: bool,
}

impl MathToolProvider {
    /// Create a new mathematical tool provider
    pub fn new() -> Self {
        Self {
            precision: 10, // Default to 10 decimal places
            advanced_functions: true,
        }
    }

    /// Set floating point precision
    pub fn with_precision(mut self, precision: u32) -> Self {
        self.precision = precision;
        self
    }

    /// Enable or disable advanced mathematical functions
    pub fn with_advanced_functions(mut self, enabled: bool) -> Self {
        self.advanced_functions = enabled;
        self
    }

    /// Parse a number from JSON value
    fn parse_number(&self, value: &Value, name: &str) -> McpResult<f64> {
        value.as_f64().ok_or_else(|| {
            McpError::invalid_request(format!("Parameter '{name}' must be a number"))
        })
    }

    /// Format a number result with specified precision
    fn format_result(&self, value: f64) -> String {
        format!("{:.prec$}", value, prec = self.precision as usize)
    }
}

#[async_trait]
impl ToolProvider for MathToolProvider {
    #[instrument(level = "debug", skip(self))]
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        info!("Listing mathematical tools");

        let mut tools = vec![
            Tool {
                name: "add".to_string(),
                description: Some("Add two or more numbers together".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "numbers": {
                            "type": "array",
                            "items": {"type": "number"},
                            "minItems": 2,
                            "description": "Numbers to add together"
                        }
                    },
                    "required": ["numbers"]
                }),
            },
            Tool {
                name: "subtract".to_string(),
                description: Some("Subtract second number from first".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "Number to subtract from"},
                        "b": {"type": "number", "description": "Number to subtract"}
                    },
                    "required": ["a", "b"]
                }),
            },
            Tool {
                name: "multiply".to_string(),
                description: Some("Multiply two or more numbers together".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "numbers": {
                            "type": "array",
                            "items": {"type": "number"},
                            "minItems": 2,
                            "description": "Numbers to multiply together"
                        }
                    },
                    "required": ["numbers"]
                }),
            },
            Tool {
                name: "divide".to_string(),
                description: Some("Divide first number by second".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "Dividend"},
                        "b": {"type": "number", "description": "Divisor"}
                    },
                    "required": ["a", "b"]
                }),
            },
            Tool {
                name: "power".to_string(),
                description: Some("Raise number to a power".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "base": {"type": "number", "description": "Base number"},
                        "exponent": {"type": "number", "description": "Exponent"}
                    },
                    "required": ["base", "exponent"]
                }),
            },
        ];

        if self.advanced_functions {
            tools.extend(vec![
                Tool {
                    name: "sqrt".to_string(),
                    description: Some("Calculate square root of a number".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "number": {"type": "number", "minimum": 0, "description": "Number to find square root of"}
                        },
                        "required": ["number"]
                    }),
                },
                Tool {
                    name: "sin".to_string(),
                    description: Some("Calculate sine of an angle in radians".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "angle": {"type": "number", "description": "Angle in radians"}
                        },
                        "required": ["angle"]
                    }),
                },
                Tool {
                    name: "cos".to_string(),
                    description: Some("Calculate cosine of an angle in radians".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "angle": {"type": "number", "description": "Angle in radians"}
                        },
                        "required": ["angle"]
                    }),
                },
                Tool {
                    name: "log".to_string(),
                    description: Some("Calculate natural logarithm of a number".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "number": {"type": "number", "minimum": 0, "exclusiveMinimum": true, "description": "Number to find logarithm of"}
                        },
                        "required": ["number"]
                    }),
                },
                Tool {
                    name: "factorial".to_string(),
                    description: Some("Calculate factorial of a non-negative integer".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "number": {"type": "integer", "minimum": 0, "maximum": 20, "description": "Non-negative integer (max 20)"}
                        },
                        "required": ["number"]
                    }),
                },
            ]);
        }

        info!(
            tool_count = tools.len(),
            advanced = self.advanced_functions,
            "Mathematical tools listed"
        );
        Ok(tools)
    }

    #[instrument(level = "debug", skip(self), fields(tool_name = %name))]
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        info!(tool_name = %name, arguments = %arguments, "Executing mathematical tool");

        let result = match name {
            "add" => {
                let numbers = arguments
                    .get("numbers")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        McpError::invalid_request("Parameter 'numbers' must be an array")
                    })?;

                let mut sum = 0.0;
                for (i, num) in numbers.iter().enumerate() {
                    sum += self.parse_number(num, &format!("numbers[{i}]"))?;
                }

                json!({
                    "operation": "addition",
                    "result": self.format_result(sum),
                    "count": numbers.len()
                })
            }
            "subtract" => {
                let a = self.parse_number(arguments.get("a").unwrap(), "a")?;
                let b = self.parse_number(arguments.get("b").unwrap(), "b")?;
                let result = a - b;

                json!({
                    "operation": "subtraction",
                    "result": self.format_result(result),
                    "operands": [a, b]
                })
            }
            "multiply" => {
                let numbers = arguments
                    .get("numbers")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        McpError::invalid_request("Parameter 'numbers' must be an array")
                    })?;

                let mut product = 1.0;
                for (i, num) in numbers.iter().enumerate() {
                    product *= self.parse_number(num, &format!("numbers[{i}]"))?;
                }

                json!({
                    "operation": "multiplication",
                    "result": self.format_result(product),
                    "count": numbers.len()
                })
            }
            "divide" => {
                let a = self.parse_number(arguments.get("a").unwrap(), "a")?;
                let b = self.parse_number(arguments.get("b").unwrap(), "b")?;

                if b == 0.0 {
                    return Err(McpError::invalid_request("Division by zero"));
                }

                let result = a / b;
                json!({
                    "operation": "division",
                    "result": self.format_result(result),
                    "dividend": a,
                    "divisor": b
                })
            }
            "power" => {
                let base = self.parse_number(arguments.get("base").unwrap(), "base")?;
                let exponent = self.parse_number(arguments.get("exponent").unwrap(), "exponent")?;
                let result = base.powf(exponent);

                json!({
                    "operation": "power",
                    "result": self.format_result(result),
                    "base": base,
                    "exponent": exponent
                })
            }
            "sqrt" if self.advanced_functions => {
                let number = self.parse_number(arguments.get("number").unwrap(), "number")?;

                if number < 0.0 {
                    return Err(McpError::invalid_request(
                        "Cannot calculate square root of negative number",
                    ));
                }

                let result = number.sqrt();
                json!({
                    "operation": "square_root",
                    "result": self.format_result(result),
                    "input": number
                })
            }
            "sin" if self.advanced_functions => {
                let angle = self.parse_number(arguments.get("angle").unwrap(), "angle")?;
                let result = angle.sin();

                json!({
                    "operation": "sine",
                    "result": self.format_result(result),
                    "angle_radians": angle
                })
            }
            "cos" if self.advanced_functions => {
                let angle = self.parse_number(arguments.get("angle").unwrap(), "angle")?;
                let result = angle.cos();

                json!({
                    "operation": "cosine",
                    "result": self.format_result(result),
                    "angle_radians": angle
                })
            }
            "log" if self.advanced_functions => {
                let number = self.parse_number(arguments.get("number").unwrap(), "number")?;

                if number <= 0.0 {
                    return Err(McpError::invalid_request(
                        "Logarithm requires positive number",
                    ));
                }

                let result = number.ln();
                json!({
                    "operation": "natural_logarithm",
                    "result": self.format_result(result),
                    "input": number
                })
            }
            "factorial" if self.advanced_functions => {
                let number = arguments
                    .get("number")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        McpError::invalid_request("Parameter 'number' must be an integer")
                    })?;

                if !(0..=20).contains(&number) {
                    return Err(McpError::invalid_request(
                        "Factorial requires non-negative integer <= 20",
                    ));
                }

                let mut result = 1u64;
                for i in 2..=number as u64 {
                    result *= i;
                }

                json!({
                    "operation": "factorial",
                    "result": result,
                    "input": number
                })
            }
            _ => return Err(McpError::tool_not_found(name)),
        };

        Ok(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .map_err(|e| McpError::internal_error(format!("JSON serialization error: {e}")))?,
        )])
    }
}

/// System operations tool provider
#[derive(Debug, Clone)]
pub struct SystemToolProvider {
    /// Whether to allow potentially dangerous operations
    allow_dangerous: bool,
    /// Maximum command execution time in seconds
    timeout_secs: u64,
}

impl SystemToolProvider {
    /// Create a new system tool provider
    pub fn new() -> Self {
        Self {
            allow_dangerous: false,
            timeout_secs: 30,
        }
    }

    /// Allow potentially dangerous operations (use with caution)
    pub fn with_dangerous_operations(mut self, allow: bool) -> Self {
        self.allow_dangerous = allow;
        self
    }

    /// Set command timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

#[async_trait]
impl ToolProvider for SystemToolProvider {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        let mut tools = vec![
            Tool {
                name: "ping".to_string(),
                description: Some("Ping a network host to test connectivity".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "host": {"type": "string", "description": "Host to ping"},
                        "count": {"type": "integer", "minimum": 1, "maximum": 10, "default": 4, "description": "Number of ping packets"}
                    },
                    "required": ["host"]
                }),
            },
            Tool {
                name: "echo".to_string(),
                description: Some("Echo back the provided text".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to echo"}
                    },
                    "required": ["text"]
                }),
            },
            Tool {
                name: "date".to_string(),
                description: Some("Get the current date and time".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "description": "Date format (optional)", "default": "ISO8601"}
                    }
                }),
            },
        ];

        if self.allow_dangerous {
            tools.push(Tool {
                name: "execute".to_string(),
                description: Some("Execute a system command (DANGEROUS - use with caution)".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Command to execute"},
                        "args": {"type": "array", "items": {"type": "string"}, "description": "Command arguments"}
                    },
                    "required": ["command"]
                }),
            });
        }

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        match name {
            "ping" => {
                let host = arguments
                    .get("host")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_request("Missing or invalid 'host' parameter")
                    })?;

                let count = arguments.get("count").and_then(|v| v.as_i64()).unwrap_or(4);

                let output = Command::new("ping")
                    .arg("-c")
                    .arg(count.to_string())
                    .arg(host)
                    .output()
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Failed to execute ping: {e}"))
                    })?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = json!({
                    "command": "ping",
                    "host": host,
                    "count": count,
                    "success": output.status.success(),
                    "stdout": stdout,
                    "stderr": stderr
                });

                Ok(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap(),
                )])
            }
            "echo" => {
                let text = arguments
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_request("Missing or invalid 'text' parameter")
                    })?;

                let result = json!({
                    "command": "echo",
                    "text": text,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                Ok(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap(),
                )])
            }
            "date" => {
                let format = arguments
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ISO8601");

                let now = chrono::Utc::now();
                let formatted = match format {
                    "ISO8601" => now.to_rfc3339(),
                    "unix" => now.timestamp().to_string(),
                    "rfc2822" => now.to_rfc2822(),
                    _ => now.to_rfc3339(), // Default fallback
                };

                let result = json!({
                    "command": "date",
                    "format": format,
                    "timestamp": formatted,
                    "unix_timestamp": now.timestamp()
                });

                Ok(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap(),
                )])
            }
            "execute" if self.allow_dangerous => {
                let command = arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_request("Missing or invalid 'command' parameter")
                    })?;

                let args = arguments
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();

                let mut cmd = Command::new(command);
                cmd.args(&args);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let output = tokio::time::timeout(
                    std::time::Duration::from_secs(self.timeout_secs),
                    cmd.output(),
                )
                .await
                .map_err(|_| McpError::internal_error("Command execution timeout"))?
                .map_err(|e| McpError::internal_error(format!("Failed to execute command: {e}")))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = json!({
                    "command": command,
                    "args": args,
                    "success": output.status.success(),
                    "exit_code": output.status.code(),
                    "stdout": stdout,
                    "stderr": stderr
                });

                Ok(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap(),
                )])
            }
            _ => Err(McpError::tool_not_found(name)),
        }
    }
}

/// Text processing tool provider
#[derive(Debug, Clone)]
pub struct TextToolProvider;

impl TextToolProvider {
    /// Create a new text processing tool provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolProvider for TextToolProvider {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(vec![
            Tool {
                name: "word_count".to_string(),
                description: Some("Count words, characters, and lines in text".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"}
                    },
                    "required": ["text"]
                }),
            },
            Tool {
                name: "reverse_text".to_string(),
                description: Some("Reverse the order of characters in text".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to reverse"}
                    },
                    "required": ["text"]
                }),
            },
            Tool {
                name: "to_uppercase".to_string(),
                description: Some("Convert text to uppercase".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to convert"}
                    },
                    "required": ["text"]
                }),
            },
            Tool {
                name: "to_lowercase".to_string(),
                description: Some("Convert text to lowercase".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to convert"}
                    },
                    "required": ["text"]
                }),
            },
            Tool {
                name: "extract_urls".to_string(),
                description: Some("Extract URLs from text".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to search for URLs"}
                    },
                    "required": ["text"]
                }),
            },
        ])
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing or invalid 'text' parameter"))?;

        let result = match name {
            "word_count" => {
                let words = text.split_whitespace().count();
                let chars = text.chars().count();
                let chars_no_spaces = text.chars().filter(|c| !c.is_whitespace()).count();
                let lines = text.lines().count();

                json!({
                    "operation": "word_count",
                    "words": words,
                    "characters": chars,
                    "characters_no_spaces": chars_no_spaces,
                    "lines": lines,
                    "bytes": text.len()
                })
            }
            "reverse_text" => {
                let reversed: String = text.chars().rev().collect();
                json!({
                    "operation": "reverse_text",
                    "original": text,
                    "reversed": reversed
                })
            }
            "to_uppercase" => {
                json!({
                    "operation": "to_uppercase",
                    "original": text,
                    "result": text.to_uppercase()
                })
            }
            "to_lowercase" => {
                json!({
                    "operation": "to_lowercase",
                    "original": text,
                    "result": text.to_lowercase()
                })
            }
            "extract_urls" => {
                // Simple URL regex pattern
                let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
                let urls: Vec<&str> = url_regex.find_iter(text).map(|m| m.as_str()).collect();

                json!({
                    "operation": "extract_urls",
                    "urls": urls,
                    "count": urls.len()
                })
            }
            _ => return Err(McpError::tool_not_found(name)),
        };

        Ok(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap(),
        )])
    }
}

impl Default for MathToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TextToolProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_math_provider_add() {
        let provider = MathToolProvider::new();
        let args = json!({"numbers": [1, 2, 3, 4, 5]});
        let result = provider.call_tool("add", args).await.unwrap();
        assert_eq!(result.len(), 1);
        // The result should contain the sum
        if let Content::Text { text, .. } = &result[0] {
            assert!(text.contains("15"));
        }
    }

    #[tokio::test]
    async fn test_math_provider_divide_by_zero() {
        let provider = MathToolProvider::new();
        let args = json!({"a": 10, "b": 0});
        let result = provider.call_tool("divide", args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_text_provider_word_count() {
        let provider = TextToolProvider::new();
        let args = json!({"text": "Hello world from MCP"});
        let result = provider.call_tool("word_count", args).await.unwrap();
        assert_eq!(result.len(), 1);
        if let Content::Text { text, .. } = &result[0] {
            assert!(text.contains("\"words\": 4"));
        }
    }

    #[tokio::test]
    async fn test_system_provider_echo() {
        let provider = SystemToolProvider::new();
        let args = json!({"text": "Hello MCP"});
        let result = provider.call_tool("echo", args).await.unwrap();
        assert_eq!(result.len(), 1);
        if let Content::Text { text, .. } = &result[0] {
            assert!(text.contains("Hello MCP"));
        }
    }
}
