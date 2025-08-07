#!/usr/bin/env python3
"""
Working MCP STDIO Test Client
Properly handles bidirectional STDIO communication with the MCP server
"""

import json
import subprocess
import threading
import time
import queue
import sys

class McpTestClient:
    def __init__(self):
        self.proc = None
        self.response_queue = queue.Queue()
        self.reader_thread = None
        self.running = False
        
    def start_server(self):
        """Start the MCP server process"""
        print("🚀 Starting MCP Server...")
        self.proc = subprocess.Popen(
            ['cargo', 'run', '--bin', 'simple-mcp-server'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        # Start response reader thread
        self.running = True
        self.reader_thread = threading.Thread(target=self._read_responses)
        self.reader_thread.daemon = True
        self.reader_thread.start()
        
        # Wait for server to initialize
        time.sleep(2)
        print("✅ Server started")
        
    def _read_responses(self):
        """Read responses from server in background thread"""
        while self.running and self.proc.poll() is None:
            try:
                line = self.proc.stdout.readline()
                if line:
                    line = line.strip()
                    if line:
                        try:
                            response = json.loads(line)
                            self.response_queue.put(response)
                        except json.JSONDecodeError:
                            # Non-JSON output (like debug messages)
                            pass
            except Exception as e:
                if self.running:
                    print(f"Error reading response: {e}")
                break
    
    def send_request(self, request):
        """Send a request and wait for response"""
        if not self.proc:
            raise Exception("Server not started")
            
        json_str = json.dumps(request)
        print(f"→ {json_str}")
        
        self.proc.stdin.write(json_str + '\n')
        self.proc.stdin.flush()
        
        # Wait for response with timeout
        try:
            response = self.response_queue.get(timeout=5)
            print(f"← {json.dumps(response, indent=2)}")
            return response
        except queue.Empty:
            print("⚠️  No response received (timeout)")
            return None
    
    def stop_server(self):
        """Stop the server and cleanup"""
        print("🛑 Stopping server...")
        self.running = False
        if self.proc:
            self.proc.terminate()
            self.proc.wait()
        print("✅ Server stopped")
    
    def run_tests(self):
        """Run comprehensive tests"""
        try:
            self.start_server()
            
            print("\n📋 Running MCP Server Tests...\n")
            
            # Test 1: Initialize
            print("1️⃣ Initializing connection...")
            response = self.send_request({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocol_version": "2024-11-05",
                    "capabilities": {
                        "roots": {"listChanged": True}
                    },
                    "client_info": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            })
            
            if not response or "error" in response:
                print("❌ Initialization failed!")
                return
                
            print("✅ Initialization successful\n")
            
            # Test 2: List Resources
            print("2️⃣ Listing resources...")
            self.send_request({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "resources/list"
            })
            print()
            
            # Test 3: Read Resource
            print("3️⃣ Reading example resource...")
            self.send_request({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "resources/read",
                "params": {
                    "uri": "file:///tmp/example.txt"
                }
            })
            print()
            
            # Test 4: List Tools
            print("4️⃣ Listing tools...")
            self.send_request({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/list"
            })
            print()
            
            # Test 5: Call Add Tool
            print("5️⃣ Calling add tool (15 + 27)...")
            self.send_request({
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
            })
            print()
            
            # Test 6: Call Greet Tool
            print("6️⃣ Calling greet tool...")
            self.send_request({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "greet",
                    "arguments": {
                        "name": "Alice"
                    }
                }
            })
            print()
            
            # Test 7: List Prompts
            print("7️⃣ Listing prompts...")
            self.send_request({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "prompts/list"
            })
            print()
            
            # Test 8: Get Code Review Prompt
            print("8️⃣ Getting code review prompt...")
            self.send_request({
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
            })
            print()
            
            print("🎉 All tests completed!")
            
        except KeyboardInterrupt:
            print("\n🛑 Tests interrupted by user")
        except Exception as e:
            print(f"❌ Test error: {e}")
        finally:
            self.stop_server()

def main():
    client = McpTestClient()
    client.run_tests()

if __name__ == "__main__":
    main()
