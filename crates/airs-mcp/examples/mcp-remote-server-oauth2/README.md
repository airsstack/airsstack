# OAuth2 MCP Remote Server

A standalone OAuth2-protected MCP (Model Context Protocol) server for testing and development. This server demonstrates how to integrate OAuth2 JWT authentication with AirsStack's MCP infrastructure.

## Features

- 🔐 **OAuth2 JWT Authentication** - Full JWT token validation with JWKS
- 🎯 **Scope-based Authorization** - Fine-grained access control for MCP operations  
- 🔑 **Mock JWKS Endpoint** - Test JWT validation infrastructure
- 🎫 **Test Token Generation** - Pre-configured tokens for different scenarios
- 🔍 **MCP Inspector Compatible** - Works with standard MCP tooling
- 📡 **HTTP Transport** - JSON-RPC over HTTP with proper CORS support

## Quick Start

### 1. Run the Server

```bash
cd mcp-remote-server-oauth2
cargo run
```

The server will start with:
- **MCP Endpoint**: `http://localhost:3001/mcp` (OAuth2 protected)
- **Test Tokens**: `http://localhost:3002/auth/tokens`
- **JWKS Endpoint**: `http://localhost:3002/.well-known/jwks.json`

### 2. Get Test Tokens

```bash
curl http://localhost:3002/auth/tokens
```

This returns test tokens for different scenarios:
- `full` - Complete access to all MCP operations
- `tools` - Access to tools operations only
- `resources` - Access to resources operations only
- `readonly` - Read-only access to listings

### 3. Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector-cli \
  --transport http \
  --server-url http://localhost:3001/mcp \
  --header "Authorization: Bearer <your-token>"
```

### 4. Manual Testing with curl

```bash
# Initialize MCP session
curl -H 'Authorization: Bearer <token>' \
     -H 'Content-Type: application/json' \
     -X POST http://localhost:3001/mcp \
     -d '{"jsonrpc":"2.0","id":"test","method":"initialize","params":{}}'

# List available resources
curl -H 'Authorization: Bearer <token>' \
     -H 'Content-Type: application/json' \
     -X POST http://localhost:3001/mcp \
     -d '{"jsonrpc":"2.0","id":"test","method":"resources/list","params":{}}'
```

## Authentication & Authorization

### OAuth2 Configuration

- **Issuer**: `oauth2-mcp-remote-issuer`
- **Audience**: `mcp-oauth2-remote-server`
- **Algorithm**: RS256
- **JWKS URL**: `http://localhost:3002/.well-known/jwks.json`

### Supported Scopes

- `mcp:*` - Full access to all MCP operations
- `mcp:tools:*` - Access to all tool operations
- `mcp:resources:*` - Access to all resource operations
- `mcp:prompts:*` - Access to all prompt operations
- `mcp:tools:list` - List available tools
- `mcp:tools:execute` - Execute tools
- `mcp:resources:list` - List available resources
- `mcp:resources:read` - Read resource contents

## Available MCP Methods

| Method | Description | Required Scopes |
|--------|-------------|----------------|
| `initialize` | Start MCP session | Any valid token |
| `resources/list` | List available resources | `mcp:*` or `mcp:resources:list` |
| `resources/read` | Read resource contents | `mcp:*` or `mcp:resources:read` |
| `tools/list` | List available tools | `mcp:*` or `mcp:tools:list` |
| `tools/call` | Execute tools | `mcp:*` or `mcp:tools:execute` |
| `prompts/list` | List available prompts | `mcp:*` or `mcp:prompts:list` |
| `prompts/get` | Get prompt content | `mcp:*` or `mcp:prompts:get` |

## Project Structure

```
mcp-remote-server-oauth2/
├── Cargo.toml              # Project configuration
├── README.md               # This file
├── src/
│   ├── main.rs            # Main server implementation
│   └── lib.rs             # Library interface
├── keys/
│   ├── test_rsa_key.pem   # RSA private key for JWT signing
│   └── test_rsa_key_pub.pem # RSA public key (for reference)
├── config/                # Configuration files (future use)
├── docs/                  # Additional documentation
└── examples/              # Usage examples (future use)
```

## Server Endpoints

### OAuth2 Protected Endpoints

- `POST /mcp` - Main MCP JSON-RPC endpoint

### Public Endpoints

- `GET /health` - Health check
- `GET /status` - Server status
- `GET /metrics` - Server metrics
- `GET /info` - Server information

### Test Endpoints (Port 3002)

- `GET /.well-known/jwks.json` - JWKS for JWT validation
- `GET /auth/tokens` - Generate test OAuth2 tokens
- `GET /info` - JWKS server information

## Development

### Prerequisites

- Rust 1.70+
- OpenSSL (for key generation)

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Generating New Test Keys

```bash
# Generate new RSA private key
openssl genrsa -out keys/test_rsa_key.pem 2048

# Extract public key
openssl rsa -in keys/test_rsa_key.pem -pubout -out keys/test_rsa_key_pub.pem
```

## Security Considerations

⚠️ **This is a test server for development purposes only**

- Uses fixed RSA keys (not suitable for production)
- JWKS endpoint is mock (real systems use proper key management)
- No rate limiting or advanced security features
- Logs may contain sensitive information

For production use:
- Use proper key rotation and management
- Implement proper JWKS with real OAuth2 provider
- Add rate limiting, monitoring, and security headers
- Use TLS/HTTPS in production
- Implement proper error handling and logging

## Integration with AirsStack

This server demonstrates how to use AirsStack MCP components:

- **Authentication**: `airs_mcp::authentication::strategies::oauth2`
- **HTTP Transport**: `airs_mcp::transport::adapters::http`
- **Providers**: File system, tools, and prompt providers
- **Infrastructure**: Connection management, session handling, JSON-RPC processing

## Troubleshooting

### Common Issues

1. **Server fails to start with "InvalidKeyFormat"**
   - Check that `keys/test_rsa_key.pem` exists and is valid
   - Regenerate keys if necessary

2. **Token validation fails**
   - Ensure JWKS server is running on port 3002
   - Check token expiration times
   - Verify scope mappings

3. **MCP Inspector connection fails**
   - Verify server is running on port 3001
   - Check OAuth2 token is included in headers
   - Ensure token has required scopes

### Debugging

Enable debug logging:

```bash
RUST_LOG=debug cargo run
```

Check server endpoints:

```bash
# Health check
curl http://localhost:3001/health

# Server info
curl http://localhost:3002/info

# JWKS endpoint
curl http://localhost:3002/.well-known/jwks.json
```

## License

MIT OR Apache-2.0
