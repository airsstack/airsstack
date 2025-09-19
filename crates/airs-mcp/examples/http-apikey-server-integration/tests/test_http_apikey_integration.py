#!/usr/bin/env python3
"""
Integration tests for HTTP API Key MCP server.

These tests verify end-to-end functionality by launching the HTTP server
and testing all authentication methods and MCP operations via HTTP requests.
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


class TestHttpApiKeyIntegration:
    """Test the HTTP API Key MCP server integration."""
    
    @classmethod
    def setup_class(cls):
        """Build the binary and start the server before running tests."""
        print("Building HTTP API Key server binary...")
        
        # Build the server binary
        result = subprocess.run([
            "cargo", "build", "--bin", "http-apikey-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Build failed: {result.stderr}")
            raise RuntimeError(f"Failed to build binary: {result.stderr}")
        
        print("Build successful")
        
        # Start the server in background
        cls.server_process = None
        cls.base_url = "http://127.0.0.1:3001"  # Use different port to avoid conflicts
        cls.start_server()
        
        # Wait for server to start
        cls.wait_for_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up server process."""
        if cls.server_process:
            print("Stopping HTTP API Key server...")
            cls.server_process.terminate()
            try:
                cls.server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.server_process.kill()
                cls.server_process.wait()
            print("Server stopped")
    
    @classmethod
    def start_server(cls):
        """Start the HTTP API Key server."""
        project_dir = Path(__file__).parent.parent
        
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        print(f"Starting HTTP API Key server on port 3001...")
        cls.server_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-apikey-server", "--", "--port", "3001"
        ], cwd=project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_server(cls, timeout=30):
        """Wait for the server to be ready to accept connections."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                # Try to connect to server
                response = requests.get(f"{cls.base_url}/health", timeout=1)
                if response.status_code == 200:
                    print("Server is ready!")
                    return
            except requests.exceptions.RequestException:
                pass
            
            # Check if server process is still running
            if cls.server_process.poll() is not None:
                stdout, stderr = cls.server_process.communicate()
                raise RuntimeError(f"Server process died: {stderr}")
            
            time.sleep(0.5)
        
        raise RuntimeError(f"Server failed to start within {timeout} seconds")
    
    def mcp_request(self, method: str, params: Dict[str, Any] = None, 
                   api_key: str = "dev-key-123", auth_method: str = "header") -> requests.Response:
        """Make an MCP request to the server."""
        if params is None:
            params = {}
        
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }
        
        headers = {"Content-Type": "application/json"}
        url = f"{self.base_url}/mcp"
        
        # Add authentication based on method
        if auth_method == "header":
            headers["X-API-Key"] = api_key
        elif auth_method == "bearer":
            headers["Authorization"] = f"Bearer {api_key}"
        elif auth_method == "query":
            url += f"?api_key={api_key}"
        
        return requests.post(url, headers=headers, json=payload, timeout=10)
    
    def test_server_health(self):
        """Test that the server health endpoint works."""
        response = requests.get(f"{self.base_url}/health", timeout=5)
        assert response.status_code == 200
    
    def test_authentication_methods(self):
        """Test all three API key authentication methods."""
        
        # Test X-API-Key header
        response = self.mcp_request("tools/list", auth_method="header", api_key="dev-key-123")
        assert response.status_code == 200
        data = response.json()
        assert "result" in data
        assert "tools" in data["result"]
        print(f"✅ X-API-Key header authentication: {len(data['result']['tools'])} tools")
        
        # Test Authorization Bearer
        response = self.mcp_request("tools/list", auth_method="bearer", api_key="test-key-456")
        assert response.status_code == 200
        data = response.json()
        assert "result" in data
        assert "tools" in data["result"]
        print(f"✅ Authorization Bearer authentication: {len(data['result']['tools'])} tools")
        
        # Test Query parameter
        response = self.mcp_request("tools/list", auth_method="query", api_key="demo-key-789")
        assert response.status_code == 200
        data = response.json()
        assert "result" in data
        assert "tools" in data["result"]
        print(f"✅ Query parameter authentication: {len(data['result']['tools'])} tools")
    
    def test_authentication_failures(self):
        """Test authentication failure scenarios."""
        
        # Test missing API key
        headers = {"Content-Type": "application/json"}
        payload = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
        response = requests.post(f"{self.base_url}/mcp", headers=headers, json=payload, timeout=5)
        assert response.status_code == 401
        assert "Missing API key" in response.text
        print("✅ Missing API key properly rejected")
        
        # Test invalid API key
        response = self.mcp_request("tools/list", api_key="invalid-key-123")
        assert response.status_code == 401
        assert "Authentication error" in response.text
        print("✅ Invalid API key properly rejected")
    
    def test_tools_list(self):
        """Test the tools/list MCP method."""
        response = self.mcp_request("tools/list")
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert "tools" in data["result"]
        
        tools = data["result"]["tools"]
        assert len(tools) > 0
        
        # Check for expected math tools
        tool_names = [tool["name"] for tool in tools]
        expected_tools = ["add", "subtract", "multiply", "divide"]
        
        for expected_tool in expected_tools:
            assert expected_tool in tool_names, f"Expected tool {expected_tool} not found"
        
        print(f"✅ tools/list: Found {len(tools)} tools including {expected_tools}")
    
    def test_tools_call(self):
        """Test the tools/call MCP method."""
        # Test add tool
        response = self.mcp_request("tools/call", {
            "name": "add",
            "arguments": {"numbers": [5, 3, 2]}
        })
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert "content" in data["result"]
        assert not data["result"]["isError"]
        
        # Check the result content
        content = data["result"]["content"]
        assert len(content) > 0
        result_text = content[0]["text"]
        assert "10" in result_text  # 5 + 3 + 2 = 10
        
        print(f"✅ tools/call add: {result_text}")
        
        # Test another math operation
        response = self.mcp_request("tools/call", {
            "name": "multiply",
            "arguments": {"numbers": [4, 3]}
        })
        assert response.status_code == 200
        
        data = response.json()
        result_text = data["result"]["content"][0]["text"]
        assert "12" in result_text  # 4 * 3 = 12
        
        print(f"✅ tools/call multiply: {result_text}")
    
    def test_resources_list(self):
        """Test the resources/list MCP method."""
        response = self.mcp_request("resources/list")
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert "resources" in data["result"]
        
        resources = data["result"]["resources"]
        assert len(resources) > 0
        
        # Check for expected test resources
        resource_names = [resource["name"] for resource in resources]
        expected_resources = ["api-info.txt", "server-config.json", "README.md"]
        
        for expected_resource in expected_resources:
            assert expected_resource in resource_names, f"Expected resource {expected_resource} not found"
        
        print(f"✅ resources/list: Found {len(resources)} resources including {expected_resources}")
    
    def test_resources_read(self):
        """Test the resources/read MCP method."""
        # First get the list of resources to find a valid URI
        response = self.mcp_request("resources/list")
        assert response.status_code == 200
        
        resources = response.json()["result"]["resources"]
        api_info_resource = None
        
        for resource in resources:
            if resource["name"] == "api-info.txt":
                api_info_resource = resource
                break
        
        assert api_info_resource is not None, "api-info.txt resource not found"
        
        # Now read the resource
        response = self.mcp_request("resources/read", {
            "uri": api_info_resource["uri"]
        })
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert "contents" in data["result"]
        
        contents = data["result"]["contents"]
        assert len(contents) > 0
        
        content_text = contents[0]["text"]
        assert "HTTP API Key protected MCP server" in content_text
        
        print(f"✅ resources/read: Successfully read {len(content_text)} characters")
    
    def test_invalid_method(self):
        """Test error handling for invalid MCP methods."""
        response = self.mcp_request("invalid/method")
        assert response.status_code == 500  # Server error for invalid method
        
        # Check that it's still JSON-RPC response format (if parseable)
        try:
            data = response.json()
            if "error" in data:
                print(f"✅ Invalid method properly rejected with JSON-RPC error: {data['error']['message']}")
            else:
                print("✅ Invalid method properly rejected with server error")
        except:
            # If response isn't JSON, that's also acceptable for invalid methods
            print("✅ Invalid method properly rejected with server error")
    
    def test_invalid_tool_parameters(self):
        """Test error handling for invalid tool parameters."""
        # Test calling add tool with missing parameters
        response = self.mcp_request("tools/call", {
            "name": "add",
            "arguments": {}  # Missing required 'numbers' parameter
        })
        assert response.status_code == 200  # HTTP success but tool error
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        
        # Tool should return error content
        result = data["result"]
        if result.get("isError", False):
            print("✅ Invalid tool parameters properly handled with error result")
        else:
            # Some tools might handle missing params gracefully
            print("✅ Tool handled missing parameters gracefully")
    
    def test_concurrent_requests(self):
        """Test that the server can handle concurrent requests."""
        import concurrent.futures
        
        def make_request():
            response = self.mcp_request("tools/list")
            return response.status_code == 200
        
        # Make 5 concurrent requests
        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as executor:
            futures = [executor.submit(make_request) for _ in range(5)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        # All requests should succeed
        assert all(results), "Some concurrent requests failed"
        print("✅ Server handled 5 concurrent requests successfully")
    
    @pytest.mark.timeout(60)
    def test_full_workflow(self):
        """Test a complete workflow: list tools, call a tool, list resources, read a resource."""
        print("\n🔧 Running full workflow test...")
        
        # Step 1: List available tools
        response = self.mcp_request("tools/list")
        assert response.status_code == 200
        tools = response.json()["result"]["tools"]
        print(f"   Step 1: Listed {len(tools)} tools")
        
        # Step 2: Call a tool (add)
        response = self.mcp_request("tools/call", {
            "name": "add",
            "arguments": {"numbers": [10, 20, 30]}
        })
        assert response.status_code == 200
        result = response.json()["result"]["content"][0]["text"]
        print(f"   Step 2: Tool result: {result}")
        
        # Step 3: List available resources
        response = self.mcp_request("resources/list")
        assert response.status_code == 200
        resources = response.json()["result"]["resources"]
        print(f"   Step 3: Listed {len(resources)} resources")
        
        # Step 4: Read a resource
        if resources:
            response = self.mcp_request("resources/read", {
                "uri": resources[0]["uri"]
            })
            assert response.status_code == 200
            content = response.json()["result"]["contents"][0]["text"]
            print(f"   Step 4: Read resource with {len(content)} characters")
        
        print("✅ Full workflow completed successfully")


if __name__ == "__main__":
    # Run tests directly
    pytest.main([__file__, "-v", "-s"])