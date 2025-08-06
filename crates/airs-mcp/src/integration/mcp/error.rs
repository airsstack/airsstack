//! MCP-specific error types
//!
//! This module provides error types specific to MCP operations,
//! extending the base integration error system.

use thiserror::Error;

use crate::integration::IntegrationError;
use crate::shared::protocol::errors::ProtocolError;

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// MCP-specific error types
#[derive(Debug, Error)]
pub enum McpError {
    /// Integration layer error (JSON-RPC, transport, correlation)
    #[error("Integration error: {0}")]
    Integration(#[from] IntegrationError),

    /// Protocol-specific error (validation, format)
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    /// Connection not established or lost
    #[error("Not connected to MCP server")]
    NotConnected,

    /// Capability negotiation failed
    #[error("Capability negotiation failed: {reason}")]
    CapabilityNegotiationFailed { reason: String },

    /// Server does not support requested capability
    #[error("Server does not support {capability}")]
    UnsupportedCapability { capability: String },

    /// Resource not found
    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },

    /// Tool not found
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    /// Tool execution failed
    #[error("Tool execution failed: {name} - {reason}")]
    ToolExecutionFailed { name: String, reason: String },

    /// Prompt not found
    #[error("Prompt not found: {name}")]
    PromptNotFound { name: String },

    /// Invalid prompt arguments
    #[error("Invalid prompt arguments for {prompt}: {reason}")]
    InvalidPromptArguments { prompt: String, reason: String },

    /// Subscription failed
    #[error("Failed to subscribe to {uri}: {reason}")]
    SubscriptionFailed { uri: String, reason: String },

    /// Server error response
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Timeout waiting for response
    #[error("Operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Invalid server response format
    #[error("Invalid server response: {reason}")]
    InvalidResponse { reason: String },

    /// Connection already established
    #[error("Already connected to MCP server")]
    AlreadyConnected,

    /// Operation not allowed in current state
    #[error("Operation not allowed in current state: {state}")]
    InvalidState { state: String },

    /// Custom error for user-defined errors
    #[error("Custom error: {message}")]
    Custom { message: String },
}

impl McpError {
    /// Create a new custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }

    /// Create a capability negotiation failed error
    pub fn capability_negotiation_failed(reason: impl Into<String>) -> Self {
        Self::CapabilityNegotiationFailed {
            reason: reason.into(),
        }
    }

    /// Create an unsupported capability error
    pub fn unsupported_capability(capability: impl Into<String>) -> Self {
        Self::UnsupportedCapability {
            capability: capability.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(uri: impl Into<String>) -> Self {
        Self::ResourceNotFound { uri: uri.into() }
    }

    /// Create a tool not found error
    pub fn tool_not_found(name: impl Into<String>) -> Self {
        Self::ToolNotFound { name: name.into() }
    }

    /// Create a tool execution failed error
    pub fn tool_execution_failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ToolExecutionFailed {
            name: name.into(),
            reason: reason.into(),
        }
    }

    /// Create a prompt not found error
    pub fn prompt_not_found(name: impl Into<String>) -> Self {
        Self::PromptNotFound { name: name.into() }
    }

    /// Create an invalid response error
    pub fn invalid_response(reason: impl Into<String>) -> Self {
        Self::InvalidResponse {
            reason: reason.into(),
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(details: impl Into<String>) -> Self {
        Self::InvalidResponse {
            reason: format!("Invalid request: {}", details.into()),
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::InvalidResponse {
            reason: format!("Method not found: {}", method.into()),
        }
    }

    /// Create a server error
    pub fn server_error(message: impl Into<String>) -> Self {
        Self::ServerError {
            message: message.into(),
        }
    }

    /// Create an already connected error
    pub fn already_connected() -> Self {
        Self::AlreadyConnected
    }

    /// Create an invalid state error
    pub fn invalid_state(state: impl Into<String>) -> Self {
        Self::InvalidState {
            state: state.into(),
        }
    }

    /// Create an internal error (alias for server_error)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::ServerError {
            message: message.into(),
        }
    }

    /// Create an invalid prompt arguments error
    pub fn invalid_prompt_arguments(prompt: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPromptArguments {
            prompt: prompt.into(),
            reason: reason.into(),
        }
    }

    /// Create a subscription failed error
    pub fn subscription_failed(uri: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::SubscriptionFailed {
            uri: uri.into(),
            reason: reason.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(seconds: u64) -> Self {
        Self::Timeout { seconds }
    }

    /// Check if this error is recoverable (can retry)
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            McpError::Integration(_) => true, // Integration errors might be recoverable
            McpError::Protocol(_) => false,
            McpError::NotConnected => true, // Can reconnect
            McpError::CapabilityNegotiationFailed { .. } => false,
            McpError::UnsupportedCapability { .. } => false,
            McpError::ResourceNotFound { .. } => false,
            McpError::ToolNotFound { .. } => false,
            McpError::ToolExecutionFailed { .. } => true, // Can retry tool
            McpError::PromptNotFound { .. } => false,
            McpError::InvalidPromptArguments { .. } => false,
            McpError::SubscriptionFailed { .. } => true, // Can retry subscription
            McpError::ServerError { .. } => true,        // Might be transient
            McpError::Timeout { .. } => true,            // Can retry
            McpError::InvalidResponse { .. } => false,
            McpError::AlreadyConnected => false,
            McpError::InvalidState { .. } => false,
            McpError::Custom { .. } => false, // Conservative default
        }
    }

    /// Check if this error indicates a connection problem
    #[must_use]
    pub fn is_connection_error(&self) -> bool {
        match self {
            McpError::Integration(_) => true, // Assume integration errors could be connection-related
            McpError::NotConnected => true,
            McpError::Timeout { .. } => true, // Could be connection timeout
            _ => false,
        }
    }

    /// Get the error category for telemetry/logging
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            McpError::Integration(_) => "integration",
            McpError::Protocol(_) => "protocol",
            McpError::NotConnected => "connection",
            McpError::CapabilityNegotiationFailed { .. } => "capability",
            McpError::UnsupportedCapability { .. } => "capability",
            McpError::ResourceNotFound { .. } => "resource",
            McpError::ToolNotFound { .. } => "tool",
            McpError::ToolExecutionFailed { .. } => "tool",
            McpError::PromptNotFound { .. } => "prompt",
            McpError::InvalidPromptArguments { .. } => "prompt",
            McpError::SubscriptionFailed { .. } => "subscription",
            McpError::ServerError { .. } => "server",
            McpError::Timeout { .. } => "timeout",
            McpError::InvalidResponse { .. } => "response",
            McpError::AlreadyConnected => "connection",
            McpError::InvalidState { .. } => "state",
            McpError::Custom { .. } => "custom",
        }
    }
}

// Helper trait to add MCP-specific context to integration errors
pub trait McpErrorExt {
    /// Convert to MCP error with additional context
    fn mcp_context(self, context: &str) -> McpError;
}

impl McpErrorExt for IntegrationError {
    fn mcp_context(self, context: &str) -> McpError {
        McpError::custom(format!("{context}: {self}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = McpError::resource_not_found("file:///test.txt");
        assert_eq!(error.to_string(), "Resource not found: file:///test.txt");
        assert_eq!(error.category(), "resource");
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_tool_execution_error() {
        let error = McpError::tool_execution_failed("grep", "Invalid regex pattern");
        assert!(error.to_string().contains("grep"));
        assert!(error.to_string().contains("Invalid regex pattern"));
        assert!(error.is_recoverable());
        assert_eq!(error.category(), "tool");
    }

    #[test]
    fn test_timeout_error() {
        let error = McpError::timeout(30);
        assert!(error.to_string().contains("30 seconds"));
        assert!(error.is_recoverable());
        assert!(error.is_connection_error());
        assert_eq!(error.category(), "timeout");
    }

    #[test]
    fn test_capability_error() {
        let error = McpError::unsupported_capability("sampling");
        assert!(error.to_string().contains("sampling"));
        assert!(!error.is_recoverable());
        assert_eq!(error.category(), "capability");
    }

    #[test]
    fn test_custom_error() {
        let error = McpError::custom("Something went wrong");
        assert_eq!(error.to_string(), "Custom error: Something went wrong");
        assert!(!error.is_recoverable());
        assert_eq!(error.category(), "custom");
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(McpError::NotConnected.category(), "connection");
        assert_eq!(McpError::server_error("test").category(), "server");
        assert_eq!(McpError::invalid_response("test").category(), "response");
    }

    #[test]
    fn test_recoverable_classification() {
        // Recoverable errors
        assert!(McpError::NotConnected.is_recoverable());
        assert!(McpError::tool_execution_failed("test", "reason").is_recoverable());
        assert!(McpError::subscription_failed("uri", "reason").is_recoverable());
        assert!(McpError::server_error("message").is_recoverable());
        assert!(McpError::timeout(30).is_recoverable());

        // Non-recoverable errors
        assert!(!McpError::unsupported_capability("test").is_recoverable());
        assert!(!McpError::resource_not_found("test").is_recoverable());
        assert!(!McpError::tool_not_found("test").is_recoverable());
        assert!(!McpError::prompt_not_found("test").is_recoverable());
        assert!(!McpError::invalid_prompt_arguments("test", "reason").is_recoverable());
        assert!(!McpError::invalid_response("reason").is_recoverable());
        assert!(!McpError::AlreadyConnected.is_recoverable());
        assert!(!McpError::invalid_state("state").is_recoverable());
    }

    #[test]
    fn test_connection_error_classification() {
        // Connection errors
        assert!(McpError::NotConnected.is_connection_error());
        assert!(McpError::timeout(30).is_connection_error());

        // Non-connection errors
        assert!(!McpError::resource_not_found("test").is_connection_error());
        assert!(!McpError::tool_execution_failed("test", "reason").is_connection_error());
        assert!(!McpError::server_error("message").is_connection_error());
        assert!(!McpError::invalid_response("reason").is_connection_error());
    }
}
