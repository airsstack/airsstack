//! Mock Server Responses
//!
//! Predefined MCP responses for the lightweight mock server.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

// Layer 3: Internal module imports
use airs_mcp::protocol::{JsonRpcRequest, JsonRpcResponse, RequestId};

/// Mock response generator for MCP operations
pub struct MockResponses;

impl MockResponses {
    /// Generate successful initialization response
    pub fn initialize_response(request_id: RequestId) -> JsonRpcResponse {
        JsonRpcResponse::success(
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {},
                    "logging": {}
                },
                "serverInfo": {
                    "name": "http-mock-server",
                    "version": "0.1.0",
                    "description": "Lightweight HTTP MCP mock server for testing"
                }
            }),
            request_id,
        )
    }

    /// Generate tools list response
    pub fn tools_list_response(request_id: RequestId) -> JsonRpcResponse {
        JsonRpcResponse::success(
            json!({
                "tools": [
                    {
                        "name": "echo",
                        "description": "Echo back the provided message",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "message": {
                                    "type": "string",
                                    "description": "Message to echo back"
                                }
                            },
                            "required": ["message"]
                        }
                    },
                    {
                        "name": "health_check",
                        "description": "Basic health check that returns server status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "get_timestamp",
                        "description": "Get current UTC timestamp",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "format": {
                                    "type": "string",
                                    "description": "Timestamp format (iso8601, unix, rfc3339)",
                                    "enum": ["iso8601", "unix", "rfc3339"]
                                }
                            },
                            "required": []
                        }
                    },
                    {
                        "name": "calculate",
                        "description": "Perform basic arithmetic calculations",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "operation": {
                                    "type": "string",
                                    "description": "Arithmetic operation",
                                    "enum": ["add", "subtract", "multiply", "divide"]
                                },
                                "a": {
                                    "type": "number",
                                    "description": "First operand"
                                },
                                "b": {
                                    "type": "number",
                                    "description": "Second operand"
                                }
                            },
                            "required": ["operation", "a", "b"]
                        }
                    }
                ]
            }),
            request_id,
        )
    }

    /// Generate tool call response
    pub fn tool_call_response(
        request_id: RequestId,
        tool_name: &str,
        arguments: &Value,
    ) -> JsonRpcResponse {
        let result = match tool_name {
            "echo" => {
                let message = arguments
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message provided");

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Echo: {}", message)
                        }
                    ]
                })
            }
            "health_check" => {
                let now: DateTime<Utc> = Utc::now();
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Mock server is healthy at {}", now.to_rfc3339())
                        }
                    ]
                })
            }
            "get_timestamp" => {
                let now: DateTime<Utc> = Utc::now();
                let format = arguments
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("iso8601");

                let timestamp = match format {
                    "unix" => now.timestamp().to_string(),
                    "rfc3339" => now.to_rfc3339(),
                    _ => now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), // ISO8601
                };

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Current timestamp ({}): {}", format, timestamp)
                        }
                    ]
                })
            }
            "calculate" => {
                let operation = arguments
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);

                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b != 0.0 {
                            a / b
                        } else {
                            return JsonRpcResponse::error(
                                json!({
                                    "code": -32602,
                                    "message": "Invalid params",
                                    "data": "Division by zero is not allowed"
                                }),
                                Some(request_id.clone()),
                            );
                        }
                    }
                    _ => {
                        return JsonRpcResponse::error(
                            json!({
                                "code": -32602,
                                "message": "Invalid params",
                                "data": format!("Unknown operation: {}", operation)
                            }),
                            Some(request_id.clone()),
                        );
                    }
                };

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("{} {} {} = {}", a, operation, b, result)
                        }
                    ]
                })
            }
            _ => {
                return JsonRpcResponse::error(
                    json!({
                        "code": -32602,
                        "message": "Invalid params",
                        "data": format!("Unknown tool: {}", tool_name)
                    }),
                    Some(request_id.clone()),
                );
            }
        };

        JsonRpcResponse::success(result, request_id)
    }

    /// Generate resources list response
    pub fn resources_list_response(request_id: RequestId) -> JsonRpcResponse {
        JsonRpcResponse::success(
            json!({
                "resources": [
                    {
                        "uri": "mock://server/info",
                        "name": "Server Information",
                        "description": "Basic information about the mock server",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "mock://server/status",
                        "name": "Server Status",
                        "description": "Current status and health of the mock server",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "mock://server/config",
                        "name": "Server Configuration",
                        "description": "Mock server configuration details",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "mock://data/sample.txt",
                        "name": "Sample Text File",
                        "description": "A sample text file for testing",
                        "mimeType": "text/plain"
                    }
                ]
            }),
            request_id,
        )
    }

    /// Generate resource read response
    pub fn resource_read_response(request_id: RequestId, uri: &str) -> JsonRpcResponse {
        let content = match uri {
            "mock://server/info" => {
                json!({
                    "contents": [
                        {
                            "type": "text",
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": json!({
                                "name": "HTTP Mock Server",
                                "version": "0.1.0",
                                "description": "Lightweight HTTP MCP mock server for testing",
                                "started_at": Utc::now().to_rfc3339(),
                                "capabilities": ["tools", "resources", "prompts"],
                                "authentication": ["X-API-Key", "Bearer", "QueryParameter"]
                            }).to_string()
                        }
                    ]
                })
            }
            "mock://server/status" => {
                json!({
                    "contents": [
                        {
                            "type": "text",
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": json!({
                                "status": "healthy",
                                "uptime": "unknown",
                                "requests_served": 0,
                                "last_request": Utc::now().to_rfc3339(),
                                "memory_usage": "low",
                                "connections": 1
                            }).to_string()
                        }
                    ]
                })
            }
            "mock://server/config" => {
                json!({
                    "contents": [
                        {
                            "type": "text",
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": json!({
                                "port": 3001,
                                "host": "127.0.0.1",
                                "auth_methods": ["X-API-Key", "Bearer", "QueryParameter"],
                                "api_keys": ["test-key-123", "dev-key-456", "mock-key-789"],
                                "debug_mode": true,
                                "cors_enabled": true
                            }).to_string()
                        }
                    ]
                })
            }
            "mock://data/sample.txt" => {
                json!({
                    "contents": [
                        {
                            "type": "text",
                            "uri": uri,
                            "mimeType": "text/plain",
                            "text": "This is a sample text file from the HTTP MCP mock server.\n\nIt demonstrates how resources can be read through the MCP protocol.\n\nYou can use this for testing client implementations.\n"
                        }
                    ]
                })
            }
            _ => {
                return JsonRpcResponse::error(
                    json!({
                        "code": -32602,
                        "message": "Invalid params",
                        "data": format!("Unknown resource URI: {}", uri)
                    }),
                    Some(request_id.clone()),
                );
            }
        };

        JsonRpcResponse::success(content, request_id)
    }

    /// Generate authentication error response
    pub fn authentication_error(request_id: Option<RequestId>) -> JsonRpcResponse {
        JsonRpcResponse::error(
            json!({
                "code": -32001,
                "message": "Authentication required",
                "data": {
                    "error": "Invalid or missing API key",
                    "supported_methods": [
                        "X-API-Key header",
                        "Authorization Bearer token",
                        "Query parameter ?api_key="
                    ],
                    "valid_keys": ["test-key-123", "dev-key-456", "mock-key-789"]
                }
            }),
            request_id,
        )
    }

    /// Generate method not found error
    pub fn method_not_found_error(request_id: Option<RequestId>, method: &str) -> JsonRpcResponse {
        JsonRpcResponse::error(
            json!({
                "code": -32601,
                "message": "Method not found",
                "data": {
                    "method": method,
                    "supported_methods": [
                        "initialize",
                        "tools/list",
                        "tools/call",
                        "resources/list",
                        "resources/read"
                    ]
                }
            }),
            request_id,
        )
    }

    /// Generate parse error response
    #[allow(dead_code)]
    pub fn parse_error(request_id: Option<RequestId>, error_message: &str) -> JsonRpcResponse {
        JsonRpcResponse::error(
            json!({
                "code": -32700,
                "message": "Parse error",
                "data": {
                    "error": error_message,
                    "expected": "Valid JSON-RPC 2.0 request"
                }
            }),
            request_id,
        )
    }

    /// Generate server error response (for fault injection)
    pub fn server_error(request_id: Option<RequestId>) -> JsonRpcResponse {
        JsonRpcResponse::error(
            json!({
                "code": -32000,
                "message": "Server error",
                "data": {
                    "error": "Simulated server error for testing",
                    "timestamp": Utc::now().to_rfc3339()
                }
            }),
            request_id,
        )
    }

    /// Generate malformed response (invalid JSON)
    #[allow(dead_code)]
    pub fn malformed_response() -> String {
        "{\"malformed\": true, \"incomplete\":".to_string()
    }

    /// Check if the request should trigger fault injection
    pub fn should_inject_fault(request: &JsonRpcRequest) -> Option<String> {
        // Check for fault injection triggers in the request
        if let Some(params) = &request.params {
            if let Some(fault) = params.get("_fault_injection") {
                if let Some(fault_type) = fault.as_str() {
                    return Some(fault_type.to_string());
                }
            }
        }

        // Check tool name for fault injection
        if request.method == "tools/call" {
            if let Some(params) = &request.params {
                if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                    match tool_name {
                        "trigger_server_error" => return Some("server_error".to_string()),
                        "trigger_timeout" => return Some("timeout".to_string()),
                        "trigger_malformed" => return Some("malformed".to_string()),
                        _ => {}
                    }
                }
            }
        }

        None
    }
}
