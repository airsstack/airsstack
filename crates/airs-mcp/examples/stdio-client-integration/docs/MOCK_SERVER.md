# STDIO Mock Server Documentation

## Overview

The STDIO Mock Server is a minimal JSON-RPC server implementation designed for testing the STDIO MCP client integration. It provides a simple, predictable environment for validating client functionality without requiring a full MCP server deployment.

## Features

- **JSON-RPC 2.0 Compliance**: Implements proper JSON-RPC 2.0 protocol
- **MCP Protocol Support**: Supports core MCP methods (initialize, tools/list, tools/call)
- **STDIO Communication**: Uses standard input/output for communication
- **Error Handling**: Provides appropriate JSON-RPC error responses
- **Tool Simulation**: Implements common test tools (echo, health_check, get_timestamp)

## Building and Running

### Build the Mock Server

```bash
# From the stdio-client-integration directory
cargo build --bin stdio-mock-server
```

### Run the Mock Server

```bash
# Run directly
cargo run --bin stdio-mock-server

# Or run the built binary
./target/debug/stdio-mock-server
```

The server will start and wait for JSON-RPC requests on standard input.

## Supported Methods

### 1. initialize

Initializes the MCP session.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize", 
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {},
    "clientInfo": {
      "name": "client-name",
      "version": "1.0"
    }
  },
  "id": "init-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "init-1",
  "result": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "stdio-mock-server",
      "version": "1.0.0"
    }
  }
}
```

### 2. tools/list

Lists available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {},
  "id": "tools-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "tools-1", 
  "result": {
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
        "description": "Perform a health check",
        "inputSchema": {
          "type": "object",
          "properties": {}
        }
      },
      {
        "name": "get_timestamp",
        "description": "Get the current timestamp",
        "inputSchema": {
          "type": "object", 
          "properties": {}
        }
      }
    ]
  }
}
```

### 3. tools/call

Calls a specific tool.

**Request (echo tool):**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "echo",
    "arguments": {
      "message": "Hello, World!"
    }
  },
  "id": "call-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "call-1",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Echo: Hello, World!"
      }
    ]
  }
}
```

**Request (health_check tool):**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "health_check",
    "arguments": {}
  },
  "id": "health-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "health-1",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Mock server is healthy"
      }
    ]
  }
}
```

**Request (get_timestamp tool):**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_timestamp",
    "arguments": {}
  },
  "id": "timestamp-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "timestamp-1",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "2025-09-19T10:30:45.123456789Z"
      }
    ]
  }
}
```

## Error Responses

The mock server provides appropriate JSON-RPC error responses for invalid requests:

### Method Not Found (-32601)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "nonexistent/method",
  "params": {},
  "id": "error-1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "error-1",
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

### Invalid Parameters (-32602)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "nonexistent_tool"
  },
  "id": "error-2"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "error-2",
  "error": {
    "code": -32602,
    "message": "Tool not found: nonexistent_tool"
  }
}
```

### Parse Error (-32700)

For malformed JSON input, the server responds with:

```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32700,
    "message": "Parse error"
  }
}
```

## Testing with the Mock Server

### Manual Testing

1. **Start the mock server:**
   ```bash
   cargo run --bin stdio-mock-server
   ```

2. **Send requests via standard input:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":"test"}' | cargo run --bin stdio-mock-server
   ```

### Automated Testing

The mock server is designed to work with the provided test suite:

```bash
# Run integration tests
cd tests/
python test_client_integration.py

# Run transport tests  
python test_transport.py

# Run error scenario tests
python test_error_scenarios.py
```

## Implementation Details

### Architecture

The mock server follows a simple request-response pattern:

1. **Input Reading**: Reads JSON-RPC requests from stdin line by line
2. **JSON Parsing**: Parses each line as JSON-RPC 2.0 request
3. **Method Routing**: Routes requests to appropriate handlers
4. **Response Generation**: Generates compliant JSON-RPC 2.0 responses
5. **Output Writing**: Writes responses to stdout

### Code Structure

```
src/mock_server.rs
├── MockServer struct
├── Request handling methods
│   ├── handle_initialize()
│   ├── handle_tools_list()  
│   ├── handle_tools_call()
│   └── handle_unknown_method()
├── Tool implementations
│   ├── tool_echo()
│   ├── tool_health_check()
│   └── tool_get_timestamp()
└── Error handling utilities
```

### Configuration

The mock server uses minimal configuration and is designed to work out-of-the-box. It automatically:

- Responds to all supported MCP methods
- Provides consistent, predictable responses
- Handles errors gracefully
- Uses UTC timestamps for time-related responses

## Limitations

1. **No Persistence**: The server doesn't maintain state between requests
2. **Limited Tools**: Only implements basic test tools
3. **No Authentication**: No security or authentication mechanisms
4. **Single Session**: Designed for single client testing scenarios
5. **No Notifications**: Doesn't support server-initiated notifications

## Troubleshooting

### Common Issues

**Server doesn't respond:**
- Ensure JSON requests are properly formatted
- Check that each request ends with a newline
- Verify JSON-RPC 2.0 format compliance

**Parse errors:**
- Validate JSON syntax
- Ensure proper escaping of strings
- Check that required fields are present

**Tool call failures:**
- Verify tool name spelling
- Check that required arguments are provided
- Ensure argument types match schema

### Debug Output

Enable debug logging to see server internals:

```bash
RUST_LOG=debug cargo run --bin stdio-mock-server
```

### Testing Connection

Quick test to verify server is working:

```bash
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":"test"}' | cargo run --bin stdio-mock-server
```

Expected output:
```json
{"jsonrpc":"2.0","id":"test","result":{"tools":[...]}}
```

## Integration with Client

The mock server is specifically designed to work with the STDIO MCP client. Set the environment variable to use the mock server:

```bash
export USE_MOCK=1
cargo run --bin stdio-mcp-client
```

This configuration allows the client to automatically use the mock server for testing and demonstration purposes.