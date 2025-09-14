#!/usr/bin/env python3
"""
Simple OAuth2 Flow Debug Script

This script helps debug the OAuth2 authorization flow step by step.
"""

import base64
import hashlib
import json
import secrets
import subprocess
import time
import urllib.parse
import requests

def generate_pkce_challenge():
    """Generate PKCE code verifier and challenge"""
    verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode('utf-8')).digest()
    ).decode('utf-8').rstrip('=')
    return verifier, challenge

def test_discovery():
    """Test OAuth2 discovery endpoint"""
    print("🔍 Testing OAuth2 Discovery...")
    try:
        response = requests.get("http://localhost:3003/.well-known/oauth-authorization-server", timeout=5)
        print(f"Discovery status: {response.status_code}")
        if response.status_code == 200:
            data = response.json()
            print(f"Discovery data: {json.dumps(data, indent=2)}")
            return data
        else:
            print(f"Discovery failed: {response.text}")
            return None
    except Exception as e:
        print(f"Discovery error: {e}")
        return None

def test_authorize(discovery_data):
    """Test OAuth2 authorize endpoint"""
    print("\n🔐 Testing OAuth2 Authorization...")
    
    if not discovery_data:
        print("No discovery data available")
        return None
    
    verifier, challenge = generate_pkce_challenge()
    print(f"PKCE verifier: {verifier[:20]}...")
    print(f"PKCE challenge: {challenge}")
    
    authorize_url = discovery_data.get("authorization_endpoint", "http://localhost:3003/authorize")
    
    params = {
        "response_type": "code",
        "client_id": "test-mcp-client",
        "redirect_uri": "http://localhost:8080/callback",
        "scope": "mcp:resources:list mcp:tools:list",
        "state": "test-state-123",
        "code_challenge": challenge,
        "code_challenge_method": "S256"
    }
    
    try:
        print(f"Making request to: {authorize_url}")
        print(f"Parameters: {params}")
        
        response = requests.get(authorize_url, params=params, allow_redirects=False, timeout=10)
        print(f"Authorization status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        
        if response.status_code in [302, 303]:
            location = response.headers.get("Location", "")
            print(f"Redirect location: {location}")
            
            # Parse authorization code from redirect
            if "code=" in location:
                parsed = urllib.parse.urlparse(location)
                query_params = urllib.parse.parse_qs(parsed.query)
                auth_code = query_params.get("code", [None])[0]
                print(f"Authorization code: {auth_code}")
                return auth_code, verifier
            else:
                print("No authorization code in redirect")
                return None, None
        else:
            print(f"Unexpected status: {response.text[:500]}")
            return None, None
    except Exception as e:
        print(f"Authorization error: {e}")
        return None, None

def test_token_exchange(discovery_data, auth_code, verifier):
    """Test OAuth2 token exchange"""
    print("\n🎫 Testing OAuth2 Token Exchange...")
    
    if not auth_code or not verifier:
        print("No authorization code or verifier available")
        return None
    
    token_url = discovery_data.get("token_endpoint", "http://localhost:3003/token")
    
    data = {
        "grant_type": "authorization_code",
        "code": auth_code,
        "redirect_uri": "http://localhost:8080/callback",
        "client_id": "test-mcp-client",
        "code_verifier": verifier
    }
    
    headers = {
        "Content-Type": "application/x-www-form-urlencoded"
    }
    
    try:
        print(f"Making token request to: {token_url}")
        print(f"Data: {data}")
        
        response = requests.post(token_url, data=data, headers=headers, timeout=10)
        print(f"Token status: {response.status_code}")
        print(f"Response headers: {dict(response.headers)}")
        
        if response.status_code == 200:
            token_data = response.json()
            print(f"Token response: {json.dumps(token_data, indent=2)}")
            return token_data.get("access_token")
        else:
            print(f"Token error: {response.text}")
            return None
    except Exception as e:
        print(f"Token exchange error: {e}")
        return None

def test_mcp_with_token(access_token):
    """Test MCP API with OAuth2 token"""
    print("\n🔧 Testing MCP API...")
    
    if not access_token:
        print("No access token available")
        return
    
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {access_token}"
    }
    
    payload = {
        "jsonrpc": "2.0",
        "id": "oauth2-debug-test",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"experimental": {}},
            "clientInfo": {"name": "OAuth2-Debug-Client", "version": "1.0.0"}
        }
    }
    
    try:
        response = requests.post("http://localhost:3002/mcp", json=payload, headers=headers, timeout=10)
        print(f"MCP status: {response.status_code}")
        
        if response.status_code == 200:
            mcp_data = response.json()
            print(f"MCP response: {json.dumps(mcp_data, indent=2)}")
        else:
            print(f"MCP error: {response.text}")
    except Exception as e:
        print(f"MCP error: {e}")

def main():
    print("🧪 OAuth2 Flow Debug Script")
    print("=" * 50)
    
    # Wait for servers to be ready
    print("⏳ Waiting for servers...")
    time.sleep(2)
    
    # Step 1: Test discovery
    discovery_data = test_discovery()
    
    # Step 2: Test authorization
    auth_code, verifier = test_authorize(discovery_data)
    
    # Step 3: Test token exchange
    access_token = test_token_exchange(discovery_data, auth_code, verifier)
    
    # Step 4: Test MCP API
    test_mcp_with_token(access_token)
    
    print("\n✅ Debug complete!")

if __name__ == "__main__":
    main()