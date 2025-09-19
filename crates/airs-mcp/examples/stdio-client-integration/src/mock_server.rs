// Layer 1: Standard library imports
use std::env;
use std::io::{self, BufRead, BufReader, Write};

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

// Layer 3: Internal module imports
use airs_mcp::protocol::{JsonRpcRequest, JsonRpcResponse};

/// Simple mock MCP server that responds to basic requests over STDIO
///
/// This mock server reads JSON-RPC requests from stdin and writes responses to stdout.
/// It supports a minimal set of MCP methods for client integration testing.
fn main() -> io::Result<()> {
    // Initialize basic logging
    eprintln!("Starting STDIO mock MCP server...");

    // Check for fault injection environment variables
    let delay_ms = env::var("MOCK_DELAY_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let should_malform = env::var("MOCK_MALFORM").is_ok();
    let should_disconnect = env::var("MOCK_DISCONNECT").is_ok();

    if should_disconnect {
        eprintln!("Mock server configured to disconnect immediately");
        return Ok(());
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    eprintln!("Mock server ready, reading from stdin...");

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        eprintln!("Received: {line}");

        // Add artificial delay if configured
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }

        // Parse the JSON-RPC request
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);

                if should_malform {
                    println!("{{\"malformed\": true}}");
                } else {
                    let response_json = serde_json::to_string(&response).unwrap();
                    println!("{response_json}");
                }

                io::stdout().flush()?;
            }
            Err(e) => {
                eprintln!("Failed to parse request: {e}");
                // Send error response
                let error_response = JsonRpcResponse::error(
                    json!({
                        "code": -32700,
                        "message": "Parse error",
                        "data": format!("Invalid JSON: {}", e)
                    }),
                    None,
                );

                let response_json = serde_json::to_string(&error_response).unwrap();
                println!("{response_json}");
                io::stdout().flush()?;
            }
        }
    }

    eprintln!("Mock server shutting down");
    Ok(())
}

/// Handle a JSON-RPC request and return an appropriate response
fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let method = &request.method;
    let params = request.params.unwrap_or_default();

    let result = match method.as_str() {
        "initialize" => handle_initialize(params),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(params),
        "resources/list" => handle_resources_list(),
        "prompts/list" => handle_prompts_list(),
        _ => {
            return JsonRpcResponse::error(
                json!({
                    "code": -32601,
                    "message": "Method not found",
                    "data": format!("Unknown method: {}", method)
                }),
                Some(request.id),
            );
        }
    };

    JsonRpcResponse::success(result, request.id)
}

fn handle_initialize(_params: Value) -> Value {
    json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "stdio-mock-server",
            "version": "0.1.0"
        }
    })
}

fn handle_tools_list() -> Value {
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
                "description": "Get current timestamp",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    })
}

fn handle_tools_call(params: Value) -> Value {
    let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or_default();

    match tool_name {
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
                        "text": format!("Server is healthy at {}", now.to_rfc3339())
                    }
                ]
            })
        }
        "get_timestamp" => {
            let now: DateTime<Utc> = Utc::now();
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": now.to_rfc3339()
                    }
                ]
            })
        }
        _ => {
            json!({
                "isError": true,
                "content": [
                    {
                        "type": "text",
                        "text": format!("Unknown tool: {}", tool_name)
                    }
                ]
            })
        }
    }
}

fn handle_resources_list() -> Value {
    json!({
        "resources": []
    })
}

fn handle_prompts_list() -> Value {
    json!({
        "prompts": []
    })
}
