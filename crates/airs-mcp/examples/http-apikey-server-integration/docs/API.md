# HTTP API Key Server API Documentation

This document describes the API and integration patterns for the HTTP API Key MCP server.

## Authentication API

### API Key Formats

The server supports three API key authentication formats:

#### 1. X-API-Key Header

```http
POST /mcp HTTP/1.1
Host: localhost:3000
Content-Type: application/json
X-API-Key: dev-key-123

{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}
```

#### 2. Authorization Bearer

```http
POST /mcp HTTP/1.1
Host: localhost:3000
Content-Type: application/json
Authorization: Bearer dev-key-123

{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}
```

#### 3. Query Parameter

```http
POST /mcp?api_key=dev-key-123 HTTP/1.1
Host: localhost:3000
Content-Type: application/json

{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}
```

### Authentication Responses

#### Successful Authentication

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "name": "math_add",
        "description": "Add two numbers",
        "inputSchema": {
          "type": "object",
          "properties": {
            "a": {"type": "number"},
            "b": {"type": "number"}
          }
        }
      }
    ]
  }
}
```

#### Authentication Failure

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Authentication failed",
    "data": {
      "reason": "Invalid API key",
      "provided_auth": "header"
    }
  }
}
```

## Tool Providers API

### Math Tool Provider

Provides mathematical operations and calculations.

#### Available Tools

- `math_add`: Add two numbers
- `math_subtract`: Subtract two numbers
- `math_multiply`: Multiply two numbers
- `math_divide`: Divide two numbers

#### Example Usage

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "name": "math_add",
      "arguments": {"a": 5, "b": 3}
    }
  }'
```

### File System Resource Provider

Provides access to test files and directories.

#### Available Resources

- `test_resources/api-info.txt`: Server information file
- `test_resources/server-config.json`: Configuration example
- `test_resources/README.md`: Documentation file
- `test_resources/examples/`: Example directory

#### Example Usage

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "read_resource",
    "params": {
      "uri": "file://test_resources/api-info.txt"
    }
  }'
```

### Code Review Prompt Provider

Provides code review templates and prompts.

#### Available Prompts

- `code_review`: General code review template
- `security_review`: Security-focused review template
- `performance_review`: Performance analysis template

#### Example Usage

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "get_prompt",
    "params": {
      "name": "code_review",
      "arguments": {"language": "rust"}
    }
  }'
```

## Configuration API

### Server Configuration

The server can be configured via command line arguments or environment variables.

#### Command Line Configuration

```bash
# Basic configuration
http-apikey-server --port 3000 --dev-mode true

# Custom API key
http-apikey-server --api-key "custom-production-key"

# Production mode (no dev keys)
http-apikey-server --dev-mode false --api-key "prod-key-123"
```

#### Configuration Structure

```rust
pub struct ServerConfig {
    pub port: u16,
    pub dev_mode: bool,
    pub custom_api_key: Option<String>,
}
```

### API Key Configuration

#### Development Keys (dev_mode = true)

| Key | User ID | Permissions | Source |
|-----|---------|-------------|--------|
| `dev-key-123` | dev-user | read, write | X-API-Key |
| `test-key-456` | test-user | read | Authorization Bearer |
| `demo-key-789` | demo-user | read, write, admin | Query Parameter |

#### Custom Key Configuration

Custom keys are configured with:
- User ID: `custom-user`
- Permissions: `read`, `write`
- Source: `X-API-Key` header

## Error Handling

### Authentication Errors

#### Invalid API Key

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Invalid API key",
    "data": {
      "provided_key": "invalid-key-123",
      "auth_method": "header"
    }
  }
}
```

#### Missing API Key

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32002,
    "message": "API key required",
    "data": {
      "supported_methods": ["header", "bearer", "query"]
    }
  }
}
```

### Tool Errors

#### Tool Not Found

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Tool not found",
    "data": {
      "tool_name": "nonexistent_tool"
    }
  }
}
```

#### Invalid Arguments

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid tool arguments",
    "data": {
      "tool_name": "math_add",
      "reason": "Missing required parameter 'b'"
    }
  }
}
```

## Integration Patterns

### Client Integration

#### Simple Client

```javascript
const apiKey = 'dev-key-123';
const baseUrl = 'http://localhost:3000/mcp';

async function callTool(toolName, args) {
  const response = await fetch(baseUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': apiKey
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: Date.now(),
      method: 'call_tool',
      params: {
        name: toolName,
        arguments: args
      }
    })
  });
  
  return response.json();
}
```

#### Python Client

```python
import requests
import json

class McpApiKeyClient:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.api_key = api_key
    
    def call_tool(self, tool_name, args):
        headers = {
            'Content-Type': 'application/json',
            'X-API-Key': self.api_key
        }
        
        payload = {
            'jsonrpc': '2.0',
            'id': 1,
            'method': 'call_tool',
            'params': {
                'name': tool_name,
                'arguments': args
            }
        }
        
        response = requests.post(
            self.base_url,
            headers=headers,
            json=payload
        )
        
        return response.json()

# Usage
client = McpApiKeyClient('http://localhost:3000/mcp', 'dev-key-123')
result = client.call_tool('math_add', {'a': 5, 'b': 3})
```

## Security Considerations

### API Key Security

1. **Transmission**: API keys should be transmitted over HTTPS in production
2. **Storage**: Never store API keys in client-side code or logs
3. **Rotation**: Implement regular API key rotation
4. **Scope**: Use principle of least privilege for key permissions

### Rate Limiting

Future implementation should include:
- Per-key rate limiting
- IP-based rate limiting
- Burst protection
- Graceful degradation

### Audit Logging

Recommended audit logging includes:
- Authentication attempts (success/failure)
- Tool invocations with user context
- Resource access patterns
- Error conditions and responses

## Health Endpoints

### Server Health

```bash
# Check server health (no auth required)
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "healthy",
  "timestamp": "2025-09-19T04:14:44.652Z",
  "version": "0.1.0",
  "uptime_seconds": 1234
}
```

### Authentication Health

```bash
# Check authentication system
curl -H "X-API-Key: dev-key-123" http://localhost:3000/auth/health
```

Response:
```json
{
  "status": "authenticated",
  "user_id": "dev-user",
  "permissions": ["read", "write"],
  "key_source": "header"
}
```