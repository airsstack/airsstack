# MCP Server Connection Troubleshooting Guide

**Created:** 2025-09-22  
**Updated:** 2025-09-22  
**Category:** Integration/Troubleshooting  
**Status:** Production-Ready Solutions

## Overview

This document captures critical troubleshooting knowledge for MCP server connection issues discovered during Claude Desktop and MCP Inspector integration testing. These solutions are essential for proper MCP protocol implementation.

## Key Issues Discovered & Solutions

### 1. 🔧 **Server Lifecycle Management**

**Problem**: Server was exiting immediately after sending initialize response instead of staying alive for subsequent requests.

**Root Cause**: Missing proper `wait_for_completion()` implementation in main server loop.

**Solution**:
```rust
// ✅ CORRECT - Wait for transport completion
if let Err(e) = transport.wait_for_completion().await {
    error!("❌ Transport error during operation: {}", e);
    process::exit(1);
}

// ❌ INCORRECT - Server exits immediately
// (no wait_for_completion call)
```

**Impact**: Critical for STDIO transport - server must block until stdin closes.

### 2. 🔧 **MCP Protocol Message Handling**

**Problem**: Server only handled `JsonRpcMessage::Request` but ignored `JsonRpcMessage::Notification`, causing "initialized" notification to be ignored.

**Root Cause**: Incomplete message type handling in MessageHandler implementation.

**Solution**:
```rust
// ✅ CORRECT - Handle all message types
match message {
    JsonRpcMessage::Request(request) => {
        // Process request and send response
    }
    JsonRpcMessage::Notification(notification) => {
        // Process notification (no response needed)
        self.process_mcp_notification(&notification).await;
    }
    JsonRpcMessage::Response(_) => {
        // STDIO servers typically don't receive responses
    }
}
```

**Critical**: The "initialized" notification completes the MCP handshake but requires no response.

### 3. 🔧 **Capability Schema Validation**

**Problem**: MCP Inspector validation errors for null capability fields:
```
Expected object, received null for "capabilities.prompts"
Expected object, received null for "capabilities.resources"
```

**Root Cause**: MCP clients expect capability fields to be objects (even empty) or completely omitted, not `null`.

**Solutions**:

**Option A - Provide Empty Capabilities**:
```rust
let capabilities = ServerCapabilities {
    prompts: Some(PromptCapabilities::default()), // Empty object
    resources: Some(ResourceCapabilities::default()), // Empty object
    tools: Some(ToolCapabilities { list_changed: Some(false) }),
    logging: None, // Omitted entirely
};
```

**Option B - Manual JSON Construction (Recommended)**:
```rust
// ✅ BEST - Only advertise what you implement
let capabilities_json = json!({
    "experimental": {},
    "tools": {
        "list_changed": false
    }
    // Omit fields for capabilities we don't provide
});
```

### 4. 🔧 **Capability-Method Consistency**

**Problem**: Server advertised logging capability but didn't implement `logging/setLevel` method, causing "Unknown method" errors.

**Root Cause**: Mismatch between advertised capabilities and implemented methods.

**Solution**: 
```rust
// ❌ PROBLEM - Advertising capability without implementation
capabilities: {
    "logging": {} // Advertised but no handler implemented
}

// ✅ SOLUTION - Only advertise implemented capabilities  
capabilities: {
    "tools": {"list_changed": false}
    // No logging field = no logging capability
}
```

**Rule**: Never advertise capabilities unless you implement ALL required methods for that capability.

### 5. 🔧 **MCP Inspector Configuration**

**Problem**: MCP Inspector spawn errors with command/args separation:
```
Error: spawn /path/to/server serve ENOENT
```

**Root Cause**: Command and arguments not properly separated in spawn configuration.

**Solution**:
- **Command**: `/path/to/airs-mcpserver-fs`
- **Args**: `serve`

**Not**: `/path/to/airs-mcpserver-fs serve` as single command path.

## Debugging Strategies

### Server-Side Debugging

1. **Check Server Logs**:
   ```bash
   tail -f ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log
   ```

2. **Manual Protocol Testing**:
   ```bash
   echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test"}}, "id": 1}' | ./target/debug/airs-mcpserver-fs serve
   ```

3. **Validate JSON Response**:
   - Check for proper capability objects (not null)
   - Verify protocol version matches
   - Ensure server info is present

### Client-Side Debugging

1. **Claude Desktop Logs**:
   - Check for connection timeout errors
   - Look for JSON parsing errors
   - Verify environment variables are set

2. **MCP Inspector**:
   - Use for interactive testing
   - Validate capability negotiation
   - Test individual tool calls

## Best Practices

### 1. Minimal Capability Advertising
```rust
// ✅ GOOD - Only advertise what you implement
{
    "experimental": {},
    "tools": {"list_changed": false}
}

// ❌ BAD - Advertising unimplemented capabilities  
{
    "logging": {}, 
    "prompts": {},
    "resources": {},
    "tools": {"list_changed": false}
}
```

### 2. Complete Message Handling
- Handle requests AND notifications
- Implement ALL methods for advertised capabilities
- Proper error responses for unknown methods

### 3. Server Lifecycle
- Always use `transport.wait_for_completion().await`
- Never exit immediately after startup
- Proper graceful shutdown handling

### 4. Protocol Compliance
- Use current protocol version: `ProtocolVersion::current()`
- Follow exact JSON-RPC 2.0 specification
- Handle "initialized" notification (no response needed)

## Testing Checklist

Before deploying MCP server:

- [ ] Server stays alive after initialize response
- [ ] Handles both requests and notifications
- [ ] Only advertises implemented capabilities
- [ ] All advertised capabilities have method implementations
- [ ] Manual protocol test succeeds
- [ ] MCP Inspector connection works
- [ ] Claude Desktop integration works
- [ ] Tool calls function correctly

## Integration Examples

### Working Claude Desktop Config
```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/path/to/airs-mcpserver-fs",
      "args": ["serve"],
      "env": {
        "AIRS_MCPSERVER_FS_LOG_DIR": "/path/to/logs"
      }
    }
  }
}
```

### Working MCP Inspector Config
- **Command**: `/path/to/airs-mcpserver-fs`
- **Args**: `serve`
- **Transport**: STDIO

## Common Error Patterns

| Error | Cause | Solution |
|-------|-------|----------|
| "Server transport closed unexpectedly" | Server exits after initialize | Add `wait_for_completion()` |
| "Expected object, received null" | Null capability fields | Use objects or omit fields |
| "Unknown method: logging/setLevel" | Capability without implementation | Remove capability or add method |
| "Unexpected end of JSON input" | Connection timing issues | Ensure server stays alive |
| "spawn ENOENT" | Wrong command format | Separate command and args |

---

**Status**: Production-tested solutions for MCP server integration
**Next**: Apply patterns to other MCP server implementations