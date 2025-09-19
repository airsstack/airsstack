#!/usr/bin/env python3
"""
Quick OAuth2 Server Test

Simple script to test individual components without full orchestration.
"""

import requests
import subprocess
import sys
import time
from pathlib import Path

def test_server_endpoints():
    """Test that servers are responding"""
    print("🌐 Testing server endpoints...")
    
    endpoints = [
        ("http://127.0.0.1:3002/health", "OAuth2 Server"),
        ("http://127.0.0.1:3002/.well-known/openid-configuration", "OIDC Discovery"),
        ("http://127.0.0.1:3002/jwks", "JWKS"),
        ("http://127.0.0.1:3003/health", "MCP Server"),
    ]
    
    for url, name in endpoints:
        try:
            response = requests.get(url, timeout=5)
            print(f"✅ {name}: {response.status_code}")
            if response.status_code == 200:
                # Print some response content for debugging
                try:
                    data = response.json()
                    print(f"   Response: {str(data)[:100]}...")
                except:
                    print(f"   Response: {response.text[:100]}...")
        except Exception as e:
            print(f"❌ {name}: {e}")

def test_authorization_url():
    """Test authorization URL generation"""
    print("\n🔗 Testing authorization URL generation...")
    
    auth_url = "http://127.0.0.1:3002/authorize"
    params = {
        'response_type': 'code',
        'client_id': 'test-client',
        'redirect_uri': 'http://localhost:8082/callback',
        'scope': 'mcp:read mcp:write',
        'code_challenge': 'test-challenge',
        'code_challenge_method': 'S256',
        'state': 'test-state'
    }
    
    try:
        response = requests.get(auth_url, params=params, timeout=10)
        print(f"✅ Authorization endpoint: {response.status_code}")
        if response.status_code == 400:
            print(f"❌ Error response: {response.text}")
        elif response.status_code == 200:
            print("✅ Authorization page returned successfully")
    except Exception as e:
        print(f"❌ Authorization test failed: {e}")

def run_quick_client_test():
    """Run a quick client test"""
    print("\n🔌 Running quick client test...")
    
    project_dir = Path(__file__).parent
    
    command = [
        'cargo', 'run', '--bin', 'http-oauth2-client', '--',
        '--auth-server', 'http://127.0.0.1:3002',
        '--mcp-server', 'http://127.0.0.1:3003'
    ]
    
    try:
        result = subprocess.run(
            command,
            cwd=project_dir,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        print(f"Exit code: {result.returncode}")
        if result.stdout:
            print("STDOUT:")
            print(result.stdout)
        if result.stderr:
            print("STDERR:")
            print(result.stderr)
            
    except subprocess.TimeoutExpired:
        print("❌ Client test timed out")
    except Exception as e:
        print(f"❌ Client test failed: {e}")

if __name__ == "__main__":
    print("🧪 Quick OAuth2 Test")
    print("=" * 50)
    
    # Install requests if needed
    try:
        import requests
    except ImportError:
        print("📦 Installing requests...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
        import requests
    
    test_server_endpoints()
    test_authorization_url()
    
    if len(sys.argv) > 1 and sys.argv[1] == "--run-client":
        run_quick_client_test()
    else:
        print("\nTo run client test, use: python quick_test.py --run-client")