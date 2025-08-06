//! Logging Messages for MCP Protocol
//!
//! This module provides structured logging support for MCP operations,
//! enabling comprehensive debugging and monitoring of MCP interactions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};

use crate::base::jsonrpc::message::JsonRpcMessage;

/// Log level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level - detailed information for diagnosing problems
    Debug,
    /// Info level - general information about program execution
    Info,
    /// Warning level - indicates potential issues
    Warning,
    /// Error level - indicates definite problems
    Error,
    /// Critical level - serious errors that may cause program termination
    Critical,
}

impl LogLevel {
    /// Get the string representation of the log level
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
        }
    }

    /// Check if this level should be logged at the given minimum level
    #[must_use]
    pub fn should_log(&self, min_level: LogLevel) -> bool {
        *self >= min_level
    }

    /// Check if this is an error level or higher
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, LogLevel::Error | LogLevel::Critical)
    }

    /// Check if this is a warning level or higher
    #[must_use]
    pub fn is_warning(&self) -> bool {
        *self >= LogLevel::Warning
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Context information for a log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogContext {
    /// Component or module generating the log
    pub component: String,
    /// Operation or function being performed
    pub operation: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// User ID if available
    pub user_id: Option<String>,
    /// Session ID if available
    pub session_id: Option<String>,
    /// Additional structured data
    #[serde(default)]
    pub data: HashMap<String, Value>,
}

impl LogContext {
    /// Create a new log context
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            operation: None,
            request_id: None,
            user_id: None,
            session_id: None,
            data: HashMap::new(),
        }
    }

    /// Set the operation
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add structured data
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Add multiple data fields
    pub fn with_data_map(mut self, data: HashMap<String, Value>) -> Self {
        self.data.extend(data);
        self
    }

    /// Add a data field
    pub fn add_data(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    /// Get a data field
    #[must_use]
    pub fn get_data(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Check if context has request correlation info
    #[must_use]
    pub fn has_correlation_info(&self) -> bool {
        self.request_id.is_some() || self.session_id.is_some()
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new("mcp")
    }
}

/// A structured log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    /// Timestamp when the log was created
    pub timestamp: DateTime<Utc>,
    /// Log level
    pub level: LogLevel,
    /// Human-readable message
    pub message: String,
    /// Context information
    pub context: LogContext,
    /// Optional error details
    pub error: Option<LogError>,
    /// Optional stack trace
    pub stack_trace: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        context: LogContext,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
            context,
            error: None,
            stack_trace: None,
        }
    }

    /// Create a debug log entry
    pub fn debug(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogLevel::Debug, message, context)
    }

    /// Create an info log entry
    pub fn info(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogLevel::Info, message, context)
    }

    /// Create a warning log entry
    pub fn warning(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogLevel::Warning, message, context)
    }

    /// Create an error log entry
    pub fn error(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogLevel::Error, message, context)
    }

    /// Create a critical log entry
    pub fn critical(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogLevel::Critical, message, context)
    }

    /// Add error details to the log entry
    pub fn with_error(mut self, error: LogError) -> Self {
        self.error = Some(error);
        self
    }

    /// Add stack trace to the log entry
    pub fn with_stack_trace(mut self, stack_trace: impl Into<String>) -> Self {
        self.stack_trace = Some(stack_trace.into());
        self
    }

    /// Set a custom timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Check if this log entry indicates an error
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.level.is_error()
    }

    /// Check if this log entry indicates a warning or higher
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.level.is_warning()
    }

    /// Get the component from context
    #[must_use]
    pub fn component(&self) -> &str {
        &self.context.component
    }

    /// Get the operation from context
    #[must_use]
    pub fn operation(&self) -> Option<&str> {
        self.context.operation.as_deref()
    }

    /// Check if this entry has correlation info
    #[must_use]
    pub fn has_correlation_info(&self) -> bool {
        self.context.has_correlation_info()
    }
}

/// Error details for logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogError {
    /// Error type or code
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(default)]
    pub data: HashMap<String, Value>,
}

impl LogError {
    /// Create a new log error
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
            data: HashMap::new(),
        }
    }

    /// Add error data
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Add multiple data fields
    pub fn with_data_map(mut self, data: HashMap<String, Value>) -> Self {
        self.data.extend(data);
        self
    }
}

/// Notification to send log entries to the client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingNotification {
    /// Log entry to send
    pub entry: LogEntry,
}

impl LoggingNotification {
    /// Create a new logging notification
    pub fn new(entry: LogEntry) -> Self {
        Self { entry }
    }

    /// Create a debug logging notification
    pub fn debug(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogEntry::debug(message, context))
    }

    /// Create an info logging notification
    pub fn info(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogEntry::info(message, context))
    }

    /// Create a warning logging notification
    pub fn warning(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogEntry::warning(message, context))
    }

    /// Create an error logging notification
    pub fn error(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogEntry::error(message, context))
    }

    /// Create a critical logging notification
    pub fn critical(message: impl Into<String>, context: LogContext) -> Self {
        Self::new(LogEntry::critical(message, context))
    }

    /// Get the log level
    #[must_use]
    pub fn level(&self) -> LogLevel {
        self.entry.level
    }

    /// Get the message
    #[must_use]
    pub fn message(&self) -> &str {
        &self.entry.message
    }

    /// Get the component
    #[must_use]
    pub fn component(&self) -> &str {
        self.entry.component()
    }
}

impl JsonRpcMessage for LoggingNotification {}

/// Configuration for logging behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    /// Minimum log level to process
    pub min_level: LogLevel,
    /// Whether to include stack traces
    #[serde(default)]
    pub include_stack_traces: bool,
    /// Whether to buffer log entries
    #[serde(default)]
    pub buffered: bool,
    /// Maximum number of buffered entries
    pub buffer_size: Option<usize>,
    /// Components to include (empty means all)
    #[serde(default)]
    pub included_components: Vec<String>,
    /// Components to exclude
    #[serde(default)]
    pub excluded_components: Vec<String>,
}

impl LoggingConfig {
    /// Create a new logging configuration
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            include_stack_traces: false,
            buffered: false,
            buffer_size: None,
            included_components: Vec::new(),
            excluded_components: Vec::new(),
        }
    }

    /// Enable buffering with the specified size
    pub fn with_buffering(mut self, buffer_size: usize) -> Self {
        self.buffered = true;
        self.buffer_size = Some(buffer_size);
        self
    }

    /// Enable stack traces
    pub fn with_stack_traces(mut self) -> Self {
        self.include_stack_traces = true;
        self
    }

    /// Include specific components
    pub fn include_components(mut self, components: Vec<String>) -> Self {
        self.included_components = components;
        self
    }

    /// Exclude specific components
    pub fn exclude_components(mut self, components: Vec<String>) -> Self {
        self.excluded_components = components;
        self
    }

    /// Check if a log entry should be processed based on configuration
    #[must_use]
    pub fn should_log(&self, entry: &LogEntry) -> bool {
        // Check level
        if !entry.level.should_log(self.min_level) {
            return false;
        }

        // Check component inclusion/exclusion
        let component = &entry.context.component;

        // If there are included components, only log those
        if !self.included_components.is_empty() {
            return self.included_components.contains(component);
        }

        // Otherwise, log everything except excluded components
        !self.excluded_components.contains(component)
    }

    /// Get the effective buffer size
    #[must_use]
    pub fn effective_buffer_size(&self) -> usize {
        if self.buffered {
            self.buffer_size.unwrap_or(1000)
        } else {
            0
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}

/// Request to set logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLoggingRequest {
    /// New logging configuration
    pub config: LoggingConfig,
}

impl SetLoggingRequest {
    /// Create a new set logging request
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Create a request to set minimum log level
    pub fn set_level(level: LogLevel) -> Self {
        Self::new(LoggingConfig::new(level))
    }

    /// Create a request to enable debug logging
    pub fn enable_debug() -> Self {
        Self::set_level(LogLevel::Debug)
    }

    /// Create a request to enable buffered logging
    pub fn enable_buffering(level: LogLevel, buffer_size: usize) -> Self {
        Self::new(LoggingConfig::new(level).with_buffering(buffer_size))
    }
}

impl JsonRpcMessage for SetLoggingRequest {}

/// Response to set logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLoggingResponse {
    /// Whether the configuration was accepted
    pub success: bool,
    /// Optional message about the configuration change
    pub message: Option<String>,
}

impl SetLoggingResponse {
    /// Create a successful response
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    /// Create a successful response with message
    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
        }
    }

    /// Create a failure response
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
        }
    }
}

impl JsonRpcMessage for SetLoggingResponse {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_log_level_should_log() {
        assert!(LogLevel::Error.should_log(LogLevel::Info));
        assert!(LogLevel::Info.should_log(LogLevel::Info));
        assert!(!LogLevel::Debug.should_log(LogLevel::Info));
    }

    #[test]
    fn test_log_level_predicates() {
        assert!(!LogLevel::Info.is_error());
        assert!(LogLevel::Error.is_error());
        assert!(LogLevel::Critical.is_error());

        assert!(!LogLevel::Info.is_warning());
        assert!(LogLevel::Warning.is_warning());
        assert!(LogLevel::Error.is_warning());
    }

    #[test]
    fn test_log_context() {
        let context = LogContext::new("test_component")
            .with_operation("test_operation")
            .with_request_id("req123")
            .with_data("key", json!("value"));

        assert_eq!(context.component, "test_component");
        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert_eq!(context.request_id, Some("req123".to_string()));
        assert_eq!(context.get_data("key"), Some(&json!("value")));
        assert!(context.has_correlation_info());
    }

    #[test]
    fn test_log_context_without_correlation() {
        let context = LogContext::new("test");
        assert!(!context.has_correlation_info());
    }

    #[test]
    fn test_log_entry() {
        let context = LogContext::new("test");
        let entry = LogEntry::info("Test message", context);

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.component(), "test");
        assert!(!entry.is_error());
        assert!(!entry.is_warning());
    }

    #[test]
    fn test_log_entry_with_error() {
        let context = LogContext::new("test");
        let error = LogError::new("TestError", "Something went wrong");
        let entry = LogEntry::error("Error occurred", context).with_error(error);

        assert!(entry.is_error());
        assert!(entry.error.is_some());
        assert_eq!(entry.error.as_ref().unwrap().error_type, "TestError");
    }

    #[test]
    fn test_log_entry_convenience_methods() {
        let context = LogContext::new("test");

        let debug_entry = LogEntry::debug("Debug message", context.clone());
        assert_eq!(debug_entry.level, LogLevel::Debug);

        let warning_entry = LogEntry::warning("Warning message", context.clone());
        assert_eq!(warning_entry.level, LogLevel::Warning);
        assert!(warning_entry.is_warning());

        let critical_entry = LogEntry::critical("Critical message", context);
        assert_eq!(critical_entry.level, LogLevel::Critical);
        assert!(critical_entry.is_error());
    }

    #[test]
    fn test_log_error() {
        let error = LogError::new("ValidationError", "Invalid input")
            .with_data("field", json!("email"))
            .with_data("value", json!("invalid-email"));

        assert_eq!(error.error_type, "ValidationError");
        assert_eq!(error.message, "Invalid input");
        assert_eq!(error.data.len(), 2);
    }

    #[test]
    fn test_logging_notification() {
        let context = LogContext::new("test");
        let notification = LoggingNotification::info("Test notification", context);

        assert_eq!(notification.level(), LogLevel::Info);
        assert_eq!(notification.message(), "Test notification");
        assert_eq!(notification.component(), "test");
    }

    #[test]
    fn test_logging_config() {
        let config = LoggingConfig::new(LogLevel::Warning)
            .with_buffering(500)
            .with_stack_traces()
            .include_components(vec!["auth".to_string(), "api".to_string()]);

        assert_eq!(config.min_level, LogLevel::Warning);
        assert!(config.buffered);
        assert!(config.include_stack_traces);
        assert_eq!(config.effective_buffer_size(), 500);
        assert_eq!(config.included_components, vec!["auth", "api"]);
    }

    #[test]
    fn test_logging_config_should_log() {
        let config = LoggingConfig::new(LogLevel::Warning)
            .include_components(vec!["auth".to_string()]);

        // Test level filtering
        let debug_context = LogContext::new("auth");
        let debug_entry = LogEntry::debug("Debug message", debug_context);
        assert!(!config.should_log(&debug_entry));

        let warning_context = LogContext::new("auth");
        let warning_entry = LogEntry::warning("Warning message", warning_context);
        assert!(config.should_log(&warning_entry));

        // Test component filtering
        let excluded_context = LogContext::new("other");
        let excluded_entry = LogEntry::error("Error message", excluded_context);
        assert!(!config.should_log(&excluded_entry));
    }

    #[test]
    fn test_logging_config_exclusion() {
        let config = LoggingConfig::new(LogLevel::Debug)
            .exclude_components(vec!["noisy".to_string()]);

        let allowed_context = LogContext::new("api");
        let allowed_entry = LogEntry::info("Info message", allowed_context);
        assert!(config.should_log(&allowed_entry));

        let excluded_context = LogContext::new("noisy");
        let excluded_entry = LogEntry::info("Info message", excluded_context);
        assert!(!config.should_log(&excluded_entry));
    }

    #[test]
    fn test_set_logging_request() {
        let request = SetLoggingRequest::enable_debug();
        assert_eq!(request.config.min_level, LogLevel::Debug);

        let buffered_request = SetLoggingRequest::enable_buffering(LogLevel::Info, 1000);
        assert!(buffered_request.config.buffered);
        assert_eq!(buffered_request.config.effective_buffer_size(), 1000);
    }

    #[test]
    fn test_set_logging_response() {
        let success = SetLoggingResponse::success();
        assert!(success.success);
        assert!(success.message.is_none());

        let success_with_msg = SetLoggingResponse::success_with_message("Logging enabled");
        assert!(success_with_msg.success);
        assert_eq!(success_with_msg.message, Some("Logging enabled".to_string()));

        let failure = SetLoggingResponse::failure("Invalid configuration");
        assert!(!failure.success);
        assert_eq!(failure.message, Some("Invalid configuration".to_string()));
    }

    #[test]
    fn test_serialization() {
        // Test log level serialization
        let level = LogLevel::Warning;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"warning\"");

        // Test log entry serialization
        let context = LogContext::new("test");
        let entry = LogEntry::info("Test message", context);
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.level, deserialized.level);
        assert_eq!(entry.message, deserialized.message);

        // Test logging notification serialization
        let notification = LoggingNotification::debug("Debug test", LogContext::new("test"));
        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: LoggingNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(notification, deserialized);
    }

    #[test]
    fn test_default_implementations() {
        let default_context = LogContext::default();
        assert_eq!(default_context.component, "mcp");

        let default_config = LoggingConfig::default();
        assert_eq!(default_config.min_level, LogLevel::Info);
        assert!(!default_config.buffered);
    }
}
