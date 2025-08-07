//! Prompt Messages for MCP Protocol
//!
//! This module provides message types for MCP prompt operations including
//! prompt template discovery, retrieval, and argument processing.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::super::types::Content;
use crate::base::jsonrpc::message::JsonRpcMessage;

/// Represents a prompt template available from the server
///
/// Prompts are reusable templates that can be filled with arguments
/// to generate contextual prompts for language models.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::Prompt;
/// use serde_json::json;
///
/// let prompt = Prompt::new(
///     "code_review",
///     Some("Code Review Assistant"),
///     Some("Generate a code review for the given code"),
///     json!([{
///         "name": "code",
///         "description": "The code to review",
///         "required": true
///     }])
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub name: String,
    /// Human-readable name for the prompt (renamed from display_name to match MCP spec)
    pub title: Option<String>,
    /// Optional description of the prompt's purpose
    pub description: Option<String>,
    /// Schema describing the prompt's arguments
    pub arguments: Value,
}

impl Prompt {
    /// Create a new prompt
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Prompt;
    /// use serde_json::json;
    ///
    /// let prompt = Prompt::new(
    ///     "summarize",
    ///     Some("Text Summarizer"),
    ///     Some("Summarize the provided text"),
    ///     json!([{
    ///         "name": "text",
    ///         "description": "Text to summarize",
    ///         "required": true
    ///     }])
    /// );
    /// ```
    pub fn new(
        name: impl Into<String>,
        title: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
        arguments: Value,
    ) -> Self {
        Self {
            name: name.into(),
            title: title.map(|t| t.into()),
            description: description.map(|d| d.into()),
            arguments,
        }
    }

    /// Create a simple prompt with a single required string argument
    pub fn simple(
        name: impl Into<String>,
        title: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
        argument_name: impl Into<String>,
        argument_description: Option<impl Into<String>>,
    ) -> Self {
        let arg_desc = argument_description.map(|d| d.into());
        let arguments = serde_json::json!([{
            "name": argument_name.into(),
            "description": arg_desc,
            "required": true
        }]);

        Self::new(name, title, description, arguments)
    }

    /// Create a prompt with no arguments
    pub fn no_args(
        name: impl Into<String>,
        title: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
    ) -> Self {
        Self::new(name, title, description, serde_json::json!([]))
    }

    /// Check if this prompt requires no arguments
    #[must_use]
    pub fn is_parameterless(&self) -> bool {
        if let Some(array) = self.arguments.as_array() {
            array.is_empty()
        } else {
            false
        }
    }

    /// Get the required argument names for this prompt
    #[must_use]
    pub fn required_arguments(&self) -> Vec<String> {
        if let Some(array) = self.arguments.as_array() {
            array
                .iter()
                .filter_map(|arg| {
                    let obj = arg.as_object()?;
                    let is_required = obj.get("required")?.as_bool().unwrap_or(false);
                    if is_required {
                        obj.get("name")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all argument names (required and optional)
    #[must_use]
    pub fn all_arguments(&self) -> Vec<String> {
        if let Some(array) = self.arguments.as_array() {
            array
                .iter()
                .filter_map(|arg| {
                    arg.as_object()?
                        .get("name")?
                        .as_str()
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Argument for a prompt template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Name of the argument
    pub name: String,
    /// Description of the argument
    pub description: Option<String>,
    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
}

impl PromptArgument {
    /// Create a new prompt argument
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.map(|d| d.into()),
            required,
        }
    }

    /// Create a required prompt argument
    pub fn required(name: impl Into<String>, description: Option<impl Into<String>>) -> Self {
        Self::new(name, description, true)
    }

    /// Create an optional prompt argument
    pub fn optional(name: impl Into<String>, description: Option<impl Into<String>>) -> Self {
        Self::new(name, description, false)
    }
}

/// Request to list available prompts
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ListPromptsRequest;
///
/// let request = ListPromptsRequest::new();
/// let request_with_cursor = ListPromptsRequest::with_cursor("next_page");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsRequest {
    /// Optional cursor for pagination
    pub cursor: Option<String>,
}

impl ListPromptsRequest {
    /// Create a new list prompts request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list prompts request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            cursor: Some(cursor.into()),
        }
    }
}

impl Default for ListPromptsRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcMessage for ListPromptsRequest {}

/// Response containing available prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsResponse {
    /// List of available prompts
    pub prompts: Vec<Prompt>,
    /// Optional cursor for next page of results
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

impl ListPromptsResponse {
    /// Create a new list prompts response
    pub fn new(prompts: Vec<Prompt>) -> Self {
        Self {
            prompts,
            next_cursor: None,
        }
    }

    /// Create a new list prompts response with pagination
    pub fn with_pagination(prompts: Vec<Prompt>, next_cursor: Option<String>) -> Self {
        Self {
            prompts,
            next_cursor,
        }
    }

    /// Check if there are more prompts available
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }

    /// Get the number of prompts in this response
    #[must_use]
    pub fn count(&self) -> usize {
        self.prompts.len()
    }

    /// Find a prompt by name
    #[must_use]
    pub fn find_prompt(&self, name: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|prompt| prompt.name == name)
    }
}

impl JsonRpcMessage for ListPromptsResponse {}

/// Request to get a specific prompt with arguments
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::GetPromptRequest;
/// use std::collections::HashMap;
///
/// let mut arguments = HashMap::new();
/// arguments.insert("topic".to_string(), "Rust programming".to_string());
///
/// let request = GetPromptRequest::new("explain_concept", arguments);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptRequest {
    /// Name of the prompt to retrieve
    pub name: String,
    /// Arguments to fill the prompt template
    pub arguments: HashMap<String, String>,
}

impl GetPromptRequest {
    /// Create a new get prompt request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::GetPromptRequest;
    /// use std::collections::HashMap;
    ///
    /// let mut args = HashMap::new();
    /// args.insert("text".to_string(), "Hello world".to_string());
    ///
    /// let request = GetPromptRequest::new("summarize", args);
    /// ```
    pub fn new(name: impl Into<String>, arguments: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }

    /// Create a get prompt request with no arguments
    pub fn no_args(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: HashMap::new(),
        }
    }

    /// Create a get prompt request with a single argument
    pub fn with_argument(
        name: impl Into<String>,
        arg_name: impl Into<String>,
        arg_value: impl Into<String>,
    ) -> Self {
        let mut arguments = HashMap::new();
        arguments.insert(arg_name.into(), arg_value.into());

        Self {
            name: name.into(),
            arguments,
        }
    }

    /// Add an argument to the request
    pub fn add_argument(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.arguments.insert(name.into(), value.into());
    }

    /// Check if the request has arguments
    #[must_use]
    pub fn has_arguments(&self) -> bool {
        !self.arguments.is_empty()
    }

    /// Get an argument value by name
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&str> {
        self.arguments.get(name).map(|s| s.as_str())
    }

    /// Get the number of arguments
    #[must_use]
    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }
}

impl JsonRpcMessage for GetPromptRequest {}

/// Message in a prompt response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: Content,
}

impl PromptMessage {
    /// Create a new prompt message
    pub fn new(role: MessageRole, content: Content) -> Self {
        Self { role, content }
    }

    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: Content::text(text),
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Content::text(text),
        }
    }

    /// Create a system message with text content
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: Content::text(text),
        }
    }
}

/// Role of a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user/human
    User,
    /// Message from the AI assistant
    Assistant,
    /// System message providing context or instructions
    System,
}

/// Response containing a filled prompt template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptResponse {
    /// Description of the prompt
    pub description: Option<String>,
    /// Messages that make up the prompt
    pub messages: Vec<PromptMessage>,
}

impl GetPromptResponse {
    /// Create a new get prompt response
    pub fn new(description: Option<String>, messages: Vec<PromptMessage>) -> Self {
        Self {
            description,
            messages,
        }
    }

    /// Create a simple prompt response with a single user message
    pub fn simple_user(text: impl Into<String>) -> Self {
        Self {
            description: None,
            messages: vec![PromptMessage::user(text)],
        }
    }

    /// Create a prompt response with system and user messages
    pub fn system_user(system_message: impl Into<String>, user_message: impl Into<String>) -> Self {
        Self {
            description: None,
            messages: vec![
                PromptMessage::system(system_message),
                PromptMessage::user(user_message),
            ],
        }
    }

    /// Add a message to the response
    pub fn add_message(&mut self, message: PromptMessage) {
        self.messages.push(message);
    }

    /// Get the number of messages
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if the response is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get messages by role
    #[must_use]
    pub fn messages_by_role(&self, role: MessageRole) -> Vec<&PromptMessage> {
        self.messages
            .iter()
            .filter(|msg| msg.role == role)
            .collect()
    }
}

impl JsonRpcMessage for GetPromptResponse {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new(
            "test_prompt",
            Some("Test Prompt"),
            Some("A test prompt"),
            json!([{
                "name": "input",
                "description": "Test input",
                "required": true
            }]),
        );

        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(prompt.title, Some("Test Prompt".to_string()));
        assert_eq!(prompt.description, Some("A test prompt".to_string()));
        assert!(!prompt.is_parameterless());
        assert_eq!(prompt.required_arguments(), vec!["input"]);
        assert_eq!(prompt.all_arguments(), vec!["input"]);
    }

    #[test]
    fn test_prompt_simple() {
        let prompt = Prompt::simple(
            "echo",
            Some("Echo Prompt"),
            Some("Echo the input"),
            "message",
            Some("Message to echo"),
        );

        assert_eq!(prompt.name, "echo");
        assert!(!prompt.is_parameterless());
        assert_eq!(prompt.required_arguments(), vec!["message"]);
    }

    #[test]
    fn test_parameterless_prompt() {
        let prompt = Prompt::no_args("status", Some("Status"), None::<&str>);
        assert!(prompt.is_parameterless());
        assert!(prompt.required_arguments().is_empty());
        assert!(prompt.all_arguments().is_empty());
    }

    #[test]
    fn test_prompt_argument() {
        let required_arg = PromptArgument::required("name", Some("User name"));
        assert!(required_arg.required);
        assert_eq!(required_arg.name, "name");

        let optional_arg = PromptArgument::optional("age", Some("User age"));
        assert!(!optional_arg.required);
        assert_eq!(optional_arg.name, "age");
    }

    #[test]
    fn test_list_prompts_request() {
        let request = ListPromptsRequest::new();
        assert!(request.cursor.is_none());

        let request_with_cursor = ListPromptsRequest::with_cursor("page2");
        assert_eq!(request_with_cursor.cursor, Some("page2".to_string()));
    }

    #[test]
    fn test_list_prompts_response() {
        let prompts = vec![
            Prompt::simple(
                "prompt1",
                Some("Prompt 1"),
                None::<&str>,
                "param",
                None::<&str>,
            ),
            Prompt::simple(
                "prompt2",
                Some("Prompt 2"),
                None::<&str>,
                "param",
                None::<&str>,
            ),
        ];

        let response = ListPromptsResponse::new(prompts.clone());
        assert_eq!(response.count(), 2);
        assert!(!response.has_more());
        assert!(response.find_prompt("prompt1").is_some());
        assert!(response.find_prompt("nonexistent").is_none());

        let response_with_pagination =
            ListPromptsResponse::with_pagination(prompts, Some("next".to_string()));
        assert!(response_with_pagination.has_more());
    }

    #[test]
    fn test_get_prompt_request() {
        let mut args = HashMap::new();
        args.insert("topic".to_string(), "Rust".to_string());

        let request = GetPromptRequest::new("explain", args);
        assert_eq!(request.name, "explain");
        assert!(request.has_arguments());
        assert_eq!(request.get_argument("topic"), Some("Rust"));
        assert!(request.get_argument("missing").is_none());
        assert_eq!(request.argument_count(), 1);
    }

    #[test]
    fn test_get_prompt_request_no_args() {
        let request = GetPromptRequest::no_args("simple");
        assert!(!request.has_arguments());
        assert_eq!(request.argument_count(), 0);
    }

    #[test]
    fn test_get_prompt_request_single_arg() {
        let request = GetPromptRequest::with_argument("summarize", "text", "Hello world");
        assert!(request.has_arguments());
        assert_eq!(request.get_argument("text"), Some("Hello world"));
    }

    #[test]
    fn test_get_prompt_request_add_argument() {
        let mut request = GetPromptRequest::no_args("test");
        assert!(!request.has_arguments());

        request.add_argument("key", "value");
        assert!(request.has_arguments());
        assert_eq!(request.get_argument("key"), Some("value"));
    }

    #[test]
    fn test_prompt_message() {
        let user_msg = PromptMessage::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content.as_text(), Some("Hello"));

        let assistant_msg = PromptMessage::assistant("Hi there");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);

        let system_msg = PromptMessage::system("You are a helpful assistant");
        assert_eq!(system_msg.role, MessageRole::System);
    }

    #[test]
    fn test_get_prompt_response() {
        let response = GetPromptResponse::simple_user("What is Rust?");
        assert_eq!(response.message_count(), 1);
        assert!(!response.is_empty());

        let user_messages = response.messages_by_role(MessageRole::User);
        assert_eq!(user_messages.len(), 1);

        let system_messages = response.messages_by_role(MessageRole::System);
        assert_eq!(system_messages.len(), 0);
    }

    #[test]
    fn test_get_prompt_response_system_user() {
        let response =
            GetPromptResponse::system_user("You are a helpful assistant", "Explain Rust");
        assert_eq!(response.message_count(), 2);

        let system_messages = response.messages_by_role(MessageRole::System);
        assert_eq!(system_messages.len(), 1);

        let user_messages = response.messages_by_role(MessageRole::User);
        assert_eq!(user_messages.len(), 1);
    }

    #[test]
    fn test_get_prompt_response_add_message() {
        let mut response = GetPromptResponse::new(None, vec![]);
        assert!(response.is_empty());

        response.add_message(PromptMessage::user("Test"));
        assert!(!response.is_empty());
        assert_eq!(response.message_count(), 1);
    }

    #[test]
    fn test_message_serialization() {
        // Test serialization of all message types
        let prompt = Prompt::simple("test", Some("Test"), None::<&str>, "param", None::<&str>);
        let json = serde_json::to_string(&prompt).unwrap();
        let deserialized: Prompt = serde_json::from_str(&json).unwrap();
        assert_eq!(prompt, deserialized);

        let request = GetPromptRequest::with_argument("test", "param", "value");
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: GetPromptRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);

        let response = GetPromptResponse::simple_user("Test message");
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: GetPromptResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");

        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");

        let role = MessageRole::System;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"system\"");
    }
}
