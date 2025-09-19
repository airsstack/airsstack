#!/usr/bin/env python3
"""
Integration tests for STDIO MCP client against mock server.

These tests verify end-to-end functionality by launching the mock server
and running the client demo against it.
"""

import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path


class TestStdioClientIntegration:
    """Test the STDIO MCP client integration with mock server."""
    
    @classmethod
    def setup_class(cls):
        """Build the binaries before running tests."""
        print("Building STDIO client integration binaries...")
        
        # Build both client and mock server
        result = subprocess.run([
            "cargo", "build", "--bin", "stdio-mcp-client", "--bin", "stdio-mock-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Build failed: {result.stderr}")
            raise RuntimeError(f"Failed to build binaries: {result.stderr}")
        
        print("Build successful")
    
    def test_client_demo_with_mock_server(self):
        """Test that the client demo runs successfully against the mock server."""
        project_dir = Path(__file__).parent.parent
        
        # Set environment to use mock server
        env = os.environ.copy()
        env["USE_MOCK"] = "1"
        env["RUST_LOG"] = "info"
        
        # Run the client demo
        result = subprocess.run([
            "cargo", "run", "--bin", "stdio-mcp-client"
        ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=30)
        
        print(f"Client stdout: {result.stdout}")
        print(f"Client stderr: {result.stderr}")
        
        # Check that the demo completed successfully
        assert result.returncode == 0, f"Client demo failed: {result.stderr}"
        
        # Verify expected output patterns
        output = result.stdout + result.stderr
        assert "STDIO MCP Client Demo" in output
        assert "Initialization successful" in output
        assert "Tools listed successfully" in output
        assert "Echo tool called successfully" in output
        assert "Health check successful" in output
        assert "Timestamp retrieved successfully" in output
        assert "Demo Complete" in output
    
    def test_client_with_custom_server_command(self):
        """Test client with custom server command configuration."""
        project_dir = Path(__file__).parent.parent
        
        # Create a simple echo script that mimics a server
        with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as f:
            f.write('#!/bin/bash\necho \'{"jsonrpc":"2.0","id":"test","result":{"status":"ok"}}\'\n')
            script_path = f.name
        
        try:
            os.chmod(script_path, 0o755)
            
            env = os.environ.copy()
            env["MCP_SERVER_COMMAND"] = script_path
            env["MCP_REQUEST_TIMEOUT"] = "5"
            env["RUST_LOG"] = "debug"
            
            # This should fail quickly since our script doesn't implement MCP protocol
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
            
            # We expect this to fail, but the important thing is that it tried to use our custom command
            output = result.stdout + result.stderr
            assert script_path in output or "Failed to create client" in output
            
        finally:
            os.unlink(script_path)
    
    def test_mock_server_standalone(self):
        """Test that the mock server responds correctly to basic requests."""
        project_dir = Path(__file__).parent.parent
        
        # Start the mock server
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send initialize request
            init_request = {
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "test", "version": "1.0"}
                },
                "id": "test-init"
            }
            
            mock_process.stdin.write(json.dumps(init_request) + "\n")
            mock_process.stdin.flush()
            
            # Read response
            response_line = mock_process.stdout.readline()
            response = json.loads(response_line)
            
            assert response["jsonrpc"] == "2.0"
            assert response["id"] == "test-init"
            assert "result" in response
            assert response["result"]["protocolVersion"] == "2025-06-18"
            
            # Test tools/list
            tools_request = {
                "jsonrpc": "2.0",
                "method": "tools/list",
                "params": {},
                "id": "test-tools"
            }
            
            mock_process.stdin.write(json.dumps(tools_request) + "\n")
            mock_process.stdin.flush()
            
            response_line = mock_process.stdout.readline()
            response = json.loads(response_line)
            
            assert response["jsonrpc"] == "2.0"
            assert response["id"] == "test-tools"
            assert "result" in response
            assert "tools" in response["result"]
            tools = response["result"]["tools"]
            assert len(tools) > 0
            
            # Verify expected tools
            tool_names = [tool["name"] for tool in tools]
            assert "echo" in tool_names
            assert "health_check" in tool_names
            assert "get_timestamp" in tool_names
            
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)


if __name__ == "__main__":
    # Run the tests
    test_instance = TestStdioClientIntegration()
    test_instance.setup_class()
    
    print("Running client demo integration test...")
    test_instance.test_client_demo_with_mock_server()
    print("✓ Client demo test passed")
    
    print("Running custom server command test...")
    test_instance.test_client_with_custom_server_command()
    print("✓ Custom server command test passed")
    
    print("Running mock server standalone test...")
    test_instance.test_mock_server_standalone()
    print("✓ Mock server standalone test passed")
    
    print("All integration tests passed!")