#!/bin/bash

# Simple MCP Client Test Script - Interactive Version
#
# This script demonstrates testing the MCP client that spawns and communicates 
# with the simple-mcp-server, showing real client ↔ server interactions.

set -e

echo "🚀 AIRS MCP Client Example Test - Interactive Version"
echo "======================================================"

# Build the examples
echo "📦 Building simple-mcp-server..."
cd ../simple-mcp-server
cargo build --quiet

echo "📦 Building simple-mcp-client..."
cd ../simple-mcp-client
cargo build --quiet

echo ""
echo "✅ Both examples built successfully!"
echo ""

echo "🔄 Running Interactive MCP Client Demonstration..."
echo "💡 This will show real JSON-RPC messages between client and server:"
echo ""

# Run the interactive client
cargo run --quiet

echo ""
echo "🎉 Interactive MCP Client Test Complete!"
echo ""
echo "💡 What you just saw:"
echo "   ✓ Real server process spawning and management"
echo "   ✓ Complete MCP protocol initialization sequence"
echo "   ✓ Actual JSON-RPC request/response message exchanges"
echo "   ✓ Resource discovery and reading operations"
echo "   ✓ Tool discovery and execution with real parameters"
echo "   ✓ Prompt template discovery and generation"
echo "   ✓ Graceful server shutdown and cleanup"
echo ""
echo "� This demonstrates the complete MCP protocol in action!"
echo "🚀 Use this as a reference for building your own MCP integrations."
