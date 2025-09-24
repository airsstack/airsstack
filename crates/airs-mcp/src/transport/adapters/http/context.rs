//! HTTP Transport Context Implementation
//!
//! This module provides the HttpContext structure that carries HTTP-specific
//! information with each message in the generic MessageHandler\<HttpContext\> pattern.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports
// (none needed for context definition)

/// HTTP-specific context data for the generic MessageHandler pattern
///
/// This structure contains HTTP request and session information that is passed
/// to message handlers along with the JSON-RPC message, enabling handlers to
/// access HTTP-specific details like request method, headers, and authentication.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::HttpContext;
/// use std::collections::HashMap;
///
/// let mut headers = HashMap::new();
/// headers.insert("content-type".to_string(), "application/json".to_string());
/// headers.insert("authorization".to_string(), "Bearer token123".to_string());
///
/// let context = HttpContext::new("POST", "/mcp")
///     .with_headers(headers)
///     .with_remote_addr("192.168.1.100:8080")
///     .with_user_agent("airs-mcp-client/1.0")
///     .with_query_param("session_id", "abc123");
///
/// assert_eq!(context.method(), "POST");
/// assert_eq!(context.path(), "/mcp");
/// assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
/// assert_eq!(context.get_header("authorization"), Some("Bearer token123"));
/// ```
#[derive(Debug, Clone)]
pub struct HttpContext {
    /// HTTP request method (GET, POST, etc.)
    method: String,

    /// Request path
    path: String,

    /// HTTP headers from the request
    headers: HashMap<String, String>,

    /// Query parameters from the request URL
    query_params: HashMap<String, String>,

    /// Remote client address
    remote_addr: Option<String>,

    /// Request timestamp
    timestamp: DateTime<Utc>,

    /// Request body content type
    content_type: Option<String>,

    /// Request content length
    content_length: Option<usize>,

    /// Authentication information (if available)
    auth_info: Option<String>,

    /// Session ID (if available)
    session_id: Option<String>,
}

impl HttpContext {
    /// Create a new HTTP context with method and path
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP request method (GET, POST, etc.)
    /// * `path` - Request path
    ///
    /// # Returns
    ///
    /// A new HttpContext with the specified method and path
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            remote_addr: None,
            timestamp: Utc::now(),
            content_type: None,
            content_length: None,
            auth_info: None,
            session_id: None,
        }
    }

    /// Get the HTTP request method
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the request timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get a specific header value
    ///
    /// # Arguments
    ///
    /// * `name` - Header name (case-insensitive)
    ///
    /// # Returns
    ///
    /// The header value if present
    pub fn get_header(&self, name: &str) -> Option<&str> {
        // Case-insensitive header lookup
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a specific query parameter value
    ///
    /// # Arguments
    ///
    /// * `name` - Query parameter name
    ///
    /// # Returns
    ///
    /// The parameter value if present
    pub fn get_query_param(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(|s| s.as_str())
    }

    /// Get all query parameters
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    /// Get the remote client address
    pub fn remote_addr(&self) -> Option<&str> {
        self.remote_addr.as_deref()
    }

    /// Get the content type
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    /// Get the content length
    pub fn content_length(&self) -> Option<usize> {
        self.content_length
    }

    /// Get authentication information
    pub fn auth_info(&self) -> Option<&str> {
        self.auth_info.as_deref()
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Add headers to the context
    ///
    /// # Arguments
    ///
    /// * `headers` - HashMap of header name-value pairs
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        // Extract common headers for convenience
        if let Some(content_type) = self.get_header("content-type") {
            self.content_type = Some(content_type.to_string());
        }
        if let Some(content_length) = self.get_header("content-length") {
            if let Ok(length) = content_length.parse() {
                self.content_length = Some(length);
            }
        }
        if let Some(auth) = self.get_header("authorization") {
            self.auth_info = Some(auth.to_string());
        }
        self
    }

    /// Add a single header to the context
    ///
    /// # Arguments
    ///
    /// * `name` - Header name
    /// * `value` - Header value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();

        // Update convenience fields for common headers
        match name.to_lowercase().as_str() {
            "content-type" => self.content_type = Some(value.clone()),
            "content-length" => {
                if let Ok(length) = value.parse() {
                    self.content_length = Some(length);
                }
            }
            "authorization" => self.auth_info = Some(value.clone()),
            "x-session-id" => self.session_id = Some(value.clone()),
            "cookie" => {
                // Extract sessionId from cookie string
                if let Some(session_id) = extract_session_from_cookie(&value) {
                    self.session_id = Some(session_id);
                }
            }
            _ => {}
        }

        self.headers.insert(name, value);
        self
    }

    /// Add query parameters to the context
    ///
    /// # Arguments
    ///
    /// * `params` - HashMap of query parameter name-value pairs
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params = params;
        // Extract session ID if present
        if let Some(session_id) = self.query_params.get("session_id") {
            self.session_id = Some(session_id.clone());
        }
        self
    }

    /// Add a single query parameter to the context
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `value` - Parameter value
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();

        // Update session ID if this is a session parameter (support both formats)
        if name == "session_id" || name == "sessionId" {
            self.session_id = Some(value.clone());
        }

        self.query_params.insert(name, value);
        self
    }

    /// Set the remote client address
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote client address
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_remote_addr(mut self, addr: impl Into<String>) -> Self {
        self.remote_addr = Some(addr.into());
        self
    }

    /// Set the user agent (convenience method for User-Agent header)
    ///
    /// # Arguments
    ///
    /// * `user_agent` - User agent string
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_user_agent(self, user_agent: impl Into<String>) -> Self {
        self.with_header("user-agent", user_agent)
    }

    /// Set the session ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session identifier
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set authentication information
    ///
    /// # Arguments
    ///
    /// * `auth_info` - Authentication information
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_auth_info(mut self, auth_info: impl Into<String>) -> Self {
        self.auth_info = Some(auth_info.into());
        self
    }

    /// Check if this is a GET request
    pub fn is_get(&self) -> bool {
        self.method.to_uppercase() == "GET"
    }

    /// Check if this is a POST request
    pub fn is_post(&self) -> bool {
        self.method.to_uppercase() == "POST"
    }

    /// Check if this is a PUT request
    pub fn is_put(&self) -> bool {
        self.method.to_uppercase() == "PUT"
    }

    /// Check if this is a DELETE request
    pub fn is_delete(&self) -> bool {
        self.method.to_uppercase() == "DELETE"
    }

    /// Check if the request accepts JSON content
    pub fn accepts_json(&self) -> bool {
        self.get_header("accept")
            .map(|accept| accept.contains("application/json"))
            .unwrap_or(false)
    }

    /// Check if the request has JSON content type
    pub fn is_json(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }
}

/// Helper function to extract session ID from cookie header
fn extract_session_from_cookie(cookie_header: &str) -> Option<String> {
    for cookie_pair in cookie_header.split(';') {
        let cookie_pair = cookie_pair.trim();
        if let Some((key, value)) = cookie_pair.split_once('=') {
            if key.trim() == "sessionId" {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_context_creation() {
        let context = HttpContext::new("POST", "/mcp");
        assert_eq!(context.method(), "POST");
        assert_eq!(context.path(), "/mcp");
        assert!(context.headers().is_empty());
        assert!(context.query_params().is_empty());
    }

    #[test]
    fn test_http_context_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("authorization".to_string(), "Bearer token123".to_string());

        let context = HttpContext::new("POST", "/mcp").with_headers(headers);

        assert_eq!(context.get_header("content-type"), Some("application/json"));
        assert_eq!(context.get_header("Content-Type"), Some("application/json")); // Case insensitive
        assert_eq!(context.get_header("authorization"), Some("Bearer token123"));
        assert_eq!(context.content_type(), Some("application/json"));
        assert_eq!(context.auth_info(), Some("Bearer token123"));
    }

    #[test]
    fn test_http_context_with_query_params() {
        let context = HttpContext::new("GET", "/mcp")
            .with_query_param("session_id", "abc123")
            .with_query_param("format", "json");

        assert_eq!(context.get_query_param("session_id"), Some("abc123"));
        assert_eq!(context.get_query_param("format"), Some("json"));
        assert_eq!(context.session_id(), Some("abc123"));
    }

    #[test]
    fn test_http_context_method_checks() {
        let get_context = HttpContext::new("GET", "/api");
        assert!(get_context.is_get());
        assert!(!get_context.is_post());

        let post_context = HttpContext::new("post", "/api"); // lowercase
        assert!(post_context.is_post());
        assert!(!post_context.is_get());
    }

    #[test]
    fn test_http_context_content_type_checks() {
        let context = HttpContext::new("POST", "/api")
            .with_header("content-type", "application/json")
            .with_header("accept", "application/json");

        assert!(context.is_json());
        assert!(context.accepts_json());
    }

    #[test]
    fn test_http_context_chaining() {
        let context = HttpContext::new("POST", "/mcp")
            .with_remote_addr("192.168.1.100:8080")
            .with_user_agent("airs-mcp-client/1.0")
            .with_session_id("session123")
            .with_auth_info("Bearer token456");

        assert_eq!(context.remote_addr(), Some("192.168.1.100:8080"));
        assert_eq!(
            context.get_header("user-agent"),
            Some("airs-mcp-client/1.0")
        );
        assert_eq!(context.session_id(), Some("session123"));
        assert_eq!(context.auth_info(), Some("Bearer token456"));
    }
}
