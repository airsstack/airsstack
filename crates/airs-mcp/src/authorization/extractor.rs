//! Method Extractor Framework
//!
//! Protocol-agnostic method extraction for authorization decisions.
//! This fixes the OAuth2 bug by extracting methods from JSON-RPC payloads
//! instead of HTTP URL paths.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports
use serde_json::Value;

// Layer 3: Internal module imports
use super::error::{AuthzError, AuthzResult};

/// Generic method extractor trait
///
/// Extracts method names from different types of requests for authorization.
/// This allows authorization policies to work with any protocol (JSON-RPC, REST, GraphQL, etc.)
pub trait MethodExtractor<R> {
    /// Extract method name from request
    ///
    /// # Arguments
    /// * `request` - Request to extract method from
    ///
    /// # Returns
    /// * `Ok(method)` - Method name extracted successfully
    /// * `Err(AuthzError)` - Failed to extract method
    fn extract_method(&self, request: &R) -> AuthzResult<String>;

    /// Get extractor name for debugging
    fn extractor_name(&self) -> &'static str;
}

/// JSON-RPC method extractor
///
/// Extracts method names from JSON-RPC request payloads.
/// This is the CORRECT way to extract methods for MCP over HTTP
/// (not from URL paths like the buggy implementation).
#[derive(Debug, Clone, Copy)]
pub struct JsonRpcMethodExtractor;

impl JsonRpcMethodExtractor {
    /// Create a new JSON-RPC method extractor
    pub const fn new() -> Self {
        Self
    }
}

impl Default for JsonRpcMethodExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON-RPC request interface
pub trait JsonRpcRequest {
    /// Get the JSON-RPC payload
    fn json_payload(&self) -> &Value;
}

impl<R> MethodExtractor<R> for JsonRpcMethodExtractor
where
    R: JsonRpcRequest,
{
    fn extract_method(&self, request: &R) -> AuthzResult<String> {
        let payload = request.json_payload();

        // Extract method from JSON-RPC payload
        match payload.get("method") {
            Some(Value::String(method)) => {
                if method.is_empty() {
                    Err(AuthzError::invalid_context("Empty method in JSON-RPC request"))
                } else {
                    Ok(method.clone())
                }
            }
            Some(_) => Err(AuthzError::invalid_context(
                "Method field in JSON-RPC request is not a string",
            )),
            None => Err(AuthzError::invalid_context(
                "Missing method field in JSON-RPC request",
            )),
        }
    }

    fn extractor_name(&self) -> &'static str {
        "JsonRpcMethodExtractor"
    }
}

/// HTTP path method extractor  
///
/// Extracts method names from HTTP URL paths for REST-style APIs.
/// This should ONLY be used for actual REST APIs, not JSON-RPC over HTTP.
#[derive(Debug, Clone)]
pub struct HttpPathMethodExtractor {
    /// Path prefix to strip (e.g., "/api/v1")
    prefix: String,
}

impl HttpPathMethodExtractor {
    /// Create a new HTTP path method extractor
    ///
    /// # Arguments
    /// * `prefix` - Path prefix to strip before extracting method
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    /// Create an extractor for MCP-style paths (/mcp/...)
    pub fn mcp() -> Self {
        Self::new("/mcp".to_string())
    }

    /// Create an extractor for API-style paths (/api/v1/...)
    pub fn api_v1() -> Self {
        Self::new("/api/v1".to_string())
    }
}

/// HTTP request interface
pub trait HttpRequest {
    /// Get the HTTP request path
    fn path(&self) -> &str;
}

impl<R> MethodExtractor<R> for HttpPathMethodExtractor
where
    R: HttpRequest,
{
    fn extract_method(&self, request: &R) -> AuthzResult<String> {
        let path = request.path();

        // Strip prefix if present
        let method_path = if let Some(stripped) = path.strip_prefix(&self.prefix) {
            stripped
        } else {
            path
        };

        // Remove leading slash and extract method
        let method = method_path.strip_prefix('/').unwrap_or(method_path);

        if method.is_empty() {
            Err(AuthzError::invalid_context(format!(
                "Cannot extract method from path: {path}"
            )))
        } else {
            Ok(method.to_string())
        }
    }

    fn extractor_name(&self) -> &'static str {
        "HttpPathMethodExtractor"
    }
}

/// Static method extractor that always returns the same method
///
/// Useful for single-purpose endpoints or testing.
#[derive(Debug, Clone)]
pub struct StaticMethodExtractor {
    method: String,
}

impl StaticMethodExtractor {
    /// Create a new static method extractor
    ///
    /// # Arguments
    /// * `method` - Method name to always return
    pub fn new(method: String) -> Self {
        Self { method }
    }
}

impl<R> MethodExtractor<R> for StaticMethodExtractor {
    fn extract_method(&self, _request: &R) -> AuthzResult<String> {
        Ok(self.method.clone())
    }

    fn extractor_name(&self) -> &'static str {
        "StaticMethodExtractor"
    }
}

/// Composite method extractor that tries multiple extractors in order
///
/// Useful for APIs that support multiple protocols (JSON-RPC + REST).
pub struct CompositeMethodExtractor<R> {
    extractors: Vec<Box<dyn MethodExtractor<R> + Send + Sync>>,
}

impl<R> CompositeMethodExtractor<R> {
    /// Create a new composite extractor
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add an extractor to try
    pub fn add_extractor(mut self, extractor: Box<dyn MethodExtractor<R> + Send + Sync>) -> Self {
        self.extractors.push(extractor);
        self
    }
}

impl<R> Default for CompositeMethodExtractor<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> MethodExtractor<R> for CompositeMethodExtractor<R> {
    fn extract_method(&self, request: &R) -> AuthzResult<String> {
        for extractor in &self.extractors {
            match extractor.extract_method(request) {
                Ok(method) => return Ok(method),
                Err(_) => continue,
            }
        }

        Err(AuthzError::invalid_context(
            "No extractor could extract method from request",
        ))
    }

    fn extractor_name(&self) -> &'static str {
        "CompositeMethodExtractor"
    }
}

impl<R> fmt::Debug for CompositeMethodExtractor<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeMethodExtractor")
            .field("extractor_count", &self.extractors.len())
            .finish()
    }
}

// Helper implementations for common request types

/// Simple JSON-RPC request wrapper
#[derive(Debug, Clone)]
pub struct SimpleJsonRpcRequest {
    payload: Value,
}

impl SimpleJsonRpcRequest {
    /// Create a new simple JSON-RPC request
    pub fn new(payload: Value) -> Self {
        Self { payload }
    }
}

impl JsonRpcRequest for SimpleJsonRpcRequest {
    fn json_payload(&self) -> &Value {
        &self.payload
    }
}

/// Simple HTTP request wrapper
#[derive(Debug, Clone)]
pub struct SimpleHttpRequest {
    path: String,
    #[allow(dead_code)] // Used in real implementations but not in tests
    headers: HashMap<String, String>,
}

impl SimpleHttpRequest {
    /// Create a new simple HTTP request
    pub fn new(path: String, headers: HashMap<String, String>) -> Self {
        Self { path, headers }
    }
}

impl HttpRequest for SimpleHttpRequest {
    fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_method_extractor() {
        let extractor = JsonRpcMethodExtractor::new();
        
        // Valid JSON-RPC request
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);
        
        let result = extractor.extract_method(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "initialize");
        assert_eq!(<JsonRpcMethodExtractor as MethodExtractor<SimpleJsonRpcRequest>>::extractor_name(&extractor), "JsonRpcMethodExtractor");
    }

    #[test]
    fn test_json_rpc_method_extractor_missing_method() {
        let extractor = JsonRpcMethodExtractor::new();
        
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);
        
        let result = extractor.extract_method(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_system_error());
    }

    #[test]
    fn test_json_rpc_method_extractor_empty_method() {
        let extractor = JsonRpcMethodExtractor::new();
        
        let payload = json!({
            "jsonrpc": "2.0",
            "method": "",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);
        
        let result = extractor.extract_method(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_path_method_extractor() {
        let extractor = HttpPathMethodExtractor::mcp();
        
        let request = SimpleHttpRequest::new(
            "/mcp/tools/call".to_string(),
            HashMap::new(),
        );
        
        let result = extractor.extract_method(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tools/call");
    }

    #[test]
    fn test_http_path_method_extractor_api_v1() {
        let extractor = HttpPathMethodExtractor::api_v1();
        
        let request = SimpleHttpRequest::new(
            "/api/v1/resources/list".to_string(),
            HashMap::new(),
        );
        
        let result = extractor.extract_method(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "resources/list");
    }

    #[test]
    fn test_http_path_method_extractor_empty_path() {
        let extractor = HttpPathMethodExtractor::mcp();
        
        let request = SimpleHttpRequest::new(
            "/mcp".to_string(),
            HashMap::new(),
        );
        
        let result = extractor.extract_method(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_static_method_extractor() {
        let extractor = StaticMethodExtractor::new("ping".to_string());
        
        let request = SimpleHttpRequest::new("/anything".to_string(), HashMap::new());
        
        let result = extractor.extract_method(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ping");
        assert_eq!(<StaticMethodExtractor as MethodExtractor<SimpleHttpRequest>>::extractor_name(&extractor), "StaticMethodExtractor");
    }

    #[test]
    fn test_composite_method_extractor_debug() {
        // Test that we can debug the composite extractor
        let composite: CompositeMethodExtractor<SimpleJsonRpcRequest> = CompositeMethodExtractor::new();
        let debug_output = format!("{composite:?}");
        assert!(debug_output.contains("CompositeMethodExtractor"));
        assert!(debug_output.contains("extractor_count"));
    }
}
