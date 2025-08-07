# Claude Desktop Integration Knowledge

**updated:** 2025-08-07T23:45:00Z  
**source:** Official MCP Documentation Analysis  
**status:** comprehensive_troubleshooting_guide_compiled

## Critical Integration Requirements

### 1. Configuration File Location - CRITICAL
**Correct Path:** `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Common Error:** Using `config.json` instead of `claude_desktop_config.json`  
**Impact:** Claude Desktop will not detect MCP servers if wrong filename is used

### 2. Configuration Structure - REQUIRED
```json
{
  "mcpServers": {
    "server-name": {
      "command": "/absolute/path/to/executable",
      "args": ["arg1", "arg2"],
      "env": {
        "OPTIONAL_ENV_VAR": "value"
      }
    }
  }
}
```

**Critical Requirements:**
- Use **absolute paths** for the `command` field
- Server name must be unique within `mcpServers`
- Environment variables inherit only subset: `USER`, `HOME`, `PATH`

### 3. STDIO Transport Logging Restrictions - CRITICAL
**Forbidden for STDIO servers:**
- Writing to stdout (standard output)
- Writing to stderr (standard error)  
- Any console output: `print()`, `console.log()`, `fmt.Println()`

**Required Approach:**
- File-based logging only
- Use `/tmp/`, `/var/log/`, or user directory for log files
- Never use stderr fallback for STDIO transport

**Current Issue in Our Implementation:**
```rust
// PROBLEMATIC - potential stderr output in fallback
tracing_subscriber::registry()
    .with(EnvFilter::new("off"))
    .init();
```

## Debugging Workflow - Official Best Practices

### Phase 1: MCP Inspector Testing (ALWAYS FIRST)
```bash
# Install and test server in isolation
npx @modelcontextprotocol/inspector /path/to/server args...

# For our server specifically:
npx @modelcontextprotocol/inspector /Users/hiraq/Projects/rstlix0x0/airs/crates/airs-mcp/examples/simple-mcp-server/target/aarch64-apple-darwin/release/simple-mcp-server
```

**Benefits:**
- Tests server without Claude Desktop complexity
- Validates JSON-RPC message format
- Confirms resource/tool/prompt availability
- Identifies protocol compliance issues

### Phase 2: Claude Desktop Configuration
**After MCP Inspector success:**
1. Create/update `claude_desktop_config.json`
2. Use exact path from Inspector testing
3. Restart Claude Desktop completely
4. Check for MCP indicator icon

### Phase 3: Claude Desktop Debugging
**Log Locations:**
```bash
# Real-time log monitoring
tail -n 20 -F ~/Library/Logs/Claude/mcp*.log

# Connection and server events
~/Library/Logs/Claude/mcp.log
```

**Chrome DevTools Access:**
```bash
# Enable developer tools
echo '{"allowDevTools": true}' > ~/Library/Application\ Support/Claude/developer_settings.json

# Access in Claude Desktop: Command-Option-Shift-i
```

**DevTools Investigation:**
- Console panel: Client-side errors
- Network panel: Message payloads and timing
- Two windows: Main content + App title bar

## Common Integration Failures

### 1. Server Not Detected
**Symptoms:** No MCP icon in Claude Desktop  
**Causes:**
- Wrong config file name/path
- Invalid JSON syntax in config
- Server executable not found
- Server crashes on startup

**Debug Steps:**
1. Verify config file exists and has correct name
2. Test JSON syntax: `cat config.json | jq '.'`
3. Test executable manually: `./server < /dev/null`
4. Check Claude Desktop logs for startup errors

### 2. Server Connection Failures
**Symptoms:** MCP icon present but tools not available  
**Causes:**
- Server initialization timeout
- JSON-RPC protocol violations
- Environment variable issues
- Permission problems

**Debug Steps:**
1. Test with MCP Inspector first
2. Check server logs for initialization errors
3. Verify environment variables in config
4. Test server process permissions

### 3. Tool Execution Failures
**Symptoms:** Tools listed but calls fail silently  
**Causes:**
- Input validation errors
- Server-side exceptions
- Response format issues
- Timeout problems

**Debug Steps:**
1. Test specific tools in MCP Inspector
2. Check server logs during tool execution
3. Verify input schema compliance
4. Monitor response structure

## Working Directory and Environment Issues

### Working Directory Behavior
- **Claude Desktop launch:** Working directory may be `/` (root)
- **Command line testing:** Working directory is current shell location
- **Solution:** Always use absolute paths in configuration and server code

### Environment Variable Management
**Default inherited variables:**
- `USER`, `HOME`, `PATH`

**Custom environment variables:**
```json
{
  "mcpServers": {
    "myserver": {
      "command": "/path/to/server",
      "env": {
        "CUSTOM_API_KEY": "value",
        "LOG_LEVEL": "debug"
      }
    }
  }
}
```

## Visual Integration Indicators

### Success Indicators
1. **MCP Icon:** Plug-like icon in conversation input area bottom-right
2. **Tools Panel:** Click icon shows available tools list
3. **Server Status:** Green/connected indicators in UI

### Failure Indicators
1. **No MCP Icon:** Configuration or server startup failure
2. **Empty Tools List:** Server running but no capabilities detected
3. **Error Messages:** Client-side connection or protocol errors

## Performance and Monitoring

### Message Exchange Monitoring
- **Log JSON-RPC messages** (to files only)
- **Track message sizes** and response times
- **Monitor connection state** changes
- **Record error patterns** and recovery

### Resource Usage Tracking
- **Memory consumption** during operation
- **File descriptor usage** for STDIO transport
- **CPU usage** during intensive operations
- **Network activity** for HTTP transport (if applicable)

## Security Considerations

### Sensitive Data Handling
- **Sanitize logs:** Remove credentials, personal information
- **Access control:** Verify file permissions for log files
- **Environment variables:** Protect API keys and secrets

### Permission Verification
- **File access:** Check server can read/write required files
- **Network access:** Verify HTTP transport permissions (if used)
- **Process isolation:** Ensure server runs with minimal privileges

## Integration Testing Strategy

### 1. Unit Testing Phase
- Test individual components with MCP Inspector
- Validate JSON-RPC message format
- Verify resource/tool/prompt schemas

### 2. Integration Testing Phase
- Test full Claude Desktop integration
- Monitor real-world usage patterns
- Verify error handling and recovery

### 3. Performance Testing Phase
- Load testing with multiple simultaneous requests
- Memory usage under sustained operation
- Response time optimization

## Next Steps for Our Implementation

### Immediate Fixes Required
1. **Fix logging strategy:** Eliminate all stderr output potential
2. **Create proper config script:** Use correct `claude_desktop_config.json` path
3. **Add MCP Inspector testing:** Before attempting Claude Desktop integration
4. **Enhance error handling:** Add connection state logging to files only

### Recommended Script Structure
1. **`build.sh`:** Build optimized release binary
2. **`test_inspector.sh`:** Test with MCP Inspector first
3. **`configure_claude.sh`:** Set up Claude Desktop configuration
4. **`debug_integration.sh`:** Monitor logs and debug issues

### Development Workflow
1. Make server changes
2. Build and test with MCP Inspector
3. Update Claude Desktop config if needed
4. Restart Claude Desktop
5. Test integration and monitor logs
6. Iterate based on findings

This knowledge base provides comprehensive guidance for successful Claude Desktop integration based on official MCP documentation and best practices.
