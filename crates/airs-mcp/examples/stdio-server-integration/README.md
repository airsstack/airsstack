# STDIO Server Integration Example

A complete MCP (Model Context Protocol) server implementation using STDIO transport with a standardized set of tools for development and testing environments.

## Overview

This example demonstrates a production-ready MCP server that communicates via stdin/stdout (STDIO transport) and provides three categories of tools:

- **File Operations**: Safe file system operations with security controls
- **System Information**: System and environment information with privacy filtering  
- **Utilities**: Common utility functions for testing and debugging

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo workspace properly configured

### Running the Server

```bash
# Run with default configuration
cargo run --bin stdio-server

# Run with custom configuration
MCP_LOG_LEVEL=debug cargo run --bin stdio-server

# Use as MCP server with client communication
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --bin stdio-server
```

### Testing with Manual Commands

```bash
# Initialize the server
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --bin stdio-server

# List available tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | cargo run --bin stdio-server

# Call a tool (echo example)
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"text":"Hello MCP!"}}}' | cargo run --bin stdio-server

# Health check
echo '{"jsonrpc":"2.0","id":4,"method":"ping","params":{}}' | cargo run --bin stdio-server
```

## Available Tools

### File Operations (4 tools)

| Tool | Description | Security Features |
|------|-------------|-------------------|
| `read_file` | Read file contents | Size limits, extension filtering, path traversal protection |
| `write_file` | Write content to file | Size limits, extension filtering, directory creation |
| `list_directory` | List directory contents | Path validation, metadata inclusion |
| `create_directory` | Create directories | Path validation, recursive option |

**Supported Extensions**: `txt`, `json`, `yaml`, `yml`, `toml`, `md`, `rs`
**Default Max File Size**: 1MB

### System Information (3 tools)

| Tool | Description | Privacy Features |
|------|-------------|------------------|
| `get_system_info` | Basic system information | OS, arch, current directory only |
| `get_environment` | Environment variables | Filtered allowlist, no sensitive vars |
| `get_process_info` | Current process info | PID, executable path, arguments |

**Environment Allowlist**: `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `LANG`, `PWD`, `CARGO_PKG_*`

### Utilities (3 tools)

| Tool | Description | Features |
|------|-------------|----------|
| `echo` | Echo text with formatting | Prefix/suffix, case conversion |
| `timestamp` | Current timestamp | Multiple formats (ISO8601, Unix, human) |
| `health_check` | Server health status | Uptime, instance ID, health checks |

## Configuration

The server supports environment-based configuration with sensible defaults:

### Server Configuration

```bash
export MCP_SERVER_NAME="My STDIO Server"
export MCP_SERVER_DESCRIPTION="Custom MCP server description"
export MCP_SERVER_INSTANCE_ID="custom-instance-123"
```

### Tool Configuration

```bash
# File Operations
export MCP_FILE_OPS_ENABLED=true
export MCP_FILE_OPS_MAX_SIZE=2097152  # 2MB
export MCP_FILE_OPS_BASE_DIR="/safe/directory"

# System Information
export MCP_SYSTEM_INFO_ENABLED=true
export MCP_SYSTEM_INFO_SENSITIVE=false

# Utilities
export MCP_UTILITIES_ENABLED=true
export MCP_UTILITIES_INSTANCE_ID="custom-util-id"
```

### Logging Configuration

```bash
export MCP_LOG_LEVEL=debug          # trace, debug, info, warn, error
export MCP_LOG_STRUCTURED=false     # true for JSON logs
```

### Security Configuration

```bash
export MCP_SECURITY_MAX_EXECUTION_TIME=30  # seconds
```

## Security Features

### File Operations Security

- **Path Traversal Protection**: Prevents `../` attacks
- **Extension Filtering**: Only allows safe file types
- **Size Limits**: Prevents large file operations
- **Base Directory Sandboxing**: Optional directory restriction

### System Information Privacy

- **Environment Filtering**: Only exposes safe environment variables
- **Sensitive Data Protection**: Optional sensitive information exclusion
- **Process Isolation**: Only current process information

### Execution Security

- **Timeout Protection**: Prevents long-running operations
- **Resource Monitoring**: Optional resource usage tracking
- **Error Isolation**: Safe error handling and reporting

## MCP Protocol Compliance

This server implements MCP protocol version `2024-11-05` with:

- **Initialize/Capabilities**: Full server capability negotiation
- **Tools**: Complete tool listing and execution
- **Error Handling**: Proper JSON-RPC error responses
- **Logging**: Protocol-compliant logging support

### Supported Methods

| Method | Description | Implementation |
|--------|-------------|----------------|
| `initialize` | Server initialization | ✅ Complete |
| `tools/list` | List available tools | ✅ Complete |
| `tools/call` | Execute tools | ✅ Complete |
| `ping` | Health check | ✅ Complete |

### Server Capabilities

```json
{
  "tools": {"listChanged": false},
  "logging": {},
  "prompts": {"listChanged": false}, 
  "resources": {"subscribe": false, "listChanged": false}
}
```

## Integration Examples

### With Python MCP Client

```python
import subprocess
import json

# Start server process
server = subprocess.Popen(
    ["cargo", "run", "--bin", "stdio-server"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

# Send initialize request
init_request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {}
}

server.stdin.write(json.dumps(init_request) + "\n")
server.stdin.flush()

# Read response
response = server.stdout.readline()
print(json.loads(response))
```

### With Shell Scripts

```bash
#!/bin/bash

SERVER_CMD="cargo run --bin stdio-server"

# Function to send MCP request
send_request() {
    echo "$1" | $SERVER_CMD | head -1
}

# Initialize server
init_response=$(send_request '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}')
echo "Initialize: $init_response"

# List tools
tools_response=$(send_request '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}')
echo "Tools: $tools_response"
```

## Testing

Comprehensive Python test suite is available in the `tests/` directory:

```bash
# Install test dependencies
pip install -r tests/requirements.txt

# Run all tests
python -m pytest tests/ -v

# Run specific test categories
python -m pytest tests/test_integration.py -v  # Integration tests
python -m pytest tests/test_tools.py -v       # Tool-specific tests
python -m pytest tests/test_negative.py -v    # Error scenarios
```

## Error Handling

The server provides comprehensive error handling:

### Common Errors

| Error Code | Description | Example |
|------------|-------------|---------|
| `-32700` | Parse error | Invalid JSON |
| `-32600` | Invalid request | Missing required fields |
| `-32601` | Method not found | Unknown method |
| `-32602` | Invalid params | Wrong parameter types |
| `-32603` | Internal error | Server-side errors |

### Tool-Specific Errors

- **File Operations**: Permission denied, file not found, size exceeded
- **System Info**: Access denied, resource unavailable
- **Utilities**: Invalid format, parameter validation

## Troubleshooting

### Common Issues

1. **Server Not Responding**
   ```bash
   # Check if binary builds
   cargo check --bin stdio-server
   
   # Test with verbose logging
   MCP_LOG_LEVEL=debug cargo run --bin stdio-server
   ```

2. **Permission Errors**
   ```bash
   # Check file permissions
   ls -la /path/to/files
   
   # Use base directory for sandboxing
   export MCP_FILE_OPS_BASE_DIR="/safe/directory"
   ```

3. **Tool Execution Timeouts**
   ```bash
   # Increase timeout
   export MCP_SECURITY_MAX_EXECUTION_TIME=60
   ```

### Debug Logging

Enable detailed logging for troubleshooting:

```bash
export MCP_LOG_LEVEL=trace
export MCP_LOG_STRUCTURED=true
cargo run --bin stdio-server 2> debug.log
```

## Development

### Adding New Tools

1. Create new tool provider in `src/tools/`
2. Implement `ToolProvider` trait
3. Add to `StandardToolSet` in `src/tools/mod.rs`
4. Update documentation and tests

### Configuration Extensions

1. Add new fields to `ServerConfig` in `src/config.rs`
2. Update environment variable parsing
3. Add validation logic
4. Update documentation

## Performance

### Benchmarks

- **Tool Execution**: < 10ms for simple tools
- **File Operations**: Limited by I/O and size constraints
- **Memory Usage**: < 10MB baseline, scales with file operations
- **Startup Time**: < 100ms

### Optimization Tips

- Use appropriate file size limits
- Enable resource monitoring for production
- Configure timeout values based on use case
- Use structured logging for performance analysis

## License

This example is part of the AIRS MCP library and is licensed under MIT OR Apache-2.0.