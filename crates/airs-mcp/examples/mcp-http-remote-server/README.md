# MCP HTTP Remote Server

A complete, production-ready implementation of an MCP (Model Context Protocol) HTTP remote server using the AIRS-MCP framework.

## Overview

This server provides a fully compliant MCP HTTP remote server that can be connected to by Claude Desktop and other MCP clients over HTTP. It follows the official MCP specification and implements proper JSON-RPC 2.0 over HTTP transport.

## Features

- **Full MCP Compliance**: Implements complete MCP specification with JSON-RPC 2.0 over HTTP
- **File System Access**: Secure resource provider for reading project files
- **Calculator Tools**: Mathematical operations and utility functions
- **Documentation Prompts**: Code review, best practices, and development guidance
- **Real-time Streaming**: Support for streaming responses and server-sent events
- **Production Ready**: Comprehensive logging, error handling, and security measures
- **Claude Desktop Integration**: Ready-to-use configuration for Claude Desktop

## MCP Capabilities

### Resources
- **File System Access**: Secure read-only access to project files
- **Directory Listing**: Browse project structure
- **Content Reading**: Read file contents with security boundaries
- **Type Detection**: Automatic MIME type detection

### Tools
- **Mathematical Operations**: Add, subtract, multiply, divide, power, sqrt, factorial
- **Random Number Generation**: Configurable range random numbers
- **Input Validation**: Comprehensive argument validation and error handling

### Prompts
- **Code Review**: Comprehensive code analysis with best practices
- **Documentation Guide**: Technical writing assistance
- **Rust Best Practices**: Language-specific guidance
- **API Design**: RESTful and other API design principles
- **Error Handling**: Robust error strategy development
- **Testing Strategy**: Comprehensive testing approaches
- **Performance Review**: Optimization and performance analysis

## Usage

### Running the Server

```bash
cd crates/airs-mcp/examples/mcp-http-remote-server
cargo run
```

The server will start on `http://localhost:3000` with the MCP endpoint at `/mcp`.

### Claude Desktop Configuration

Add the following to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "http-remote": {
      "command": "curl",
      "args": ["-X", "POST", "http://localhost:3000/mcp", "-H", "Content-Type: application/json", "-d", "@-"]
    }
  }
}
```

### Environment Configuration

Set logging level (optional):
```bash
export RUST_LOG=mcp_http_remote_server=info,airs_mcp=info
```

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Claude        │    │  MCP HTTP        │    │   Providers     │
│   Desktop       │◄──►│  Remote Server   │◄──►│                 │
│                 │    │                  │    │ • FileSystem    │
└─────────────────┘    │ • JSON-RPC 2.0   │    │ • Calculator    │
                       │ • HTTP Transport │    │ • Documentation │
                       │ • Session Mgmt   │    │                 │
                       └──────────────────┘    └─────────────────┘
```

## MCP Protocol Support

### Standard Methods
- `initialize` - Start MCP session with capability negotiation
- `initialized` - Confirm initialization completion
- `shutdown` - Graceful session termination

### Resource Methods
- `resources/list` - List available file system resources
- `resources/read` - Read specific file content

### Tool Methods
- `tools/list` - List available calculator and utility tools
- `tools/call` - Execute specific tool with parameters

### Prompt Methods
- `prompts/list` - List available development prompts
- `prompts/get` - Get specific prompt with context

### Notification Methods
- `notifications/progress` - Progress updates for long-running operations
- `notifications/message` - General server messages

## Security Features

- **Path Validation**: Prevents directory traversal attacks
- **File Size Limits**: Configurable maximum file size (1MB default)
- **Extension Filtering**: Only allowed file types accessible
- **Input Sanitization**: Comprehensive argument validation
- **Error Boundaries**: Safe error handling without information leakage

## Development

### Project Structure
```
src/
├── main.rs              # Server initialization and configuration
├── utils.rs             # Logging and utility functions
└── providers/
    ├── mod.rs           # Provider module exports
    ├── filesystem.rs    # File system resource provider
    ├── calculator.rs    # Mathematical tool provider
    └── documentation.rs # Development prompt provider
```

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

### Documentation
```bash
cargo doc --open
```

## Configuration

The server accepts configuration through environment variables and command-line arguments:

- `HOST`: Server host (default: 127.0.0.1)
- `PORT`: Server port (default: 3000)
- `BASE_PATH`: File system access base path (default: current directory)
- `MAX_CONNECTIONS`: Maximum concurrent connections (default: 100)
- `REQUEST_TIMEOUT`: Request timeout in seconds (default: 30)

## Troubleshooting

### Common Issues

1. **Connection Refused**: Ensure server is running and port is available
2. **Permission Denied**: Check file system permissions for base path
3. **Timeout Errors**: Increase request timeout for large files
4. **Invalid JSON**: Verify Claude Desktop configuration format

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

Check server health:
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
```

## License

This example is part of the AIRS-MCP framework and follows the same licensing terms.
