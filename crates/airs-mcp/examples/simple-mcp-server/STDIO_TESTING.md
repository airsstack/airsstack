# Manual MCP Server Testing

## The Problem with STDIO Testing

MCP servers use STDIO (standard input/output) for communication, which makes testing a bit tricky because:

1. The server reads from STDIN and writes to STDOUT
2. It's designed for persistent connections, not one-off requests
3. The JSON-RPC protocol expects a session-based interaction

## Recommended Testing Methods

### Method 1: Interactive Manual Testing

**Terminal 1** - Start the server:
```bash
cd examples/simple-mcp-server
cargo run --bin simple-mcp-server
```

**Terminal 2** - Send requests using echo and pipes:
```bash
# Initialize (required first)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | nc localhost 8080

# For STDIO, you can use:
(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'; sleep 1; echo '{"jsonrpc":"2.0","id":2,"method":"resources/list"}'; sleep 10) | cargo run --bin simple-mcp-server
```

### Method 2: Using `socat` for Bidirectional Communication

If you have `socat` installed:
```bash
# Install socat first
brew install socat  # macOS
# or
sudo apt-get install socat  # Ubuntu

# Create a bidirectional connection
socat - EXEC:"cargo run --bin simple-mcp-server",pty,ctty
```

Then type JSON-RPC messages directly:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"resources/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/list"}
```

### Method 3: Creating a Test Client

The most reliable way is to create a proper MCP client. Let me show you a simple approach:

**Create a file called `test_client.py`:**
```python
import json
import subprocess
import sys
import threading
import time

def read_responses(proc):
    """Read responses from the server in a separate thread"""
    while True:
        try:
            line = proc.stdout.readline()
            if not line:
                break
            if line.strip():
                try:
                    response = json.loads(line.strip())
                    print(f"← {json.dumps(response, indent=2)}")
                except json.JSONDecodeError:
                    print(f"← Raw: {line.strip()}")
        except Exception as e:
            print(f"Error reading: {e}")
            break

def send_request(proc, request):
    """Send a request to the server"""
    json_str = json.dumps(request)
    print(f"→ {json_str}")
    proc.stdin.write(json_str + '\n')
    proc.stdin.flush()
    time.sleep(0.5)  # Give server time to respond

# Start the server
proc = subprocess.Popen(
    ['cargo', 'run', '--bin', 'simple-mcp-server'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=0
)

# Start response reader thread
reader_thread = threading.Thread(target=read_responses, args=(proc,))
reader_thread.daemon = True
reader_thread.start()

time.sleep(2)  # Let server start

try:
    # Initialize
    send_request(proc, {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    })
    
    # List resources
    send_request(proc, {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/list"
    })
    
    # Read a resource
    send_request(proc, {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/read",
        "params": {"uri": "file:///tmp/example.txt"}
    })
    
    # List tools
    send_request(proc, {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/list"
    })
    
    # Call add tool
    send_request(proc, {
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "add",
            "arguments": {"a": 15, "b": 27}
        }
    })
    
    time.sleep(2)  # Wait for final responses
    
except KeyboardInterrupt:
    pass
finally:
    proc.terminate()
    proc.wait()
```

### Method 4: Using curl with HTTP (Future Enhancement)

For easier testing, you might want to add HTTP transport support:

```rust
// In your main.rs, add HTTP transport option
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--http" {
        // HTTP transport for testing
        let transport = airs_mcp::transport::HttpTransport::new("127.0.0.1:3000").await?;
        // ... rest of setup
    } else {
        // Default STDIO transport
        let transport = airs_mcp::transport::StdioTransport::new().await?;
        // ... rest of setup
    }
}
```

Then test with curl:
```bash
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"resources/list"}'
```

## Why STDIO Testing is Challenging

1. **Session-based**: MCP expects persistent connections
2. **Bidirectional**: Both client and server can initiate messages
3. **State Management**: Some operations depend on initialization
4. **Buffering**: STDIO buffering can interfere with real-time testing

## Best Practices for MCP Testing

1. **Use proper MCP clients** when possible
2. **Test initialization first** - it's required for most operations
3. **Allow processing time** between requests
4. **Check error responses** for protocol violations
5. **Use dedicated testing tools** like the Python client above

The MCP server you've built is working correctly - the challenge is just in how to test STDIO-based servers effectively!
