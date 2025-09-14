#!/usr/bin/env python3
"""
OAuth2 Authorization Flow Test Suite

This test validates the complete OAuth2 Authorization Code Flow with PKCE (RFC 6749 + RFC 7636)
for MCP Inspector compatibility. It tests the actual /authorize and /token endpoints,
not just the simplified test token generator.

Usage:
    python3 test_oauth2_authorization_flow.py [--debug] [--no-cleanup]

Options:
    --debug      Enable debug output
    --no-cleanup Don't stop the server after tests
"""

import argparse
import base64
import hashlib
import json
import os
import secrets
import signal
import subprocess
import sys
import time
import urllib.parse
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass

import requests


@dataclass
class TestResult:
    """Result of a test operation"""
    name: str
    passed: bool
    message: str
    details: Optional[str] = None


@dataclass 
class PKCEChallenge:
    """PKCE code challenge and verifier pair"""
    verifier: str
    challenge: str
    method: str = "S256"


class OAuth2FlowTester:
    """OAuth2 Authorization Flow Tester"""
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self.server_process: Optional[subprocess.Popen] = None
        self.server_pid: Optional[int] = None
        self.results: List[TestResult] = []
        
        # Server configuration (three-server proxy architecture)
        self.mcp_direct_url = "http://localhost:3001/mcp"  # Direct MCP server
        self.proxy_url = "http://localhost:3002"           # Proxy server
        self.server_url = f"{self.proxy_url}/mcp"          # MCP through proxy
        self.oauth2_server_url = "http://localhost:3003"   # OAuth2 authorization server
        self.jwks_url = "http://localhost:3004"            # JWKS server
        
        # OAuth2 flow endpoints
        self.discovery_url = f"{self.oauth2_server_url}/.well-known/oauth-authorization-server"
        self.authorize_url = f"{self.oauth2_server_url}/authorize"
        self.token_url = f"{self.oauth2_server_url}/token"
        
        # Test OAuth2 client configuration
        self.client_id = "test-mcp-client"
        self.redirect_uri = "http://localhost:8080/callback"
        self.scopes = ["mcp:resources:list", "mcp:tools:list", "mcp:prompts:list"]
        
        # Flow state
        self.authorization_code: Optional[str] = None
        self.access_token: Optional[str] = None
        self.pkce: Optional[PKCEChallenge] = None
        
        # Test counters
        self.tests_run = 0
        self.tests_passed = 0

    def log(self, message: str, level: str = "INFO"):
        """Log a message with optional debug level"""
        if level == "DEBUG" and not self.debug:
            return
        
        timestamp = time.strftime("%H:%M:%S")
        if level == "SUCCESS":
            print(f"✅ {message}")
        elif level == "ERROR":
            print(f"❌ {message}")
        elif level == "WARNING":
            print(f"⚠️ {message}")
        elif level == "DEBUG":
            print(f"🐛 [{timestamp}] {message}")
        else:
            print(f"ℹ️ [{timestamp}] {message}")

    def log_success(self, message: str):
        """Log a success message"""
        self.log(message, "SUCCESS")

    def log_error(self, message: str):
        """Log an error message"""
        self.log(message, "ERROR")

    def log_warning(self, message: str):
        """Log a warning message"""
        self.log(message, "WARNING")

    def add_result(self, name: str, passed: bool, message: str, details: str = None):
        """Add a test result"""
        self.results.append(TestResult(name, passed, message, details))
        self.tests_run += 1
        if passed:
            self.tests_passed += 1
            self.log_success(f"{name}: {message}")
        else:
            self.log_error(f"{name}: {message}")
            if details and self.debug:
                self.log(f"Details: {details}", "DEBUG")

    def generate_pkce_challenge(self) -> PKCEChallenge:
        """Generate PKCE code verifier and challenge"""
        # Generate cryptographically secure random code verifier (43-128 chars)
        verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
        
        # Generate S256 code challenge
        challenge = base64.urlsafe_b64encode(
            hashlib.sha256(verifier.encode('utf-8')).digest()
        ).decode('utf-8').rstrip('=')
        
        return PKCEChallenge(verifier=verifier, challenge=challenge, method="S256")

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
            log_file = open('../logs/server_oauth2_flow.log', 'w')
            
            self.server_process = subprocess.Popen(
                ['../target/debug/oauth2-mcp-server'],  # Path relative to tests directory
                stdout=log_file,
                stderr=subprocess.STDOUT,
                env=env,
                text=True
            )
            self.server_pid = self.server_process.pid
            print(f"Server started with PID: {self.server_pid}")
            print(f"Server logs: ../logs/server_oauth2_flow.log")
            
            # Wait for all servers to be ready
            return self.wait_for_server()
            
        except Exception as e:
            print(f"Failed to start server: {e}")
            return False

    def wait_for_server(self, max_attempts: int = 30) -> bool:
        """Wait for the OAuth2 three-server architecture to be ready"""
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
        
        # Step 3: Wait for OAuth2 Authorization Server (port 3003)
        self.log("  🔍 Checking OAuth2 Authorization Server (port 3003)...")
        for attempt in range(max_attempts):
            try:
                response = requests.get(self.discovery_url, timeout=2)
                if response.status_code == 200:
                    self.log_success("  ✅ OAuth2 Authorization Server ready!")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.add_result("OAuth2 Authorization Server", False, "Failed to start within timeout")
            return False
        
        # Step 4: Wait for JWKS Server (port 3004)
        self.log("  🔍 Checking JWKS Server (port 3004)...")
        jwks_url = f"{self.jwks_url}/.well-known/jwks.json"
        for attempt in range(max_attempts):
            try:
                response = requests.get(jwks_url, timeout=2)
                if response.status_code == 200:
                    self.log_success("  ✅ JWKS Server ready!")
                    break
            except requests.exceptions.RequestException:
                pass
            time.sleep(1)
        else:
            self.add_result("JWKS Server", False, "Failed to start within timeout")
            return False
        
        self.log_success("🚀 All servers in three-server architecture are ready!")
        return True

    def test_oauth2_discovery(self) -> bool:
        """Test OAuth2 discovery metadata endpoint"""
        self.log("🔍 Testing OAuth2 Discovery Metadata...")
        
        try:
            response = requests.get(self.discovery_url, timeout=10)
            if response.status_code != 200:
                self.add_result("OAuth2 Discovery", False, 
                              f"HTTP {response.status_code}")
                return False
            
            metadata = response.json()
            self.log(f"Discovery metadata received: {len(json.dumps(metadata))} bytes", "DEBUG")
            
            # Validate required OAuth2 metadata fields
            required_fields = [
                "authorization_endpoint",
                "token_endpoint", 
                "response_types_supported",
                "code_challenge_methods_supported"
            ]
            
            for field in required_fields:
                if field not in metadata:
                    self.add_result("OAuth2 Discovery", False, 
                                  f"Missing required field: {field}")
                    return False
            
            # Validate endpoint URLs (accept both localhost and 127.0.0.1)
            authorize_endpoint = metadata["authorization_endpoint"]
            token_endpoint = metadata["token_endpoint"]
            
            # Check if endpoints point to correct paths on port 3003
            if not (":3003/authorize" in authorize_endpoint):
                self.add_result("OAuth2 Discovery", False, 
                              f"Wrong authorization_endpoint: {authorize_endpoint}")
                return False
                
            if not (":3003/token" in token_endpoint):
                self.add_result("OAuth2 Discovery", False, 
                              f"Wrong token_endpoint: {token_endpoint}")
                return False
            
            # Update our URLs to match what the server actually returns
            self.authorize_url = authorize_endpoint
            self.token_url = token_endpoint
            
            # Validate PKCE support
            if "S256" not in metadata.get("code_challenge_methods_supported", []):
                self.add_result("OAuth2 Discovery", False, 
                              "S256 PKCE method not supported")
                return False
            
            self.add_result("OAuth2 Discovery", True, 
                          "Discovery metadata valid and complete")
            return True
            
        except Exception as e:
            self.add_result("OAuth2 Discovery", False, f"Error: {e}")
            return False

    def test_authorize_endpoint(self) -> bool:
        """Test OAuth2 /authorize endpoint with PKCE"""
        self.log("🔐 Testing OAuth2 Authorization Endpoint...")
        
        # Generate PKCE challenge
        self.pkce = self.generate_pkce_challenge()
        self.log(f"Generated PKCE verifier: {self.pkce.verifier[:20]}...", "DEBUG")
        self.log(f"Generated PKCE challenge: {self.pkce.challenge}", "DEBUG")
        
        # Prepare authorization request parameters
        state = secrets.token_urlsafe(32)
        params = {
            "response_type": "code",
            "client_id": self.client_id,
            "redirect_uri": self.redirect_uri,
            "scope": " ".join(self.scopes),
            "state": state,
            "code_challenge": self.pkce.challenge,
            "code_challenge_method": self.pkce.method
        }
        
        try:
            # Make authorization request
            response = requests.get(self.authorize_url, params=params, 
                                  allow_redirects=False, timeout=10)
            
            self.log(f"Authorization request: HTTP {response.status_code}", "DEBUG")
            
            # For testing, we expect either:
            # 1. HTTP 302/303 redirect to redirect_uri with authorization code
            # 2. HTTP 200 with authorization form (if user consent required)
            # 3. HTTP 302/303 redirect with authorization code directly (auto-approval for testing)
            
            if response.status_code in [302, 303]:
                # Check redirect location
                location = response.headers.get("Location", "")
                self.log(f"Redirect location: {location}", "DEBUG")
                
                # Parse redirect URI and extract authorization code
                if location.startswith(self.redirect_uri):
                    parsed_url = urllib.parse.urlparse(location)
                    query_params = urllib.parse.parse_qs(parsed_url.query)
                    
                    if "code" in query_params:
                        self.authorization_code = query_params["code"][0]
                        self.log(f"Authorization code received: {self.authorization_code[:20]}...", "DEBUG")
                        
                        # Validate state parameter
                        returned_state = query_params.get("state", [None])[0]
                        if returned_state != state:
                            self.add_result("Authorize Endpoint", False, 
                                          "State parameter mismatch")
                            return False
                        
                        self.add_result("Authorize Endpoint", True, 
                                      "Authorization code generated successfully")
                        return True
                    elif "error" in query_params:
                        error = query_params["error"][0]
                        error_desc = query_params.get("error_description", [""])[0]
                        self.add_result("Authorize Endpoint", False, 
                                      f"OAuth2 error: {error} - {error_desc}")
                        return False
                    else:
                        self.add_result("Authorize Endpoint", False, 
                                      "No authorization code or error in redirect")
                        return False
                else:
                    self.add_result("Authorize Endpoint", False, 
                                  f"Unexpected redirect location: {location}")
                    return False
            
            elif response.status_code == 200:
                # Check if this is an authorization form or direct approval
                content = response.text
                self.log(f"Authorization response content: {len(content)} bytes", "DEBUG")
                
                # For automated testing, we expect direct approval.
                # If there's a form, it means user interaction is required.
                if "authorization" in content.lower() or "consent" in content.lower():
                    self.add_result("Authorize Endpoint", False, 
                                  "User consent form returned (expected auto-approval for testing)")
                    return False
                else:
                    # Check if authorization code is in the response body
                    try:
                        data = response.json()
                        if "code" in data:
                            self.authorization_code = data["code"]
                            self.add_result("Authorize Endpoint", True, 
                                          "Authorization code in response body")
                            return True
                    except:
                        pass
                    
                    self.add_result("Authorize Endpoint", False, 
                                  "Unexpected response format")
                    return False
            
            else:
                self.add_result("Authorize Endpoint", False, 
                              f"Unexpected HTTP status: {response.status_code}")
                return False
                
        except Exception as e:
            self.add_result("Authorize Endpoint", False, f"Error: {e}")
            return False

    def test_token_endpoint(self) -> bool:
        """Test OAuth2 /token endpoint with authorization code exchange"""
        self.log("🎫 Testing OAuth2 Token Exchange...")
        
        if not self.authorization_code:
            self.add_result("Token Exchange", False, "No authorization code available")
            return False
        
        if not self.pkce:
            self.add_result("Token Exchange", False, "No PKCE verifier available")
            return False
        
        # Prepare token exchange request
        data = {
            "grant_type": "authorization_code",
            "code": self.authorization_code,
            "redirect_uri": self.redirect_uri,
            "client_id": self.client_id,
            "code_verifier": self.pkce.verifier
        }
        
        headers = {
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        try:
            self.log(f"Token exchange with code: {self.authorization_code[:20]}...", "DEBUG")
            self.log(f"Using PKCE verifier: {self.pkce.verifier[:20]}...", "DEBUG")
            
            response = requests.post(self.token_url, data=data, headers=headers, timeout=10)
            
            self.log(f"Token exchange request: HTTP {response.status_code}", "DEBUG")
            
            if response.status_code != 200:
                self.add_result("Token Exchange", False, 
                              f"HTTP {response.status_code}: {response.text}")
                return False
            
            token_response = response.json()
            self.log(f"Token response received: {len(json.dumps(token_response))} bytes", "DEBUG")
            
            # Validate token response structure
            required_fields = ["access_token", "token_type", "expires_in"]
            for field in required_fields:
                if field not in token_response:
                    self.add_result("Token Exchange", False, 
                                  f"Missing required field: {field}")
                    return False
            
            # Validate token type
            if token_response["token_type"].lower() != "bearer":
                self.add_result("Token Exchange", False, 
                              f"Unexpected token type: {token_response['token_type']}")
                return False
            
            # Store access token for further testing
            self.access_token = token_response["access_token"]
            self.log(f"Access token received: {self.access_token[:50]}...", "DEBUG")
            
            # Validate token format (should be JWT)
            if not self.access_token.count('.') == 2:
                self.add_result("Token Exchange", False, 
                              "Access token is not a valid JWT format")
                return False
            
            self.add_result("Token Exchange", True, 
                          f"Access token received successfully (expires in {token_response['expires_in']}s)")
            return True
            
        except Exception as e:
            self.add_result("Token Exchange", False, f"Error: {e}")
            return False

    def test_mcp_with_oauth2_token(self) -> bool:
        """Test MCP API access with OAuth2-obtained token"""
        self.log("🔧 Testing MCP API with OAuth2 Token...")
        
        if not self.access_token:
            self.add_result("MCP with OAuth2 Token", False, "No access token available")
            return False
        
        # Test MCP initialize with OAuth2 token
        try:
            headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.access_token}"
            }
            
            payload = {
                "jsonrpc": "2.0",
                "id": "oauth2-flow-test",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "experimental": {},
                        "sampling": {}
                    },
                    "clientInfo": {
                        "name": "OAuth2-Flow-Test-Client",
                        "version": "1.0.0"
                    }
                }
            }
            
            response = requests.post(self.server_url, json=payload, headers=headers, timeout=10)
            
            self.log(f"MCP initialize request: HTTP {response.status_code}", "DEBUG")
            
            if response.status_code != 200:
                self.add_result("MCP with OAuth2 Token", False, 
                              f"HTTP {response.status_code}")
                return False
            
            mcp_response = response.json()
            self.log(f"MCP response received: {len(json.dumps(mcp_response))} bytes", "DEBUG")
            
            # Validate MCP response
            if "result" not in mcp_response:
                self.add_result("MCP with OAuth2 Token", False, 
                              "No result in MCP response")
                return False
            
            result = mcp_response["result"]
            if "protocolVersion" not in result or "serverInfo" not in result:
                self.add_result("MCP with OAuth2 Token", False, 
                              "Invalid MCP initialize response")
                return False
            
            server_name = result["serverInfo"].get("name", "unknown")
            protocol_version = result["protocolVersion"]
            
            self.add_result("MCP with OAuth2 Token", True, 
                          f"MCP API accessible - Server: {server_name}, Protocol: {protocol_version}")
            return True
            
        except Exception as e:
            self.add_result("MCP with OAuth2 Token", False, f"Error: {e}")
            return False

    def test_invalid_authorization_code(self) -> bool:
        """Test token exchange with invalid authorization code"""
        self.log("❌ Testing Invalid Authorization Code Handling...")
        
        if not self.pkce:
            self.add_result("Invalid Auth Code", False, "No PKCE verifier available")
            return False
        
        # Use invalid authorization code
        invalid_code = "invalid_" + secrets.token_urlsafe(16)
        
        data = {
            "grant_type": "authorization_code",
            "code": invalid_code,
            "redirect_uri": self.redirect_uri,
            "client_id": self.client_id,
            "code_verifier": self.pkce.verifier
        }
        
        headers = {
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        try:
            response = requests.post(self.token_url, data=data, headers=headers, timeout=10)
            
            # Should return error response
            if response.status_code == 200:
                self.add_result("Invalid Auth Code", False, 
                              "Server accepted invalid authorization code")
                return False
            
            if response.status_code == 400:
                # Check for proper OAuth2 error response
                try:
                    error_response = response.json()
                    if "error" in error_response:
                        self.add_result("Invalid Auth Code", True, 
                                      f"Properly rejected invalid code: {error_response['error']}")
                        return True
                except:
                    pass
            
            self.add_result("Invalid Auth Code", True, 
                          f"Invalid code rejected with HTTP {response.status_code}")
            return True
            
        except Exception as e:
            self.add_result("Invalid Auth Code", False, f"Error: {e}")
            return False

    def test_invalid_pkce_verifier(self) -> bool:
        """Test token exchange with invalid PKCE verifier"""
        self.log("❌ Testing Invalid PKCE Verifier Handling...")
        
        if not self.authorization_code:
            self.add_result("Invalid PKCE Verifier", False, "No authorization code available")
            return False
        
        # Use invalid PKCE verifier
        invalid_verifier = "invalid_" + secrets.token_urlsafe(32)
        
        data = {
            "grant_type": "authorization_code",
            "code": self.authorization_code,
            "redirect_uri": self.redirect_uri,
            "client_id": self.client_id,
            "code_verifier": invalid_verifier
        }
        
        headers = {
            "Content-Type": "application/x-www-form-urlencoded"
        }
        
        try:
            response = requests.post(self.token_url, data=data, headers=headers, timeout=10)
            
            # Should return error response
            if response.status_code == 200:
                self.add_result("Invalid PKCE Verifier", False, 
                              "Server accepted invalid PKCE verifier")
                return False
            
            if response.status_code == 400:
                # Check for proper OAuth2 error response
                try:
                    error_response = response.json()
                    if "error" in error_response:
                        self.add_result("Invalid PKCE Verifier", True, 
                                      f"Properly rejected invalid verifier: {error_response['error']}")
                        return True
                except:
                    pass
            
            self.add_result("Invalid PKCE Verifier", True, 
                          f"Invalid verifier rejected with HTTP {response.status_code}")
            return True
            
        except Exception as e:
            self.add_result("Invalid PKCE Verifier", False, f"Error: {e}")
            return False

    def run_all_tests(self) -> bool:
        """Run the complete OAuth2 authorization flow test suite"""
        print("🧪 OAuth2 Authorization Flow Test Suite")
        print("=" * 60)
        
        try:
            # Setup
            self.cleanup()
            if not self.start_server():
                return False
            
            # Test 1: OAuth2 Discovery Metadata
            if not self.test_oauth2_discovery():
                return False
            
            # Test 2: Authorization Endpoint with PKCE
            if not self.test_authorize_endpoint():
                return False
            
            # Test 3: Token Exchange with Authorization Code
            if not self.test_token_endpoint():
                return False
            
            # Test 4: MCP API Access with OAuth2 Token
            if not self.test_mcp_with_oauth2_token():
                return False
            
            # Test 5: Error Handling - Invalid Authorization Code
            if not self.test_invalid_authorization_code():
                return False
            
            # Test 6: Error Handling - Invalid PKCE Verifier  
            if not self.test_invalid_pkce_verifier():
                return False
            
            return True
            
        except KeyboardInterrupt:
            self.log_warning("Test interrupted by user")
            return False
        except Exception as e:
            self.log_error(f"Unexpected error: {e}")
            return False

    def print_summary(self):
        """Print test results summary"""
        print()
        print("=" * 60)
        print("📊 OAuth2 Authorization Flow Test Results")
        print("=" * 60)
        
        for result in self.results:
            status = "✅ PASS" if result.passed else "❌ FAIL"
            print(f"{status}: {result.name} - {result.message}")
        
        print("-" * 60)
        print(f"Results: {self.tests_passed}/{self.tests_run} tests passed")
        
        if self.tests_passed == self.tests_run:
            print("🎉 All OAuth2 authorization flow tests passed!")
            print()
            print("📖 OAuth2 Flow Validation Complete:")
            print("  • Authorization endpoint (/authorize) working with PKCE")
            print("  • Token exchange endpoint (/token) working with code verification")
            print("  • Complete OAuth2 Authorization Code Flow validated")
            print("  • Error handling for invalid codes and verifiers confirmed")
            print("  • MCP API integration with OAuth2 tokens confirmed")
        else:
            print(f"💥 {self.tests_run - self.tests_passed} test(s) failed")


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="OAuth2 Authorization Flow Test Suite")
    parser.add_argument("--debug", action="store_true", help="Enable debug output")
    parser.add_argument("--no-cleanup", action="store_true", help="Don't stop server after tests")
    
    args = parser.parse_args()
    
    tester = OAuth2FlowTester(debug=args.debug)
    
    try:
        success = tester.run_all_tests()
        tester.print_summary()
        
        if not args.no_cleanup:
            tester.cleanup()
        else:
            print()
            print("🔄 Server endpoints (still running):")
            print("  • Direct MCP: http://localhost:3001/mcp")
            print("  • Proxy MCP: http://localhost:3002/mcp")
            print("  • OAuth2 Discovery: http://localhost:3003/.well-known/oauth-authorization-server")
            print("  • OAuth2 Authorize: http://localhost:3003/authorize")
            print("  • OAuth2 Token: http://localhost:3003/token")
            print("  • JWKS: http://localhost:3004/.well-known/jwks.json")
        
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\n⚠️ Test interrupted by user")
        tester.cleanup()
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Unexpected error: {e}")
        tester.cleanup()
        sys.exit(1)


if __name__ == "__main__":
    main()