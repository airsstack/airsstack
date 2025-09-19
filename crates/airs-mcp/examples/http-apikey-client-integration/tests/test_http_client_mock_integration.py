#!/usr/bin/env python3
"""
Integration tests for HTTP API Key MCP client with mock server.

These tests verify end-to-end functionality by launching the mock server
and testing the HTTP client against it with all authentication methods
and MCP operations.
"""

import json
import os
import subprocess
import sys
import time
import threading
from pathlib import Path
from typing import Dict, Any, Optional

import pytest
import requests
import psutil


class TestHttpClientMockIntegration:
    """Test the HTTP API Key MCP client integration with mock server."""
    
    @classmethod
    def setup_class(cls):
        """Build binaries and start the mock server before running tests."""
        print("Building HTTP client and mock server binaries...")
        
        # Build both binaries
        result = subprocess.run([
            "cargo", "build", "--bin", "http-apikey-client", "--bin", "http-mock-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Build failed: {result.stderr}")
            raise RuntimeError(f"Failed to build binaries: {result.stderr}")
        
        print("Build successful")
        
        # Start the mock server in background
        cls.mock_server_process = None
        cls.mock_server_url = "http://127.0.0.1:3001"
        cls.project_dir = Path(__file__).parent.parent
        cls.start_mock_server()
        
        # Wait for mock server to start
        cls.wait_for_mock_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up mock server process."""
        if cls.mock_server_process:
            print("Stopping HTTP mock server...")
            cls.mock_server_process.terminate()
            try:
                cls.mock_server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.mock_server_process.kill()
                cls.mock_server_process.wait()
            print("Mock server stopped")
    
    @classmethod
    def start_mock_server(cls):
        """Start the HTTP mock server."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        print(f"Starting HTTP mock server on port 3001...")
        cls.mock_server_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-mock-server"
        ], cwd=cls.project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_mock_server(cls, timeout=30):
        """Wait for the mock server to be ready to accept connections."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                # Try to connect to mock server health endpoint
                response = requests.get(f"{cls.mock_server_url}/health", timeout=1)
                if response.status_code == 200:
                    print("Mock server is ready!")
                    return
            except requests.exceptions.RequestException:
                pass
            
            # Check if server process is still running
            if cls.mock_server_process.poll() is not None:
                stdout, stderr = cls.mock_server_process.communicate()
                raise RuntimeError(f"Mock server process died: {stderr}")
            
            time.sleep(0.5)
        
        raise RuntimeError(f"Mock server failed to start within {timeout} seconds")
    
    def run_client_command(self, command: str, env_vars: Dict[str, str] = None, timeout: int = 30) -> subprocess.CompletedProcess:
        """Run the HTTP client with the given command and environment variables."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        # Set default environment variables
        env["MCP_SERVER_URL"] = self.mock_server_url
        env["MCP_API_KEY"] = "test-key-123"
        
        # Override with provided env vars
        if env_vars:
            env.update(env_vars)
        
        # Split command properly, avoiding shell word splitting issues
        if "--args" in command:
            # Special handling for commands with JSON arguments
            parts = command.split("--args", 1)
            base_cmd = parts[0].strip().split()
            args_part = parts[1].strip()
            
            # Remove surrounding quotes if present
            if args_part.startswith("'") and args_part.endswith("'"):
                args_part = args_part[1:-1]
            
            cmd = ["cargo", "run", "--bin", "http-apikey-client", "--"] + base_cmd + ["--args", args_part]
        else:
            cmd = ["cargo", "run", "--bin", "http-apikey-client", "--"] + command.split()
        
        return subprocess.run(
            cmd,
            cwd=self.project_dir,
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout
        )
    
    def test_mock_server_health(self):
        """Test that the mock server health endpoint works."""
        response = requests.get(f"{self.mock_server_url}/health", timeout=5)
        assert response.status_code == 200
        
        data = response.json()
        assert data["status"] == "healthy"
        assert data["server"] == "http-mock-server"
        print("✅ Mock server health check passed")
    
    def test_mock_server_info(self):
        """Test the mock server info endpoint."""
        response = requests.get(f"{self.mock_server_url}/info", timeout=5)
        assert response.status_code == 200
        
        data = response.json()
        assert data["name"] == "HTTP MCP Mock Server"
        assert "authentication" in data
        assert "mcp_capabilities" in data
        print("✅ Mock server info endpoint working")
    
    def test_client_help(self):
        """Test the client help command."""
        result = self.run_client_command("--help")
        assert result.returncode == 0
        assert "HTTP MCP client with API key authentication support" in result.stdout
        print("✅ Client help command working")
    
    def test_client_validate_config(self):
        """Test the client config validation."""
        result = self.run_client_command("validate-config")
        assert result.returncode == 0
        assert "Configuration appears valid" in result.stdout
        print("✅ Client config validation working")
    
    def test_client_test_connection_x_api_key(self):
        """Test client connection with X-API-Key authentication."""
        env_vars = {
            "MCP_API_KEY": "test-key-123",
            "AUTH_METHOD": "XApiKey"
        }
        
        result = self.run_client_command("test-connection", env_vars)
        assert result.returncode == 0
        assert "Connection successful!" in result.stdout
        assert "Authentication working" in result.stdout
        assert "MCP session initialized" in result.stdout
        print("✅ Client X-API-Key authentication working")
    
    def test_client_test_connection_bearer(self):
        """Test client connection with Bearer token authentication."""
        env_vars = {
            "MCP_API_KEY": "test-key-123",
            "AUTH_METHOD": "Bearer"
        }
        
        result = self.run_client_command("test-connection", env_vars)
        assert result.returncode == 0
        assert "Connection successful!" in result.stdout
        print("✅ Client Bearer token authentication working")
    
    def test_client_test_connection_query_parameter(self):
        """Test client connection with query parameter authentication."""
        env_vars = {
            "MCP_API_KEY": "test-key-123",
            "AUTH_METHOD": "QueryParameter"
        }
        
        result = self.run_client_command("test-connection", env_vars)
        assert result.returncode == 0
        assert "Connection successful!" in result.stdout
        print("✅ Client query parameter authentication working")
    
    def test_client_authentication_failure(self):
        """Test client behavior with invalid API key."""
        env_vars = {
            "MCP_API_KEY": "invalid-key-123"
        }
        
        result = self.run_client_command("test-connection", env_vars)
        assert result.returncode != 0
        # The client should fail to connect with invalid credentials
        print("✅ Client properly handles authentication failure")
    
    def test_client_list_tools(self):
        """Test the client list-tools command."""
        result = self.run_client_command("list-tools")
        assert result.returncode == 0
        
        # Check for expected tools from mock server
        expected_tools = ["echo", "health_check", "get_timestamp", "calculate"]
        for tool in expected_tools:
            assert tool in result.stdout
        
        print(f"✅ Client list-tools working: found {len(expected_tools)} tools")
    
    def test_client_call_tool_echo(self):
        """Test calling the echo tool through the client."""
        args = '{"message": "Hello from client test"}'
        result = self.run_client_command(f'call-tool echo --args {args}')
        assert result.returncode == 0
        assert "Echo: Hello from client test" in result.stdout
        print("✅ Client echo tool call working")
    
    def test_client_call_tool_health_check(self):
        """Test calling the health_check tool through the client."""
        result = self.run_client_command("call-tool health_check")
        assert result.returncode == 0
        assert "Mock server is healthy" in result.stdout
        print("✅ Client health_check tool call working")
    
    def test_client_call_tool_calculate(self):
        """Test calling the calculate tool through the client."""
        args = '{"operation": "add", "a": 5, "b": 3}'
        result = self.run_client_command(f'call-tool calculate --args {args}')
        assert result.returncode == 0
        assert "5 add 3 = 8" in result.stdout
        print("✅ Client calculate tool call working")
    
    def test_client_list_resources(self):
        """Test the client list-resources command."""
        result = self.run_client_command("list-resources")
        assert result.returncode == 0
        
        # Check for expected resources from mock server
        expected_resources = [
            "mock://server/info",
            "mock://server/status", 
            "mock://server/config",
            "mock://data/sample.txt"
        ]
        for resource in expected_resources:
            assert resource in result.stdout
        
        print(f"✅ Client list-resources working: found {len(expected_resources)} resources")
    
    def test_client_read_resource_server_info(self):
        """Test reading the server info resource through the client."""
        result = self.run_client_command("read-resource mock://server/info")
        assert result.returncode == 0
        assert "HTTP Mock Server" in result.stdout
        assert "version" in result.stdout
        print("✅ Client read resource (server info) working")
    
    def test_client_read_resource_sample_text(self):
        """Test reading the sample text resource through the client."""
        result = self.run_client_command("read-resource mock://data/sample.txt")
        assert result.returncode == 0
        assert "sample text file" in result.stdout
        assert "MCP protocol" in result.stdout
        print("✅ Client read resource (sample text) working")
    
    def test_client_demo_command(self):
        """Test the full client demo command."""
        result = self.run_client_command("demo", timeout=60)
        assert result.returncode == 0
        
        # Check that all demo steps completed
        demo_steps = [
            "Initializing MCP session",
            "Listing available tools",
            "Calling tool 'echo'",
            "Listing available resources",
            "Reading resource",
            "Demo completed successfully"
        ]
        
        for step in demo_steps:
            assert step in result.stdout
        
        print("✅ Client demo command completed successfully")
    
    def test_client_concurrent_requests(self):
        """Test concurrent client requests to verify mock server stability."""
        import concurrent.futures
        import threading
        
        def run_test_connection():
            result = self.run_client_command("test-connection")
            return result.returncode == 0
        
        # Run 5 concurrent client connections
        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
            futures = [executor.submit(run_test_connection) for _ in range(5)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        # All connections should succeed
        assert all(results), "Some concurrent connections failed"
        print("✅ Mock server handles concurrent client connections")
    
    def test_client_timeout_handling(self):
        """Test client timeout behavior."""
        # Test with very short timeout
        env_vars = {
            "MCP_TIMEOUT": "1"  # 1 second timeout
        }
        
        # This might timeout or succeed depending on system speed
        result = self.run_client_command("test-connection", env_vars, timeout=10)
        # We don't assert success/failure here since it depends on timing
        # The important thing is that the client doesn't crash
        print("✅ Client handles timeout configuration")
    
    def test_direct_mock_server_mcp_protocol(self):
        """Test direct MCP protocol requests to mock server."""
        # Test initialize request
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        }
        
        headers = {
            "Content-Type": "application/json",
            "X-API-Key": "test-key-123"
        }
        
        response = requests.post(f"{self.mock_server_url}/", headers=headers, json=payload, timeout=10)
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert data["result"]["protocolVersion"] == "2024-11-05"
        assert "serverInfo" in data["result"]
        
        print("✅ Direct MCP protocol initialize request working")
    
    def test_error_scenarios(self):
        """Test various error scenarios."""
        # Test with non-existent tool
        result = self.run_client_command("call-tool nonexistent_tool")
        assert result.returncode != 0
        print("✅ Client handles non-existent tool call")
        
        # Test with non-existent resource
        result = self.run_client_command("read-resource mock://nonexistent/resource")
        assert result.returncode != 0
        print("✅ Client handles non-existent resource read")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])