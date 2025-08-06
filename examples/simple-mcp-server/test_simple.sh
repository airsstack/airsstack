#!/bin/bash
# Simple MCP Server STDIO Test
# Direct line-by-line testing

echo "🧪 MCP Server STDIO Testing"
echo "================================"

echo ""
echo "📋 Method 1: Direct JSON-RPC Testing"
echo "Starting server and sending test requests..."

# Test individual requests
test_request() {
    local request="$1"
    local description="$2"
    
    echo ""
    echo "🔹 $description"
    echo "Request: $request"
    echo "Response:"
    echo "$request" | timeout 3s cargo run --bin simple-mcp-server 2>/dev/null
    echo "---"
}

# Test each capability
test_request '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' "Initialize"

test_request '{"jsonrpc":"2.0","id":2,"method":"resources/list"}' "List Resources"

test_request '{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"file:///tmp/example.txt"}}' "Read Resource"

test_request '{"jsonrpc":"2.0","id":4,"method":"tools/list"}' "List Tools"

test_request '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"add","arguments":{"a":10,"b":5}}}' "Call Add Tool"

test_request '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}' "Call Greet Tool"

test_request '{"jsonrpc":"2.0","id":7,"method":"prompts/list"}' "List Prompts"

echo ""
echo "✅ Basic STDIO testing complete!"
echo ""
echo "📖 For interactive testing:"
echo "1. Run: cargo run --bin simple-mcp-server"
echo "2. In another terminal, send JSON-RPC messages manually"
echo "3. Use CTRL+C to stop the server"
