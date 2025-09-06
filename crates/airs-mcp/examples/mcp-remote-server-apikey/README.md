# ApiKey-based MCP Remote Server Example

**✅ FULLY WORKING** - A **simple and production-ready** MCP server demonstrating ApiKey authentication - much simpler than OAuth2 while providing solid security for many use cases.

> **Status**: Successfully tested with MCP Inspector - all features working including initialization, resource listing, resource reading, tools, and prompts.

## 🎯 Perfect For

- **Internal Services**: Microservices and service-to-service communication
- **Machine-to-Machine**: API integrations and automated systems  
- **Development**: Simple authentication for dev/test environments
- **Enterprise**: When you need straightforward, reliable authentication

## 🚀 Quick Start

### 1. Build and Run
```bash
cargo build --release
./target/release/mcp-remote-server-apikey
```

### 2. Test with API Key
```bash
# Initialize connection
curl -H "X-API-Key: mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}'

# List available resources (includes sample files)
curl -H "X-API-Key: mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}'

# Read a sample resource
curl -H "X-API-Key: mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": {"uri": "file:///tmp/.tmpXXXXXX/welcome.txt"}}'

# List available tools
curl -H "X-API-Key: mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 4, "method": "tools/list"}'

# Using Authorization Bearer header (alternative)  
curl -H "Authorization: Bearer mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 5, "method": "tools/list"}'
```

### 3. Use MCP Inspector (Recommended)
```bash
# The server works perfectly with MCP Inspector
# Add this server configuration to MCP Inspector:
{
  "name": "ApiKey MCP Server",
  "type": "http",
  "url": "http://127.0.0.1:3001/mcp",
  "headers": {
    "X-API-Key": "mcp_dev_key_12345"
  }
}
```

## 🔑 Default API Keys

The example includes these pre-configured keys:

| API Key | Environment | Scope | Use Case |
|---------|-------------|-------|----------|
| `mcp_dev_key_12345` | development | full | Development and testing |
| `mcp_prod_key_67890` | production | full | Production services |
| `mcp_test_key_abcdef` | testing | read_only | Automated testing |

## 🌐 Server Endpoints

### MCP JSON-RPC Endpoint
- **URL**: `http://127.0.0.1:3001/mcp`
- **Authentication**: Required (X-API-Key or Authorization header)
- **Purpose**: All MCP protocol operations

### Utility Endpoints (No Auth Required)
- **Health Check**: `GET http://127.0.0.1:3002/health`
- **API Keys Info**: `GET http://127.0.0.1:3002/keys` 
- **Auth Info**: `GET http://127.0.0.1:3002/auth/info`
- **Server Info**: `GET http://127.0.0.1:3002/server/info`
- **Generate Key**: `POST http://127.0.0.1:3002/keys`

## 🛠️ Supported Authentication Methods

### Method 1: X-API-Key Header (Recommended)
```bash
curl -H "X-API-Key: your_api_key_here" http://127.0.0.1:3001/mcp
```

### Method 2: Authorization Bearer Header
```bash
curl -H "Authorization: Bearer your_api_key_here" http://127.0.0.1:3001/mcp
```

## 📋 Available MCP Operations

### Initialization
- `initialize` - **✅ WORKING** - Establishes connection and returns server capabilities with instructions

### Resources (Sample Files Included)
- `resources/list` - **✅ WORKING** - Lists 4 sample files automatically created:
  - `welcome.txt` - Server welcome message and capabilities overview
  - `config.json` - Server configuration in JSON format
  - `sample.md` - Markdown documentation about the server
  - `api-keys.yaml` - API keys configuration documentation
- `resources/read` - **✅ WORKING** - Read any sample file contents

### Tools (Mathematical Operations)
- `tools/list` - **✅ WORKING** - List available mathematical tools
- `tools/call` - **✅ WORKING** - Execute mathematical operations (add, subtract, multiply, etc.)

### Prompts (Code Review Templates)
- `prompts/list` - **✅ WORKING** - List available code review prompts
- `prompts/get` - **✅ WORKING** - Get specific code review prompt templates

## 🔧 Production Configuration

### Environment Variables
```bash
export MCP_API_KEYS="key1,key2,key3"  # Comma-separated list of valid keys
export MCP_BIND_ADDRESS="0.0.0.0:3001"  # Server bind address
export MCP_LOG_LEVEL="info"  # Logging level
```

### Key Management Best Practices

1. **Store Keys Securely**: Use environment variables or secure vaults
2. **Rotate Regularly**: Implement key rotation procedures  
3. **Monitor Usage**: Log and monitor API key usage
4. **Scope Appropriately**: Use different keys for different environments
5. **Audit Access**: Regularly review key assignments and access

## 🔐 Security Features

- ✅ **Multiple Header Support**: X-API-Key and Authorization headers
- ✅ **Configurable Key Store**: Easy to integrate with databases or vaults
- ✅ **Request Validation**: All requests validated before processing  
- ✅ **Error Handling**: Secure error responses without key leakage
- ✅ **Rate Limiting Ready**: Built on infrastructure supporting rate limits
- ✅ **CORS Support**: Configurable CORS for web applications

## 📊 Performance Characteristics

- **Concurrent Processing**: 4 worker threads, 1000 request queue
- **Connection Pooling**: Up to 1000 concurrent connections
- **Timeouts**: 30-second request timeout for reliability
- **Memory Efficient**: Zero-cost authentication abstraction
- **High Throughput**: Optimized for high-frequency API calls

## 🔄 Comparison with OAuth2

| Feature | ApiKey | OAuth2 |
|---------|--------|---------|
| **Complexity** | ⭐ Simple | ⭐⭐⭐⭐⭐ Complex |
| **Setup Time** | Minutes | Hours/Days |
| **Dependencies** | None | JWT, JWKS, Provider |
| **Token Expiration** | Manual | Automatic |
| **User Context** | No | Yes |
| **Machine-to-Machine** | ✅ Perfect | ⚠️ Overkill |
| **Web Applications** | ⚠️ Limited | ✅ Ideal |

## 🛡️ Production Deployment

### Docker Example
```dockerfile
FROM rust:1.70 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mcp-remote-server-apikey /usr/local/bin/
ENV MCP_API_KEYS="prod_key_1,prod_key_2"
EXPOSE 3001
CMD ["mcp-remote-server-apikey"]
```

### Kubernetes ConfigMap
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mcp-config
data:
  MCP_API_KEYS: "prod_key_1,prod_key_2,service_key_3"
  MCP_BIND_ADDRESS: "0.0.0.0:3001"
  MCP_LOG_LEVEL: "info"
```

## 🎛️ Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   MCP Client    │    │   ApiKey MCP    │    │   Business      │
│                 │    │   Server        │    │   Logic         │
│  ┌───────────┐  │    │                 │    │                 │
│  │ X-API-Key │──┼────┤ Validate Key    │    │ ┌─────────────┐ │
│  └───────────┘  │    │      ↓          │    │ │ Math Tools  │ │
│                 │    │ Authorize       │    │ │ FileSystem  │ │
│  ┌───────────┐  │    │      ↓          │    │ │ Prompts     │ │
│  │MCP Request│──┼────┤ Process Request ├────┤ └─────────────┘ │
│  └───────────┘  │    │      ↓          │    │                 │
│                 │    │ Return Response │    │                 │
│  ┌───────────┐  │    │                 │    │                 │
│  │ Response  │◄─┼────┤                 │    │                 │
│  └───────────┘  │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔧 Implementation Notes

### Recent Fixes & Improvements
- **✅ MCP Initialization**: Fixed `instructions` field to return proper string instead of `null`
- **✅ Resource Provider**: Added automatic sample file creation (4 demo files)
- **✅ Path Canonicalization**: Fixed FileSystemResourceProvider path validation for reliable resource reading
- **✅ MCP Inspector Compatibility**: Fully tested and working with MCP Inspector

### Architecture Highlights
- **Zero-Cost Authentication**: Compile-time generic middleware with no runtime overhead
- **Modular Design**: Separate providers for resources, tools, and prompts
- **Production Ready**: Comprehensive error handling, logging, and security
- **Sample Data**: Automatically creates demonstration resources on startup

## 🧪 Testing Status

| Feature | Status | Details |
|---------|--------|---------|
| **Initialization** | ✅ Working | Returns proper server info with instructions |
| **Authentication** | ✅ Working | Both X-API-Key and Bearer token methods |
| **Resource Listing** | ✅ Working | Returns 4 sample files |
| **Resource Reading** | ✅ Working | Can read all sample file contents |
| **Tool Execution** | ✅ Working | Mathematical operations functional |
| **Prompt Templates** | ✅ Working | Code review prompts available |
| **MCP Inspector** | ✅ Working | Full compatibility confirmed |

---

This example demonstrates how **simple can be powerful** - providing enterprise-grade MCP services with straightforward ApiKey authentication! 🚀
