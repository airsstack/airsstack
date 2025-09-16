#!/usr/bin/env python3
"""
OAuth2 MCP Integration Test - Basic Functionality Demo

This script demonstrates the core OAuth2 MCP integration functionality:
- Server startup and token generation
- Token extraction and validation  
- Basic MCP initialize with full access token

Usage:
    python3 test_oauth2_basic.py [--debug]
"""

import json
import requests
import subprocess
import time
import argparse
import sys
import os
import signal
from typing import Dict, Optional


class BasicOAuth2Test:
    """Basic OAuth2 MCP functionality test"""
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self.server_process: Optional[subprocess.Popen] = None
        self.server_pid: Optional[int] = None
        
        # Server configuration (updated for three-server proxy architecture)
        self.mcp_direct_url = "http://localhost:3001/mcp"    # Direct MCP server
        self.proxy_url = "http://localhost:3002"             # Proxy server
        self.server_url = f"{self.proxy_url}/mcp"            # MCP through proxy (recommended)
        self.auth_tokens_url = "http://localhost:3004/auth/tokens"  # JWKS server
        
    def log(self, message: str, level: str = "INFO"):
        """Log a message"""
        if level == "DEBUG" and not self.debug:
            return
        prefix = "🐛" if level == "DEBUG" else "ℹ️"
        print(f"{prefix} {message}")
        
    def success(self, message: str):
        """Log success"""
        print(f"✅ {message}")
        
    def error(self, message: str):
        """Log error"""
        print(f"❌ {message}")
    
    def cleanup(self):
        """Clean up processes"""
        self.log("🧹 Cleaning up...")
        try:
            subprocess.run(['pkill', '-f', 'http-oauth2-server'], 
                         capture_output=True, check=False)
            time.sleep(1)
        except Exception as e:
            self.log(f"Cleanup warning: {e}", "DEBUG")
    
    def start_server(self) -> bool:
        """Start the OAuth2 MCP server"""
        self.log("🚀 Starting OAuth2 MCP Server...")
        
        # First, build the server
        try:
            build_result = subprocess.run(
                ['cargo', 'build', '--bin', 'http-oauth2-server'],
                cwd='..',  # Run from parent directory where Cargo.toml is
                capture_output=True, text=True, check=True
            )
            self.log("Server build completed", "DEBUG")
        except subprocess.CalledProcessError as e:
            self.add_result("Server Build", False, f"Failed to build server: {e}")
            if self.debug:
                self.log(f"Build stderr: {e.stderr}", "DEBUG")
            return False
        
        # Start the server
        try:
            env = os.environ.copy()
            env['RUST_LOG'] = 'debug' if self.debug else 'info'
            
            # Create logs directory if it doesn't exist
            os.makedirs('../logs', exist_ok=True)
            
            # Open log file for server output
            log_file = open('../logs/server.log', 'w')
            
            self.server_process = subprocess.Popen(
                ['../target/debug/http-oauth2-server'],  # Path relative to tests directory
                stdout=log_file,
                stderr=subprocess.STDOUT,
                env=env,
                text=True
            )
            self.server_pid = self.server_process.pid
            self.log(f"Server started with PID: {self.server_pid}", "DEBUG")
            self.log(f"Server logs: ../logs/server.log", "DEBUG")
            
        except Exception as e:
            self.add_result("Server Start", False, f"Failed to start server: {e}")
            return False
        
        # Wait for ready
        self.log("⏳ Waiting for server to be ready...")
        for attempt in range(30):
            try:
                # Check MCP endpoint
                response = requests.get(self.server_url, timeout=2)
                if response.status_code in [401, 403, 405]:
                    self.success(f"OAuth2 MCP Server ready! (HTTP {response.status_code})")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.error("Server startup timeout")
            return False
        
        # Check JWKS
        for attempt in range(10):
            try:
                response = requests.get(self.auth_tokens_url, timeout=2)
                if response.status_code == 200:
                    self.success("OAuth2 JWKS Server ready!")
                    return True
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        
        self.error("JWKS server startup timeout")
        return False
    
    def test_token_generation(self) -> Optional[str]:
        """Test token generation and return a valid token"""
        self.log("🔐 Testing Token Generation...")
        
        try:
            response = requests.get(self.auth_tokens_url, timeout=10)
            if response.status_code != 200:
                self.error(f"Token fetch failed: HTTP {response.status_code}")
                return None
            
            data = response.json()
            self.log(f"Token response: {len(json.dumps(data))} bytes", "DEBUG")
            
            # Extract full access token
            if 'tokens' in data and 'full' in data['tokens'] and 'token' in data['tokens']['full']:
                token = data['tokens']['full']['token']
                self.success(f"Token extracted: {token[:50]}...")
                
                # Display all available tokens
                print("\n📋 Available Token Types:")
                for token_type, token_data in data['tokens'].items():
                    scopes = token_data.get('scopes', [])
                    expires = token_data.get('expires_minutes', 'unknown')
                    print(f"  • {token_type}: {len(scopes)} scopes, expires in {expires} minutes")
                    for scope in scopes:
                        print(f"    - {scope}")
                
                return token
            else:
                self.error("Token structure invalid")
                return None
                
        except Exception as e:
            self.error(f"Token generation failed: {e}")
            return None
    
    def test_mcp_initialize(self, token: str) -> bool:
        """Test MCP initialize with token"""
        self.log("🔧 Testing MCP Initialize...")
        
        payload = {
            "jsonrpc": "2.0",
            "id": "test-init",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {
                    "name": "OAuth2-Basic-Test",
                    "version": "1.0.0"
                }
            }
        }
        
        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {token}"
        }
        
        try:
            response = requests.post(self.server_url, 
                                   json=payload, 
                                   headers=headers, 
                                   timeout=10)
            
            self.log(f"MCP initialize: HTTP {response.status_code}", "DEBUG")
            
            if response.status_code == 200:
                result = response.json()
                if 'result' in result:
                    protocol = result['result'].get('protocolVersion', 'unknown')
                    server_name = result['result'].get('serverInfo', {}).get('name', 'unknown')
                    capabilities = result['result'].get('capabilities', {})
                    
                    self.success(f"MCP Initialize successful!")
                    print(f"  • Protocol Version: {protocol}")
                    print(f"  • Server Name: {server_name}")
                    print(f"  • Server Capabilities: {list(capabilities.keys())}")
                    
                    return True
                else:
                    self.error(f"Invalid MCP response: {result}")
                    return False
            else:
                self.error(f"MCP request failed: HTTP {response.status_code}")
                return False
                
        except Exception as e:
            self.error(f"MCP request error: {e}")
            return False
    
    def run_basic_test(self) -> bool:
        """Run the basic functionality test"""
        print("🧪 OAuth2 MCP Basic Integration Test")
        print("=" * 50)
        
        try:
            # Setup
            self.cleanup()
            if not self.start_server():
                return False
            
            # Test token generation
            token = self.test_token_generation()
            if not token:
                return False
            
            # Test MCP initialize
            if not self.test_mcp_initialize(token):
                return False
            
            print("\n🎉 Basic OAuth2 MCP integration is working correctly!")
            print("\n📖 Next Steps:")
            print("  1. Copy one of the tokens from the server response")
            print("  2. Use the MCP Inspector: npx @modelcontextprotocol/inspector-cli \\")
            print("     --transport http --server-url http://localhost:3001/mcp \\")
            print("     --header \"Authorization: Bearer YOUR_TOKEN_HERE\"")
            print("  3. Or test with curl using the provided curl_test commands")
            
            return True
            
        except KeyboardInterrupt:
            print("\n⚠️  Test interrupted by user")
            return False
        except Exception as e:
            self.error(f"Test error: {e}")
            return False


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="OAuth2 MCP Basic Integration Test")
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    parser.add_argument("--no-cleanup", action="store_true", help="Keep server running")
    
    args = parser.parse_args()
    
    tester = BasicOAuth2Test(debug=args.debug)
    
    try:
        success = tester.run_basic_test()
        
        if not args.no_cleanup:
            tester.cleanup()
            tester.success("Cleanup completed")
        else:
            print(f"\n🔄 Server left running on:")
            print(f"  • MCP Endpoint: http://localhost:3001/mcp")
            print(f"  • Token Endpoint: http://localhost:3002/auth/tokens")
        
        sys.exit(0 if success else 1)
        
    except Exception as e:
        tester.error(f"Fatal error: {e}")
        if not args.no_cleanup:
            tester.cleanup()
        sys.exit(1)


if __name__ == "__main__":
    main()