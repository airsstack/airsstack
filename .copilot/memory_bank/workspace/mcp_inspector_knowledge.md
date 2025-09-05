# MCP Inspector Knowledge Base

**Document Type**: Tool Analysis & Integration Guide  
**Created**: 2025-09-05  
**Source**: https://modelcontextprotocol.io/legacy/tools/inspector  
**Relevance**: Critical for airsstack MCP development and testing  

## Overview

The **MCP Inspector** is an interactive developer tool for testing and debugging Model Context Protocol (MCP) servers. It serves as the primary testing and validation tool in the MCP ecosystem, providing a visual interface for exploring server capabilities, testing functionality, and debugging issues.

## Key Architecture & Purpose

### Core Function
- **Primary Purpose**: Interactive testing and debugging of MCP servers
- **Installation Method**: No installation required - runs directly via `npx`
- **Target Audience**: MCP server developers and integrators
- **Development Stage**: Production-ready tool maintained by MCP ecosystem

### Technical Foundation
- **Runtime**: Node.js-based tool executed via npx
- **Interface**: Web-based interactive interface with multiple functional tabs
- **Transport Support**: Configurable transport layer support for different server connection methods
- **Protocol Compliance**: Fully compliant with MCP 2024-11-05 specification

## Installation & Usage Patterns

### Basic Installation
```bash
npx @modelcontextprotocol/inspector <command> <args>
```

### Common Usage Scenarios

#### 1. NPM Package Servers
```bash
npx -y @modelcontextprotocol/inspector npx <package-name> <args>
# Example:
npx -y @modelcontextprotocol/inspector npx @modelcontextprotocol/server-filesystem /Users/username/Desktop
```

#### 2. PyPI Package Servers  
```bash
npx -y @modelcontextprotocol/inspector uvx <package-name> <args>
# Example:
npx -y @modelcontextprotocol/inspector uvx mcp-server-git --repository /path/to/repo
```

#### 3. Local Development Servers
```bash
# TypeScript/Node.js
npx @modelcontextprotocol/inspector node path/to/server/index.js args...

# Python
npx @modelcontextprotocol/inspector python path/to/server.py args...
```

## Interface Components & Features

### 1. Server Connection Pane
**Purpose**: Manage server connection and transport configuration
- **Transport Selection**: Choose appropriate transport method for server communication
- **Command Configuration**: Customize command-line arguments and environment variables
- **Connection Status**: Real-time connection state monitoring

### 2. Resources Tab
**Purpose**: Explore and test MCP resource capabilities
- **Resource Discovery**: Lists all available resources from the server
- **Metadata Inspection**: Display MIME types, descriptions, and resource properties
- **Content Access**: View and inspect actual resource content
- **Subscription Testing**: Test resource subscription functionality where supported

### 3. Prompts Tab  
**Purpose**: Test and validate prompt template functionality
- **Template Discovery**: Display available prompt templates
- **Argument Analysis**: Show prompt arguments, types, and descriptions
- **Interactive Testing**: Execute prompts with custom argument values
- **Message Preview**: Preview generated messages and prompt outputs

### 4. Tools Tab
**Purpose**: Test and debug tool execution capabilities
- **Tool Enumeration**: List all available tools provided by server
- **Schema Validation**: Display tool schemas, input requirements, and descriptions
- **Interactive Execution**: Execute tools with custom input parameters
- **Result Analysis**: View tool execution results and error handling

### 5. Notifications Pane
**Purpose**: Monitor server communication and debugging information
- **Log Aggregation**: Collect and display all server logs
- **Notification Handling**: Show server-sent notifications and events
- **Debug Information**: Real-time debugging output and protocol messages

## Development Workflow Integration

### Recommended Development Process

#### Phase 1: Initial Development
1. **Launch Inspector**: Start with basic server implementation
2. **Verify Connectivity**: Ensure basic MCP protocol handshake works
3. **Capability Negotiation**: Confirm server capabilities are properly advertised

#### Phase 2: Iterative Testing
1. **Code Changes**: Make server modifications
2. **Server Rebuild**: Rebuild/restart the server
3. **Inspector Reconnect**: Reconnect Inspector to updated server
4. **Feature Testing**: Test affected functionality through Inspector interface
5. **Message Monitoring**: Use notifications pane to monitor protocol communication

#### Phase 3: Edge Case Validation
1. **Invalid Input Testing**: Test server behavior with malformed inputs
2. **Missing Argument Handling**: Validate prompt argument validation
3. **Concurrent Operation Testing**: Test server behavior under load
4. **Error Response Validation**: Verify proper error handling and reporting

## Integration with airsstack Development

### Relevance to airs-mcp Implementation

#### Server Testing Capabilities
- **Protocol Validation**: Verify our airs-mcp server implementations comply with MCP specification
- **Transport Testing**: Test our HTTP and subprocess transports work correctly with Inspector
- **Authentication Testing**: Validate our zero-cost authentication system integration
- **Performance Monitoring**: Monitor server performance and response times

#### Client Testing Opportunities  
- **Client Compatibility**: Ensure our airs-mcp client can interact with Inspector-validated servers
- **Protocol Compliance**: Verify our client correctly handles all MCP protocol features
- **Error Handling**: Test client robustness against various server responses

### Recommended airsstack Testing Workflow

#### 1. Server Development Testing
```bash
# Test our airs-mcp-fs filesystem server
cd crates/airs-mcp-fs
cargo build --release
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-fs generate-config
```

#### 2. Authentication System Validation
```bash
# Test various authentication configurations
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-server --auth-config api-key
npx @modelcontextprotocol/inspector ./target/release/airs-mcp-server --auth-config oauth2
```

#### 3. Example Server Validation
```bash
# Test our MCP server examples
cd crates/airs-mcp/examples/simple-mcp-server
cargo run &
npx @modelcontextprotocol/inspector node dist/server.js
```

## Advanced Testing Scenarios

### Security Testing Integration
- **Authentication Flow Testing**: Validate authentication mechanisms work with Inspector
- **Permission Validation**: Test security frameworks through Inspector's resource/tool access
- **Error Boundary Testing**: Verify security failures are handled gracefully

### Performance Testing
- **Response Time Monitoring**: Use Inspector to monitor server response characteristics  
- **Concurrent Request Testing**: Test server behavior under multiple Inspector sessions
- **Resource Usage Analysis**: Monitor server resource consumption during Inspector testing

### Protocol Compliance Testing
- **MCP 2024-11-05 Compliance**: Verify all protocol features work correctly
- **Transport Layer Testing**: Test various transport implementations
- **Error Handling Validation**: Ensure proper JSON-RPC error responses

## Best Practices & Recommendations

### Development Workflow Best Practices
1. **Early Integration**: Use Inspector from the beginning of server development
2. **Continuous Testing**: Integrate Inspector testing into development loops  
3. **Edge Case Focus**: Use Inspector to systematically test error conditions
4. **Performance Monitoring**: Monitor Inspector's notifications pane for performance insights

### Quality Assurance Integration
1. **Automated Testing Complement**: Use Inspector alongside automated test suites
2. **Manual Validation**: Use Inspector for manual testing scenarios automated tests can't cover
3. **Debugging Workflow**: Use Inspector as primary debugging tool for MCP issues
4. **Client Integration Testing**: Test both server and client implementations together

### Production Readiness Validation
1. **Full Feature Testing**: Validate all MCP capabilities work through Inspector
2. **Error Response Testing**: Ensure all error conditions produce appropriate responses
3. **Performance Benchmarking**: Use Inspector to establish performance baselines
4. **Compatibility Testing**: Test server compatibility with Inspector and other MCP clients

## Technical Debt & Limitations

### Known Limitations
- **Node.js Dependency**: Requires Node.js runtime environment
- **Local Development Focus**: Primarily designed for local development testing
- **Transport Limitations**: May not support all custom transport implementations
- **Performance Testing Scope**: Limited performance testing capabilities compared to dedicated tools

### Integration Considerations for airsstack
- **Rust Server Compatibility**: Ensure our Rust servers work correctly with Node.js-based Inspector
- **Authentication Integration**: Verify Inspector can handle our authentication systems
- **Custom Transport Support**: Test our custom HTTP transports work with Inspector
- **Performance Monitoring**: Supplement Inspector with dedicated performance testing tools

## Repository & Resources

- **GitHub Repository**: https://github.com/modelcontextprotocol/inspector
- **Official Documentation**: https://modelcontextprotocol.io/legacy/tools/inspector  
- **Related Tools**: MCP Debugging Guide (https://modelcontextprotocol.io/legacy/tools/debugging)
- **Specification**: MCP 2024-11-05 protocol specification

## Summary for airsstack Development

The MCP Inspector is a **critical tool for airsstack MCP development**, providing:

1. **Validation Framework**: Essential for verifying airs-mcp implementations work correctly
2. **Debugging Capabilities**: Primary tool for debugging MCP protocol issues
3. **Development Workflow Integration**: Natural integration point for iterative development
4. **Quality Assurance**: Validation that our implementations work with standard MCP ecosystem tools

**Recommendation**: Integrate MCP Inspector testing into our development workflow for both airs-mcp server and client implementations, using it as the primary validation tool for MCP protocol compliance and functionality testing.
