#!/bin/bash
# Manual MCP Server Testing Script
# Tests individual MCP server capabilities

echo "🚀 Starting MCP Server in background..."

# Start server in background
cargo run --bin simple-mcp-server &
SERVER_PID=$!

# Wait for server to start
sleep 2

echo "📋 Testing MCP Server Capabilities..."

# Function to send JSON-RPC message
send_request() {
    local message="$1"
    local description="$2"
    
    echo ""
    echo "🔹 $description"
    echo "→ Sending: $message"
    
    # Send message to server and capture response
    echo "$message" | timeout 5s socat - EXEC:"cargo run --bin simple-mcp-server" 2>/dev/null | head -1
}

# Test 1: Initialize
send_request '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"roots":{"listChanged":true}},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' "Initializing connection"

# Test 2: List Resources
send_request '{"jsonrpc":"2.0","id":2,"method":"resources/list"}' "Listing available resources"

# Test 3: Read Resource
send_request '{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"file:///tmp/example.txt"}}' "Reading example.txt resource"

# Test 4: List Tools
send_request '{"jsonrpc":"2.0","id":4,"method":"tools/list"}' "Listing available tools"

# Test 5: Call Add Tool
send_request '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"add","arguments":{"a":15,"b":27}}}' "Calling add tool (15 + 27)"

# Test 6: Call Greet Tool
send_request '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"greet","arguments":{"name":"Bob"}}}' "Calling greet tool"

# Test 7: List Prompts
send_request '{"jsonrpc":"2.0","id":7,"method":"prompts/list"}' "Listing available prompts"

# Test 8: Get Code Review Prompt
send_request '{"jsonrpc":"2.0","id":8,"method":"prompts/get","params":{"name":"code_review","arguments":{"language":"rust","code":"fn hello() { println!(\"Hi!\"); }"}}}' "Getting code review prompt"

echo ""
echo "✅ Testing complete!"

# Cleanup
kill $SERVER_PID 2>/dev/null
