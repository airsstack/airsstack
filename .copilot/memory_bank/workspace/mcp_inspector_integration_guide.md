# MCP Inspector Integration Guide for airsstack

**Document Type**: Development Workflow Integration Guide  
**Created**: 2025-09-05  
**Purpose**: Practical integration of MCP Inspector with airsstack development  
**Status**: Active Development Tool  

## Integration Overview

This guide provides practical instructions for integrating the MCP Inspector tool into airsstack development workflows, specifically tailored for our airs-mcp and airs-mcp-fs implementations.

## Quick Start Testing Commands

### Test airs-mcp-fs Filesystem Server
```bash
# Build and test the filesystem server
cd crates/airs-mcp-fs
cargo build --release

# Generate config and test with Inspector
./target/release/airs-mcp-fs generate-config
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs /Users/hiraq/Projects/test-directory
```

### Test airs-mcp Simple Server Example  
```bash
# Build and run simple MCP server example
cd crates/airs-mcp/examples/simple-mcp-server
cargo run &

# Test with Inspector (assuming server outputs connection details)
npx @modelcontextprotocol/inspector <connection-command>
```

### Test airs-mcp HTTP Server
```bash
# Build and run HTTP server example
cd crates/airs-mcp/examples
cargo run --example axum_server_with_handlers &

# Test HTTP transport (may require additional configuration)
npx @modelcontextprotocol/inspector <http-connection-command>
```

## airsstack-Specific Testing Scenarios

### 1. Zero-Cost Authentication Testing
Based on our TASK005 zero-cost authentication implementation:

```bash
# Test API Key authentication
cargo run --example axum_server_with_handlers -- --auth api-key
npx @modelcontextprotocol/inspector <server-command-with-auth>

# Test OAuth2 authentication  
cargo run --example axum_server_with_handlers -- --auth oauth2
npx @modelcontextprotocol/inspector <server-command-with-auth>

# Test NoAuth (default)
cargo run --example axum_server_with_handlers
npx @modelcontextprotocol/inspector <server-command-no-auth>
```

### 2. Security Framework Validation
Test our comprehensive security framework from airs-mcp-fs:

```bash
# Test with different security levels
./target/release/airs-mcp-fs --security-level high /safe/directory
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs --security-level high /safe/directory

# Test path traversal protection
./target/release/airs-mcp-fs --security-level maximum /restricted/path
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs --security-level maximum /restricted/path

# Test with human-in-the-loop approval
./target/release/airs-mcp-fs --approval-mode manual /sensitive/directory  
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs --approval-mode manual /sensitive/directory
```

### 3. Performance Validation Testing
Test our sub-100ms performance claims:

```bash
# Build optimized release version
cargo build --release --workspace

# Test with Inspector monitoring response times
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs /performance/test/directory

# Monitor notifications pane for:
# - Response time metrics
# - Memory usage patterns  
# - Concurrent request handling
# - Transport layer performance
```

## Development Workflow Integration

### Phase 1: Feature Development
1. **Implement Feature**: Add new MCP capability to server
2. **Build**: `cargo build --workspace`
3. **Inspector Test**: `npx @modelcontextprotocol/inspector <server-command>`
4. **Validate**: Check all tabs (Resources, Tools, Prompts) show new functionality
5. **Debug**: Use Notifications pane to identify issues

### Phase 2: Integration Testing  
1. **Multi-Transport Testing**: Test STDIO and HTTP transports
2. **Authentication Flow**: Validate authentication systems work with Inspector
3. **Error Handling**: Test malformed inputs and edge cases
4. **Performance**: Monitor response times and resource usage

### Phase 3: Production Readiness
1. **Full Protocol Compliance**: Verify all MCP 2024-11-05 features work
2. **Security Validation**: Test security frameworks hold up under Inspector testing
3. **Documentation Sync**: Ensure server capabilities match documentation
4. **Cross-Platform Testing**: Test on different environments

## Inspector Interface Usage for airsstack

### Server Connection Pane Usage
- **Transport Selection**: Choose STDIO for local development, HTTP for production testing
- **Arguments Configuration**: Test different security levels, authentication modes
- **Environment Variables**: Test various configuration scenarios

### Resources Tab Testing
For airs-mcp-fs filesystem resources:
- **File Discovery**: Verify filesystem resources are properly enumerated
- **MIME Type Detection**: Ensure proper file type detection
- **Content Access**: Test file reading with security restrictions
- **Path Validation**: Verify path traversal protection works

### Tools Tab Testing  
For airs-mcp tool implementations:
- **Tool Schema Validation**: Verify JSON schemas are correct
- **Parameter Testing**: Test tool execution with various inputs
- **Error Handling**: Test invalid parameter handling
- **Result Processing**: Verify tool outputs are properly formatted

### Prompts Tab Testing
For any prompt templates we implement:
- **Template Discovery**: Verify prompts are properly exposed
- **Argument Validation**: Test required vs optional parameters
- **Generation Testing**: Test prompt generation with various inputs
- **Output Formatting**: Verify generated content is properly formatted

### Notifications Pane Monitoring
Critical for airsstack debugging:
- **Authentication Logs**: Monitor authentication success/failure
- **Security Events**: Watch for security framework activations
- **Performance Metrics**: Monitor response times and resource usage
- **Error Tracking**: Catch and analyze any protocol errors

## Common Testing Patterns

### 1. Security Testing Pattern
```bash
# Test increasing security levels
for level in low medium high maximum; do
  echo "Testing security level: $level"
  ./target/release/airs-mcp-fs --security-level $level /test/directory &
  SERVER_PID=$!
  
  npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs --security-level $level /test/directory
  
  kill $SERVER_PID
  sleep 2
done
```

### 2. Authentication Testing Pattern
```bash
# Test different authentication modes
for auth in none api-key oauth2; do
  echo "Testing authentication: $auth"
  cargo run --example axum_server_with_handlers -- --auth $auth &
  SERVER_PID=$!
  
  # Manual Inspector testing here
  echo "Press enter when testing complete..."
  read
  
  kill $SERVER_PID
  sleep 2  
done
```

### 3. Performance Testing Pattern
```bash
# Performance monitoring setup
cargo build --release --workspace

# Start server with performance logging
RUST_LOG=info ./target/release/airs-mcp-fs /large/directory &
SERVER_PID=$!

# Run Inspector and monitor notifications pane
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs /large/directory

# Check for sub-100ms response times in Inspector logs
kill $SERVER_PID
```

## Expected Inspector Behavior with airsstack

### Successful Integration Indicators
1. **Connection Success**: Server connection pane shows "Connected" status
2. **Capability Negotiation**: All implemented capabilities appear in respective tabs
3. **Resource Access**: File system resources appear with correct metadata
4. **Tool Execution**: Tools execute successfully with expected outputs
5. **Authentication Flow**: Authentication challenges/responses work correctly
6. **Error Handling**: Proper JSON-RPC error responses for invalid inputs

### Common Issues & Solutions

#### Authentication Issues
- **Symptom**: Connection fails with authentication errors
- **Solution**: Verify authentication configuration matches server setup
- **Debug**: Check notifications pane for authentication protocol messages

#### Transport Issues  
- **Symptom**: Inspector cannot connect to server
- **Solution**: Verify server is running and transport method is correct
- **Debug**: Check server logs and Inspector connection parameters

#### Protocol Compliance Issues
- **Symptom**: Some features don't work properly in Inspector
- **Solution**: Verify MCP 2024-11-05 specification compliance
- **Debug**: Check JSON-RPC message format in notifications pane

## Integration with Automated Testing

### Complementary to Unit Tests
- **Unit Tests**: Test individual components and functions
- **Inspector Tests**: Test full protocol integration and user workflows
- **Integration**: Use Inspector findings to inform additional unit tests

### CI/CD Integration Considerations
- **Local Development**: Primary Inspector usage during development
- **CI Testing**: Inspector could be integrated for automated protocol compliance testing
- **Release Validation**: Use Inspector as final validation before releases

## Best Practices for airsstack Development

### Development Phase
1. **Early Inspector Usage**: Start using Inspector from first MCP implementation
2. **Iterative Testing**: Use Inspector after each feature addition
3. **Security Focus**: Emphasize security testing through Inspector interface
4. **Performance Monitoring**: Always monitor performance metrics during testing

### Quality Assurance Phase
1. **Complete Feature Testing**: Test all MCP capabilities through Inspector
2. **Edge Case Validation**: Use Inspector to test error conditions systematically
3. **Cross-Platform Testing**: Test on different development environments
4. **Documentation Validation**: Verify Inspector behavior matches documentation

### Production Readiness Phase
1. **Full Protocol Compliance**: Verify complete MCP 2024-11-05 compliance
2. **Performance Benchmarking**: Establish baseline performance metrics
3. **Security Validation**: Comprehensive security testing through Inspector
4. **Client Compatibility**: Verify compatibility with Inspector and other MCP clients

This integration guide provides the practical foundation for incorporating MCP Inspector into our airsstack development workflow, ensuring high-quality MCP implementations that work seamlessly with the broader MCP ecosystem.
