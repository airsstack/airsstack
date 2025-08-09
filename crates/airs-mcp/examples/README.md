# AIRS-MCP Examples

This directory contains example implementations demonstrating how to use the `airs-mcp` library.

## Examples

### [simple-mcp-server](./simple-mcp-server/)

A comprehensive example demonstrating the core features of the MCP (Model Context Protocol) server implementation:

- **Resources**: File system resource provider
- **Tools**: Calculator and greeting tools  
- **Prompts**: Code review and concept explanation prompts
- **Testing Scripts**: Python and shell scripts for manual testing

**To run:**
```bash
cd examples/simple-mcp-server
cargo run
```

**To test:**
```bash
cd examples/simple-mcp-server
./test_simple.sh
```

### [simple-mcp-client](./simple-mcp-client/) ✨ **NEW** - Interactive Protocol Demo

A comprehensive example demonstrating **real client ↔ server communication** that spawns an MCP server and shows the complete JSON-RPC message exchange:

- **📡 Real Server Interaction**: Spawns and communicates with actual MCP server processes
- **📋 Complete Protocol Demo**: Shows every step of MCP initialization, resources, tools, and prompts
- **📤📥 Message Logging**: See actual JSON-RPC requests and responses being exchanged  
- **🔄 Process Management**: Proper server spawning, communication, and cleanup
- **🎯 Educational Value**: Perfect for understanding how MCP works under the hood

**To see real client ↔ server interactions:**
```bash
# Build both examples
cd examples/simple-mcp-server && cargo build
cd examples/simple-mcp-client && cargo build

# Run the interactive demo (automatically spawns server)
cd examples/simple-mcp-client
cargo run
```

**Sample output shows real protocol messages:**
```
📤 Sending: {"id":"init-1","jsonrpc":"2.0","method":"initialize",...}
📥 Received: {"jsonrpc":"2.0","result":{"capabilities":{"prompts":...}
✅ Initialization successful! Server responded with capabilities.

📤 Sending: {"id":"call-tool-1","jsonrpc":"2.0","method":"tools/call",...}
📥 Received: {"jsonrpc":"2.0","result":{"content":[{"text":"{\n  \"result\": 42.0\n}"...}
🎯 Tool result: { "operation": "addition", "result": 42.0 }
```

**Use with any MCP server:**
```bash
cargo run -- --server-path /path/to/your/mcp-server
```

## Creating New Examples

Each example should be a self-contained Rust project with its own `Cargo.toml`:

1. Create a new directory: `mkdir examples/my-example`
2. Add standalone `Cargo.toml` with `airs-mcp` dependency
3. Include documentation and testing scripts
4. Add description to this README

## Structure

```
examples/
├── README.md                    # This file
├── simple-mcp-server/          # Basic MCP server implementation
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── README.md
│   └── test_*.{sh,py}          # Testing scripts
└── future-example/             # Template for new examples
    ├── Cargo.toml
    ├── src/main.rs
    └── README.md
```

Each example demonstrates specific aspects of the `airs-mcp` library and serves as both documentation and reference implementation.
