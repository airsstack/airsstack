//! Message Context and Session Management
//!
//! Context management for MCP message handling with session support.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

/// Message context for session and metadata management
///
/// This structure carries session information and metadata for each message,
/// enabling proper handling of multi-session transports like HTTP.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::mcp::MessageContext;
/// use chrono::Utc;
///
/// let context = MessageContext::new("session-123".to_string())
///     .with_remote_addr("192.168.1.100:8080".to_string())
///     .with_user_agent("airs-mcp-client/1.0".to_string());
///
/// assert_eq!(context.session_id(), Some("session-123"));
/// assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
/// ```
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Session identifier (if applicable)
    session_id: Option<String>,

    /// Timestamp when message was received
    timestamp: DateTime<Utc>,

    /// Remote address/endpoint information
    remote_addr: Option<String>,

    /// Additional metadata
    metadata: HashMap<String, String>,
}

impl MessageContext {
    /// Create a new message context
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new message context without session ID
    pub fn without_session() -> Self {
        Self {
            session_id: None,
            timestamp: Utc::now(),
            remote_addr: None,
            metadata: HashMap::new(),
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get message timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get remote address
    pub fn remote_addr(&self) -> Option<&str> {
        self.remote_addr.as_deref()
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Set remote address
    pub fn with_remote_addr(mut self, addr: String) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Convenience method to add user agent
    pub fn with_user_agent(self, user_agent: String) -> Self {
        self.with_metadata("user-agent".to_string(), user_agent)
    }

    /// Convenience method to add content type
    pub fn with_content_type(self, content_type: String) -> Self {
        self.with_metadata("content-type".to_string(), content_type)
    }
}

impl Default for MessageContext {
    fn default() -> Self {
        Self::without_session()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_context() {
        let context = MessageContext::new("session-123")
            .with_remote_addr("192.168.1.100:8080".to_string())
            .with_user_agent("airs-mcp-client/1.0".to_string())
            .with_metadata("custom-header".to_string(), "custom-value".to_string());

        assert_eq!(context.session_id(), Some("session-123"));
        assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
        assert_eq!(
            context.get_metadata("user-agent"),
            Some("airs-mcp-client/1.0")
        );
        assert_eq!(context.get_metadata("custom-header"), Some("custom-value"));
    }

    #[test]
    fn test_context_without_session() {
        let context = MessageContext::without_session();
        assert!(context.session_id().is_none());
        assert!(context.timestamp() <= Utc::now());
    }

    #[test]
    fn test_context_default() {
        let context = MessageContext::default();
        assert!(context.session_id().is_none());
    }
}
