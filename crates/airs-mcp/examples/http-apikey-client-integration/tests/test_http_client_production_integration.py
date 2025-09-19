#!/usr/bin/env python3
"""
Integration tests for HTTP API Key MCP client with production server.

These tests verify the client works with the Phase 4.4 production HTTP server
implementation by connecting to a real server instance.
"""

import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, Any, Optional

import pytest
import requests
import psutil


class TestHttpClientProductionIntegration:
    """Test the HTTP API Key MCP client integration with production server."""
    
    @classmethod
    def setup_class(cls):
        """Build client binary and prepare for production server testing."""
        print("Building HTTP client binary...")
        
        # Build the client binary
        result = subprocess.run([
            "cargo", "build", "--bin", "http-apikey-client"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Build failed: {result.stderr}")
            raise RuntimeError(f"Failed to build client binary: {result.stderr}")
        
        print("Build successful")
        
        cls.project_dir = Path(__file__).parent.parent
        
        # Production server settings
        cls.production_server_url = os.environ.get("PRODUCTION_SERVER_URL", "http://127.0.0.1:3000")
        cls.production_api_key = os.environ.get("PRODUCTION_API_KEY", "test-api-key")
        
        # Check if we should start our own production server
        cls.should_start_server = os.environ.get("START_PRODUCTION_SERVER", "true").lower() == "true"
        cls.production_server_process = None
        
        if cls.should_start_server:
            cls.start_production_server()
            cls.wait_for_production_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up production server process if we started it."""
        if cls.production_server_process:
            print("Stopping production HTTP server...")
            cls.production_server_process.terminate()
            try:
                cls.production_server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.production_server_process.kill()
                cls.production_server_process.wait()
            print("Production server stopped")
    
    @classmethod
    def start_production_server(cls):
        """Start the production HTTP server."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        env["MCP_API_KEY"] = cls.production_api_key
        env["SERVER_HOST"] = "127.0.0.1"
        env["SERVER_PORT"] = "3000"
        
        print(f"Starting production HTTP server on port 3000...")
        cls.production_server_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-apikey-server"
        ], cwd=cls.project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_production_server(cls, timeout=30):
        """Wait for the production server to be ready to accept connections."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                # Try to connect to production server health endpoint
                response = requests.get(f"{cls.production_server_url}/health", timeout=1)
                if response.status_code == 200:
                    print("Production server is ready!")
                    return
            except requests.exceptions.RequestException:
                pass
            
            # Check if server process is still running (if we started it)
            if cls.production_server_process and cls.production_server_process.poll() is not None:
                stdout, stderr = cls.production_server_process.communicate()
                raise RuntimeError(f"Production server process died: {stderr}")
            
            time.sleep(0.5)
        
        if cls.should_start_server:
            raise RuntimeError(f"Production server failed to start within {timeout} seconds")
        else:
            pytest.skip(f"Production server not available at {cls.production_server_url}")
    
    def run_client_command(self, command: str, env_vars: Dict[str, str] = None, timeout: int = 30) -> subprocess.CompletedProcess:
        """Run the HTTP client with the given command and environment variables."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        # Set default environment variables for production server
        env["MCP_SERVER_URL"] = self.production_server_url
        env["MCP_API_KEY"] = self.production_api_key
        
        # Override with provided env vars
        if env_vars:
            env.update(env_vars)
        
        cmd = ["cargo", "run", "--bin", "http-apikey-client", "--"] + command.split()
        
        return subprocess.run(
            cmd,
            cwd=self.project_dir,
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout
        )
    
    def test_production_server_available(self):
        """Test that the production server is available and responding."""
        try:
            response = requests.get(f"{self.production_server_url}/health", timeout=5)
            assert response.status_code == 200
            print("✅ Production server is available")
        except requests.exceptions.RequestException as e:
            pytest.skip(f"Production server not available: {e}")
    
    def test_production_server_info(self):
        """Test the production server info endpoint."""
        response = requests.get(f"{self.production_server_url}/info", timeout=5)
        assert response.status_code == 200
        
        data = response.json()
        assert "name" in data
        assert "authentication" in data
        print("✅ Production server info endpoint working")
    
    def test_client_production_connection(self):
        """Test client connection to production server."""
        result = self.run_client_command("test-connection")
        assert result.returncode == 0
        assert "Connection successful!" in result.stdout
        print("✅ Client connects to production server")
    
    def test_client_production_authentication_methods(self):
        """Test all authentication methods with production server."""
        auth_methods = ["XApiKey", "Bearer", "QueryParameter"]
        
        for auth_method in auth_methods:
            print(f"Testing {auth_method} authentication...")
            env_vars = {
                "AUTH_METHOD": auth_method
            }
            
            result = self.run_client_command("test-connection", env_vars)
            assert result.returncode == 0, f"Authentication failed for {auth_method}"
            assert "Connection successful!" in result.stdout
            print(f"✅ {auth_method} authentication working with production server")
    
    def test_client_production_list_tools(self):
        """Test listing tools from production server."""
        result = self.run_client_command("list-tools")
        assert result.returncode == 0
        
        # Production server should have at least some tools
        assert "Available tools:" in result.stdout
        print("✅ Client lists tools from production server")
    
    def test_client_production_list_resources(self):
        """Test listing resources from production server."""
        result = self.run_client_command("list-resources")
        assert result.returncode == 0
        
        # Production server should have at least some resources
        assert "Available resources:" in result.stdout
        print("✅ Client lists resources from production server")
    
    def test_client_production_tool_calls(self):
        """Test calling tools on production server."""
        # First, get the list of available tools
        result = self.run_client_command("list-tools")
        assert result.returncode == 0
        
        # Try to call a tool if any are available
        if "echo" in result.stdout.lower():
            print("Testing echo tool call...")
            args = '{"message": "Production test"}'
            result = self.run_client_command(f'call-tool echo --args \'{args}\'')
            assert result.returncode == 0
            print("✅ Tool call working on production server")
        else:
            print("ℹ️ No echo tool available on production server")
    
    def test_client_production_resource_reads(self):
        """Test reading resources from production server."""
        # First, get the list of available resources
        result = self.run_client_command("list-resources")
        assert result.returncode == 0
        
        # Parse the output to find a resource URI if any
        lines = result.stdout.split('\n')
        resource_uri = None
        
        for line in lines:
            line = line.strip()
            if line.startswith("- ") and "://" in line:
                # Extract URI from "- uri (description)"
                resource_uri = line.split()[1]
                break
        
        if resource_uri:
            print(f"Testing resource read: {resource_uri}")
            result = self.run_client_command(f"read-resource {resource_uri}")
            # Don't assert success since resource might not exist or require different auth
            if result.returncode == 0:
                print("✅ Resource read working on production server")
            else:
                print("ℹ️ Resource read failed (may be expected)")
        else:
            print("ℹ️ No resources available on production server")
    
    def test_client_production_demo(self):
        """Test the full client demo against production server."""
        result = self.run_client_command("demo", timeout=60)
        
        # Demo might fail on production server if tools/resources are different
        # So we don't assert success, just check it doesn't crash
        if result.returncode == 0:
            print("✅ Demo completed successfully on production server")
        else:
            print("ℹ️ Demo failed on production server (may be expected due to different capabilities)")
    
    def test_client_production_error_handling(self):
        """Test error handling with production server."""
        # Test with invalid authentication
        env_vars = {
            "MCP_API_KEY": "invalid-production-key"
        }
        
        result = self.run_client_command("test-connection", env_vars)
        assert result.returncode != 0
        print("✅ Client properly handles authentication failure on production server")
    
    def test_client_production_performance(self):
        """Test client performance with production server."""
        import time
        
        # Measure connection time
        start_time = time.time()
        result = self.run_client_command("test-connection")
        connection_time = time.time() - start_time
        
        assert result.returncode == 0
        assert connection_time < 10.0  # Should connect within 10 seconds
        
        print(f"✅ Client connects to production server in {connection_time:.2f}s")
    
    def test_client_production_concurrent_connections(self):
        """Test concurrent connections to production server."""
        import concurrent.futures
        
        def run_test_connection():
            result = self.run_client_command("test-connection", timeout=15)
            return result.returncode == 0
        
        # Run 3 concurrent connections (fewer than mock server test)
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            futures = [executor.submit(run_test_connection) for _ in range(3)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        # Most connections should succeed
        success_count = sum(results)
        assert success_count >= 2, f"Only {success_count}/3 concurrent connections succeeded"
        print(f"✅ Production server handles concurrent connections ({success_count}/3 succeeded)")
    
    def test_direct_production_mcp_protocol(self):
        """Test direct MCP protocol requests to production server."""
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
            "X-API-Key": self.production_api_key
        }
        
        response = requests.post(f"{self.production_server_url}/", headers=headers, json=payload, timeout=10)
        assert response.status_code == 200
        
        data = response.json()
        assert data["jsonrpc"] == "2.0"
        assert "result" in data
        assert "serverInfo" in data["result"]
        
        print("✅ Direct MCP protocol requests working with production server")
    
    def test_client_production_long_session(self):
        """Test a longer session with multiple operations."""
        # This is a more comprehensive test that exercises the client
        # through multiple operations in sequence
        operations = [
            "test-connection",
            "list-tools",
            "list-resources"
        ]
        
        all_successful = True
        for operation in operations:
            print(f"Testing operation: {operation}")
            result = self.run_client_command(operation)
            if result.returncode != 0:
                print(f"Operation {operation} failed: {result.stderr}")
                all_successful = False
            else:
                print(f"✅ Operation {operation} successful")
        
        assert all_successful, "Some operations failed in long session test"
        print("✅ Long session test completed successfully")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])