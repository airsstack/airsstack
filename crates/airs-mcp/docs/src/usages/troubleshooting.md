# Troubleshooting Guide

Comprehensive troubleshooting guide for AIRS MCP integration, common issues, debugging techniques, and resolution strategies.

## Quick Diagnosis

### Is Your Server Working?

```bash
# 1. Basic server test
timeout 3 /path/to/your/server < /dev/null

# 2. Test with MCP Inspector (browser-based testing)
cd your-server-directory
./scripts/test_inspector.sh  # If using example server

# 3. Manual JSON-RPC test
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | your-server
```

### Is Claude Desktop Integration Working?

```bash
# 1. Check configuration file
python3 -m json.tool ~/.config/Claude/claude_desktop_config.json

# 2. Monitor integration status
./scripts/debug_integration.sh  # If using example server

# 3. Check for MCP icon in Claude Desktop chat interface
```

## Common Issues and Solutions

### 1. Claude Desktop Integration Problems

#### Issue: MCP Icon Not Visible in Claude Desktop

**Symptoms:**
- No MCP plug icon in chat interface
- Tools not accessible through Claude Desktop
- Integration appears to fail silently

**Diagnosis:**
```bash
# Check Claude Desktop configuration
ls -la ~/.config/Claude/claude_desktop_config.json

# Validate JSON syntax
python3 -m json.tool ~/.config/Claude/claude_desktop_config.json

# Check server binary exists and is executable
ls -la /path/to/your/server
```

**Solutions:**

1. **Restart Claude Desktop Completely:**
   ```bash
   # macOS
   killall Claude
   open /Applications/Claude.app
   
   # Wait 10 seconds for full startup
   sleep 10
   ```

2. **Verify Configuration File Location:**
   ```json
   // Correct file: ~/.config/Claude/claude_desktop_config.json
   {
     "mcpServers": {
       "your-server": {
         "command": "/absolute/path/to/your/server"
       }
     }
   }
   ```

3. **Use Absolute Paths:**
   ```json
   // ❌ Wrong - relative path
   "command": "./my-server"
   
   // ✅ Correct - absolute path
   "command": "/home/user/projects/my-server"
   ```

4. **Check File Permissions:**
   ```bash
   chmod +x /path/to/your/server
   ```

#### Issue: Tools Showing But Not Working

**Symptoms:**
- MCP icon appears in Claude Desktop
- Tools are listed but execution fails
- Error messages in Claude Desktop

**Diagnosis:**
```rust
// Add comprehensive error handling to your tools
use airs_mcp::integration::mcp::error::McpError;

async fn handle_tool_call(
    &self,
    name: &str,
    arguments: serde_json::Value,
) -> Result<ToolResult, McpError> {
    match name {
        "my_tool" => {
            // Validate arguments
            let args = arguments.as_object()
                .ok_or_else(|| McpError::invalid_params("Arguments must be an object"))?;
            
            // Add detailed error context
            let result = self.execute_tool(args)
                .await
                .map_err(|e| McpError::tool_execution_failed("my_tool", &e.to_string()))?;
            
            Ok(ToolResult::success(vec![Content::text(result)]))
        }
        _ => Err(McpError::tool_not_found(name))
    }
}
```

**Solutions:**

1. **Enhance Error Reporting:**
   ```rust
   // Provide detailed error messages
   ToolResult {
       content: vec![Content::text(format!(
           "Tool execution failed: {}. Please check parameters: {:?}",
           error_msg, arguments
       ))],
       is_error: Some(true),
       meta: Some(json!({
           "error_type": "validation_error",
           "timestamp": chrono::Utc::now().to_rfc3339()
       })),
   }
   ```

2. **Validate Tool Arguments:**
   ```rust
   // Strong argument validation
   #[derive(Deserialize)]
   struct ToolArgs {
       required_param: String,
       optional_param: Option<i32>,
   }
   
   let args: ToolArgs = serde_json::from_value(arguments)
       .map_err(|e| McpError::invalid_params(&format!("Invalid arguments: {}", e)))?;
   ```

### 2. Server Startup and Connection Issues

#### Issue: Server Not Starting

**Symptoms:**
- Server process exits immediately
- Connection refused errors
- Timeout errors during initialization

**Diagnosis:**
```bash
# Check server logs
tail -f /tmp/your-server/server.log

# Test server in isolation
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | your-server

# Check for stderr output (violates STDIO transport)
your-server 2>&1 | grep -v "^{" || echo "No stderr output (good)"
```

**Solutions:**

1. **Eliminate stderr Output:**
   ```rust
   // ❌ Wrong - stderr output violates STDIO transport
   eprintln!("Debug info: {}", message);
   
   // ✅ Correct - use file logging for STDIO transport
   use log::info;
   info!("Debug info: {}", message);  // Goes to file via env_logger
   ```

2. **Implement Proper Logging:**
   ```rust
   use env_logger::Env;
   
   fn main() {
       // Initialize file-based logging for STDIO transport
       env_logger::Builder::from_env(Env::default().default_filter_or("info"))
           .target(env_logger::Target::Stdout)  // Or use file target
           .init();
       
       // Your server code
   }
   ```

3. **Add Initialization Timeout:**
   ```rust
   use tokio::time::{timeout, Duration};
   
   async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
       let server = timeout(
           Duration::from_secs(30),
           McpServerBuilder::new().build()
       ).await??;
       
       server.run().await
   }
   ```

#### Issue: Transport Errors

**Symptoms:**
- Connection drops unexpectedly
- Message parsing failures
- Buffer overflow errors

**Diagnosis:**
```rust
// Add transport-level debugging
use airs_mcp::transport::error::TransportError;

match error {
    TransportError::Closed => {
        log::warn!("Transport connection closed unexpectedly");
    }
    TransportError::BufferFull => {
        log::error!("Buffer overflow - consider increasing buffer size");
    }
    TransportError::IoError(io_err) => {
        log::error!("IO error: {} (kind: {:?})", io_err, io_err.kind());
    }
    TransportError::FormatError { message, .. } => {
        log::error!("Message format error: {}", message);
    }
}
```

**Solutions:**

1. **Increase Buffer Sizes:**
   ```rust
   use airs_mcp::transport::stdio::StdioTransport;
   
   let transport = StdioTransport::builder()
       .read_buffer_size(16384)   // Increase from default 8192
       .write_buffer_size(16384)
       .build().await?;
   ```

2. **Implement Connection Recovery:**
   ```rust
   async fn run_with_recovery() -> Result<(), Box<dyn std::error::Error>> {
       loop {
           match run_server().await {
               Ok(_) => break,
               Err(e) if is_recoverable(&e) => {
                   log::warn!("Recoverable error, retrying: {}", e);
                   tokio::time::sleep(Duration::from_secs(1)).await;
                   continue;
               }
               Err(e) => return Err(e),
           }
       }
       Ok(())
   }
   ```

### 3. Protocol and Schema Issues

#### Issue: Schema Validation Errors

**Symptoms:**
- "Method not found" errors for valid methods
- Content type mismatches
- Invalid JSON-RPC responses

**Diagnosis:**
```rust
// Validate protocol compliance
use airs_mcp::shared::protocol::ProtocolVersion;

let protocol = ProtocolVersion::new("2024-11-05")?;
// Ensure you're using the correct MCP schema version
```

**Solutions:**

1. **Use Correct Content Types:**
   ```rust
   // ✅ Correct - include required URI fields for resources
   Content::Text {
       text: content,
       uri: Some(Uri::new("file:///path/to/file")?),
       mime_type: Some(MimeType::new("text/plain")?),
   }
   
   // ❌ Wrong - missing URI for resource responses
   Content::Text { text: content }
   ```

2. **Validate Method Implementations:**
   ```rust
   // Ensure all required MCP methods are implemented
   async fn handle_request(&self, method: &str, params: Option<Value>) -> Result<Value, McpError> {
       match method {
           "initialize" => self.handle_initialize(params).await,
           "tools/list" => self.handle_tools_list(params).await,
           "tools/call" => self.handle_tools_call(params).await,
           "resources/list" => self.handle_resources_list(params).await,
           "resources/read" => self.handle_resources_read(params).await,
           _ => Err(McpError::method_not_found(method))
       }
   }
   ```

#### Issue: JSON-RPC Compliance Problems

**Solutions:**

1. **Ensure JSON-RPC 2.0 Compliance:**
   ```rust
   // All responses must include jsonrpc: "2.0"
   let response = JsonRpcResponse::success(result, request_id);
   assert!(response.to_json()?.contains(r#""jsonrpc":"2.0""#));
   ```

2. **Handle All Error Cases:**
   ```rust
   use airs_mcp::base::jsonrpc::ErrorCode;
   
   let error_response = match validation_error {
       ValidationError::InvalidParams => JsonRpcResponse::error(
           ErrorCode::InvalidParams,
           "Invalid parameters provided",
           Some(json!({"details": error_details})),
           request_id
       ),
       ValidationError::MethodNotFound => JsonRpcResponse::error(
           ErrorCode::MethodNotFound,
           &format!("Method '{}' not found", method),
           None,
           request_id
       ),
       _ => JsonRpcResponse::error(
           ErrorCode::InternalError,
           "Internal server error",
           None,
           request_id
       ),
   };
   ```

### 4. Performance and Resource Issues

#### Issue: High Memory Usage

**Symptoms:**
- Server consumes excessive memory
- Out of memory errors
- Performance degradation over time

**Diagnosis:**
```rust
// Monitor memory usage
use std::alloc::{GlobalAlloc, Layout, System};

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            log::debug!("Allocated {} bytes", layout.size());
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        log::debug!("Deallocated {} bytes", layout.size());
        System.dealloc(ptr, layout)
    }
}
```

**Solutions:**

1. **Use Buffer Pooling:**
   ```rust
   use airs_mcp::transport::buffer_pool::BufferManager;
   
   let buffer_manager = BufferManager::builder()
       .pool_size(100)
       .buffer_size(8192)
       .max_pools(10)
       .build();
   
   // Reuse buffers instead of allocating new ones
   let buffer = buffer_manager.acquire_read_buffer().await?;
   // ... use buffer ...
   // Buffer automatically returned to pool when dropped
   ```

2. **Implement Connection Limits:**
   ```rust
   use tokio::sync::Semaphore;
   
   struct Server {
       connection_semaphore: Semaphore,
   }
   
   impl Server {
       fn new(max_connections: usize) -> Self {
           Self {
               connection_semaphore: Semaphore::new(max_connections),
           }
       }
       
       async fn handle_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
           let _permit = self.connection_semaphore.acquire().await?;
           // Handle connection (permit released on drop)
           Ok(())
       }
   }
   ```

#### Issue: Slow Response Times

**Solutions:**

1. **Optimize JSON Processing:**
   ```rust
   // Use streaming for large responses
   use airs_mcp::transport::streaming::StreamingJsonParser;
   
   let parser = StreamingJsonParser::new(8192);
   
   // Process JSON in chunks instead of loading entire response
   while let Some(chunk) = reader.read_chunk().await? {
       if let Some(message) = parser.parse_chunk(&chunk)? {
           handle_message(message).await?;
       }
   }
   ```

2. **Implement Request Batching:**
   ```rust
   // Batch multiple requests for efficiency
   let batched_requests = vec![
       JsonRpcRequest::new("method1", params1, RequestId::new_number(1)),
       JsonRpcRequest::new("method2", params2, RequestId::new_number(2)),
   ];
   
   let results = client.call_batch(batched_requests).await?;
   ```

## Debugging Tools and Scripts

### Server Debugging Scripts

```bash
#!/bin/bash
# debug_server.sh - Comprehensive server debugging

# 1. Test server startup
echo "Testing server startup..."
timeout 5 /path/to/server < /dev/null
echo "Exit code: $?"

# 2. Test JSON-RPC compliance
echo "Testing JSON-RPC..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | /path/to/server

# 3. Check for stderr violations
echo "Checking for stderr output..."
OUTPUT=$(echo '{"jsonrpc":"2.0","method":"ping","id":1}' | /path/to/server 2>&1)
STDERR=$(echo "$OUTPUT" | grep -v '^{')
if [ -n "$STDERR" ]; then
    echo "❌ STDERR output detected (violates STDIO transport):"
    echo "$STDERR"
else
    echo "✅ No STDERR output (STDIO compliant)"
fi
```

### Integration Testing

```bash
#!/bin/bash
# integration_test.sh - End-to-end integration testing

# Function to test MCP capabilities
test_capabilities() {
    echo "Testing MCP capabilities..."
    
    # Initialize
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}' | /path/to/server
    
    # List tools
    echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | /path/to/server
    
    # List resources
    echo '{"jsonrpc":"2.0","method":"resources/list","params":{},"id":3}' | /path/to/server
    
    # List prompts
    echo '{"jsonrpc":"2.0","method":"prompts/list","params":{},"id":4}' | /path/to/server
}

test_capabilities
```

### Monitoring and Logging

```rust
// Real-time debugging with structured logging
use tracing::{info, warn, error, debug, span, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// Initialize comprehensive logging
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let file_appender = tracing_appender::rolling::daily("/tmp/mcp-server", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking))
        .with(EnvFilter::from_default_env())
        .init();
    
    Ok(())
}

// Instrument critical functions
#[tracing::instrument(level = "debug")]
async fn handle_request(request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
    let span = span!(Level::INFO, "handle_request", method = %request.method);
    let _enter = span.enter();
    
    info!("Processing request: {}", request.method);
    
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(request.params).await,
        "tools/call" => {
            debug!("Tool call with params: {:?}", request.params);
            handle_tool_call(request.params).await
        }
        _ => {
            warn!("Unknown method: {}", request.method);
            Err(McpError::method_not_found(&request.method))
        }
    };
    
    match &result {
        Ok(_) => info!("Request completed successfully"),
        Err(e) => error!("Request failed: {}", e),
    }
    
    result
}
```

## Environment-Specific Issues

### macOS

```bash
# Check app permissions
ls -la@ /Applications/Claude.app
xattr -l /Applications/Claude.app

# macOS-specific configuration path
CONFIG_PATH="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

# Check for file system permissions
sudo fs_usage | grep Claude
```

### Linux

```bash
# Check systemd logs if running as service
journalctl -u claude-desktop --follow

# Check AppImage permissions (if using AppImage)
chmod +x Claude.AppImage

# Linux configuration path
CONFIG_PATH="$HOME/.config/Claude/claude_desktop_config.json"
```

### Windows

```powershell
# Windows configuration path
$CONFIG_PATH = "$env:APPDATA\Claude\claude_desktop_config.json"

# Check Windows Defender exclusions
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath

# Test with Windows-specific paths
Test-Path "C:\path\to\server.exe"
```

## Getting Help

### Diagnostic Information to Gather

When seeking help, include:

1. **Environment Information:**
   ```bash
   echo "OS: $(uname -a)"
   echo "Claude Desktop Version: [check in app]"
   echo "Server Version: [your version]"
   echo "Rust Version: $(rustc --version)"
   ```

2. **Configuration:**
   ```bash
   # Sanitized configuration (remove sensitive paths)
   python3 -m json.tool ~/.config/Claude/claude_desktop_config.json
   ```

3. **Logs:**
   ```bash
   # Last 50 lines of server logs
   tail -50 /tmp/your-server/server.log
   
   # Claude Desktop logs (macOS)
   tail -50 ~/Library/Logs/Claude/mcp*.log
   ```

4. **Error Reproduction:**
   ```bash
   # Minimal test case that reproduces the issue
   echo '{"jsonrpc":"2.0","method":"failing_method","params":{},"id":1}' | your-server
   ```

### Resources

- **GitHub Issues**: Report bugs with diagnostic information
- **Documentation**: Check updated examples for working patterns
- **MCP Inspector**: Use browser-based testing for protocol validation
- **Community Forums**: Share troubleshooting experiences

Remember: Most integration issues stem from configuration problems, STDIO transport violations, or schema compliance issues. Start with the basic diagnostics and work systematically through the solution steps.
