# AIRS-MCP Examples

This directory contains **production-ready example implementations** demonstrating modern usage patterns for the `airs-mcp` library with the **TransportClient architecture**.

## 🏗️ Example Architecture

All examples follow the **modernized integration pattern** with consistent structure and comprehensive testing:

### **Server Integration Examples** (`*-server-integration`)
- **Full MCP servers** with standardized tool sets
- **Production configuration patterns** for development environments  
- **Comprehensive test suites** with automated validation
- **Transport-specific implementations** (HTTP, STDIO)

### **Client Integration Examples** (`*-client-integration`) - *Coming Soon*
- **MCP clients** using the new TransportClient interface
- **Mock server patterns** for simplified integration testing
- **Transport abstraction demonstrations**
- **Error handling and retry patterns**

## 📋 Available Examples

### [http-oauth2-server-integration](./http-oauth2-server-integration/) ✅ **Ready**

**Production-ready OAuth2 MCP server** with three-server architecture:

- **🔐 OAuth2 Authorization Flow**: Complete PKCE implementation with `/authorize` and `/token` endpoints
- **🏗️ Three-Server Architecture**: Smart proxy (3002) + Custom routes (3003) + MCP server (3001) + JWKS (3004)
- **🎫 Token-Based Authentication**: JWT tokens with scope-based authorization
- **📊 Comprehensive Testing**: 34/34 tests passing with full OAuth2 flow validation
- **🔍 MCP Inspector Compatible**: Direct integration with `@modelcontextprotocol/inspector-cli`

**Features:**
- Complete OAuth2 Authorization Code Flow with PKCE
- Thread-safe authorization code management with expiration
- RFC 8414 compliant OAuth2 discovery metadata
- Four token types with different scopes and expiration times
- Comprehensive error handling for invalid codes and verifiers

**To run:**
```bash
cd examples/http-oauth2-server-integration
cargo run
```

**To test:**
```bash
cd examples/http-oauth2-server-integration/tests
python run_tests.py all
```

## 🚀 Coming Soon - Phase 4 Modernization

### Planned Server Examples
- **`stdio-server-integration`**: MCP server using STDIO transport
- **`http-apikey-server-integration`**: HTTP server with API key authentication

### Planned Client Examples  
- **`http-oauth2-client-integration`**: OAuth2 client with mock authorization server
- **`stdio-client-integration`**: STDIO client with simplified mock server
- **`http-apikey-client-integration`**: HTTP client with API key authentication

## 🛠️ Development Standards

All examples follow the **AIRS workspace standards**:

- **3-Layer Import Organization** (std → third-party → internal)
- **chrono DateTime<Utc>** for all time operations
- **Zero Warning Policy** with comprehensive error handling
- **Standardized Tool Set**: file operations, system info, utilities
- **Comprehensive Documentation** with setup guides and API references
- **Automated Testing** with Python-based test suites

## 📖 Getting Started

1. **Choose an example** based on your transport and authentication needs
2. **Read the example's README.md** for specific setup instructions
3. **Run the automated tests** to verify functionality
4. **Explore the source code** to understand implementation patterns
5. **Adapt the patterns** for your specific use case

## 🔗 Related Documentation

- **[AIRS-MCP Library Documentation](../README.md)**: Core library usage
- **[Transport Architecture](../src/transport/)**: Understanding transport abstractions  
- **[Authentication Patterns](../src/authentication/)**: Authentication and authorization
- **[MCP Protocol Implementation](../src/protocol/)**: JSON-RPC and MCP compliance
