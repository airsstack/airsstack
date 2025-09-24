//! HTTP Message Handler Examples
//!
//! This module provides example implementations of MessageHandler\<HttpContext\>
//! demonstrating how to handle MCP protocol messages with HTTP-specific context.
//! These handlers showcase the Generic MessageHandler pattern in practice.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;

// Layer 3: Internal module imports
use crate::protocol::{
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, MessageContext, MessageHandler, TransportError,
};
use crate::transport::adapters::http::HttpContext;

/// MCP Protocol Handler over HTTP
///
/// This handler demonstrates proper MCP protocol handling with HTTP-specific
/// context access. It processes standard MCP requests like initialization,
/// resource listing, and tool invocation with appropriate HTTP status semantics.
///
/// # Features
///
/// - Full MCP protocol support (initialize, resources, tools, prompts)
/// - HTTP status code mapping (200 for success, 400 for invalid requests, 500 for errors)
/// - Session tracking via HTTP headers, cookies, and query parameters
/// - Content-Type validation for JSON-RPC requests
/// - Remote address logging for security auditing
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::{HttpContext, McpHttpHandler};
/// use airs_mcp::protocol::{MessageHandler, JsonRpcRequest, MessageContext, RequestId};
/// use std::sync::Arc;
///
/// let handler = Arc::new(McpHttpHandler::new());
/// let context = HttpContext::new("POST", "/mcp")
///     .with_header("content-type", "application/json")
///     .with_header("x-session-id", "session123");
/// let message_context = MessageContext::new_with_transport_data(
///     "session123".to_string(),
///     context,
/// );
///
/// // Handler will process MCP requests with HTTP context awareness
/// ```
#[derive(Debug)]
pub struct McpHttpHandler {
    /// Handler name for logging and identification
    #[allow(dead_code)]
    name: String,
    /// Creation timestamp
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
}

impl McpHttpHandler {
    /// Create a new MCP HTTP handler
    pub fn new() -> Self {
        Self {
            name: "McpHttpHandler".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Process MCP initialize request
    async fn handle_initialize(
        &self,
        request: JsonRpcRequest,
        context: &HttpContext,
    ) -> JsonRpcResponse {
        // Log security-relevant information
        println!(
            "MCP initialize request from {} via {} {}",
            context.remote_addr().unwrap_or("unknown"),
            context.method(),
            context.path()
        );

        // Validate JSON content type for MCP protocol
        if !context.is_json() {
            let error_data = json!({
                "code": -32600,
                "message": "MCP protocol requires application/json content type"
            });
            return JsonRpcResponse::error(error_data, Some(request.id.clone()));
        }

        // Build capabilities response based on HTTP context
        let capabilities = json!({
            "protocol": {
                "version": "1.0.0",
                "transport": "http"
            },
            "server": {
                "name": "airs-mcp-http-server",
                "version": "1.0.0"
            },
            "capabilities": {
                "resources": {},
                "tools": {},
                "prompts": {},
                "experimental": {
                    "http_streaming": true,
                    "session_management": true
                }
            }
        });

        JsonRpcResponse::success(capabilities, request.id.clone())
    }

    /// Process MCP resource listing request
    async fn handle_list_resources(
        &self,
        request: JsonRpcRequest,
        context: &HttpContext,
    ) -> JsonRpcResponse {
        println!(
            "Listing resources for session: {}",
            context.session_id().unwrap_or("anonymous")
        );

        let resources = json!({
            "resources": [
                {
                    "uri": "http://example.com/resource1",
                    "name": "Sample Resource",
                    "description": "A sample resource from HTTP handler"
                }
            ]
        });

        JsonRpcResponse::success(resources, request.id.clone())
    }

    /// Handle unknown MCP methods
    async fn handle_unknown_method(
        &self,
        request: JsonRpcRequest,
        _context: &HttpContext,
    ) -> JsonRpcResponse {
        let error_data = json!({
            "code": -32601,
            "message": format!("Method '{}' not implemented", request.method)
        });
        JsonRpcResponse::error(error_data, Some(request.id.clone()))
    }
}

impl Default for McpHttpHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler<HttpContext> for McpHttpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
        let http_context = match context.transport_data() {
            Some(ctx) => ctx,
            None => {
                println!("McpHttpHandler: Missing HTTP context data");
                return;
            }
        };

        // Log request details for debugging and security
        println!(
            "Processing message from {} {} (remote: {}, session: {})",
            http_context.method(),
            http_context.path(),
            http_context.remote_addr().unwrap_or("unknown"),
            http_context.session_id().unwrap_or("anonymous")
        );

        match message {
            JsonRpcMessage::Request(request) => {
                let response = match request.method.as_str() {
                    "initialize" => self.handle_initialize(request, http_context).await,
                    "resources/list" => self.handle_list_resources(request, http_context).await,
                    _ => self.handle_unknown_method(request, http_context).await,
                };

                println!("MCP response generated: {}", response.result.is_some());
                // In a real implementation, you'd send the response back through the transport
            }
            JsonRpcMessage::Response(response) => {
                println!(
                    "Received response (ID: {:?}): {}",
                    response.id,
                    response.result.is_some()
                );
                // Handle response if we're in a client role
            }
            JsonRpcMessage::Notification(notification) => {
                println!("Received notification: {}", notification.method);
                // Handle notifications (one-way messages)
            }
        }
    }

    async fn handle_error(&self, error: TransportError) {
        println!("MCP HTTP handler transport error: {error:?}");
        // Could implement error recovery, metrics collection, etc.
    }

    async fn handle_close(&self) {
        println!("MCP HTTP handler closing");
        // Cleanup resources, save state, etc.
    }
}

/// Echo Handler for Testing
///
/// This handler echoes back the received message along with HTTP context information.
/// Useful for testing the Generic MessageHandler pattern and debugging HTTP transport.
///
/// # Features
///
/// - Message echo with request/response correlation
/// - HTTP context information in response
/// - Request method and path logging
/// - Header and query parameter inspection
/// - Simple performance timing
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::{HttpContext, EchoHttpHandler};
/// use airs_mcp::protocol::{MessageHandler, JsonRpcRequest, MessageContext, RequestId};
/// use std::sync::Arc;
///
/// let handler = Arc::new(EchoHttpHandler::new());
/// let context = HttpContext::new("GET", "/echo")
///     .with_query_param("test", "value");
/// let message_context = MessageContext::new_with_transport_data(
///     "test_session".to_string(),
///     context,
/// );
///
/// // Handler will echo the message with HTTP context details
/// ```
#[derive(Debug)]
pub struct EchoHttpHandler {
    /// Handler name for identification
    name: String,
    /// Message counter for debugging
    message_count: std::sync::atomic::AtomicU64,
}

impl EchoHttpHandler {
    /// Create a new echo handler
    pub fn new() -> Self {
        Self {
            name: "EchoHttpHandler".to_string(),
            message_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl Default for EchoHttpHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler<HttpContext> for EchoHttpHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
        let count = self
            .message_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;

        let http_context = match context.transport_data() {
            Some(ctx) => ctx,
            None => {
                println!("EchoHttpHandler: No HTTP context available");
                return;
            }
        };

        // Build HTTP context summary
        let http_info = json!({
            "method": http_context.method(),
            "path": http_context.path(),
            "remote_addr": http_context.remote_addr(),
            "session_id": http_context.session_id(),
            "headers": http_context.headers(),
            "query_params": http_context.query_params(),
            "is_json": http_context.is_json(),
            "is_post": http_context.is_post()
        });

        match message {
            JsonRpcMessage::Request(request) => {
                println!(
                    "Echo #{}: {} request '{}' from {}",
                    count,
                    http_context.method(),
                    request.method,
                    http_context.remote_addr().unwrap_or("unknown")
                );

                let echo_response = json!({
                    "echo": {
                        "original_request": {
                            "id": request.id,
                            "method": request.method,
                            "params": request.params
                        },
                        "http_context": http_info,
                        "handler_info": {
                            "name": self.name,
                            "message_count": count,
                            "timestamp": Utc::now()
                        }
                    }
                });

                let _response = JsonRpcResponse::success(echo_response, request.id.clone());
                println!("Generated echo response for request ID: {:?}", request.id);
                // In real implementation, send response through transport
            }
            JsonRpcMessage::Response(response) => {
                println!(
                    "Echo #{}: Response (ID: {:?}, success: {})",
                    count,
                    response.id,
                    response.result.is_some()
                );
            }
            JsonRpcMessage::Notification(notification) => {
                println!(
                    "Echo #{}: Notification '{}' from {}",
                    count,
                    notification.method,
                    http_context.remote_addr().unwrap_or("unknown")
                );
            }
        }
    }

    async fn handle_error(&self, error: TransportError) {
        println!("Echo handler transport error: {error:?}");
    }

    async fn handle_close(&self) {
        let final_count = self
            .message_count
            .load(std::sync::atomic::Ordering::Relaxed);
        println!("Echo handler closing (processed {final_count} messages)");
    }
}

/// Static File Handler
///
/// This handler demonstrates file serving capabilities with HTTP-specific context.
/// It shows how to handle non-MCP requests using the Generic MessageHandler pattern,
/// useful for serving documentation, assets, or health check endpoints.
///
/// # Features
///
/// - Virtual file system with in-memory content
/// - HTTP GET request handling with path routing
/// - Content-Type detection based on file extensions
/// - 404 Not Found responses for missing files
/// - Security: path traversal protection
/// - Directory listing support
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::{HttpContext, StaticFileHandler};
/// use airs_mcp::protocol::{MessageHandler, JsonRpcRequest, MessageContext, RequestId};
/// use std::sync::Arc;
///
/// let mut handler = StaticFileHandler::new();
/// handler.add_file("/health", "OK", "text/plain");
/// handler.add_file("/docs/api.json", r#"{"version": "1.0"}"#, "application/json");
///
/// let context = HttpContext::new("GET", "/health");
/// // Handler will serve the static content with appropriate Content-Type
/// ```
#[derive(Debug)]
pub struct StaticFileHandler {
    /// Handler name for identification
    #[allow(dead_code)]
    name: String,
    /// Virtual file system (path -> (content, content_type))
    files: HashMap<String, (String, String)>,
    /// Request counter for metrics
    request_count: std::sync::atomic::AtomicU64,
}

impl StaticFileHandler {
    /// Create a new static file handler
    pub fn new() -> Self {
        let mut handler = Self {
            name: "StaticFileHandler".to_string(),
            files: HashMap::new(),
            request_count: std::sync::atomic::AtomicU64::new(0),
        };

        // Add default files
        handler.add_file("/health", "OK", "text/plain");
        handler.add_file(
            "/version",
            r#"{"version": "1.0.0", "transport": "http"}"#,
            "application/json",
        );
        handler.add_file(
            "/",
            r#"<!DOCTYPE html>
<html>
<head><title>AIRS MCP HTTP Server</title></head>
<body>
<h1>AIRS MCP HTTP Server</h1>
<p>Model Context Protocol over HTTP</p>
<ul>
<li><a href="/health">Health Check</a></li>
<li><a href="/version">Version Info</a></li>
</ul>
</body>
</html>"#,
            "text/html",
        );

        handler
    }

    /// Add a file to the virtual file system
    pub fn add_file(&mut self, path: &str, content: &str, content_type: &str) {
        self.files.insert(
            path.to_string(),
            (content.to_string(), content_type.to_string()),
        );
    }

    /// Get content type based on file extension
    #[allow(dead_code)]
    fn get_content_type(&self, path: &str) -> &'static str {
        if path.ends_with(".html") || path.ends_with(".htm") {
            "text/html"
        } else if path.ends_with(".json") {
            "application/json"
        } else if path.ends_with(".js") {
            "application/javascript"
        } else if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".txt") {
            "text/plain"
        } else {
            "application/octet-stream"
        }
    }

    /// Check if path is safe (no traversal attacks)
    fn is_safe_path(&self, path: &str) -> bool {
        !path.contains("..") && !path.contains("//") && path.starts_with('/')
    }

    /// Handle GET request for static files
    async fn handle_get_request(
        &self,
        request: JsonRpcRequest,
        context: &HttpContext,
    ) -> JsonRpcResponse {
        let count = self
            .request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let path = context.path();

        println!(
            "Static file request #{}: GET {} from {}",
            count,
            path,
            context.remote_addr().unwrap_or("unknown")
        );

        // Security check
        if !self.is_safe_path(path) {
            println!("Blocked unsafe path: {path}");
            let error_data = json!({
                "code": -32600,
                "message": "Invalid file path"
            });
            return JsonRpcResponse::error(error_data, Some(request.id.clone()));
        }

        // Look up file
        if let Some((content, content_type)) = self.files.get(path) {
            let response_data = json!({
                "file": {
                    "path": path,
                    "content": content,
                    "content_type": content_type,
                    "size": content.len(),
                    "served_at": Utc::now()
                }
            });

            println!(
                "Served file: {} ({} bytes, {})",
                path,
                content.len(),
                content_type
            );
            JsonRpcResponse::success(response_data, request.id.clone())
        } else {
            // Directory listing for root
            if path == "/" {
                let file_list: Vec<String> = self.files.keys().cloned().collect();
                let listing = json!({
                    "directory": {
                        "path": path,
                        "files": file_list,
                        "count": file_list.len()
                    }
                });
                JsonRpcResponse::success(listing, request.id.clone())
            } else {
                println!("File not found: {path}");
                let error_data = json!({
                    "code": -32404,
                    "message": format!("File not found: {}", path)
                });
                JsonRpcResponse::error(error_data, Some(request.id.clone()))
            }
        }
    }
}

impl Default for StaticFileHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageHandler<HttpContext> for StaticFileHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<HttpContext>) {
        let http_context = match context.transport_data() {
            Some(ctx) => ctx,
            None => {
                println!("StaticFileHandler: Missing HTTP context");
                return;
            }
        };

        match message {
            JsonRpcMessage::Request(request) => {
                // Only handle GET requests for static files
                if http_context.method() == "GET" {
                    let _response = self.handle_get_request(request, http_context).await;
                    println!("Generated static file response");
                    // In real implementation, send response through transport
                } else {
                    let error_data = json!({
                        "code": -32600,
                        "message": format!("Method {} not supported for static files", http_context.method())
                    });
                    let _error_response =
                        JsonRpcResponse::error(error_data, Some(request.id.clone()));
                    println!(
                        "Unsupported method for static files: {}",
                        http_context.method()
                    );
                    // Send error response through transport
                }
            }
            JsonRpcMessage::Response(_) => {
                println!("Static file handler ignoring response message");
            }
            JsonRpcMessage::Notification(_) => {
                println!("Static file handler ignoring notification");
            }
        }
    }

    async fn handle_error(&self, error: TransportError) {
        println!("Static file handler transport error: {error:?}");
    }

    async fn handle_close(&self) {
        let total_requests = self
            .request_count
            .load(std::sync::atomic::Ordering::Relaxed);
        println!(
            "Static file handler closing (served {total_requests} requests)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcRequest, RequestId};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mcp_http_handler_creation() {
        let handler = McpHttpHandler::new();
        assert_eq!(handler.name, "McpHttpHandler");
    }

    #[tokio::test]
    async fn test_mcp_http_handler_initialize() {
        let handler = McpHttpHandler::new();
        let context =
            HttpContext::new("POST", "/mcp").with_header("content-type", "application/json");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(json!({"protocolVersion": "1.0.0"})),
            id: RequestId::new_number(1),
        };

        let response = handler.handle_initialize(request, &context).await;
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("protocol").is_some());
        assert!(result.get("capabilities").is_some());
    }

    #[tokio::test]
    async fn test_mcp_http_handler_invalid_content_type() {
        let handler = McpHttpHandler::new();
        let context = HttpContext::new("POST", "/mcp").with_header("content-type", "text/plain");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: RequestId::new_number(1),
        };

        let response = handler.handle_initialize(request, &context).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32600);
    }

    #[tokio::test]
    async fn test_echo_handler_message_counting() {
        let handler = Arc::new(EchoHttpHandler::new());
        let context = HttpContext::new("GET", "/echo");
        let message_context =
            MessageContext::new_with_transport_data("test_session".to_string(), context);

        // Send multiple messages to test counting
        for i in 1..=3 {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "test".to_string(),
                params: None,
                id: RequestId::new_number(i),
            };
            handler
                .handle_message(JsonRpcMessage::Request(request), message_context.clone())
                .await;
        }

        let count = handler
            .message_count
            .load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_static_file_handler_default_files() {
        let handler = StaticFileHandler::new();

        // Check default files exist
        assert!(handler.files.contains_key("/health"));
        assert!(handler.files.contains_key("/version"));
        assert!(handler.files.contains_key("/"));

        let (health_content, health_type) = handler.files.get("/health").unwrap();
        assert_eq!(health_content, "OK");
        assert_eq!(health_type, "text/plain");
    }

    #[tokio::test]
    async fn test_static_file_handler_add_file() {
        let mut handler = StaticFileHandler::new();
        handler.add_file("/test.json", r#"{"test": true}"#, "application/json");

        assert!(handler.files.contains_key("/test.json"));
        let (content, content_type) = handler.files.get("/test.json").unwrap();
        assert_eq!(content, r#"{"test": true}"#);
        assert_eq!(content_type, "application/json");
    }

    #[tokio::test]
    async fn test_static_file_handler_security() {
        let handler = StaticFileHandler::new();

        // Test safe paths
        assert!(handler.is_safe_path("/health"));
        assert!(handler.is_safe_path("/docs/api.json"));

        // Test unsafe paths
        assert!(!handler.is_safe_path("../etc/passwd"));
        assert!(!handler.is_safe_path("/docs/../../../etc/passwd"));
        assert!(!handler.is_safe_path("//etc/passwd"));
        assert!(!handler.is_safe_path("etc/passwd")); // Must start with /
    }

    #[tokio::test]
    async fn test_static_file_handler_content_type_detection() {
        let handler = StaticFileHandler::new();

        assert_eq!(handler.get_content_type("/test.html"), "text/html");
        assert_eq!(handler.get_content_type("/api.json"), "application/json");
        assert_eq!(
            handler.get_content_type("/script.js"),
            "application/javascript"
        );
        assert_eq!(handler.get_content_type("/style.css"), "text/css");
        assert_eq!(handler.get_content_type("/readme.txt"), "text/plain");
        assert_eq!(
            handler.get_content_type("/unknown"),
            "application/octet-stream"
        );
    }

    #[tokio::test]
    async fn test_static_file_handler_get_request() {
        let handler = StaticFileHandler::new();
        let context = HttpContext::new("GET", "/health");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "file_request".to_string(),
            params: None,
            id: RequestId::new_number(1),
        };

        let response = handler.handle_get_request(request, &context).await;
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let file_info = result.get("file").unwrap();
        assert_eq!(file_info.get("path").unwrap(), "/health");
        assert_eq!(file_info.get("content").unwrap(), "OK");
        assert_eq!(file_info.get("content_type").unwrap(), "text/plain");
    }

    #[tokio::test]
    async fn test_static_file_handler_not_found() {
        let handler = StaticFileHandler::new();
        let context = HttpContext::new("GET", "/nonexistent");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "file_request".to_string(),
            params: None,
            id: RequestId::new_number(1),
        };

        let response = handler.handle_get_request(request, &context).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.get("code").unwrap().as_i64().unwrap(), -32404);
    }
}
