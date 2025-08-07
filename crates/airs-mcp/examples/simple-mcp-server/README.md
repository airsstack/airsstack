# Simple MCP Server Example

This example demonstrates how to create a complete MCP (Model Context Protocol) server using the `airs-mcp` library.

## What This Example Shows

The example implements a fully functional MCP server with three types of providers:

### 🗂️ Resource Provider
- **File System Resources**: Exposes example files that can be read by MCP clients
- **Available Resources**:
  - `file:///tmp/example.txt` - Simple text content
  - `file:///tmp/config.json` - JSON configuration example

### 🔧 Tool Provider
- **Calculator Tool** (`add`): Adds two numbers together
- **Greeting Tool** (`greet`): Generates personalized greeting messages

### 📝 Prompt Provider
- **Code Review** (`code_review`): Generates code review prompts for any programming language
- **Concept Explanation** (`explain_concept`): Creates educational prompts for technical concepts

## How to Run

### Development Testing
```bash
# From the example directory
cd examples/simple-mcp-server

# Run the server for testing
cargo run
```

### Claude Desktop Integration

We provide comprehensive management tools for Claude Desktop integration:

#### 🚀 Quick Setup (Recommended)
```bash
# Automated setup - builds and configures everything
./setup_claude_integration.sh
```

#### 🔧 Manual Setup
```bash
# Build optimized binary and generate configuration
./build.sh

# Follow the manual integration steps in CLAUDE_INTEGRATION.md
```

#### 🧪 Testing & Validation
```bash
# Test your integration setup
./test_integration.sh
```

#### 🛠️ Troubleshooting
If you encounter issues, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for detailed diagnostics and solutions.

### Integration Management Features

- **✅ Automated Setup**: Complete build and configuration in one command
- **✅ Smart Configuration Merging**: Safely merges with existing Claude Desktop config
- **✅ Cross-Platform Support**: macOS, Linux, and Windows detection
- **✅ Comprehensive Testing**: Protocol validation and integration verification
- **✅ Backup & Recovery**: Automatic config backups before changes
- **✅ Detailed Troubleshooting**: Step-by-step diagnostic guidance

See [CLAUDE_INTEGRATION.md](CLAUDE_INTEGRATION.md) for detailed integration information.

## How It Works

1. **Server Initialization**: Creates STDIO transport for communication
2. **Provider Registration**: Registers all three provider types with the server
3. **Protocol Handling**: Automatically handles MCP protocol messages
4. **Client Communication**: Responds to client requests via STDIO

## Example Usage with MCP Client

When connected to an MCP client, you can:

### List Available Resources
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list"
}
```

### Read a Resource
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/read",
  "params": {
    "uri": "file:///tmp/example.txt"
  }
}
```

### Call the Add Tool
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add",
    "arguments": {
      "a": 15,
      "b": 27
    }
  }
}
```

### Get a Code Review Prompt
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "prompts/get",
  "params": {
    "name": "code_review",
    "arguments": {
      "language": "rust",
      "code": "fn main() { println!(\"Hello\"); }"
    }
  }
}
```

## Architecture

The example demonstrates the high-level `airs-mcp` APIs:

- **`McpServerBuilder`**: Fluent API for server configuration
- **Provider Traits**: Clean abstractions for extending functionality
  - `ResourceProvider`: For exposing readable resources
  - `ToolProvider`: For executable tools and functions
  - `PromptProvider`: For prompt templates and generation
- **STDIO Transport**: Standard input/output communication
- **Async Architecture**: Full async/await support throughout

## Key Features Demonstrated

✅ **Complete MCP Implementation**: All major MCP protocol features  
✅ **Type-Safe APIs**: Rust's type system ensures protocol correctness  
✅ **Error Handling**: Proper error propagation and MCP error responses  
✅ **Async Support**: Non-blocking I/O with tokio  
✅ **Extensible Design**: Easy to add new providers and capabilities  
✅ **Production Ready**: Includes logging, error handling, and proper resource management  

## Next Steps

This example provides a foundation for building more sophisticated MCP servers:

- Add database connectivity for dynamic resources
- Implement authentication and authorization
- Create specialized tools for your domain
- Add configuration management
- Implement custom transport layers
- Add metrics and monitoring

The `airs-mcp` library handles all the protocol complexity, letting you focus on your application logic.
