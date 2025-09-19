# HTTP API Key Server Integration

A complete HTTP MCP (Model Context Protocol) server implementation with API key authentication, demonstrating secure HTTP transport for MCP operations.

## Overview

This example provides a production-ready HTTP MCP server with:

- **HTTP Transport**: Full HTTP server using Axum framework
- **API Key Authentication**: Multiple authentication methods (header, bearer, query parameter)
- **Standardized Tools**: Math operations, file system access, system information
- **Comprehensive Error Handling**: Proper MCP error responses and HTTP status codes
- **Security**: Request validation, authentication middleware, and secure defaults

## Features

### Authentication Methods

The server supports three API key authentication methods:

1. **X-API-Key Header**:
   ```bash
   curl -H "X-API-Key: dev-key-123" ...
   ```

2. **Authorization Bearer**:
   ```bash
   curl -H "Authorization: Bearer dev-key-123" ...
   ```

3. **Query Parameter**:
   ```bash
   curl "http://localhost:3000/mcp?api_key=dev-key-123" ...
   ```

### Available Tools

- **Mathematical Operations**: `add`, `subtract`, `multiply`, `divide`, `power`, `sqrt`, `sin`, `cos`, `log`, `factorial`
- **File System Resources**: Read files, list directories from the test resources
- **System Information**: Environment variables, process info, system details

### Development API Keys

For development and testing, the server includes pre-configured API keys:

- `dev-key-123`
- `test-key-456` 
- `demo-key-789`

> **Security Note**: These are development keys only. In production, use proper API key management.

## Quick Start

### 1. Start the Server

```bash
# Navigate to the example directory
cd crates/airs-mcp/examples/http-apikey-server-integration

# Start the server (default port 3000)
cargo run --bin http-apikey-server

# Or specify a different port
cargo run --bin http-apikey-server -- --port 8080
```

### 2. Test MCP Operations

#### List Available Tools

```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

#### Call a Tool

```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{
    "jsonrpc":"2.0",
    "id":2,
    "method":"tools/call",
    "params":{
      "name":"add",
      "arguments":{"numbers":[5,3,2]}
    }
  }'
```

#### List Resources

```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}'
```

#### Read a Resource

```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{
    "jsonrpc":"2.0",
    "id":4,
    "method":"resources/read",
    "params":{
      "uri":"file:///path/to/resource/file.txt"
    }
  }'
```

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────┐
│                   main.rs                       │
│  (CLI argument parsing, server lifecycle)       │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│            HttpApiKeyServer                     │
│  (Server configuration and startup)             │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│        AxumMcpRequestHandler                    │
│  (MCP request routing and processing)           │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│              Providers                          │
│  • FileSystemResourceProvider                  │
│  • MathToolProvider                             │
│  • CodeReviewPromptProvider                     │
│  • StructuredLoggingHandler                     │
└─────────────────────────────────────────────────┘
```

### Authentication Flow

```
┌─────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client    │───▶│  Auth Middle-   │───▶│   MCP Request   │
│   Request   │    │     ware        │    │    Handler      │
└─────────────┘    └─────────────────┘    └─────────────────┘
                           │
                           ▼
                   ┌─────────────────┐
                   │  API Key        │
                   │  Validator      │
                   └─────────────────┘
```

### Key Files

- **`src/main.rs`**: Application entry point and CLI handling
- **`src/config.rs`**: Server configuration management
- **`src/transport/server.rs`**: HTTP server implementation with API key auth
- **`src/tools/mod.rs`**: Tool provider setup and management
- **`Cargo.toml`**: Dependencies and build configuration

## Configuration

### Command Line Options

```bash
cargo run --bin http-apikey-server -- --help
```

```
HTTP MCP Server with API Key Authentication

Usage: http-apikey-server [OPTIONS]

Options:
  -p, --port <PORT>          Server port [default: 3000]
      --api-key <API_KEY>    API key for development
      --dev-mode <DEV_MODE>  Enable development mode [default: true]
  -h, --help                 Print help
```

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `RUST_LOG=debug`)

## Testing

This example includes comprehensive test suites to validate all functionality:

### Quick Testing

Run all tests with the provided script:

```bash
./tests/run_tests.sh
```

This will:
- Set up a Python virtual environment
- Install testing dependencies  
- Run comprehensive integration tests
- Run stress tests and edge case validation

### Manual Testing

Use the curl commands shown above to manually test the server functionality:

```bash
# Start the server
cargo run --bin http-apikey-server

# Test X-API-Key header authentication
curl -H "X-API-Key: dev-key-123" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
     http://127.0.0.1:3000/mcp

# Test Authorization Bearer authentication  
curl -H "Authorization: Bearer test-key-456" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
     http://127.0.0.1:3000/mcp

# Test query parameter authentication
curl -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
     "http://127.0.0.1:3000/mcp?api_key=demo-key-789"
```

### Test Coverage

The test suites provide comprehensive coverage:

**Integration Tests** (`tests/test_http_apikey_integration.py`):
- ✅ All three API key authentication methods
- ✅ Complete MCP protocol operations (tools/list, tools/call, resources/list, resources/read)  
- ✅ Authentication failure scenarios
- ✅ Error handling and invalid requests
- ✅ Concurrent request handling
- ✅ End-to-end workflow testing

**Stress Tests** (`tests/test_stress_validation.py`):
- ✅ Malformed JSON payload handling
- ✅ Missing required fields validation
- ✅ Large payload processing
- ✅ Concurrent mixed operations
- ✅ Rapid authentication method switching
- ✅ Invalid content type handling
- ✅ Unicode character support
- ✅ Response time consistency
- ✅ Sustained load testing

All tests automatically manage server lifecycle and provide detailed logging for debugging.

### Authentication Testing

```bash
# Test without API key (should fail)
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Test with invalid API key (should fail)
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: invalid-key" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Test with valid API key (should succeed)
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-123" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

## Error Handling

The server provides proper error responses for various scenarios:

### Authentication Errors

- **Missing API Key**: `"Missing API key"`
- **Invalid API Key**: `"Authentication error"`

### MCP Protocol Errors

- **Invalid Method**: JSON-RPC error response with method not found
- **Invalid Parameters**: JSON-RPC error response with parameter validation details
- **Tool Execution Errors**: Proper MCP error responses with error content

### HTTP Errors

- **404 Not Found**: For invalid endpoints
- **405 Method Not Allowed**: For unsupported HTTP methods
- **500 Internal Server Error**: For server-side errors

## Security Considerations

### Production Deployment

1. **API Key Management**: 
   - Use secure random API keys
   - Implement proper key rotation
   - Store keys securely (environment variables, secret management systems)

2. **HTTPS**:
   - Always use HTTPS in production
   - Configure proper TLS certificates

3. **Rate Limiting**:
   - Implement rate limiting per API key
   - Add request throttling

4. **Logging**:
   - Log authentication attempts
   - Monitor for suspicious activity
   - Implement proper audit trails

5. **Validation**:
   - Validate all input parameters
   - Sanitize file paths for resource access
   - Implement request size limits

### Development vs Production

This example is configured for development with:
- Pre-configured API keys
- Debug logging enabled
- Permissive error messages

For production deployment, consider:
- Dynamic API key management
- Reduced error message verbosity
- Enhanced monitoring and alerting

## Troubleshooting

### Common Issues

1. **Port Already in Use**:
   ```bash
   # Check what's using the port
   lsof -i :3000
   # Use a different port
   cargo run --bin http-apikey-server -- --port 8080
   ```

2. **Authentication Failures**:
   - Verify API key format and spelling
   - Check the authentication method being used
   - Review server logs for detailed error messages

3. **Tool Execution Errors**:
   - Verify tool parameters match the expected schema
   - Check that required parameters are provided
   - Review tool-specific documentation

### Debug Logging

Enable debug logging to see detailed request processing:

```bash
RUST_LOG=debug cargo run --bin http-apikey-server
```

## Related Examples

- **[STDIO Server Integration](../stdio-server-integration/)**: Basic MCP server with STDIO transport
- **[HTTP OAuth2 Server Integration](../http-oauth2-server-integration/)**: HTTP server with OAuth2 authentication
- **[STDIO Client Integration](../stdio-client-integration/)**: MCP client implementation

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../../../LICENSE-MIT))

at your option.