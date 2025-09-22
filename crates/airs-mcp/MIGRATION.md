# Migration Guide: v0.1.x → v0.2.0

## Recommended Migration Strategy

**⚠️ Important**: Due to extensive architectural changes in v0.2.0, we **strongly recommend starting fresh** rather than attempting to migrate existing v0.1.x code.

## Why Fresh Start is Better

The v0.2.0 release includes fundamental architectural improvements that make migration complex:

- ✅ **Complete transport architecture redesign** with TransportClient pattern
- ✅ **New MessageHandler<T> system** for type-safe message processing  
- ✅ **Module reorganization** with unified protocol structure
- ✅ **Enhanced OAuth2 + PKCE** authentication system
- ✅ **Zero-dyn HTTP architecture** with generic design patterns

These changes provide significant benefits but require adopting new patterns entirely.

## Deprecation Notice

**v0.1.x End of Life**: 
- v0.1.0 and v0.1.1 will not receive further updates
- All future development continues on v0.2.0+
- No security patches or bug fixes for v0.1.x versions

## Fresh Start Migration Steps

### 1. Update Dependencies

```toml
[dependencies]
airs-mcp = "0.2.0"
```

### 2. Choose Your Starting Point

Pick the most relevant integration example for your use case:

```
crates/airs-mcp/examples/
├── stdio-server-integration/       # STDIO server implementation
├── stdio-client-integration/       # STDIO client with testing
├── http-oauth2-server-integration/ # OAuth2 authentication server  
├── http-oauth2-client-integration/ # OAuth2 client implementation
├── http-apikey-server-integration/ # API key authentication
└── http-apikey-client-integration/ # API key client
```

### 3. Study the New Architecture

**Key Pattern Changes**:

```rust
// OLD v0.1.x Pattern (deprecated)
let server = McpServerBuilder::new()
    .with_transport(stdio_transport)
    .build()?;

// NEW v0.2.0 Pattern  
let transport = StdioTransportBuilder::new().build()?;
let handler = YourMessageHandler::new();
let server = McpServer::new(handler);
server.run(transport).await?;
```

### 4. Follow Example Patterns

Rather than migrating code line-by-line:

1. **Copy example structure** that matches your needs
2. **Adapt the business logic** to your requirements
3. **Use new import paths** and API patterns
4. **Test with the new architecture** from the start

### 5. Key Import Changes

```rust
// OLD imports (v0.1.x)
use airs_mcp::shared::protocol::{Content, Tool};
use airs_mcp::integration::mcp::{McpError, McpResult};

// NEW imports (v0.2.0)
use airs_mcp::protocol::types::{Content, Tool};
use airs_mcp::integration::{McpError, McpResult};
use airs_mcp::transport::{TransportClient, StdioTransportBuilder};
use airs_mcp::integration::MessageHandler;
```

## What You Gain in v0.2.0

### Performance Improvements
- **Zero-dyn architecture** with compile-time optimizations
- **Reduced allocations** in JSON-RPC processing
- **Better async patterns** with proper resource management

### Enhanced Security  
- **OAuth2 + PKCE** authentication with production-ready implementation
- **API key authentication** with proper validation
- **Transport security** with comprehensive error handling

### Developer Experience
- **6 working examples** with comprehensive documentation
- **Clear architecture patterns** that scale to production
- **Professional documentation** with verified code examples
- **Comprehensive testing** with Python test suites

### Reliability
- **100% JSON-RPC 2.0 compliance** with validation testing
- **Transport layer reliability** with proper lifecycle management
- **Error handling** with comprehensive coverage
- **Type safety** throughout the protocol implementation

## Getting Help

### Documentation
- **Examples**: Start with `crates/airs-mcp/examples/`
- **API Docs**: https://docs.rs/airs-mcp/0.2.0
- **Architecture Guide**: See mdBook documentation in `docs/`

### Common Patterns
- **Server Implementation**: See `stdio-server-integration` example
- **Client Implementation**: See `stdio-client-integration` example  
- **Authentication**: See `http-oauth2-*` examples
- **Custom Transports**: See architecture documentation

### Support
- **GitHub Issues**: Report problems or ask questions
- **Example Code**: All examples are working and tested
- **Documentation**: Comprehensive guides in the repository

## Timeline

- **v0.2.0 Release**: September 22, 2025
- **v0.1.x End of Support**: Immediate (no further updates)
- **Migration Window**: Indefinite (no forced migration timeline)
- **Support**: v0.2.0+ only going forward

## Bottom Line

v0.2.0 represents a mature, production-ready architecture that's worth adopting fresh rather than migrating incrementally. The examples provide excellent starting points for any use case.

**Start with an example, adapt to your needs, and enjoy the improved architecture!**