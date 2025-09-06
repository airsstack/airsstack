# Production-Ready OAuth2 MCP Server Example

This example demonstrates a production-grade MCP (Model Context Protocol) server with OAuth2 authentication and authorization, comprehensive error handling, structured logging, and real-world security practices.

## Features

- **Zero-Cost OAuth2 Authorization**: Uses compile-time generics for maximum performance
- **JSON-RPC Method Extraction**: Proper method extraction from JSON-RPC payloads (fixes the old path-based extraction bug)
- **Production Providers**: Includes filesystem, math tools, and code review prompt providers
- **Comprehensive Logging**: Structured logging with tracing support
- **Health Checks**: Built-in health monitoring and status endpoints
- **Development Token Endpoint**: Mock token generation for testing (not for production use)

## Architecture

The server uses the refactored `AxumHttpServer` with its fluent API for OAuth2 integration, showcasing:

- **Layered Authorization**: Authentication happens at the HTTP transport layer, authorization at the JSON-RPC layer
- **Zero-Cost Abstractions**: No dynamic dispatch or runtime overhead for authentication/authorization
- **Proper Scope Validation**: MCP method-to-OAuth scope mappings with fine-grained access control

## MCP Capabilities

The server provides the following MCP capabilities:

### Resources
- **Filesystem Provider**: Secure access to files and directories with path traversal protection
- Resource endpoints: `resources/list`, `resources/read`

### Tools
- **Math Provider**: Mathematical calculations and operations
- Tool endpoints: `tools/list`, `tools/call` (for math operations)

### Prompts
- **Code Review Provider**: Code review and analysis prompts
- Prompt endpoints: `prompts/list`, `prompts/get`

## OAuth2 Configuration

### Environment Variables

Configure OAuth2 settings via environment variables:

```bash
export OAUTH2_ISSUER="https://your-auth-server.com"
export OAUTH2_JWKS_URL="https://your-auth-server.com/.well-known/jwks.json" 
export OAUTH2_AUDIENCE="mcp-server"
```

### Default Scope Mappings

The server uses these default OAuth scope mappings:

- `tools/call` → `mcp:tools:execute`
- `tools/list` → `mcp:tools:read`
- `resources/read` → `mcp:resources:read`
- `resources/list` → `mcp:resources:list`
- `prompts/get` → `mcp:prompts:read`
- `prompts/list` → `mcp:prompts:list`

## Running the Server

### Prerequisites

- Rust 1.70+
- Valid OAuth2 configuration (or use development mode)

### Development Mode

For testing and development, you can run the server with default/mock settings:

```bash
cargo run
```

The server will start on `127.0.0.1:3001` with:
- Mock OAuth2 configuration pointing to `https://auth.example.com`
- Development token endpoint at `/auth/token` 
- Temporary filesystem provider directory

### Production Mode

For production deployment:

1. Set proper OAuth2 environment variables
2. Update authentication configuration:
   - Set `include_error_details: false`
   - Set `require_https: true` (when HTTPS is configured)
   - Configure proper JWKS endpoint and issuer

```bash
export OAUTH2_ISSUER="https://your-production-auth-server.com"
export OAUTH2_JWKS_URL="https://your-production-auth-server.com/.well-known/jwks.json"
export OAUTH2_AUDIENCE="production-mcp-server"

cargo run --release
```

## API Endpoints

### MCP Protocol
- `POST /mcp` - Main MCP JSON-RPC endpoint (requires OAuth2 Bearer token)

### Health & Status
- `GET /health` - Health check (no authentication required)
- `GET /server/info` - Server information and capabilities
- `GET /auth/info` - OAuth2 authentication information

### Development/Testing
- `POST /auth/token` - Generate development token (⚠️ **NOT FOR PRODUCTION**)

## Testing the Server

### 1. Start the Server

```bash
cargo run
```

### 2. Get a Development Token

```bash
curl -X POST http://127.0.0.1:3001/auth/token
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "mcp:tools:execute mcp:resources:read mcp:prompts:read",
  "note": "Development token - not for production use"
}
```

### 3. Test MCP Endpoints

#### List Available Tools

```bash
curl -X POST http://127.0.0.1:3001/mcp \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list"
  }'
```

#### Call a Math Tool

```bash
curl -X POST http://127.0.0.1:3001/mcp \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "calculate",
      "arguments": {"expression": "2 + 2"}
    }
  }'
```

#### List Resources

```bash
curl -X POST http://127.0.0.1:3001/mcp \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0", 
    "id": 3,
    "method": "resources/list"
  }'
```

## Integration with MCP Inspector

This server is designed to work with the MCP Inspector CLI tool for comprehensive testing:

1. **Start the server**: `cargo run`
2. **Configure MCP Inspector** with the server endpoint and OAuth2 settings
3. **Test authentication flow**: Verify that the old `mcp:mcp:*` scope error is gone
4. **Test method authorization**: Confirm proper JSON-RPC method extraction and scope validation

## Security Features

### Path Traversal Protection
The filesystem provider includes comprehensive path traversal protection to prevent access to unauthorized directories.

### Scope-Based Authorization
Each MCP method requires specific OAuth scopes, providing fine-grained access control.

### Audit Logging
All authentication attempts, authorization decisions, and MCP operations are logged for security monitoring.

### Production Hardening
- HTTPS enforcement (configurable)
- Proper error message handling (no sensitive info leakage)
- Secure session management
- Rate limiting and connection limits

## Performance Characteristics

- **Zero-Cost Authorization**: No runtime performance penalty for authentication/authorization
- **Concurrent Processing**: Handles up to 100 concurrent JSON-RPC requests
- **Connection Pooling**: Supports up to 1000 concurrent HTTP connections
- **Efficient Memory Usage**: Stack-allocated configurations and zero-copy request processing

## Monitoring and Observability

The server provides comprehensive monitoring capabilities:

- **Structured Logging**: JSON-formatted logs with correlation IDs
- **Health Endpoints**: Real-time server health and uptime information
- **Connection Metrics**: Active connections, request counts, and performance statistics
- **OAuth2 Metrics**: Authentication success/failure rates and token validation times

## Troubleshooting

### Common Issues

1. **Authentication Failures**: 
   - Verify JWKS URL is accessible
   - Check token expiration
   - Confirm issuer and audience configuration

2. **Authorization Errors**:
   - Verify token contains required scopes
   - Check method-to-scope mappings
   - Confirm JSON-RPC method extraction is working

3. **Connection Issues**:
   - Check server binding address and port
   - Verify firewall and network settings
   - Review connection limits and timeouts

### Debug Mode

Enable detailed debug logging:

```bash
RUST_LOG=airs_mcp=debug,mcp_oauth2_server=debug cargo run
```

This will provide comprehensive logging of:
- OAuth2 token validation
- Scope checking decisions  
- MCP method routing
- Provider interactions
- Transport layer operations

## Contributing

This example serves as both a working server and a reference implementation. When contributing:

1. Follow the layered import standards (stdlib, third-party, internal)
2. Maintain zero-cost abstractions where possible
3. Add comprehensive error handling and logging
4. Update tests and documentation
5. Verify compatibility with MCP Inspector CLI

## License

This example is part of the AirsStack project and follows the same MIT OR Apache-2.0 dual licensing.
