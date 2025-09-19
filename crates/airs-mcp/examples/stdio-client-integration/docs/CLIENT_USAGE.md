# STDIO MCP Client Usage Guide

## Overview

The STDIO MCP Client is a demonstration implementation showing how to use the AIRS MCP TransportClient architecture with STDIO transport. It provides a complete example of building MCP clients using the new transport abstraction layer.

## Quick Start

### Build and Run

```bash
# Build the client and mock server
cargo build --bin stdio-mcp-client --bin stdio-mock-server

# Run with mock server (easiest for testing)
USE_MOCK=1 cargo run --bin stdio-mcp-client

# Run with custom server command
MCP_SERVER_COMMAND="your-mcp-server" cargo run --bin stdio-mcp-client
```

### Environment Variables

- `USE_MOCK=1`: Use the built-in mock server for testing
- `MCP_SERVER_COMMAND`: Command to start the MCP server (default: "stdio-mock-server")
- `MCP_SERVER_ARGS`: Space-separated arguments for the server command
- `MCP_REQUEST_TIMEOUT`: Request timeout in seconds (default: 30)
- `RUST_LOG`: Logging level (debug, info, warn, error)

## Architecture Overview

The STDIO MCP Client demonstrates the layered architecture of the AIRS MCP system:

```
┌─────────────────────────────┐
│     Demo Application        │  <- main.rs
├─────────────────────────────┤
│     MCP Client Wrapper      │  <- client.rs (StdioMcpClient)
├─────────────────────────────┤
│     AIRS MCP Client         │  <- McpClient (high-level API)
├─────────────────────────────┤
│   STDIO Transport Client    │  <- StdioTransportClient
├─────────────────────────────┤
│     Transport Layer         │  <- TransportClient trait
└─────────────────────────────┘
```

## Core Components

### 1. Configuration (config.rs)

Manages client configuration settings:

```rust
pub struct ClientConfig {
    pub server_command: String,       // Command to start MCP server
    pub server_args: Vec<String>,     // Arguments for server command
    pub request_timeout: Duration,    // Request timeout
    pub connection_timeout: Duration, // Connection timeout
}
```

**Usage:**
```rust
let config = ClientConfig::from_env()?;
let client = StdioMcpClient::new(&config).await?;
```

### 2. Client Wrapper (client.rs)

High-level wrapper around the MCP client:

```rust
pub struct StdioMcpClient {
    client: McpClient<StdioTransportClient>,
}

impl StdioMcpClient {
    pub async fn new(config: &ClientConfig) -> Result<Self>;
    pub async fn initialize(&mut self) -> Result<()>;
    pub async fn list_tools(&mut self) -> Result<Vec<String>>;
    pub async fn call_tool(&mut self, name: &str, args: Option<Value>) -> Result<String>;
    pub async fn run_demo(&mut self) -> Result<()>;
}
```

### 3. Transport Client

Uses the AIRS MCP StdioTransportClient:

```rust
// Build transport client
let transport = StdioTransportClientBuilder::new()
    .command(&config.server_command)
    .timeout(config.request_timeout)
    .arg("--verbose")
    .build()
    .await?;

// Create MCP client
let client = McpClientBuilder::new().build(transport);
```

## Configuration Methods

### 1. Environment Variables (Recommended)

```bash
export USE_MOCK=1
export MCP_REQUEST_TIMEOUT=30
export RUST_LOG=info
cargo run --bin stdio-mcp-client
```

### 2. Command Line Arguments

```bash
cargo run --bin stdio-mcp-client -- --help
```

### 3. Configuration File

Create a `client_config.json` file:

```json
{
  "server_command": "your-mcp-server",
  "server_args": ["--config", "server.json"],
  "request_timeout_secs": 30,
  "connection_timeout_secs": 10
}
```

## Usage Examples

### Basic Demo Run

```bash
# Use built-in mock server
USE_MOCK=1 cargo run --bin stdio-mcp-client
```

**Expected Output:**
```
=== STDIO MCP Client Demo ===

1. Initializing MCP session...
✓ Initialization successful

2. Listing available tools...
✓ Tools listed successfully
   - echo
   - health_check
   - get_timestamp

3. Testing echo tool...
✓ Echo tool called successfully
   Response: Tool 'echo' result: ...

4. Testing health check...
✓ Health check successful
   Status: Tool 'health_check' result: ...

5. Getting current timestamp...
✓ Timestamp retrieved successfully
   Timestamp: Tool 'get_timestamp' result: ...

=== Demo Complete ===
```

### Custom Server Integration

```bash
# Run with a real MCP server
MCP_SERVER_COMMAND="/path/to/your/mcp-server" \
MCP_SERVER_ARGS="--config server.json --verbose" \
MCP_REQUEST_TIMEOUT=60 \
cargo run --bin stdio-mcp-client
```

### Debug Mode

```bash
# Enable detailed logging
RUST_LOG=debug USE_MOCK=1 cargo run --bin stdio-mcp-client
```

## Integration Patterns

### 1. Using as Library

```rust
use stdio_client_integration::{ClientConfig, StdioMcpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ClientConfig::from_env()?;
    let mut client = StdioMcpClient::new(&config).await?;
    
    // Initialize session
    client.initialize().await?;
    
    // List available tools
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);
    
    // Call a tool
    let result = client.call_tool(
        "echo", 
        Some(serde_json::json!({"message": "Hello!"}))
    ).await?;
    println!("Tool result: {}", result);
    
    Ok(())
}
```

### 2. Custom Transport Builder

```rust
use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
use airs_mcp::integration::McpClientBuilder;

// Build custom transport
let transport = StdioTransportClientBuilder::new()
    .command("my-custom-server")
    .arg("--mode=strict")
    .arg("--timeout=60")
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// Create MCP client
let client = McpClientBuilder::new()
    .build(transport);
```

### 3. Error Handling

```rust
match client.initialize().await {
    Ok(()) => println!("✓ Initialization successful"),
    Err(e) => {
        eprintln!("✗ Initialization failed: {}", e);
        
        // Handle specific error types
        if e.to_string().contains("timeout") {
            eprintln!("  Try increasing MCP_REQUEST_TIMEOUT");
        } else if e.to_string().contains("not found") {
            eprintln!("  Check MCP_SERVER_COMMAND path");
        }
        
        return Err(e);
    }
}
```

## Testing and Validation

### Running Tests

```bash
# Install Python dependencies
cd tests/
pip install -r requirements.txt

# Run all tests
python -m pytest

# Run specific test files
python test_client_integration.py
python test_transport.py
python test_error_scenarios.py
```

### Manual Testing Steps

1. **Build verification:**
   ```bash
   cargo check
   cargo clippy --all-targets --all-features
   cargo test
   ```

2. **Mock server test:**
   ```bash
   USE_MOCK=1 cargo run --bin stdio-mcp-client
   ```

3. **Timeout behavior:**
   ```bash
   MCP_REQUEST_TIMEOUT=1 USE_MOCK=1 cargo run --bin stdio-mcp-client
   ```

4. **Error handling:**
   ```bash
   MCP_SERVER_COMMAND="/nonexistent" cargo run --bin stdio-mcp-client
   ```

## Performance Considerations

### Timeout Configuration

- **Connection timeout**: Time to establish server connection (default: 10s)
- **Request timeout**: Time to wait for individual responses (default: 30s)
- **Overall timeout**: Total time for demo sequence (60s recommended)

```bash
# For slow servers
MCP_REQUEST_TIMEOUT=120 cargo run --bin stdio-mcp-client

# For fast local testing
MCP_REQUEST_TIMEOUT=5 USE_MOCK=1 cargo run --bin stdio-mcp-client
```

### Resource Usage

- **Memory**: ~5-10MB for basic operation
- **CPU**: Minimal during idle, brief spikes during JSON processing
- **File Descriptors**: 2 per server connection (stdin/stdout)

### Optimization Tips

1. **Reuse clients**: Create one client instance for multiple operations
2. **Batch operations**: Group related tool calls when possible
3. **Appropriate timeouts**: Set realistic timeouts based on server performance
4. **Logging level**: Use `info` or `warn` in production, `debug` for troubleshooting

## Troubleshooting

### Common Issues

**"Failed to create client"**
- Check `MCP_SERVER_COMMAND` is valid executable
- Verify server command exists in PATH
- Ensure sufficient permissions

**"Initialization timed out"**
- Increase `MCP_REQUEST_TIMEOUT`
- Check server startup time
- Verify server implements `initialize` method

**"Tools listed successfully" but no tools shown**
- Server may not implement any tools
- Check server's `tools/list` response format
- Enable debug logging to see raw responses

**"Tool call failed"**
- Verify tool exists in server's tool list
- Check argument format matches tool schema
- Review error messages for parameter issues

### Debug Techniques

1. **Enable debug logging:**
   ```bash
   RUST_LOG=debug cargo run --bin stdio-mcp-client
   ```

2. **Test server independently:**
   ```bash
   echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":"test"}' | your-mcp-server
   ```

3. **Use mock server for comparison:**
   ```bash
   USE_MOCK=1 RUST_LOG=debug cargo run --bin stdio-mcp-client
   ```

4. **Check transport connectivity:**
   ```bash
   # Test if server starts
   timeout 5s your-mcp-server
   echo "Exit code: $?"
   ```

### Error Messages Reference

| Error Pattern | Likely Cause | Solution |
|---------------|--------------|----------|
| "No such file or directory" | Invalid server command | Check `MCP_SERVER_COMMAND` path |
| "Connection refused" | Server not responding | Verify server startup |
| "Timeout" | Server too slow | Increase `MCP_REQUEST_TIMEOUT` |
| "Parse error" | Invalid JSON | Check server JSON-RPC format |
| "Method not found" | Unsupported method | Verify server MCP compliance |

## Advanced Usage

### Custom Tool Implementation

To test with your own tools, modify the client wrapper:

```rust
impl StdioMcpClient {
    pub async fn call_custom_tool(
        &mut self,
        tool_name: &str,
        args: MyCustomArgs,
    ) -> Result<MyCustomResult, Box<dyn std::error::Error + Send + Sync>> {
        let json_args = serde_json::to_value(args)?;
        let result = self.client.call_tool(tool_name, Some(json_args)).await?;
        
        // Parse custom result format
        let custom_result: MyCustomResult = serde_json::from_value(result)?;
        Ok(custom_result)
    }
}
```

### Async Patterns

```rust
use tokio::time::{timeout, Duration};

// Concurrent tool calls
let echo_future = client.call_tool("echo", Some(json!({"msg": "1"})));
let health_future = client.call_tool("health_check", None);

let (echo_result, health_result) = tokio::try_join!(echo_future, health_future)?;

// Timeout handling
match timeout(Duration::from_secs(10), client.initialize()).await {
    Ok(Ok(())) => println!("Initialized successfully"),
    Ok(Err(e)) => println!("Initialization error: {}", e),
    Err(_) => println!("Initialization timed out"),
}
```

### Production Deployment

For production use:

1. **Configuration management**: Use structured config files
2. **Error recovery**: Implement retry logic with exponential backoff
3. **Monitoring**: Add metrics and health checks
4. **Security**: Validate server responses and sanitize inputs
5. **Resource limits**: Set appropriate timeouts and resource constraints

```rust
// Production-ready error handling
use tokio::time::{sleep, Duration};

async fn robust_tool_call(&mut self, tool: &str, args: Option<Value>) -> Result<String> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    while attempts < max_attempts {
        match self.call_tool(tool, args.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) if attempts + 1 < max_attempts => {
                warn!("Tool call failed, retrying: {}", e);
                attempts += 1;
                sleep(Duration::from_millis(1000 * attempts)).await;
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

## Standards Compliance

This client implementation follows workspace standards:

- **3-Layer Import Organization** (§2.1): Standard → Third-party → Internal
- **chrono DateTime<Utc> Standard** (§3.2): All timestamps use UTC
- **Module Architecture Patterns** (§4.3): Clean module separation
- **Zero Warning Policy**: Compiles with zero warnings
- **Dependency Management** (§5.1): AIRS foundation crates prioritized

## Next Steps

- Review the [Mock Server Documentation](MOCK_SERVER.md) for testing details
- Explore the test suite in `tests/` for comprehensive examples
- Check the source code for implementation patterns
- Adapt the client for your specific MCP server requirements