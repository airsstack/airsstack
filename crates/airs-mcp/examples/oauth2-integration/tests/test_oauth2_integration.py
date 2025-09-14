#!/usr/bin/env python3
"""
OAuth2 MCP Integration Test Suite

This script provides comprehensive testing of the OAuth2 MCP server integration,
including token generation, MCP protocol operations, and scope-based authorization.

Usage:
    python3 test_oauth2_integration.py [--debug] [--no-cleanup]
    
Options:
    --debug      Enable debug output
    --no-cleanup Don't stop the server after tests
"""

import json
import requests
import subprocess
import time
import argparse
import sys
import os
import signal
from typing import Dict, List, Optional, Any
from dataclasses import dataclass


@dataclass
class TestResult:
    """Result of a test operation"""
    name: str
    passed: bool
    message: str
    details: Optional[str] = None


class OAuth2MCPTester:
    """OAuth2 MCP Integration Tester"""
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self.server_process: Optional[subprocess.Popen] = None
        self.server_pid: Optional[int] = None
        self.results: List[TestResult] = []
        
        # Server configuration (updated for three-server proxy architecture)
        self.mcp_direct_url = "http://localhost:3001/mcp"  # Direct MCP server
        self.proxy_url = "http://localhost:3002"           # Proxy server (recommended endpoint)
        self.server_url = f"{self.proxy_url}/mcp"          # MCP through proxy
        self.custom_routes_url = "http://localhost:3003"   # Custom OAuth2 routes
        self.jwks_url = "http://localhost:3004"            # JWKS server
        self.auth_tokens_url = f"{self.jwks_url}/auth/tokens"
        
        # Test tokens storage
        self.tokens: Dict[str, str] = {}
        
    def log(self, message: str, level: str = "INFO"):
        """Log a message with optional debug level"""
        if level == "DEBUG" and not self.debug:
            return
        timestamp = time.strftime("%H:%M:%S")
        prefix = "🐛" if level == "DEBUG" else "ℹ️"
        print(f"{prefix} [{timestamp}] {message}")
        
    def log_success(self, message: str):
        """Log a success message"""
        print(f"✅ {message}")
        
    def log_error(self, message: str):
        """Log an error message"""
        print(f"❌ {message}")
        
    def log_warning(self, message: str):
        """Log a warning message"""
        print(f"⚠️  {message}")

    def add_result(self, name: str, passed: bool, message: str, details: str = None):
        """Add a test result"""
        self.results.append(TestResult(name, passed, message, details))
        if passed:
            self.log_success(f"{name}: {message}")
        else:
            self.log_error(f"{name}: {message}")
            if details and self.debug:
                self.log(f"Details: {details}", "DEBUG")
    
    def cleanup(self):
        """Clean up any running processes"""
        self.log("🧹 Cleaning up any existing servers...")
        
        # Kill any existing oauth2-mcp-server processes
        try:
            subprocess.run(['pkill', '-f', 'oauth2-mcp-server'], 
                         capture_output=True, check=False)
            time.sleep(1)
        except Exception as e:
            self.log(f"Cleanup warning: {e}", "DEBUG")
            
        # Kill our specific process if we have the PID
        if self.server_pid:
            try:
                os.kill(self.server_pid, signal.SIGTERM)
                time.sleep(2)
                # Force kill if still running
                try:
                    os.kill(self.server_pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass  # Process already dead
            except ProcessLookupError:
                pass  # Process not found
            except Exception as e:
                self.log(f"Cleanup error: {e}", "DEBUG")
    
    def start_server(self) -> bool:
        """Start the OAuth2 MCP server"""
        print("🚀 Starting OAuth2 MCP Server...")
        
        # First, build the server
        try:
            build_result = subprocess.run(
                ['cargo', 'build', '--bin', 'oauth2-mcp-server'],
                cwd='..',  # Run from parent directory where Cargo.toml is
                capture_output=True, text=True, check=True
            )
            print("Server build completed")
        except subprocess.CalledProcessError as e:
            print(f"Failed to build server: {e}")
            if hasattr(e, 'stderr') and e.stderr:
                print(f"Build stderr: {e.stderr}")
            return False
        
        # Start the server
        try:
            env = os.environ.copy()
            env['RUST_LOG'] = 'info'
            
            # Create logs directory if it doesn't exist
            os.makedirs('../logs', exist_ok=True)
            
            # Open log file for server output
            log_file = open('../logs/server_integration.log', 'w')
            
            self.server_process = subprocess.Popen(
                ['../target/debug/oauth2-mcp-server'],  # Path relative to tests directory
                stdout=log_file,
                stderr=subprocess.STDOUT,
                env=env,
                text=True
            )
            self.server_pid = self.server_process.pid
            print(f"Server started with PID: {self.server_pid}")
            print(f"Server logs: ../logs/server_integration.log")
            
            # Wait for all servers to be ready
            return self.wait_for_server()
            
        except Exception as e:
            print(f"Failed to start server: {e}")
            return False
    
    def wait_for_server(self, max_attempts: int = 30) -> bool:
        """Wait for the three-server architecture to be ready"""
        self.log("⏳ Waiting for OAuth2 Three-Server Architecture to be ready...")
        
        # Step 1: Wait for Direct MCP Server (port 3001)
        self.log("  🔍 Checking Direct MCP Server (port 3001)...")
        for attempt in range(max_attempts):
            try:
                response = requests.get(self.mcp_direct_url, timeout=2)
                if response.status_code in [401, 403, 405]:
                    self.log_success(f"  ✅ Direct MCP Server ready (HTTP {response.status_code})")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.add_result("Direct MCP Server", False, "Failed to start within timeout")
            return False
        
        # Step 2: Wait for Proxy Server (port 3002)
        self.log("  🔍 Checking Proxy Server (port 3002)...")
        for attempt in range(max_attempts):
            try:
                response = requests.get(self.server_url, timeout=2)
                if response.status_code in [401, 403, 405]:
                    self.log_success(f"  ✅ Proxy Server ready (HTTP {response.status_code})")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.add_result("Proxy Server", False, "Failed to start within timeout")
            return False
        
        # Step 3: Wait for JWKS Server (port 3004)
        self.log("  🔍 Checking JWKS Server (port 3004)...")
        for attempt in range(max_attempts):
            try:
                response = requests.get(self.auth_tokens_url, timeout=2)
                if response.status_code == 200:
                    self.log_success("  ✅ JWKS Server ready!")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.add_result("JWKS Server", False, "Failed to start within timeout")
            return False
        
        # Step 4: Test OAuth2 Discovery (optional but useful)
        try:
            discovery_url = f"{self.custom_routes_url}/.well-known/oauth-authorization-server"
            response = requests.get(discovery_url, timeout=2)
            if response.status_code == 200:
                self.log_success("  ✅ OAuth2 Discovery endpoint ready!")
            else:
                self.log_warning(f"  OAuth2 Discovery returned HTTP {response.status_code}")
        except requests.exceptions.RequestException:
            self.log_warning("  OAuth2 Discovery endpoint not accessible")
        
        self.log_success("🚀 All servers in three-server architecture are ready!")
        return True
    
    def test_connectivity(self) -> bool:
        """Test basic connectivity to all servers in the architecture"""
        self.log("🔍 Testing Three-Server Architecture Connectivity...")
        
        # Test Direct MCP endpoint (should require auth)
        try:
            response = requests.get(self.mcp_direct_url, timeout=5)
            if response.status_code in [401, 403, 404, 405]:
                self.add_result("Direct MCP Endpoint", True, 
                              f"Accessible (HTTP {response.status_code} - auth required as expected)")
            else:
                self.add_result("Direct MCP Endpoint", False, 
                              f"Unexpected status code: {response.status_code}")
                return False
        except Exception as e:
            self.add_result("Direct MCP Endpoint", False, f"Connection failed: {e}")
            return False
        
        # Test Proxy MCP endpoint (should require auth)
        try:
            response = requests.get(self.server_url, timeout=5)
            if response.status_code in [401, 403, 404, 405]:
                self.add_result("Proxy MCP Endpoint", True, 
                              f"Accessible (HTTP {response.status_code} - auth required as expected)")
            else:
                self.add_result("Proxy MCP Endpoint", False, 
                              f"Unexpected status code: {response.status_code}")
                return False
        except Exception as e:
            self.add_result("Proxy MCP Endpoint", False, f"Connection failed: {e}")
            return False
        
        # Test JWKS endpoint
        try:
            response = requests.get(self.auth_tokens_url, timeout=5)
            if response.status_code == 200:
                self.add_result("JWKS Server", True, "Accessible")
            else:
                self.add_result("JWKS Server", False, 
                              f"Unexpected status code: {response.status_code}")
                return False
        except Exception as e:
            self.add_result("JWKS Server", False, f"Connection failed: {e}")
            return False
        
        # Test OAuth2 Discovery endpoint
        try:
            discovery_url = f"{self.custom_routes_url}/.well-known/oauth-authorization-server"
            response = requests.get(discovery_url, timeout=5)
            if response.status_code == 200:
                self.add_result("OAuth2 Discovery", True, "Accessible")
            else:
                self.add_result("OAuth2 Discovery", False, 
                              f"Unexpected status code: {response.status_code}")
                # Don't return False - this is optional
        except Exception as e:
            self.add_result("OAuth2 Discovery", False, f"Connection failed: {e}")
            # Don't return False - this is optional
        
        return True
    
    def test_proxy_vs_direct_access(self) -> bool:
        """Test that proxy and direct access both work with the same token"""
        self.log("🔄 Testing Proxy vs Direct Access...")
        
        if not self.tokens or 'full' not in self.tokens:
            self.add_result("Proxy vs Direct", False, "No full access token available")
            return False
        
        full_token = self.tokens['full']
        initialize_params = {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "Proxy-Test-Client",
                "version": "1.0.0"
            }
        }
        
        # Test direct access to MCP server
        self.log("  Testing direct MCP server access...", "DEBUG")
        direct_response = None
        try:
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {full_token}"
            }
            payload = {
                "jsonrpc": "2.0",
                "id": "proxy-test-direct",
                "method": "initialize",
                "params": initialize_params
            }
            
            response = requests.post(self.mcp_direct_url, 
                                   json=payload, 
                                   headers=headers, 
                                   timeout=10)
            
            if response.status_code == 200:
                direct_response = response.json()
                if 'result' in direct_response:
                    self.add_result("Direct MCP Access", True, "Initialize successful")
                else:
                    self.add_result("Direct MCP Access", False, "No result in response")
            else:
                self.add_result("Direct MCP Access", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.add_result("Direct MCP Access", False, f"Error: {e}")
        
        # Test proxy access to MCP server
        self.log("  Testing proxy MCP server access...", "DEBUG")
        proxy_response = None
        try:
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {full_token}"
            }
            payload = {
                "jsonrpc": "2.0",
                "id": "proxy-test-proxy",
                "method": "initialize",
                "params": initialize_params
            }
            
            response = requests.post(self.server_url, 
                                   json=payload, 
                                   headers=headers, 
                                   timeout=10)
            
            if response.status_code == 200:
                proxy_response = response.json()
                if 'result' in proxy_response:
                    self.add_result("Proxy MCP Access", True, "Initialize successful")
                else:
                    self.add_result("Proxy MCP Access", False, "No result in response")
            else:
                self.add_result("Proxy MCP Access", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.add_result("Proxy MCP Access", False, f"Error: {e}")
        
        # Compare responses
        if direct_response and proxy_response and 'result' in direct_response and 'result' in proxy_response:
            direct_server = direct_response['result'].get('serverInfo', {}).get('name', 'unknown')
            proxy_server = proxy_response['result'].get('serverInfo', {}).get('name', 'unknown')
            
            if direct_server == proxy_server:
                self.add_result("Response Consistency", True, 
                              f"Both return same server: {direct_server}")
            else:
                self.add_result("Response Consistency", False, 
                              f"Different servers: direct={direct_server}, proxy={proxy_server}")
        
        return True
    
    def fetch_tokens(self) -> bool:
        """Fetch OAuth2 tokens from the server"""
        self.log("🔐 Fetching OAuth2 Authentication Tokens...")
        
        try:
            response = requests.get(self.auth_tokens_url, timeout=10)
            if response.status_code != 200:
                self.add_result("Token Fetch", False, 
                              f"Failed to fetch tokens: HTTP {response.status_code}")
                return False
            
            data = response.json()
            self.log(f"Token response received: {len(json.dumps(data))} bytes", "DEBUG")
            
            # Extract tokens
            if 'tokens' not in data:
                self.add_result("Token Extract", False, "No 'tokens' field in response")
                return False
            
            token_types = ['full', 'readonly', 'tools', 'resources']
            for token_type in token_types:
                if token_type in data['tokens'] and 'token' in data['tokens'][token_type]:
                    self.tokens[token_type] = data['tokens'][token_type]['token']
                    self.log(f"Extracted {token_type} token: {self.tokens[token_type][:50]}...", "DEBUG")
                else:
                    self.add_result("Token Extract", False, f"Missing {token_type} token")
                    return False
            
            self.add_result("Token Fetch", True, f"Successfully extracted {len(self.tokens)} tokens")
            return True
            
        except Exception as e:
            self.add_result("Token Fetch", False, f"Error fetching tokens: {e}")
            return False
    
    def mcp_request(self, method: str, params: Dict[str, Any], token: str, timeout: int = 15) -> Optional[Dict[str, Any]]:
        """Make an MCP JSON-RPC request"""
        payload = {
            "jsonrpc": "2.0",
            "id": f"test-{int(time.time())}-{len(token[:8])}",  # Unique ID per request
            "method": method,
            "params": params
        }
        
        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {token}"
        }
        
        self.log(f"Making MCP {method} request with {token[:8]}... token", "DEBUG")
        
        try:
            # Add a small delay between requests to avoid overwhelming the server
            time.sleep(0.5)
            
            response = requests.post(self.server_url, 
                                   json=payload, 
                                   headers=headers, 
                                   timeout=timeout)
            
            self.log(f"MCP {method} request: HTTP {response.status_code}", "DEBUG")
            
            if response.status_code == 200:
                result = response.json()
                self.log(f"MCP {method} response received successfully", "DEBUG")
                return result
            elif response.status_code == 401:
                self.log(f"MCP request unauthorized (token may be invalid)", "DEBUG")
                return None
            elif response.status_code == 403:
                self.log(f"MCP request forbidden (insufficient scope)", "DEBUG")
                return None
            else:
                self.log(f"MCP request failed: {response.status_code} - {response.text[:200]}", "DEBUG")
                return None
                
        except requests.exceptions.Timeout as e:
            self.log(f"MCP request timeout after {timeout}s: {e}", "DEBUG")
            return None
        except requests.exceptions.ConnectionError as e:
            self.log(f"MCP connection error: {e}", "DEBUG")
            return None
        except Exception as e:
            self.log(f"MCP request error: {e}", "DEBUG")
            return None
    
    def test_mcp_initialize(self) -> bool:
        """Test MCP initialize with different tokens"""
        self.log("🔧 Testing MCP Initialize...")
        
        success_count = 0
        for token_type, token in self.tokens.items():
            self.log(f"Testing initialize with {token_type} token...", "DEBUG")
            
            # Use a shorter timeout and add retry logic
            for attempt in range(2):  # Try twice per token
                response = self.mcp_request("initialize", {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": f"OAuth2-Test-Client-{token_type}",
                        "version": "1.0.0"
                    }
                }, token, timeout=8)
                
                if response and 'result' in response:
                    protocol_version = response['result'].get('protocolVersion', 'unknown')
                    server_name = response['result'].get('serverInfo', {}).get('name', 'unknown')
                    self.add_result(f"Initialize ({token_type})", True, 
                                  f"Success - Protocol: {protocol_version}, Server: {server_name}")
                    success_count += 1
                    break  # Success, no need to retry
                elif response and 'error' in response:
                    error_msg = response['error'].get('message', 'Unknown error')
                    error_code = response['error'].get('code', 'unknown')
                    self.add_result(f"Initialize ({token_type})", False, 
                                  f"Server error {error_code}: {error_msg}")
                    break  # Server error, don't retry
                else:
                    if attempt == 0:
                        self.log(f"Retrying {token_type} token (attempt {attempt + 2}/2)...", "DEBUG")
                        time.sleep(1)  # Wait before retry
                    else:
                        self.add_result(f"Initialize ({token_type})", False, 
                                      "Failed: No response after retries")
        
        return success_count == len(self.tokens)
    
    def test_scope_authorization(self) -> bool:
        """Test scope-based authorization"""
        self.log("🔐 Testing Scope-Based Authorization...")
        
        # Test resources access
        readonly_token = self.tokens.get('readonly')
        full_token = self.tokens.get('full')
        
        if not readonly_token or not full_token:
            self.add_result("Scope Test Setup", False, "Missing required tokens")
            return False
        
        # Test: readonly token should be able to list resources
        response = self.mcp_request("resources/list", {}, readonly_token)
        if response and 'result' in response:
            resources = response['result'].get('resources', [])
            self.add_result("Readonly Resources List", True, 
                          f"Success - Found {len(resources)} resources")
        else:
            self.add_result("Readonly Resources List", False, "Failed to list resources")
            return False
        
        # Test: full token should also be able to list resources
        response = self.mcp_request("resources/list", {}, full_token)
        if response and 'result' in response:
            resources = response['result'].get('resources', [])
            self.add_result("Full Access Resources List", True, 
                          f"Success - Found {len(resources)} resources")
        else:
            self.add_result("Full Access Resources List", False, "Failed to list resources")
            return False
        
        return True
    
    def test_tools_access(self) -> bool:
        """Test tools access with different tokens"""
        self.log("🔧 Testing Tools Access...")
        
        tools_token = self.tokens.get('tools')
        if not tools_token:
            self.add_result("Tools Test Setup", False, "Missing tools token")
            return False
        
        # Test: tools token should be able to list tools
        response = self.mcp_request("tools/list", {}, tools_token)
        if response and 'result' in response:
            tools = response['result'].get('tools', [])
            self.add_result("Tools List", True, f"Success - Found {len(tools)} tools")
            return True
        else:
            self.add_result("Tools List", False, "Failed to list tools")
            return False
    
    def test_prompts_access(self) -> bool:
        """Test prompts access"""
        self.log("💬 Testing Prompts Access...")
        
        full_token = self.tokens.get('full')
        if not full_token:
            self.add_result("Prompts Test Setup", False, "Missing full token")
            return False
        
        # Test: full token should be able to list prompts
        response = self.mcp_request("prompts/list", {}, full_token)
        if response and 'result' in response:
            prompts = response['result'].get('prompts', [])
            self.add_result("Prompts List", True, f"Success - Found {len(prompts)} prompts")
            return True
        else:
            self.add_result("Prompts List", False, "Failed to list prompts")
            return False
    
    def run_all_tests(self) -> bool:
        """Run the complete test suite"""
        print("🧪 OAuth2 MCP Integration Test Suite")
        print("=" * 50)
        
        try:
            # Setup
            self.cleanup()
            if not self.start_server():
                return False
            
            # Basic connectivity
            if not self.test_connectivity():
                return False
            
            # Authentication
            if not self.fetch_tokens():
                return False
            
            # Test proxy vs direct access (requires tokens)
            if not self.test_proxy_vs_direct_access():
                return False
            
            # MCP Protocol tests
            if not self.test_mcp_initialize():
                return False
            
            # Authorization tests
            if not self.test_scope_authorization():
                return False
            
            if not self.test_tools_access():
                return False
            
            if not self.test_prompts_access():
                return False
            
            return True
            
        except KeyboardInterrupt:
            self.log_warning("Test interrupted by user")
            return False
        except Exception as e:
            self.log_error(f"Test suite error: {e}")
            return False
    
    def print_summary(self):
        """Print test results summary"""
        print("\n" + "=" * 50)
        print("📊 Test Results Summary")
        print("=" * 50)
        
        passed = sum(1 for r in self.results if r.passed)
        total = len(self.results)
        
        for result in self.results:
            status = "✅ PASS" if result.passed else "❌ FAIL"
            print(f"{status}: {result.name} - {result.message}")
        
        print("-" * 50)
        print(f"Results: {passed}/{total} tests passed")
        
        if passed == total:
            print("🎉 All tests passed! OAuth2 MCP integration is working correctly.")
            return True
        else:
            print(f"💥 {total - passed} tests failed. Check the logs above for details.")
            return False


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="OAuth2 MCP Integration Test Suite")
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    parser.add_argument("--no-cleanup", action="store_true", help="Don't stop server after tests")
    
    args = parser.parse_args()
    
    tester = OAuth2MCPTester(debug=args.debug)
    
    try:
        success = tester.run_all_tests()
        final_success = tester.print_summary()
        
        if not args.no_cleanup:
            tester.cleanup()
            tester.log_success("Cleanup completed")
        else:
            tester.log_warning("Server left running (--no-cleanup specified)")
        
        sys.exit(0 if final_success else 1)
        
    except Exception as e:
        tester.log_error(f"Fatal error: {e}")
        if not args.no_cleanup:
            tester.cleanup()
        sys.exit(1)


if __name__ == "__main__":
    main()