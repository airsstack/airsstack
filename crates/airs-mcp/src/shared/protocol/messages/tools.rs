//! Tool Messages for MCP Protocol
//!
//! This module provides message types for MCP tool operations including
//! tool discovery, invocation, and result handling with safety controls.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::types::Content;
use crate::base::jsonrpc::message::JsonRpcMessage;

/// Represents a tool available for execution
///
/// Tools are functions that can be invoked by clients to perform operations
/// such as file manipulation, API calls, calculations, or other actions.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::Tool;
/// use serde_json::json;
///
/// let tool = Tool::new(
///     "file_read",
///     Some("Read File"),
///     Some("Read content from a file"),
///     json!({
///         "type": "object",
///         "properties": {
///             "path": {
///                 "type": "string",
///                 "description": "Path to the file to read"
///             }
///         },
///         "required": ["path"]
///     })
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// Unique identifier for the tool
    pub name: String,
    /// Human-readable name for the tool (renamed from display_name to match MCP spec)
    pub title: Option<String>,
    /// Optional description of what the tool does
    pub description: Option<String>,
    /// JSON Schema describing the tool's input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

impl Tool {
    /// Create a new tool
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Tool;
    /// use serde_json::json;
    ///
    /// let tool = Tool::new(
    ///     "calculator",
    ///     Some("Calculator"),
    ///     Some("Perform mathematical calculations"),
    ///     json!({
    ///         "type": "object",
    ///         "properties": {
    ///             "expression": {"type": "string"}
    ///         }
    ///     })
    /// );
    /// ```
    pub fn new(
        name: impl Into<String>,
        title: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            title: title.map(|t| t.into()),
            description: description.map(|d| d.into()),
            input_schema,
        }
    }

    /// Create a simple tool with basic string parameter
    pub fn simple(
        name: impl Into<String>,
        title: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
        parameter_name: impl Into<String>,
        parameter_description: Option<impl Into<String>>,
    ) -> Self {
        let param_name = parameter_name.into();
        let param_desc = parameter_description.map(|d| d.into());
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": {
                param_name.clone(): {
                    "type": "string",
                    "description": param_desc
                }
            },
            "required": [param_name]
        });

        Self::new(name, title, description, input_schema)
    }

    /// Check if this tool requires no parameters
    #[must_use]
    pub fn is_parameterless(&self) -> bool {
        if let Some(properties) = self.input_schema.get("properties") {
            properties.as_object().map_or(true, |obj| obj.is_empty())
        } else {
            true
        }
    }

    /// Get the required parameters for this tool
    #[must_use]
    pub fn required_parameters(&self) -> Vec<String> {
        self.input_schema
            .get("required")
            .and_then(|req| req.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Request to list available tools
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ListToolsRequest;
///
/// let request = ListToolsRequest::new();
/// let request_with_cursor = ListToolsRequest::with_cursor("next_page");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsRequest {
    /// Optional cursor for pagination
    pub cursor: Option<String>,
}

impl ListToolsRequest {
    /// Create a new list tools request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list tools request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            cursor: Some(cursor.into()),
        }
    }
}

impl Default for ListToolsRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcMessage for ListToolsRequest {}

/// Response containing available tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsResponse {
    /// List of available tools
    pub tools: Vec<Tool>,
    /// Optional cursor for next page of results
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ListToolsResponse {
    /// Create a new list tools response
    pub fn new(tools: Vec<Tool>) -> Self {
        Self {
            tools,
            next_cursor: None,
        }
    }

    /// Create a new list tools response with pagination
    pub fn with_pagination(tools: Vec<Tool>, next_cursor: Option<String>) -> Self {
        Self { tools, next_cursor }
    }

    /// Check if there are more tools available
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }

    /// Get the number of tools in this response
    #[must_use]
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Find a tool by name
    #[must_use]
    pub fn find_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|tool| tool.name == name)
    }
}

impl JsonRpcMessage for ListToolsResponse {}

/// Request to call/execute a tool
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::CallToolRequest;
/// use serde_json::json;
///
/// let request = CallToolRequest::new(
///     "file_read",
///     json!({"path": "/path/to/file.txt"})
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolRequest {
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    pub arguments: Value,
}

impl CallToolRequest {
    /// Create a new call tool request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::CallToolRequest;
    /// use serde_json::json;
    ///
    /// let request = CallToolRequest::new(
    ///     "calculator",
    ///     json!({"expression": "2 + 2"})
    /// );
    /// ```
    pub fn new(name: impl Into<String>, arguments: Value) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }

    /// Create a call tool request with no arguments
    pub fn no_args(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a call tool request with a single string argument
    pub fn with_string_arg(
        name: impl Into<String>,
        arg_name: impl Into<String>,
        arg_value: impl Into<String>,
    ) -> Self {
        let mut args = serde_json::Map::new();
        args.insert(arg_name.into(), Value::String(arg_value.into()));

        Self {
            name: name.into(),
            arguments: Value::Object(args),
        }
    }

    /// Check if the request has arguments
    #[must_use]
    pub fn has_arguments(&self) -> bool {
        match &self.arguments {
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }

    /// Get an argument value by name
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Value> {
        self.arguments.get(name)
    }
}

impl JsonRpcMessage for CallToolRequest {}

/// Response from tool execution
///
/// Contains the result of tool execution, which can be successful content
/// or an error message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolResponse {
    /// Content returned by the tool
    pub content: Vec<Content>,
    /// Whether the tool execution was successful
    #[serde(default = "default_false")]
    #[serde(rename = "isError")]
    pub is_error: bool,
}

fn default_false() -> bool {
    false
}

impl CallToolResponse {
    /// Create a successful tool response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{CallToolResponse, Content};
    ///
    /// let response = CallToolResponse::success(vec![
    ///     Content::text("Operation completed successfully")
    /// ]);
    /// ```
    pub fn success(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: false,
        }
    }

    /// Create a successful tool response with text content
    pub fn success_text(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: false,
        }
    }

    /// Create an error tool response
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{CallToolResponse, Content};
    ///
    /// let response = CallToolResponse::error(vec![
    ///     Content::text("Tool execution failed: invalid parameters")
    /// ]);
    /// ```
    pub fn error(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: true,
        }
    }

    /// Create an error tool response with text content
    pub fn error_text(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: true,
        }
    }

    /// Check if this response represents an error
    #[must_use]
    pub fn is_success(&self) -> bool {
        !self.is_error
    }

    /// Get the number of content items
    #[must_use]
    pub fn content_count(&self) -> usize {
        self.content.len()
    }

    /// Add content to the response
    pub fn add_content(&mut self, content: Content) {
        self.content.push(content);
    }

    /// Get the text content if this is a single text response
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        if self.content.len() == 1 {
            self.content[0].as_text()
        } else {
            None
        }
    }
}

impl JsonRpcMessage for CallToolResponse {}

/// Progress notification during tool execution
///
/// Sent by the server to inform the client about progress during
/// long-running tool operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolProgressNotification {
    /// Identifier for the progress session
    #[serde(rename = "progressToken")]
    pub progress_token: String,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Optional progress message
    pub message: Option<String>,
}

impl ToolProgressNotification {
    /// Create a new progress notification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::ToolProgressNotification;
    ///
    /// let notification = ToolProgressNotification::new(
    ///     "task_123",
    ///     0.5,
    ///     Some("Processing file 5 of 10")
    /// );
    /// ```
    pub fn new(
        progress_token: impl Into<String>,
        progress: f64,
        message: Option<impl Into<String>>,
    ) -> Self {
        Self {
            progress_token: progress_token.into(),
            progress: progress.clamp(0.0, 1.0),
            message: message.map(|m| m.into()),
        }
    }

    /// Create a progress notification without a message
    pub fn simple(progress_token: impl Into<String>, progress: f64) -> Self {
        Self::new(progress_token, progress, None::<String>)
    }

    /// Check if this represents completion (progress >= 1.0)
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    /// Get progress as a percentage (0-100)
    #[must_use]
    pub fn progress_percentage(&self) -> u32 {
        (self.progress * 100.0) as u32
    }
}

impl JsonRpcMessage for ToolProgressNotification {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new(
            "test_tool",
            Some("Test Tool"),
            Some("A test tool"),
            json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                },
                "required": ["input"]
            }),
        );

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.title, Some("Test Tool".to_string()));
        assert_eq!(tool.description, Some("A test tool".to_string()));
        assert!(!tool.is_parameterless());
        assert_eq!(tool.required_parameters(), vec!["input"]);
    }

    #[test]
    fn test_tool_simple() {
        let tool = Tool::simple(
            "echo",
            Some("Echo Tool"),
            Some("Echo input"),
            "message",
            Some("Message to echo"),
        );

        assert_eq!(tool.name, "echo");
        assert!(!tool.is_parameterless());
        assert_eq!(tool.required_parameters(), vec!["message"]);
    }

    #[test]
    fn test_parameterless_tool() {
        let tool = Tool::new(
            "status",
            Some("Status Check"),
            None::<&str>,
            json!({"type": "object", "properties": {}}),
        );

        assert!(tool.is_parameterless());
        assert!(tool.required_parameters().is_empty());
    }

    #[test]
    fn test_list_tools_request() {
        let request = ListToolsRequest::new();
        assert!(request.cursor.is_none());

        let request_with_cursor = ListToolsRequest::with_cursor("page2");
        assert_eq!(request_with_cursor.cursor, Some("page2".to_string()));
    }

    #[test]
    fn test_list_tools_response() {
        let tools = vec![
            Tool::simple("tool1", Some("Tool 1"), None::<&str>, "param", None::<&str>),
            Tool::simple("tool2", Some("Tool 2"), None::<&str>, "param", None::<&str>),
        ];

        let response = ListToolsResponse::new(tools.clone());
        assert_eq!(response.count(), 2);
        assert!(!response.has_more());
        assert!(response.find_tool("tool1").is_some());
        assert!(response.find_tool("nonexistent").is_none());

        let response_with_pagination =
            ListToolsResponse::with_pagination(tools, Some("next".to_string()));
        assert!(response_with_pagination.has_more());
    }

    #[test]
    fn test_call_tool_request() {
        let request = CallToolRequest::new("test_tool", json!({"param": "value"}));

        assert_eq!(request.name, "test_tool");
        assert!(request.has_arguments());
        assert_eq!(request.get_argument("param"), Some(&json!("value")));
        assert!(request.get_argument("missing").is_none());
    }

    #[test]
    fn test_call_tool_request_no_args() {
        let request = CallToolRequest::no_args("simple_tool");
        assert!(!request.has_arguments());
    }

    #[test]
    fn test_call_tool_request_string_arg() {
        let request = CallToolRequest::with_string_arg("echo", "message", "hello");
        assert!(request.has_arguments());
        assert_eq!(request.get_argument("message"), Some(&json!("hello")));
    }

    #[test]
    fn test_call_tool_response_success() {
        let response = CallToolResponse::success_text("Success!");
        assert!(response.is_success());
        assert_eq!(response.content_count(), 1);
        assert_eq!(response.as_text(), Some("Success!"));
    }

    #[test]
    fn test_call_tool_response_error() {
        let response = CallToolResponse::error_text("Error occurred");
        assert!(!response.is_success());
        assert_eq!(response.as_text(), Some("Error occurred"));
    }

    #[test]
    fn test_tool_progress_notification() {
        let notification = ToolProgressNotification::new("task_1", 0.75, Some("75% complete"));

        assert_eq!(notification.progress_token, "task_1");
        assert_eq!(notification.progress, 0.75);
        assert!(!notification.is_complete());
        assert_eq!(notification.progress_percentage(), 75);

        let complete = ToolProgressNotification::simple("task_2", 1.0);
        assert!(complete.is_complete());
    }

    #[test]
    fn test_progress_clamping() {
        let notification = ToolProgressNotification::simple("task", 1.5);
        assert_eq!(notification.progress, 1.0);

        let notification = ToolProgressNotification::simple("task", -0.1);
        assert_eq!(notification.progress, 0.0);
    }

    #[test]
    fn test_message_serialization() {
        // Test serialization of all message types
        let tool = Tool::simple("test", Some("Test"), None::<&str>, "param", None::<&str>);
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);

        let request = CallToolRequest::with_string_arg("test", "param", "value");
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CallToolRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);

        let response = CallToolResponse::success_text("OK");
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CallToolResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
