//! Calculator tool provider with mathematical operations
//!
//! Provides safe mathematical calculations and utility functions through MCP tools.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, instrument, warn};

// Layer 3: Internal module imports
use airs_mcp::integration::mcp::{ToolProvider, McpError, McpResult};
use airs_mcp::shared::protocol::{Tool, Content};

/// Calculator tool provider with mathematical operations
#[derive(Debug, Clone)]
pub struct CalculatorToolProvider {
    supported_operations: Vec<String>,
}

impl CalculatorToolProvider {
    /// Create new calculator tool provider
    pub fn new() -> Self {
        Self {
            supported_operations: vec![
                "add".to_string(),
                "subtract".to_string(),
                "multiply".to_string(),
                "divide".to_string(),
                "power".to_string(),
                "sqrt".to_string(),
                "factorial".to_string(),
                "random".to_string(),
            ],
        }
    }

    /// Perform mathematical operation
    #[instrument(skip(self))]
    fn calculate(&self, operation: &str, args: &HashMap<String, Value>) -> Result<Content, String> {
        match operation {
            "add" => {
                let a = self.get_number_arg(args, "a")?;
                let b = self.get_number_arg(args, "b")?;
                let result = a + b;
                Ok(Content::text(format!("{} + {} = {}", a, b, result)))
            }
            "subtract" => {
                let a = self.get_number_arg(args, "a")?;
                let b = self.get_number_arg(args, "b")?;
                let result = a - b;
                Ok(Content::text(format!("{} - {} = {}", a, b, result)))
            }
            "multiply" => {
                let a = self.get_number_arg(args, "a")?;
                let b = self.get_number_arg(args, "b")?;
                let result = a * b;
                Ok(Content::text(format!("{} × {} = {}", a, b, result)))
            }
            "divide" => {
                let a = self.get_number_arg(args, "a")?;
                let b = self.get_number_arg(args, "b")?;
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                let result = a / b;
                Ok(Content::text(format!("{} ÷ {} = {}", a, b, result)))
            }
            "power" => {
                let base = self.get_number_arg(args, "base")?;
                let exponent = self.get_number_arg(args, "exponent")?;
                let result = base.powf(exponent);
                Ok(Content::text(format!("{}^{} = {}", base, exponent, result)))
            }
            "sqrt" => {
                let number = self.get_number_arg(args, "number")?;
                if number < 0.0 {
                    return Err("Cannot calculate square root of negative number".to_string());
                }
                let result = number.sqrt();
                Ok(Content::text(format!("√{} = {}", number, result)))
            }
            "factorial" => {
                let number = self.get_number_arg(args, "number")?;
                if number < 0.0 || number.fract() != 0.0 {
                    return Err("Factorial only defined for non-negative integers".to_string());
                }
                let n = number as u64;
                if n > 20 {
                    return Err("Factorial too large (max 20)".to_string());
                }
                let result = (1..=n).fold(1u64, |acc, x| acc * x) as f64;
                Ok(Content::text(format!("{}! = {}", n, result)))
            }
            "random" => {
                let min = self.get_optional_number_arg(args, "min").unwrap_or(0.0);
                let max = self.get_optional_number_arg(args, "max").unwrap_or(1.0);
                if min >= max {
                    return Err("Minimum must be less than maximum".to_string());
                }
                // Simple pseudo-random using current time
                let seed = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
                let result = min + ((seed % 1000) as f64 / 1000.0) * (max - min);
                Ok(Content::text(format!("random({}, {}) = {}", min, max, result)))
            }
            _ => Err(format!("Unknown operation: {}", operation)),
        }
    }

    /// Extract required number argument
    fn get_number_arg(&self, args: &HashMap<String, Value>, key: &str) -> Result<f64, String> {
        args.get(key)
            .ok_or_else(|| format!("Missing required argument: {}", key))?
            .as_f64()
            .ok_or_else(|| format!("Argument '{}' must be a number", key))
    }

    /// Extract optional number argument
    fn get_optional_number_arg(&self, args: &HashMap<String, Value>, key: &str) -> Option<f64> {
        args.get(key)?.as_f64()
    }
}

#[async_trait]
impl ToolProvider for CalculatorToolProvider {
    #[instrument(skip(self))]
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        let tools = vec![
            Tool::new(
                "add",
                Some("Add Numbers"),
                Some("Add two numbers"),
                json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }),
            ),
            Tool::new(
                "subtract",
                Some("Subtract Numbers"),
                Some("Subtract two numbers"),
                json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number to subtract"}
                    },
                    "required": ["a", "b"]
                }),
            ),
            Tool::new(
                "multiply",
                Some("Multiply Numbers"),
                Some("Multiply two numbers"),
                json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }),
            ),
            Tool::new(
                "divide",
                Some("Divide Numbers"),
                Some("Divide two numbers"),
                json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "Dividend"},
                        "b": {"type": "number", "description": "Divisor (cannot be zero)"}
                    },
                    "required": ["a", "b"]
                }),
            ),
            Tool::new(
                "power",
                Some("Power Operation"),
                Some("Raise a number to a power"),
                json!({
                    "type": "object",
                    "properties": {
                        "base": {"type": "number", "description": "Base number"},
                        "exponent": {"type": "number", "description": "Exponent"}
                    },
                    "required": ["base", "exponent"]
                }),
            ),
            Tool::new(
                "sqrt",
                Some("Square Root"),
                Some("Calculate square root of a number"),
                json!({
                    "type": "object",
                    "properties": {
                        "number": {"type": "number", "description": "Number (must be non-negative)"}
                    },
                    "required": ["number"]
                }),
            ),
            Tool::new(
                "factorial",
                Some("Factorial"),
                Some("Calculate factorial of a non-negative integer (max 20)"),
                json!({
                    "type": "object",
                    "properties": {
                        "number": {"type": "number", "description": "Non-negative integer (max 20)"}
                    },
                    "required": ["number"]
                }),
            ),
            Tool::new(
                "random",
                Some("Random Number"),
                Some("Generate a random number within a range"),
                json!({
                    "type": "object",
                    "properties": {
                        "min": {"type": "number", "description": "Minimum value (default: 0)"},
                        "max": {"type": "number", "description": "Maximum value (default: 1)"}
                    },
                    "required": []
                }),
            ),
        ];

        debug!("Listed {} calculator tools", tools.len());
        Ok(tools)
    }

    #[instrument(skip(self))]
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        if !self.supported_operations.contains(&name.to_string()) {
            let error_msg = format!("Unknown tool: {}", name);
            warn!("{}", error_msg);
            return Err(McpError::invalid_request(error_msg));
        }

        let args: HashMap<String, Value> = arguments.as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        match self.calculate(name, &args) {
            Ok(result) => {
                debug!("Calculator tool '{}' executed successfully", name);
                Ok(vec![result])
            }
            Err(error) => {
                warn!("Calculator tool '{}' failed: {}", name, error);
                Err(McpError::internal_error(error))
            }
        }
    }
}
