#!/usr/bin/env python3
"""
STDIO MCP Integration Test Suite

This script demonstrates and tests the STDIO MCP server integration functionality:
- Direct STDIO communication with the MCP server
- JSON-RPC request/response validation
- All MCP protocol methods testing
- Provider functionality verification

Usage:
    python3 test_stdio_integration.py [--debug] [--server-path PATH]
"""

import json
import subprocess
import argparse
import sys
import os
import tempfile
import shutil
from typing import Dict, Optional, Any, List
from pathlib import Path


class StdioMcpTest:
    """STDIO MCP server integration test suite"""
    
    def __init__(self, debug: bool = False, server_path: Optional[str] = None):
        self.debug = debug
        self.server_path = server_path or self._find_server_binary()
        self.test_count = 0
        self.passed_count = 0
        self.failed_tests: List[str] = []
        
    def _find_server_binary(self) -> str:
        """Find the STDIO server binary"""
        # Check relative to this test file
        test_dir = Path(__file__).parent
        possible_paths = [
            test_dir / "../target/debug/stdio-server",
            test_dir / "../target/release/stdio-server", 
            test_dir / "../../../../target/debug/examples/stdio-server-integration",
            test_dir / "../../../../target/release/examples/stdio-server-integration",
            "./stdio-server"
        ]
        
        for path in possible_paths:
            if Path(path).exists():
                return str(path.resolve())
        
        raise FileNotFoundError(
            "Could not find stdio-server binary. "
            "Please build the project or specify --server-path"
        )
        
    def log(self, message: str, level: str = "INFO"):
        """Log a message"""
        if level == "DEBUG" and not self.debug:
            return
        prefix = "🐛" if level == "DEBUG" else "ℹ️"
        print(f"{prefix} {message}")
        
    def success(self, message: str):
        """Log success"""
        print(f"✅ {message}")
        self.passed_count += 1
        
    def error(self, message: str):
        """Log error"""
        print(f"❌ {message}")
        
    def start_test(self, test_name: str):
        """Start a test"""
        self.test_count += 1
        self.log(f"🧪 Test {self.test_count}: {test_name}")
        
    def fail_test(self, test_name: str, reason: str):
        """Mark test as failed"""
        self.error(f"Test failed: {test_name} - {reason}")
        self.failed_tests.append(f"{test_name}: {reason}")
        
    def send_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send a JSON-RPC request to the STDIO server"""
        try:
            # Convert request to JSON
            request_json = json.dumps(request)
            self.log(f"Sending request: {request_json}", "DEBUG")
            
            # Run the server process with the request
            env = os.environ.copy()
            env["STDIO_LOG_LEVEL"] = "error"  # Suppress logs for clean JSON output
            
            result = subprocess.run(
                [self.server_path],
                input=request_json,
                capture_output=True,
                text=True,
                timeout=10,
                env=env
            )
            
            if result.returncode != 0:
                self.error(f"Server process failed with exit code {result.returncode}")
                self.error(f"Stderr: {result.stderr}")
                return None
                
            # Parse the JSON response
            if not result.stdout.strip():
                self.error("No response received from server")
                return None
                
            response = json.loads(result.stdout.strip())
            self.log(f"Received response: {response}", "DEBUG")
            return response
            
        except subprocess.TimeoutExpired:
            self.error("Server request timed out")
            return None
        except json.JSONDecodeError as e:
            self.error(f"Failed to parse JSON response: {e}")
            self.error(f"Raw output: {result.stdout}")
            return None
        except Exception as e:
            self.error(f"Request failed: {e}")
            return None
    
    def validate_json_rpc_response(self, response: Dict[str, Any], expected_id: Any) -> bool:
        """Validate JSON-RPC response format"""
        if not isinstance(response, dict):
            return False
            
        # Check required fields
        if response.get("jsonrpc") != "2.0":
            self.error(f"Invalid jsonrpc version: {response.get('jsonrpc')}")
            return False
            
        if response.get("id") != expected_id:
            self.error(f"ID mismatch: expected {expected_id}, got {response.get('id')}")
            return False
            
        # Must have either result or error, but not both
        has_result = "result" in response
        has_error = "error" in response
        
        if has_result and has_error:
            self.error("Response has both result and error")
            return False
            
        if not has_result and not has_error:
            self.error("Response has neither result nor error")
            return False
            
        return True
    
    def test_ping(self) -> bool:
        """Test ping method"""
        self.start_test("Ping")
        
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping",
            "params": {}
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Ping", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 1):
            self.fail_test("Ping", "Invalid JSON-RPC response")
            return False
            
        if response.get("result") != "pong":
            self.fail_test("Ping", f"Expected 'pong', got {response.get('result')}")
            return False
            
        self.success("Ping test passed")
        return True
    
    def test_initialize(self) -> bool:
        """Test initialize method"""
        self.start_test("Initialize")
        
        request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "stdio-test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Initialize", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 2):
            self.fail_test("Initialize", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        
        # Validate initialize response structure
        if "protocolVersion" not in result:
            self.fail_test("Initialize", "Missing protocolVersion in response")
            return False
            
        if "capabilities" not in result:
            self.fail_test("Initialize", "Missing capabilities in response")
            return False
            
        if "serverInfo" not in result:
            self.fail_test("Initialize", "Missing serverInfo in response")
            return False
            
        server_info = result["serverInfo"]
        if server_info.get("name") != "airs-mcp-stdio-server":
            self.fail_test("Initialize", f"Unexpected server name: {server_info.get('name')}")
            return False
            
        self.success("Initialize test passed")
        return True
    
    def test_tools_list(self) -> bool:
        """Test tools/list method"""
        self.start_test("Tools List")
        
        request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {}
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Tools List", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 3):
            self.fail_test("Tools List", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        tools = result.get("tools", [])
        
        if not isinstance(tools, list):
            self.fail_test("Tools List", "Tools should be a list")
            return False
            
        if len(tools) == 0:
            self.fail_test("Tools List", "No tools returned")
            return False
            
        # Check for expected math tools
        tool_names = [tool.get("name") for tool in tools]
        expected_tools = ["add", "subtract", "multiply", "divide"]
        
        for expected_tool in expected_tools:
            if expected_tool not in tool_names:
                self.fail_test("Tools List", f"Missing expected tool: {expected_tool}")
                return False
                
        self.success(f"Tools List test passed ({len(tools)} tools found)")
        return True
    
    def test_tool_call(self) -> bool:
        """Test tools/call method"""
        self.start_test("Tool Call (add)")
        
        request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": {
                    "numbers": [2, 3, 5]
                }
            }
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Tool Call", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 4):
            self.fail_test("Tool Call", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        
        # The exact structure depends on the MathToolProvider implementation
        # Let's just verify we got a reasonable result
        if "content" not in result:
            self.fail_test("Tool Call", "Missing content in tool result")
            return False
            
        self.success("Tool Call test passed")
        return True
    
    def test_resources_list(self) -> bool:
        """Test resources/list method"""
        self.start_test("Resources List")
        
        request = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/list",
            "params": {}
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Resources List", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 5):
            self.fail_test("Resources List", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        resources = result.get("resources", [])
        
        if not isinstance(resources, list):
            self.fail_test("Resources List", "Resources should be a list")
            return False
            
        if len(resources) == 0:
            self.fail_test("Resources List", "No resources returned")
            return False
            
        # Check for expected test files
        resource_names = [resource.get("name") for resource in resources]
        expected_files = ["config.json", "README.md", "stdio-test.txt"]
        
        for expected_file in expected_files:
            if expected_file not in resource_names:
                self.fail_test("Resources List", f"Missing expected file: {expected_file}")
                return False
                
        self.success(f"Resources List test passed ({len(resources)} resources found)")
        return True
    
    def test_resource_read(self) -> bool:
        """Test resources/read method"""
        self.start_test("Resource Read")
        
        # First get the list of resources to find a valid URI
        list_request = {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "resources/list",
            "params": {}
        }
        
        list_response = self.send_request(list_request)
        if not list_response:
            self.fail_test("Resource Read", "Failed to get resource list")
            return False
            
        resources = list_response.get("result", {}).get("resources", [])
        if not resources:
            self.fail_test("Resource Read", "No resources available to read")
            return False
            
        # Pick the first resource to read
        test_resource = resources[0]
        resource_uri = test_resource.get("uri")
        
        if not resource_uri:
            self.fail_test("Resource Read", "Resource missing URI")
            return False
        
        read_request = {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "resources/read",
            "params": {
                "uri": resource_uri
            }
        }
        
        response = self.send_request(read_request)
        if not response:
            self.fail_test("Resource Read", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 7):
            self.fail_test("Resource Read", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        
        if "contents" not in result:
            self.fail_test("Resource Read", "Missing contents in resource result")
            return False
            
        self.success(f"Resource Read test passed (read {test_resource.get('name')})")
        return True
    
    def test_prompts_list(self) -> bool:
        """Test prompts/list method"""
        self.start_test("Prompts List")
        
        request = {
            "jsonrpc": "2.0",
            "id": 8,
            "method": "prompts/list",
            "params": {}
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Prompts List", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 8):
            self.fail_test("Prompts List", "Invalid JSON-RPC response")
            return False
            
        result = response.get("result", {})
        prompts = result.get("prompts", [])
        
        if not isinstance(prompts, list):
            self.fail_test("Prompts List", "Prompts should be a list")
            return False
            
        # Code review prompts should be available
        if len(prompts) > 0:
            self.success(f"Prompts List test passed ({len(prompts)} prompts found)")
        else:
            self.success("Prompts List test passed (no prompts configured)")
        return True
    
    def test_invalid_method(self) -> bool:
        """Test error handling for invalid methods"""
        self.start_test("Invalid Method")
        
        request = {
            "jsonrpc": "2.0",
            "id": 9,
            "method": "invalid/method",
            "params": {}
        }
        
        response = self.send_request(request)
        if not response:
            self.fail_test("Invalid Method", "No response received")
            return False
            
        if not self.validate_json_rpc_response(response, 9):
            self.fail_test("Invalid Method", "Invalid JSON-RPC response")
            return False
            
        # Should have an error, not a result
        if "error" not in response:
            self.fail_test("Invalid Method", "Expected error response for invalid method")
            return False
            
        error = response["error"]
        if error.get("code") != -32601:  # Method not found
            self.fail_test("Invalid Method", f"Expected error code -32601, got {error.get('code')}")
            return False
            
        self.success("Invalid Method test passed")
        return True
    
    def run_all_tests(self) -> bool:
        """Run all tests"""
        self.log("🚀 Starting STDIO MCP Integration Test Suite")
        self.log(f"📍 Server binary: {self.server_path}")
        
        # Check if server binary exists
        if not Path(self.server_path).exists():
            self.error(f"Server binary not found: {self.server_path}")
            return False
        
        tests = [
            self.test_ping,
            self.test_initialize,
            self.test_tools_list,
            self.test_tool_call,
            self.test_resources_list,
            self.test_resource_read,
            self.test_prompts_list,
            self.test_invalid_method,
        ]
        
        all_passed = True
        
        for test in tests:
            try:
                if not test():
                    all_passed = False
            except Exception as e:
                test_name = test.__name__.replace("test_", "").replace("_", " ").title()
                self.fail_test(test_name, f"Exception: {e}")
                all_passed = False
        
        # Print summary
        print("\n" + "="*60)
        print(f"📊 Test Summary:")
        print(f"   Total tests: {self.test_count}")
        print(f"   Passed: {self.passed_count}")
        print(f"   Failed: {len(self.failed_tests)}")
        
        if self.failed_tests:
            print(f"\n❌ Failed tests:")
            for failure in self.failed_tests:
                print(f"   • {failure}")
        
        if all_passed:
            print(f"\n🎉 All tests passed! STDIO MCP server is working correctly.")
        else:
            print(f"\n💥 Some tests failed. Please check the server implementation.")
            
        return all_passed


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="STDIO MCP Integration Test Suite")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    parser.add_argument("--server-path", help="Path to the stdio-server binary")
    
    args = parser.parse_args()
    
    try:
        tester = StdioMcpTest(debug=args.debug, server_path=args.server_path)
        success = tester.run_all_tests()
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\n🛑 Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Test suite failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()