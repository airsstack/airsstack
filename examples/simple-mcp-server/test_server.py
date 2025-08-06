#!/usr/bin/env python3
"""
Simple MCP Server Tester
Sends JSON-RPC messages to test the MCP server functionality
"""

import json
import subprocess
import time
import sys

def send_message(proc, message):
    """Send a JSON-RPC message to the server"""
    json_str = json.dumps(message)
    print(f"→ Sending: {json_str}")
    proc.stdin.write(json_str + '\n')
    proc.stdin.flush()
    
    # Give server time to respond
    time.sleep(0.1)

def read_response(proc):
    """Read response from server"""
    try:
        line = proc.stdout.readline()
        if line:
            response = json.loads(line.strip())
            print(f"← Received: {json.dumps(response, indent=2)}")
            return response
    except json.JSONDecodeError as e:
        print(f"Error parsing response: {e}")
        print(f"Raw line: {line}")
    return None

def test_mcp_server():
    """Test the MCP server with various requests"""
    
    # Start the server
    print("🚀 Starting MCP Server...")
    proc = subprocess.Popen(
        ['cargo', 'run', '--bin', 'simple-mcp-server'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Wait for server to initialize
    time.sleep(2)
    
    try:
        print("\n📋 Testing Server Capabilities...")
        
        # Test 1: Initialize connection
        print("\n1️⃣ Initializing connection...")
        init_msg = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {
                        "listChanged": True
                    },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        send_message(proc, init_msg)
        read_response(proc)
        
        # Test 2: List Resources
        print("\n2️⃣ Testing Resources...")
        list_resources_msg = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list"
        }
        send_message(proc, list_resources_msg)
        read_response(proc)
        
        # Test 3: Read Resource
        print("\n3️⃣ Reading a resource...")
        read_resource_msg = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/read",
            "params": {
                "uri": "file:///tmp/example.txt"
            }
        }
        send_message(proc, read_resource_msg)
        read_response(proc)
        
        # Test 4: List Tools
        print("\n4️⃣ Testing Tools...")
        list_tools_msg = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/list"
        }
        send_message(proc, list_tools_msg)
        read_response(proc)
        
        # Test 5: Call Add Tool
        print("\n5️⃣ Calling add tool...")
        call_tool_msg = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": {
                    "a": 15,
                    "b": 27
                }
            }
        }
        send_message(proc, call_tool_msg)
        read_response(proc)
        
        # Test 6: Call Greet Tool
        print("\n6️⃣ Calling greet tool...")
        greet_tool_msg = {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "greet",
                "arguments": {
                    "name": "Alice"
                }
            }
        }
        send_message(proc, greet_tool_msg)
        read_response(proc)
        
        # Test 7: List Prompts
        print("\n7️⃣ Testing Prompts...")
        list_prompts_msg = {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "prompts/list"
        }
        send_message(proc, list_prompts_msg)
        read_response(proc)
        
        # Test 8: Get Code Review Prompt
        print("\n8️⃣ Getting code review prompt...")
        get_prompt_msg = {
            "jsonrpc": "2.0",
            "id": 8,
            "method": "prompts/get",
            "params": {
                "name": "code_review",
                "arguments": {
                    "language": "rust",
                    "code": "fn main() {\n    println!(\"Hello, world!\");\n}"
                }
            }
        }
        send_message(proc, get_prompt_msg)
        read_response(proc)
        
        print("\n✅ All tests completed!")
        
    except KeyboardInterrupt:
        print("\n🛑 Test interrupted by user")
    finally:
        print("\n🔚 Terminating server...")
        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    test_mcp_server()
