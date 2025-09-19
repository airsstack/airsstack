#!/usr/bin/env python3
"""
Simple script to debug what headers the AIRS MCP client sends.
"""

import subprocess
import time
import requests
import os
import json
from pathlib import Path

def get_real_jwt_token(oauth2_base_url):
    """Get a real JWT token from the OAuth2 server."""
    print("🎫 Getting real JWT token from OAuth2 server...")
    
    # Step 1: Exchange demo auth code for token
    token_data = {
        "grant_type": "authorization_code",
        "code": "demo_auth_code_automatic",  # Pre-stored demo code
        "client_id": "test-client",
        "redirect_uri": "http://localhost:8082/callback",  # Must match what client uses
        "code_verifier": "dummy_verifier_for_demo"  # Demo verifier
    }
    
    try:
        response = requests.post(f"{oauth2_base_url}/token", data=token_data, timeout=5)
        print(f"Token response status: {response.status_code}")
        print(f"Token response body: {response.text}")
        
        if response.status_code == 200:
            token_response = response.json()
            access_token = token_response.get("access_token")
            if access_token:
                print(f"✅ Got real JWT token: {access_token[:50]}...")
                return access_token
            else:
                print(f"❌ No access_token in response: {token_response}")
                return None
        else:
            print(f"❌ Failed to get token: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print(f"❌ Error getting token: {e}")
        return None

def main():
    project_dir = Path(__file__).parent
    oauth2_base_url = "http://127.0.0.1:3002"
    mcp_base_url = "http://127.0.0.1:3003"
    
    print("🚀 Starting OAuth2 server...")
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    # Start OAuth2 server
    oauth2_process = subprocess.Popen([
        "cargo", "run", "--bin", "http-oauth2-mock-server", "--",
        "--host", "127.0.0.1", "--port", "3002"
    ], cwd=project_dir, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    
    # Start MCP server in a way where we can see its output
    print("🚀 Starting MCP server with debug output...")
    env["RUST_LOG"] = "debug"
    
    mcp_process = subprocess.Popen([
        "cargo", "run", "--bin", "http-mcp-mock-server", "--",
        "--host", "127.0.0.1", "--port", "3003",
        "--jwks-url", f"{oauth2_base_url}/jwks"
    ], cwd=project_dir, env=env)
    
    try:
        # Wait for servers to start
        print("⏳ Waiting for servers to start...")
        time.sleep(5)
        
        # Check if OAuth2 server is ready
        try:
            health_response = requests.get(f"{oauth2_base_url}/health", timeout=2)
            if health_response.status_code == 200:
                print("✅ OAuth2 server is ready!")
            else:
                print(f"⚠️  OAuth2 server health check returned: {health_response.status_code}")
        except Exception as e:
            print(f"❌ OAuth2 server not responding: {e}")
            return
        
        # Get a real JWT token
        real_token = get_real_jwt_token(oauth2_base_url)
        if not real_token:
            print("❌ Failed to get real token, cannot proceed")
            return
        
        # Test with the real JWT token
        print(f"\n📡 Testing MCP server with real JWT token...")
        result = subprocess.run([
            "curl", "-X", "POST", f"{mcp_base_url}/",
            "-H", "Content-Type: application/json",
            "-H", f"Authorization: Bearer {real_token}",
            "-d", '{"jsonrpc": "2.0", "method": "initialize", "id": 1}'
        ], capture_output=True, text=True)
        
        print(f"Curl exit code: {result.returncode}")
        print(f"Curl stdout: {result.stdout}")
        print(f"Curl stderr: {result.stderr}")
        
        print("\n" + "="*50)
        print("Check the MCP server output above for debug info!")
        print("="*50)
        
        # Keep the server running for a bit so we can see the output
        time.sleep(2)
        
    finally:
        print("\n🛑 Stopping servers...")
        oauth2_process.terminate()
        mcp_process.terminate()
        oauth2_process.wait()
        mcp_process.wait()

if __name__ == "__main__":
    main()