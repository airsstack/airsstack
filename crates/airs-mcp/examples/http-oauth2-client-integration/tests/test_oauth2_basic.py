#!/usr/bin/env python3
"""
Basic OAuth2 flow validation tests.

Tests fundamental OAuth2 operations without complex integration scenarios.
"""

import json
import os
import subprocess
import time
from pathlib import Path
from typing import Dict, Any

import pytest
import requests
import psutil


class TestOAuth2Basic:
    """Basic OAuth2 flow validation tests."""
    
    @classmethod
    def setup_class(cls):
        """Set up test environment and start OAuth2 server."""
        cls.project_dir = Path(__file__).parent.parent
        cls.oauth2_port = 3002
        cls.oauth2_base_url = f"http://127.0.0.1:{cls.oauth2_port}"
        cls.oauth2_process = None
        
        # Kill any existing processes on our port
        cls.kill_processes_on_port(cls.oauth2_port)
        
        # Build and start OAuth2 server
        cls.build_oauth2_server()
        cls.start_oauth2_server()
        cls.wait_for_oauth2_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up OAuth2 server process."""
        if cls.oauth2_process:
            print("Stopping OAuth2 server...")
            cls.oauth2_process.terminate()
            try:
                cls.oauth2_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.oauth2_process.kill()
                cls.oauth2_process.wait()
            print("OAuth2 server stopped")
    
    @classmethod
    def kill_processes_on_port(cls, port: int):
        """Kill any processes running on the specified port."""
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
    def build_oauth2_server(cls):
        """Build the OAuth2 mock server binary."""
        print("Building OAuth2 mock server...")
        result = subprocess.run([
            "cargo", "build", "--bin", "http-oauth2-mock-server"
        ], cwd=cls.project_dir, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to build OAuth2 server: {result.stderr}")
        print("OAuth2 server build successful")
    
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
    def wait_for_oauth2_server(cls, timeout=30):
        """Wait for the OAuth2 server to be ready."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                response = requests.get(f"{cls.oauth2_base_url}/health", timeout=2)
                if response.status_code == 200:
                    print("OAuth2 server is ready!")
                    return
            except requests.exceptions.RequestException:
                pass
            
            # Check if server process is still running
            if cls.oauth2_process.poll() is not None:
                stdout, stderr = cls.oauth2_process.communicate()
                raise RuntimeError(f"OAuth2 server process died: {stderr}")
            
            time.sleep(0.5)
        
        raise RuntimeError(f"OAuth2 server failed to start within {timeout} seconds")
    
    def test_oauth2_server_health(self):
        """Test that the OAuth2 server health endpoint works."""
        response = requests.get(f"{self.oauth2_base_url}/health", timeout=5)
        assert response.status_code == 200
        print("✅ OAuth2 server health check passed")
    
    def test_oauth2_oidc_discovery(self):
        """Test OAuth2 OpenID Connect discovery endpoint."""
        response = requests.get(
            f"{self.oauth2_base_url}/.well-known/openid-configuration", 
            timeout=5
        )
        assert response.status_code == 200
        
        data = response.json()
        assert "issuer" in data
        assert "authorization_endpoint" in data
        assert "token_endpoint" in data
        assert "jwks_uri" in data
        print("✅ OIDC discovery endpoint working")
        print(f"   Issuer: {data['issuer']}")
        print(f"   Authorization endpoint: {data['authorization_endpoint']}")
        print(f"   Token endpoint: {data['token_endpoint']}")
    
    def test_oauth2_jwks_endpoint(self):
        """Test OAuth2 JWKS endpoint."""
        response = requests.get(f"{self.oauth2_base_url}/jwks", timeout=5)
        assert response.status_code == 200
        
        data = response.json()
        assert "keys" in data
        assert len(data["keys"]) > 0
        
        # Validate JWKS structure
        key = data["keys"][0]
        assert "kty" in key  # Key type
        assert "use" in key  # Public key use
        assert "n" in key    # RSA modulus
        assert "e" in key    # RSA exponent
        print("✅ JWKS endpoint working")
        print(f"   Found {len(data['keys'])} key(s)")
    
    def test_oauth2_authorization_endpoint_basic(self):
        """Test OAuth2 authorization endpoint with basic parameters."""
        auth_url = f"{self.oauth2_base_url}/authorize"
        params = {
            'response_type': 'code',
            'client_id': 'test-client',
            'redirect_uri': 'http://localhost:8082/callback',
            'scope': 'mcp:read mcp:write',
            'code_challenge': 'test-challenge-' + 'x' * 32,  # Valid length
            'code_challenge_method': 'S256',
            'state': 'test-state-12345'
        }
        
        response = requests.get(auth_url, params=params, timeout=10)
        assert response.status_code == 200
        assert 'Authorization Successful' in response.text
        assert 'test-client' in response.text
        print("✅ OAuth2 authorization endpoint working")
        print("   Authorization page returned successfully")
    
    def test_oauth2_authorization_invalid_client(self):
        """Test OAuth2 authorization endpoint with invalid client."""
        auth_url = f"{self.oauth2_base_url}/authorize"
        params = {
            'response_type': 'code',
            'client_id': 'invalid-client',
            'redirect_uri': 'http://localhost:8082/callback',
            'scope': 'mcp:read mcp:write',
            'code_challenge': 'test-challenge-' + 'x' * 32,
            'code_challenge_method': 'S256',
            'state': 'test-state-12345'
        }
        
        response = requests.get(auth_url, params=params, timeout=10)
        assert response.status_code == 400
        
        error_data = response.json()
        assert "error" in error_data
        assert error_data["error"] == "invalid_client"
        print("✅ OAuth2 invalid client handling working")
    
    def test_oauth2_authorization_invalid_redirect_uri(self):
        """Test OAuth2 authorization endpoint with invalid redirect URI."""
        auth_url = f"{self.oauth2_base_url}/authorize"
        params = {
            'response_type': 'code',
            'client_id': 'test-client',
            'redirect_uri': 'http://evil.com/callback',  # Not registered
            'scope': 'mcp:read mcp:write',
            'code_challenge': 'test-challenge-' + 'x' * 32,
            'code_challenge_method': 'S256',
            'state': 'test-state-12345'
        }
        
        response = requests.get(auth_url, params=params, timeout=10)
        assert response.status_code == 400
        
        error_data = response.json()
        assert "error" in error_data
        assert error_data["error"] == "invalid_client"
        assert "redirect_uri" in error_data["error_description"]
        print("✅ OAuth2 invalid redirect URI handling working")
    
    def test_oauth2_authorization_unsupported_response_type(self):
        """Test OAuth2 authorization endpoint with unsupported response type."""
        auth_url = f"{self.oauth2_base_url}/authorize"
        params = {
            'response_type': 'token',  # Only 'code' is supported
            'client_id': 'test-client',
            'redirect_uri': 'http://localhost:8082/callback',
            'scope': 'mcp:read mcp:write',
            'state': 'test-state-12345'
        }
        
        response = requests.get(auth_url, params=params, timeout=10)
        assert response.status_code == 400
        
        error_data = response.json()
        assert "error" in error_data
        assert error_data["error"] == "unsupported_response_type"
        print("✅ OAuth2 unsupported response type handling working")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])