# MCP Protocol Implementation

## Protocol Overview

The Model Context Protocol (MCP) is a standardized communication protocol for AI applications to interact with external systems. This implementation provides a complete JSON-RPC 2.0 foundation with MCP-specific extensions.

## Core Protocol Features

### JSON-RPC 2.0 Foundation

The implementation is built on a complete JSON-RPC 2.0 message system:

```rust
use airs_mcp::protocol::types::{JsonRpcRequest, JsonRpcResponse, RequestId};
use serde_json::json;

// Create a request
let request = JsonRpcRequest {
    jsonrpc: "2.0".to_string(),
    method: "tools/list".to_string(),
    params: None,
    id: RequestId::new_string("req-1".to_string()),
};

// Handle responses
match response {
    JsonRpcResponse::Success(success) => {
        println!("Result: {:?}", success.result);
    }
    JsonRpcResponse::Error(error) => {
        println!("Error: {:?}", error.error);
    }
}
```

### Message Types

The protocol supports three core message types:

- **Requests**: Method calls that expect responses
- **Responses**: Results or errors for requests  
- **Notifications**: One-way messages that don't expect responses

### Protocol Lifecycle

MCP connections follow a three-phase lifecycle:

1. **Initialization**: Capability negotiation and handshake
2. **Operation**: Normal request/response operations
3. **Termination**: Clean connection shutdown

## Protocol Capabilities

### Client Capabilities

Clients can advertise support for:

- **Experimental Features**: Optional protocol extensions
- **Sampling**: Server-initiated AI request capabilities

### Server Capabilities

Servers can provide:

- **Resources**: File and data access capabilities
- **Tools**: Function calling capabilities  
- **Prompts**: Template and prompt management
- **Logging**: Audit and monitoring capabilities

## Transport Layer

The protocol is transport-agnostic and supports:

### STDIO Transport

Process-based communication for local servers:

```rust
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;

let transport = StdioTransportClientBuilder::new()
    .command("your-mcp-server")
    .build()
    .await?;
```

### HTTP Transport

RESTful communication with authentication:

```rust
use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};

let transport = HttpTransportClientBuilder::new()
    .endpoint("http://localhost:3000/mcp")?
    .auth(AuthMethod::Bearer { token: "token".to_string() })
    .build()
    .await?;
```

## Error Handling

The protocol defines standard error types:

- **Parse Errors**: Invalid JSON or malformed messages
- **Invalid Request**: Missing or invalid fields
- **Method Not Found**: Unsupported operations
- **Invalid Params**: Incorrect parameter types or values
- **Internal Error**: Server implementation errors

## Security Considerations

### Authentication

HTTP transport supports multiple authentication methods:

- Bearer token authentication
- OAuth2 with PKCE
- API key authentication

### Validation

All messages are validated for:

- JSON-RPC 2.0 compliance
- MCP schema requirements
- Parameter type checking
- Required field presence
