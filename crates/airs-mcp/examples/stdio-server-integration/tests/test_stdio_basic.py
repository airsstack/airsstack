#!/usr/bin/env python3
"""
Basic STDIO MCP Test

Simple test to verify the STDIO MCP server is working.
This is equivalent to test_oauth2_basic.py but for STDIO transport.

Usage:
    python3 test_stdio_basic.py
"""

import json
import subprocess
import sys
import os
from pathlib import Path


class BasicStdioTest:
    """Basic STDIO MCP server test"""
    
    def __init__(self):
        self.server_path = self._find_server_binary()
        
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
        
    def send_request(self, request: dict) -> dict:
        """Send request to STDIO server"""
        request_json = json.dumps(request)
        
        env = os.environ.copy()
        env["STDIO_LOG_LEVEL"] = "error"  # Suppress logs
        
        result = subprocess.run(
            [self.server_path],
            input=request_json,
            capture_output=True,
            text=True,
            timeout=5,
            env=env
        )
        
        if result.returncode != 0:
            raise RuntimeError(f"Server failed: {result.stderr}")
            
        return json.loads(result.stdout.strip())
    
    def test_ping(self):
        """Test ping functionality"""
        print("Testing ping...")
        
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping",
            "params": {}
        }
        
        response = self.send_request(request)
        
        assert response["jsonrpc"] == "2.0"
        assert response["id"] == 1
        assert response["result"] == "pong"
        
        print("✅ Ping test passed")
    
    def test_initialize(self):
        """Test initialize functionality"""
        print("Testing initialize...")
        
        request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "basic-test-client",
                    "version": "1.0.0"
                }
            }
        }
        
        response = self.send_request(request)
        
        assert response["jsonrpc"] == "2.0"
        assert response["id"] == 2
        assert "result" in response
        
        result = response["result"]
        assert "protocolVersion" in result
        assert "capabilities" in result
        assert "serverInfo" in result
        assert result["serverInfo"]["name"] == "airs-mcp-stdio-server"
        
        print("✅ Initialize test passed")
    
    def test_tools_list(self):
        """Test tools list functionality"""
        print("Testing tools/list...")
        
        request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list",
            "params": {}
        }
        
        response = self.send_request(request)
        
        assert response["jsonrpc"] == "2.0"
        assert response["id"] == 3
        assert "result" in response
        
        result = response["result"]
        assert "tools" in result
        assert isinstance(result["tools"], list)
        assert len(result["tools"]) > 0
        
        # Check for math tools
        tool_names = [tool["name"] for tool in result["tools"]]
        assert "add" in tool_names
        assert "multiply" in tool_names
        
        print(f"✅ Tools list test passed ({len(result['tools'])} tools found)")
    
    def run_basic_tests(self):
        """Run basic test suite"""
        print("🚀 Running Basic STDIO MCP Tests")
        print(f"📍 Server: {self.server_path}")
        
        try:
            self.test_ping()
            self.test_initialize() 
            self.test_tools_list()
            
            print("\n🎉 All basic tests passed!")
            return True
            
        except Exception as e:
            print(f"\n❌ Test failed: {e}")
            return False


def main():
    """Main entry point"""
    try:
        tester = BasicStdioTest()
        success = tester.run_basic_tests()
        sys.exit(0 if success else 1)
        
    except Exception as e:
        print(f"❌ Test setup failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()