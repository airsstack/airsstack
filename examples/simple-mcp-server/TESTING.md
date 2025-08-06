# MCP Server Testing Guide

## Quick Testing Methods

### Method 1: Using the Python Test Script
```bash
# Run comprehensive automated tests
python3 test_server.py
```

### Method 2: Manual JSON Testing
Start the server in one terminal:
```bash
cargo run --bin simple-mcp-server
```

In another terminal, send JSON-RPC messages:

#### Initialize Connection
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"roots":{"listChanged":true}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### List Resources
```bash
echo '{"jsonrpc":"2.0","id":2,"method":"resources/list"}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### Read a Resource
```bash
echo '{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"file:///tmp/example.txt"}}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### List Tools
```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/list"}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### Call Add Tool
```bash
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"add","arguments":{"a":15,"b":27}}}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### Call Greet Tool
```bash
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"greet","arguments":{"name":"Alice"}}}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### List Prompts
```bash
echo '{"jsonrpc":"2.0","id":7,"method":"prompts/list"}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

#### Get Code Review Prompt
```bash
echo '{"jsonrpc":"2.0","id":8,"method":"prompts/get","params":{"name":"code_review","arguments":{"language":"rust","code":"fn main() { println!(\"Hello!\"); }"}}}' | socat - EXEC:"cargo run --bin simple-mcp-server"
```

### Method 3: Using `nc` (netcat) for Interactive Testing
Start server:
```bash
cargo run --bin simple-mcp-server
```

In another terminal, create an interactive session:
```bash
# Send requests line by line
(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; echo '{"jsonrpc":"2.0","id":2,"method":"resources/list"}') | cargo run --bin simple-mcp-server
```

### Method 4: Using curl (if you add HTTP transport later)
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"resources/list"}'
```

## Expected Responses

### Resources List Response
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "resources": [
      {
        "uri": "file:///tmp/example.txt",
        "name": "Example File",
        "description": "A simple example file",
        "mimeType": "text/plain"
      },
      {
        "uri": "file:///tmp/config.json",
        "name": "Config File", 
        "description": "Application configuration",
        "mimeType": "application/json"
      }
    ]
  }
}
```

### Tool Call Response (Add)
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"result\": 42,\n  \"operation\": \"addition\"\n}"
      }
    ]
  }
}
```

### Prompt Response
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "description": "Code review prompt template",
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "Please review the following rust code and provide feedback:\n\n```rust\nfn main() { println!(\"Hello!\"); }\n```\n\nFocus on:\n- Code quality and best practices\n- Potential bugs or issues\n- Performance considerations\n- Readability and maintainability"
        }
      }
    ]
  }
}
```

## Debugging Tips

1. **Check Server Logs**: The server outputs initialization messages
2. **Validate JSON**: Ensure your JSON is properly formatted
3. **Check Method Names**: MCP methods are case-sensitive
4. **Required Fields**: Include `jsonrpc`, `id`, and `method` in all requests
5. **Parameter Structure**: Check the expected parameter structure for each method

## Installation Requirements

For the Python test script:
```bash
# No additional dependencies needed - uses standard library
python3 test_server.py
```

For manual testing with socat:
```bash
# On macOS:
brew install socat

# On Ubuntu/Debian:
sudo apt-get install socat

# On other systems, use curl or create your own HTTP client
```
