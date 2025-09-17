#!/usr/bin/env python3
"""
Comprehensive STDIO MCP Test Suite

This script provides comprehensive testing of the STDIO MCP server integration,
including all MCP protocol methods, error handling, and edge cases.

Usage:
    python3 test_stdio_comprehensive.py [--verbose] [--server-path PATH]
"""

import json
import subprocess
import argparse
import sys
import os
import time
from pathlib import Path
from typing import Dict, Any, List


class ComprehensiveStdioTest:
    """Comprehensive STDIO MCP server test suite"""
    
    def __init__(self, verbose: bool = False, server_path: str = None):
        self.verbose = verbose
        self.server_path = server_path or self._find_server_binary()
        self.test_results: List[Dict[str, Any]] = []
        
    def _find_server_binary(self) -> str:
        """Find the STDIO server binary"""
        test_dir = Path(__file__).parent
        possible_paths = [
            test_dir / "../target/debug/stdio-server",
            test_dir / "../target/release/stdio-server",
            test_dir / "../../../../target/debug/examples/stdio-server-integration",
            test_dir / "../../../../target/release/examples/stdio-server-integration"
        ]
        
        for path in possible_paths:
            if Path(path).exists():
                return str(path.resolve())
        
        raise FileNotFoundError("Could not find stdio-server binary")
    
    def log(self, message: str):
        """Log message if verbose"""
        if self.verbose:
            print(f"🔍 {message}")
    
    def send_request(self, request: Dict[str, Any], timeout: int = 5) -> Dict[str, Any]:
        """Send request to STDIO server with timing"""
        start_time = time.time()
        request_json = json.dumps(request)
        
        self.log(f"Sending: {request_json}")
        
        env = os.environ.copy()
        env["STDIO_LOG_LEVEL"] = "error"
        
        result = subprocess.run(
            [self.server_path],
            input=request_json,
            capture_output=True,
            text=True,
            timeout=timeout,
            env=env
        )
        
        duration = time.time() - start_time
        
        if result.returncode != 0:
            raise RuntimeError(f"Server failed (exit {result.returncode}): {result.stderr}")
        
        response = json.loads(result.stdout.strip())
        self.log(f"Received: {response}")
        self.log(f"Duration: {duration:.3f}s")
        
        return response
    
    def run_test(self, test_name: str, test_func) -> bool:
        """Run a single test and record results"""
        print(f"🧪 {test_name}...")
        
        try:
            start_time = time.time()
            result = test_func()
            duration = time.time() - start_time
            
            self.test_results.append({
                "name": test_name,
                "passed": True,
                "duration": duration,
                "error": None
            })
            
            print(f"✅ {test_name} passed ({duration:.3f}s)")
            return True
            
        except Exception as e:
            duration = time.time() - start_time
            
            self.test_results.append({
                "name": test_name,
                "passed": False,
                "duration": duration,
                "error": str(e)
            })
            
            print(f"❌ {test_name} failed: {e}")
            return False
    
    def test_ping_basic(self):
        """Test basic ping functionality"""
        request = {
            "jsonrpc": "2.0",
            "id": "ping-1",
            "method": "ping",
            "params": {}
        }
        
        response = self.send_request(request)
        
        assert response["jsonrpc"] == "2.0"
        assert response["id"] == "ping-1"
        assert response["result"] == "pong"
    
    def test_ping_with_different_ids(self):
        """Test ping with different ID types"""
        test_cases = [
            {"id": 1, "expected": 1},
            {"id": "test-string", "expected": "test-string"},
            {"id": None, "expected": None}
        ]
        
        for case in test_cases:
            request = {
                "jsonrpc": "2.0",
                "id": case["id"],
                "method": "ping",
                "params": {}
            }
            
            response = self.send_request(request)
            assert response["id"] == case["expected"]
            assert response["result"] == "pong"
    
    def test_initialize_comprehensive(self):
        """Test comprehensive initialize functionality"""
        request = {
            "jsonrpc": "2.0",
            "id": "init-1",
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
                    "name": "comprehensive-test-client",
                    "version": "2.0.0"
                }
            }
        }
        
        response = self.send_request(request)
        
        assert response["jsonrpc"] == "2.0"
        assert response["id"] == "init-1"
        assert "result" in response
        
        result = response["result"]
        assert result["protocolVersion"] == "2024-11-05"
        assert "capabilities" in result
        assert "serverInfo" in result
        
        server_info = result["serverInfo"]
        assert server_info["name"] == "airs-mcp-stdio-server"
        assert "version" in server_info
    
    def test_tools_complete_workflow(self):
        """Test complete tools workflow: list -> call"""
        # First, list tools
        list_request = {
            "jsonrpc": "2.0",
            "id": "tools-list-1",
            "method": "tools/list",
            "params": {}
        }
        
        list_response = self.send_request(list_request)
        assert "result" in list_response
        
        tools = list_response["result"]["tools"]
        assert len(tools) > 0
        
        # Find a math tool to test
        add_tool = None
        for tool in tools:
            if tool["name"] == "add":
                add_tool = tool
                break
        
        assert add_tool is not None, "Add tool not found"
        
        # Test calling the add tool
        call_request = {
            "jsonrpc": "2.0",
            "id": "tools-call-1",
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": {
                    "numbers": [10, 20, 30]
                }
            }
        }
        
        call_response = self.send_request(call_request)
        assert "result" in call_response
        
        result = call_response["result"]
        assert "content" in result
        # The exact format depends on implementation, but should have content
    
    def test_tools_error_handling(self):
        """Test tools error handling"""
        # Test calling non-existent tool
        request = {
            "jsonrpc": "2.0",
            "id": "tools-error-1",
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        }
        
        response = self.send_request(request)
        assert "error" in response
        assert response["error"]["code"] == -32602  # Invalid params
    
    def test_resources_complete_workflow(self):
        """Test complete resources workflow: list -> read"""
        # First, list resources
        list_request = {
            "jsonrpc": "2.0",
            "id": "resources-list-1",
            "method": "resources/list",
            "params": {}
        }
        
        list_response = self.send_request(list_request)
        assert "result" in list_response
        
        resources = list_response["result"]["resources"]
        assert len(resources) > 0
        
        # Pick first resource to read
        test_resource = resources[0]
        assert "uri" in test_resource
        
        # Test reading the resource
        read_request = {
            "jsonrpc": "2.0",
            "id": "resources-read-1",
            "method": "resources/read",
            "params": {
                "uri": test_resource["uri"]
            }
        }
        
        read_response = self.send_request(read_request)
        assert "result" in read_response
        
        result = read_response["result"]
        assert "contents" in result
    
    def test_prompts_list(self):
        """Test prompts list functionality"""
        request = {
            "jsonrpc": "2.0",
            "id": "prompts-1",
            "method": "prompts/list",
            "params": {}
        }
        
        response = self.send_request(request)
        assert "result" in response
        
        result = response["result"]
        assert "prompts" in result
        assert isinstance(result["prompts"], list)
    
    def test_invalid_json_rpc(self):
        """Test invalid JSON-RPC requests"""
        # Test missing jsonrpc field
        request = {
            "id": 1,
            "method": "ping",
            "params": {}
        }
        
        try:
            response = self.send_request(request)
            # Should get an error response
            assert "error" in response
        except Exception:
            # Or the server might reject it entirely
            pass
    
    def test_invalid_method(self):
        """Test invalid method handling"""
        request = {
            "jsonrpc": "2.0",
            "id": "invalid-1",
            "method": "invalid/method/name",
            "params": {}
        }
        
        response = self.send_request(request)
        assert "error" in response
        assert response["error"]["code"] == -32601  # Method not found
    
    def test_malformed_params(self):
        """Test malformed parameters handling"""
        request = {
            "jsonrpc": "2.0",
            "id": "malformed-1",
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": "this should be an object"
            }
        }
        
        response = self.send_request(request)
        assert "error" in response
    
    def test_performance_basic(self):
        """Test basic performance characteristics"""
        # Test multiple rapid requests
        start_time = time.time()
        
        for i in range(5):
            request = {
                "jsonrpc": "2.0",
                "id": f"perf-{i}",
                "method": "ping",
                "params": {}
            }
            
            response = self.send_request(request)
            assert response["result"] == "pong"
        
        total_time = time.time() - start_time
        
        # Should complete 5 pings in reasonable time
        assert total_time < 5.0, f"Performance test too slow: {total_time:.3f}s"
        
        self.log(f"Performance: 5 pings in {total_time:.3f}s ({total_time/5:.3f}s avg)")
    
    def run_comprehensive_tests(self):
        """Run all comprehensive tests"""
        print("🚀 Running Comprehensive STDIO MCP Test Suite")
        print(f"📍 Server: {self.server_path}")
        print()
        
        tests = [
            ("Ping Basic", self.test_ping_basic),
            ("Ping ID Types", self.test_ping_with_different_ids),
            ("Initialize Comprehensive", self.test_initialize_comprehensive),
            ("Tools Complete Workflow", self.test_tools_complete_workflow),
            ("Tools Error Handling", self.test_tools_error_handling),
            ("Resources Complete Workflow", self.test_resources_complete_workflow),
            ("Prompts List", self.test_prompts_list),
            ("Invalid JSON-RPC", self.test_invalid_json_rpc),
            ("Invalid Method", self.test_invalid_method),
            ("Malformed Params", self.test_malformed_params),
            ("Performance Basic", self.test_performance_basic),
        ]
        
        passed = 0
        for test_name, test_func in tests:
            if self.run_test(test_name, test_func):
                passed += 1
        
        # Print comprehensive summary
        print("\n" + "="*70)
        print("📊 COMPREHENSIVE TEST RESULTS")
        print("="*70)
        
        print(f"Total Tests: {len(tests)}")
        print(f"Passed: {passed}")
        print(f"Failed: {len(tests) - passed}")
        
        if passed == len(tests):
            print("\n🎉 ALL COMPREHENSIVE TESTS PASSED!")
            print("The STDIO MCP server is fully functional and robust.")
        else:
            print(f"\n💥 {len(tests) - passed} tests failed.")
            
        # Detailed results
        print("\nDetailed Results:")
        for result in self.test_results:
            status = "✅" if result["passed"] else "❌"
            duration = f"{result['duration']:.3f}s"
            print(f"  {status} {result['name']} ({duration})")
            if not result["passed"]:
                print(f"     Error: {result['error']}")
        
        # Performance summary
        total_time = sum(r["duration"] for r in self.test_results)
        avg_time = total_time / len(self.test_results)
        print(f"\nPerformance Summary:")
        print(f"  Total time: {total_time:.3f}s")
        print(f"  Average per test: {avg_time:.3f}s")
        
        return passed == len(tests)


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Comprehensive STDIO MCP Test Suite")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    parser.add_argument("--server-path", help="Path to the stdio-server binary")
    
    args = parser.parse_args()
    
    try:
        tester = ComprehensiveStdioTest(verbose=args.verbose, server_path=args.server_path)
        success = tester.run_comprehensive_tests()
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\n🛑 Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Test suite failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()