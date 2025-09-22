# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-09-22

### 🚀 Major Features

#### Transport Architecture Refactoring (TASK-034)
- **Complete transport client-server architecture** with new `TransportClient` pattern
- **Unified transport layer** supporting both STDIO and HTTP protocols
- **MessageHandler<T> pattern** for type-safe, transport-agnostic message processing
- **6 comprehensive integration examples** demonstrating real-world usage patterns
- **100% JSON-RPC 2.0 compliance** with comprehensive validation testing

#### HTTP Transport Zero-Dyn Architecture (TASK-030)
- **Generic HttpTransport<E>** eliminating dynamic dispatch patterns
- **Engine-agnostic design** supporting any HTTP engine implementation
- **Progressive developer experience** with 4-tier learning curve (zero-config → async)
- **Compile-time optimization** with true generic design

#### OAuth2 + PKCE Authentication (TASK-032)
- **Complete OAuth2 authorization flow** with PKCE support
- **MCP Inspector compatibility** for testing and development
- **Three-server proxy architecture** for production deployment
- **Authorization code management** with thread-safe in-memory storage
- **RFC 8414 compliant discovery** endpoint

#### Documentation Overhaul (TASK-010)
- **Professional documentation standards** with eliminated fictional APIs
- **Architecture accuracy** reflecting current TransportClient patterns  
- **Working code examples** verified against actual implementation
- **Custom transport implementation guides** for extensibility

### 💥 Breaking Changes

#### API Architecture Changes
- **Replaced McpServerBuilder pattern** with `TransportClient + MessageHandler<T>`
- **Updated server initialization**:
  ```rust
  // OLD (v0.1.x)
  let server = McpServerBuilder::new()
      .with_transport(stdio_transport)
      .build()?;

  // NEW (v0.2.0)  
  let transport = StdioTransportBuilder::new().build()?;
  let server = McpServer::new(MessageHandler::new());
  server.run(transport).await?;
  ```

#### Module Organization Changes
- **Unified src/protocol/ module** replacing scattered protocol modules
- **Transport-agnostic design** with consistent patterns across STDIO/HTTP
- **Updated import paths**:
  ```rust
  // OLD import paths
  use airs_mcp::shared::protocol::{Content, Tool};
  use airs_mcp::integration::mcp::{McpError, McpResult};
  
  // NEW import paths  
  use airs_mcp::protocol::types::{Content, Tool};
  use airs_mcp::integration::{McpError, McpResult};
  ```

#### HTTP Transport API Changes
- **Generic HttpTransport<E>** replaces trait object patterns
- **New builder patterns** for HTTP transport configuration
- **Engine-specific optimizations** with compile-time dispatch

### ✨ New Features

#### Transport Layer
- **Custom transport support** with comprehensive implementation guides
- **Transport lifecycle management** with proper startup/shutdown handling
- **Background task coordination** with synchronization primitives
- **Request/response correlation** with JSON-RPC 2.0 compliance

#### Authentication & Authorization
- **OAuth2 + PKCE flow** with production-ready implementation
- **Authorization server** with discovery endpoint support
- **Token management** with expiration and cleanup
- **MCP Inspector integration** for development workflow

#### Integration Examples
- **stdio-server-integration**: Complete STDIO server with modular architecture
- **stdio-client-integration**: Client implementation with comprehensive testing
- **http-oauth2-server-integration**: OAuth2 authentication server
- **http-oauth2-client-integration**: OAuth2 client implementation
- **http-apikey-server-integration**: API key authentication
- **http-apikey-client-integration**: API key client implementation

### 🔧 Internal Improvements

#### Code Quality
- **Zero warnings compliance** across entire codebase
- **Workspace standards adherence** with proper import organization
- **Comprehensive test coverage** with integration and unit tests
- **Memory safety** with proper resource management

#### Performance
- **Compile-time optimizations** through generic design patterns
- **Reduced dynamic dispatch** with zero-dyn architecture
- **Efficient JSON-RPC processing** with minimal allocations
- **Background task optimization** with proper async patterns

### 📚 Documentation

#### Comprehensive Guides
- **mdBook documentation** with professional quality standards
- **Architecture documentation** reflecting current implementation
- **Integration examples** with step-by-step guides
- **Custom transport guides** for extension development

#### API Documentation
- **Complete rustdoc coverage** for all public APIs
- **Working code examples** in documentation
- **Clear error handling patterns** with proper documentation
- **Migration guidance** for version transitions

### 🔧 Migration

#### Migration Strategy
**Recommendation**: Start fresh with v0.2.0 architecture rather than migrating v0.1.x code.

**Key Changes Required**:
1. **Update dependencies**: `airs-mcp = "0.2.0"`
2. **Adopt new patterns**: Use `TransportClient + MessageHandler<T>` architecture
3. **Update imports**: Use new module organization paths
4. **Follow examples**: 6 integration examples provide implementation patterns

**Deprecation Notice**: v0.1.x will not receive further updates. All development continues on v0.2.0+.

### 🏗️ Development

#### Examples Structure
```
crates/airs-mcp/examples/
├── stdio-server-integration/     # Complete STDIO server implementation
├── stdio-client-integration/     # STDIO client with testing suite
├── http-oauth2-server-integration/   # OAuth2 authentication server  
├── http-oauth2-client-integration/   # OAuth2 client implementation
├── http-apikey-server-integration/   # API key authentication
└── http-apikey-client-integration/   # API key client
```

#### Testing
- **16 comprehensive Python tests** for client integration validation
- **JSON-RPC 2.0 compliance testing** with 15/15 validation tests passing
- **Transport layer testing** with error scenario coverage
- **OAuth2 flow testing** with complete authorization cycle validation

## [0.1.1] - Previous Release

### Bug Fixes
- Minor bug fixes and improvements
- Documentation updates

## [0.1.0] - Initial Release

### Features
- Initial MCP implementation
- Basic transport layer
- JSON-RPC 2.0 foundation

---

## Migration from v0.1.x

Due to the extensive architectural improvements in v0.2.0, we recommend starting fresh rather than attempting to migrate existing v0.1.x code. The new architecture provides significant benefits:

- **Better Performance**: Zero-dyn design with compile-time optimizations
- **Improved Reliability**: Comprehensive testing and validation
- **Enhanced Security**: OAuth2 + PKCE authentication support
- **Developer Experience**: 6 working examples and comprehensive documentation

For detailed implementation patterns, see the integration examples in `crates/airs-mcp/examples/`.

## Support

- **Documentation**: [airs-mcp documentation](https://docs.rs/airs-mcp)
- **Examples**: See `crates/airs-mcp/examples/` for working implementations
- **Issues**: [GitHub Issues](https://github.com/airsstack/airsstack/issues)