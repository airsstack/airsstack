#!/usr/bin/env python3
"""
OAuth2 integration tests.

Tests complete OAuth2 + MCP integration scenarios.
"""

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Dict, Any
import asyncio

import pytest
import requests
import psutil


class TestOAuth2Integration:
    """Complete OAuth2 + MCP integration tests."""
    
    @classmethod
    def setup_class(cls):
        """Set up test environment with OAuth2 and MCP servers."""
        cls.project_dir = Path(__file__).parent.parent
        cls.oauth2_port = 3002
        cls.mcp_port = 3003
        cls.oauth2_base_url = f"http://127.0.0.1:{cls.oauth2_port}"
        cls.mcp_base_url = f"http://127.0.0.1:{cls.mcp_port}"
        cls.oauth2_process = None
        cls.mcp_process = None
        
        # Kill any existing processes on our ports
        cls.kill_processes_on_ports([cls.oauth2_port, cls.mcp_port])
        
        # Build and start both servers
        cls.build_servers()
        cls.start_oauth2_server()
        cls.start_mcp_server()
        cls.wait_for_servers()
    
    @classmethod
    def teardown_class(cls):
        """Clean up server processes."""
        if cls.oauth2_process:
            print("Stopping OAuth2 server...")
            cls.oauth2_process.terminate()
            try:
                cls.oauth2_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.oauth2_process.kill()
                cls.oauth2_process.wait()
            print("OAuth2 server stopped")
            
        if cls.mcp_process:
            print("Stopping MCP server...")
            cls.mcp_process.terminate()
            try:
                cls.mcp_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.mcp_process.kill()
                cls.mcp_process.wait()
            print("MCP server stopped")
    
    @classmethod
    def kill_processes_on_ports(cls, ports: list):
        """Kill any processes running on the specified ports."""
        for port in ports:
            try:
                result = subprocess.run(['lsof', '-ti', f':{port}'], 
                                      capture_output=True, text=True)
                if result.stdout.strip():
                    pids = result.stdout.strip().split('\n')
                    for pid in pids:
                        subprocess.run(['kill', '-9', pid], capture_output=True)
                    print(f"Killed existing processes on port {port}")
            except Exception:
                pass  # No processes to kill
    
    @classmethod
    def build_servers(cls):
        """Build all required server binaries."""
        print("Building OAuth2 and MCP servers...")
        
        # Build OAuth2 server
        result = subprocess.run([
            "cargo", "build", "--bin", "http-oauth2-mock-server"
        ], cwd=cls.project_dir, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to build OAuth2 server: {result.stderr}")
        
        # Build MCP server
        result = subprocess.run([
            "cargo", "build", "--bin", "http-mcp-mock-server"
        ], cwd=cls.project_dir, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to build MCP server: {result.stderr}")
        
        print("Server builds successful")
    
    @classmethod
    def start_oauth2_server(cls):
        """Start the OAuth2 mock server."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,http_oauth2_client_integration=debug"
        
        print(f"Starting OAuth2 server on port {cls.oauth2_port}...")
        cls.oauth2_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-oauth2-mock-server", "--",
            "--host", "127.0.0.1", "--port", str(cls.oauth2_port)
        ], cwd=cls.project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def start_mcp_server(cls):
        """Start the MCP mock server."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,http_oauth2_client_integration=debug"
        
        print(f"Starting MCP server on port {cls.mcp_port}...")
        cls.mcp_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-mcp-mock-server", "--",
            "--host", "127.0.0.1", "--port", str(cls.mcp_port),
            "--jwks-url", f"{cls.oauth2_base_url}/jwks"
        ], cwd=cls.project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_servers(cls, timeout=30):
        """Wait for both servers to be ready."""
        start_time = time.time()
        oauth2_ready = False
        mcp_ready = False
        
        while time.time() - start_time < timeout:
            # Check OAuth2 server
            if not oauth2_ready:
                try:
                    response = requests.get(f"{cls.oauth2_base_url}/health", timeout=2)
                    if response.status_code == 200:
                        print("OAuth2 server is ready!")
                        oauth2_ready = True
                except requests.exceptions.RequestException:
                    pass
            
            # Check MCP server
            if not mcp_ready:
                try:
                    response = requests.get(f"{cls.mcp_base_url}/health", timeout=2)
                    if response.status_code == 200:
                        print("MCP server is ready!")
                        mcp_ready = True
                except requests.exceptions.RequestException:
                    pass
            
            if oauth2_ready and mcp_ready:
                return
            
            # Check if processes are still running
            if cls.oauth2_process.poll() is not None:
                stdout, stderr = cls.oauth2_process.communicate()
                raise RuntimeError(f"OAuth2 server process died: {stderr}")
            
            if cls.mcp_process.poll() is not None:
                stdout, stderr = cls.mcp_process.communicate()
                raise RuntimeError(f"MCP server process died: {stderr}")
            
            time.sleep(0.5)
        
        if not oauth2_ready:
            raise RuntimeError("OAuth2 server failed to start")
        if not mcp_ready:
            raise RuntimeError("MCP server failed to start")
    
    def test_servers_health(self):
        """Test that both servers are healthy."""
        # Test OAuth2 server health
        response = requests.get(f"{self.oauth2_base_url}/health", timeout=5)
        assert response.status_code == 200
        print("✅ OAuth2 server health check passed")
        
        # Test MCP server health
        response = requests.get(f"{self.mcp_base_url}/health", timeout=5)
        assert response.status_code == 200
        print("✅ MCP server health check passed")
    
    def test_mcp_server_configuration(self):
        """Test that MCP server is properly configured for OAuth2 with JSON-RPC."""
        # The MCP server should reject JSON-RPC requests without valid OAuth2 tokens
        mcp_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }
        
        # Request without authentication should fail (proper JSON-RPC to root endpoint)
        response = requests.post(
            f"{self.mcp_base_url}/",
            json=mcp_request,
            headers={"Content-Type": "application/json"},
            timeout=5
        )
        assert response.status_code == 401
        print("✅ MCP server properly rejects unauthenticated JSON-RPC requests")
    
    @pytest.mark.asyncio
    async def test_oauth2_client_integration(self):
        """Test the complete OAuth2 client integration."""
        print("🔌 Running OAuth2 client integration test...")
        
        command = [
            'cargo', 'run', '--bin', 'http-oauth2-client', '--',
            '--auth-server', self.oauth2_base_url,
            '--mcp-server', self.mcp_base_url
        ]
        
        try:
            process = await asyncio.create_subprocess_exec(
                *command,
                cwd=self.project_dir,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            # Wait for completion with timeout
            try:
                stdout, stderr = await asyncio.wait_for(
                    process.communicate(), timeout=60
                )
            except asyncio.TimeoutError:
                print("❌ OAuth2 client timed out")
                process.kill()
                await process.wait()
                assert False, "OAuth2 client timed out"
            
            # Decode bytes to string
            stdout_text = stdout.decode('utf-8') if stdout else ""
            stderr_text = stderr.decode('utf-8') if stderr else ""
            
            # Analyze the output
            print("📊 Client output analysis:")
            print(f"Exit code: {process.returncode}")
            
            if stdout_text:
                print("STDOUT:")
                for line in stdout_text.split('\n'):
                    if line.strip():
                        print(f"  {line}")
            
            if stderr_text:
                print("STDERR:")
                for line in stderr_text.split('\n'):
                    if line.strip():
                        print(f"  {line}")
            
            # Check for success indicators
            combined_output = f"{stdout_text}\n{stderr_text}"
            
            success_indicators = [
                "Authorization URL:",
                "Authorization code generated",
                "Token exchange successful",
                "MCP operations completed"
            ]
            
            error_indicators = [
                "AuthorizationFailed",
                "400 Bad Request",
                "Configuration error",
                "Connection refused"
            ]
            
            # Check for errors first
            for error in error_indicators:
                if error in combined_output:
                    print(f"❌ Found error indicator: {error}")
                    assert False, f"OAuth2 client failed with error: {error}"
            
            # Check for success indicators
            success_count = 0
            for success in success_indicators:
                if success in combined_output:
                    print(f"✅ Found success indicator: {success}")
                    success_count += 1
            
            # We expect at least authorization URL generation to work
            assert success_count >= 1, f"Expected success indicators, found {success_count}"
            print(f"✅ OAuth2 client integration test passed with {success_count} success indicators")
            
        except Exception as e:
            print(f"❌ OAuth2 client integration test failed: {e}")
            raise


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])