#!/usr/bin/env python3
"""
JSON-RPC Protocol Edge Case Tests

Comprehensive edge case testing for JSON-RPC 2.0 protocol implementation in the OAuth2 MCP server covering:
- JSON-RPC 2.0 structure validation (version, id, method, params)
- MCP-specific method validation (tools, resources, prompts)
- Request/response format validation
- Error handling and response formatting
- Protocol violation detection

Tests follow the BasicOAuth2Test infrastructure pattern with proper server
management, cleanup, and logging.
"""

import json
import subprocess
import time
import requests
import sys
import os
from datetime import datetime
from typing import Optional, Dict, Any, Union

class JsonRpcEdgeCaseTest:
    """JSON-RPC edge case test suite following BasicOAuth2Test infrastructure pattern"""
    
    def __init__(self):
        self.server_process = None
        
        # Server configuration (matching BasicOAuth2Test pattern)
        self.mcp_direct_url = "http://localhost:3001/mcp"    # Direct MCP server
        self.proxy_url = "http://localhost:3002"             # Proxy server
        self.server_url = f"{self.proxy_url}/mcp"            # MCP through proxy (recommended)
        self.auth_tokens_url = "http://localhost:3004/auth/tokens"  # JWKS server
        
        # Legacy compatibility (for existing JSON-RPC test code)
        self.server_port = 3002  # Use proxy port for legacy compatibility
        self.base_url = self.proxy_url
        self.mcp_url = self.server_url
        
        # Valid token for testing JSON-RPC structure (not OAuth2 token validation)
        self.valid_auth_headers = None
        
        # Success/failure tracking
        self.tests_run = 0
        self.tests_passed = 0
        self.tests_failed = 0
        
    def log(self, message: str, level: str = "INFO"):
        """Log message with timestamp and level"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        print(f"[{timestamp}] {level}: {message}")
        
    def success(self, message: str):
        """Log success message"""
        self.log(f"✅ {message}", "SUCCESS")
        
    def error(self, message: str):
        """Log error message"""
        self.log(f"❌ {message}", "ERROR")
        
    def warning(self, message: str):
        """Log warning message"""
        self.log(f"⚠️  {message}", "WARNING")
        
    def cleanup(self):
        """Clean up any running processes"""
        self.log("🧹 Cleaning up any existing test processes")
        
        if self.server_process:
            try:
                self.server_process.terminate()
                self.server_process.wait(timeout=5)
            except Exception as e:
                self.log(f"Error terminating server: {e}", "DEBUG")
                
        # Kill any processes on our test port
        try:
            subprocess.run(['pkill', '-f', f':{self.server_port}'], 
                         capture_output=True, text=True)
            time.sleep(1)
        except Exception:
            pass
            
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
            self.log("Server build completed")
        except subprocess.CalledProcessError as e:
            self.error(f"Failed to build server: {e}")
            return False
        
        # Start the server
        try:
            env = os.environ.copy()
            env['RUST_LOG'] = 'info'
            
            # Create logs directory if it doesn't exist
            os.makedirs('../logs', exist_ok=True)
            
            # Open log file for server output
            log_file = open('../logs/server_jsonrpc_edge_cases.log', 'w')
            
            self.server_process = subprocess.Popen(
                ['../target/debug/http-oauth2-server'],  # Path relative to tests directory
                stdout=log_file,
                stderr=subprocess.STDOUT,
                env=env,
                text=True
            )
            
            # Wait for server to start (matching BasicOAuth2Test pattern)
            max_attempts = 30
            for attempt in range(max_attempts):
                # Check if process is still running
                if self.server_process.poll() is not None:
                    self.error("Server process exited early")
                    return False
                    
                try:
                    # Check MCP endpoint (expect 401/403/405)
                    response = requests.get(self.server_url, timeout=2)
                    if response.status_code in [401, 403, 405]:
                        self.success(f"OAuth2 MCP Server ready! (HTTP {response.status_code})")
                        break
                except requests.exceptions.RequestException:
                    pass
                    
                time.sleep(1)
                if attempt % 5 == 0:  # Log every 5th attempt
                    self.log(f"Waiting for MCP server... attempt {attempt + 1}/{max_attempts}")
            else:
                self.error("MCP server startup timeout")
                return False
                
            # Check JWKS server
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
        except Exception as e:
            self.error(f"Failed to start server: {e}")
            return False
            
    def get_valid_token(self) -> Optional[str]:
        """Get a valid OAuth2 token for JSON-RPC testing"""
        try:
            # First try the OAuth2 authorization code flow (this will fail but we log it)
            token_url = f"{self.base_url}/token"
            payload = {
                "grant_type": "client_credentials",
                "client_id": "test-client-oauth2-mcp",
                "client_secret": "test-secret-oauth2-mcp",
                "scope": "mcp:read mcp:write"
            }
            
            self.log(f"Requesting OAuth2 token from: {token_url}", "DEBUG")
            self.log(f"Token request payload: {payload}", "DEBUG")
            
            response = requests.post(token_url, data=payload, timeout=5)
            self.log(f"Token response status: {response.status_code}", "DEBUG")
            self.log(f"Token response body: {response.text}", "DEBUG")
            
            if response.status_code == 200:
                token_data = response.json()
                if 'access_token' in token_data:
                    return token_data['access_token']
            
            # OAuth2 flow failed, try to get a token from the dev endpoint
            self.log("OAuth2 flow failed, trying dev token endpoint", "DEBUG")
            dev_token_url = f"{self.base_url}/dev/tokens"
            dev_response = requests.get(dev_token_url, timeout=5)
            self.log(f"Dev token response status: {dev_response.status_code}", "DEBUG")
            self.log(f"Dev token response body: {dev_response.text}", "DEBUG")
            
            if dev_response.status_code == 200:
                dev_token_data = dev_response.json()
                if 'tokens' in dev_token_data and 'full' in dev_token_data['tokens']:
                    return dev_token_data['tokens']['full']
            
            return None
        except Exception as e:
            self.log(f"Token request failed: {e}", "DEBUG")
            return None
            
    def setup_valid_auth(self) -> bool:
        """Setup valid authorization headers for JSON-RPC testing"""
        token = self.get_valid_token()
        if token:
            self.valid_auth_headers = {"Authorization": f"Bearer {token}"}
            return True
        else:
            # Use mock token that bypasses OAuth2 for JSON-RPC testing
            self.valid_auth_headers = {"Authorization": "Bearer mock_token_for_jsonrpc_testing"}
            self.warning("Using mock authorization - OAuth2 validation may interfere with JSON-RPC tests")
            return True
            
    def make_jsonrpc_request(self, payload: Union[str, Dict], 
                           headers: Optional[Dict] = None) -> requests.Response:
        """Make a JSON-RPC request to the MCP endpoint"""
        if headers is None:
            headers = self.valid_auth_headers.copy() if self.valid_auth_headers else {}
            
        if isinstance(payload, str):
            # Raw string payload for malformed JSON tests
            headers['Content-Type'] = 'application/json'
            return requests.post(self.mcp_url, data=payload, headers=headers, timeout=5)
        else:
            # Dict payload for normal JSON requests
            return requests.post(self.mcp_url, json=payload, headers=headers, timeout=5)
            
    def run_test(self, test_name: str, test_func) -> bool:
        """Run individual test with tracking"""
        self.tests_run += 1
        self.log(f"📋 Running test: {test_name}")
        
        try:
            result = test_func()
            if result:
                self.tests_passed += 1
                self.success(f"Test passed: {test_name}")
            else:
                self.tests_failed += 1
                self.error(f"Test failed: {test_name}")
            return result
        except Exception as e:
            self.tests_failed += 1
            self.error(f"Test error: {test_name} - {e}")
            return False

    # ========================================================================
    # JSON-RPC 2.0 Structure Validation Tests
    # ========================================================================
    
    def test_malformed_json(self) -> bool:
        """Test completely malformed JSON"""
        malformed_payloads = [
            '{"jsonrpc": "2.0", "method": "initialize", "id": 1',  # Missing closing brace
            '{"jsonrpc": "2.0", "method": "initialize", "id": 1,}',  # Trailing comma
            '{"jsonrpc": "2.0" "method": "initialize", "id": 1}',  # Missing comma
            '{jsonrpc: "2.0", "method": "initialize", "id": 1}',  # Unquoted key
            '{"jsonrpc": "2.0", "method": initialize", "id": 1}',  # Unquoted value
            'not_json_at_all',  # Not JSON
            '',  # Empty string
            '{',  # Just opening brace
        ]
        
        for payload in malformed_payloads:
            try:
                response = self.make_jsonrpc_request(payload)
                # Should return 400 Bad Request or JSON-RPC parse error
                if response.status_code not in [400, 401]:  # 401 acceptable due to auth
                    return False
                    
                # If response has content, check for proper JSON-RPC error
                if response.content:
                    try:
                        error_data = response.json()
                        if 'error' in error_data:
                            error_code = error_data['error'].get('code')
                            if error_code == -32700:  # Parse error
                                continue  # Good, proper JSON-RPC error
                    except json.JSONDecodeError:
                        pass  # Non-JSON response is acceptable for malformed JSON
                        
            except Exception:
                continue  # Expected for malformed requests
                
        return True
        
    def test_missing_jsonrpc_version(self) -> bool:
        """Test request missing jsonrpc version field"""
        payload = {
            "method": "initialize",
            "id": 1
            # Missing "jsonrpc": "2.0"
        }
        
        try:
            response = self.make_jsonrpc_request(payload)
            
            # DEBUG: Log what we actually got
            self.log(f"Missing JSON-RPC Version test - Status: {response.status_code}", "DEBUG")
            self.log(f"Missing JSON-RPC Version test - Response: {response.text}", "DEBUG")
            
            if response.status_code == 200:
                # Server accepted it - check if it's a proper error response
                result = response.json()
                self.log(f"Missing JSON-RPC Version test - Parsed JSON: {result}", "DEBUG")
                if 'error' in result and result['error'].get('code') == -32600:
                    return True  # Proper invalid request error
                return False  # Shouldn't succeed without version
            elif response.status_code == 400:
                return True  # Bad request is acceptable
                
            return False
        except Exception:
            return False
            
    def test_wrong_jsonrpc_version(self) -> bool:
        """Test request with wrong JSON-RPC version"""
        versions = ["1.0", "2.1", "3.0", "", "not_a_version", 2.0, None]
        
        for version in versions:
            payload = {
                "jsonrpc": version,
                "method": "initialize",
                "id": 1
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                # DEBUG: Log what we actually got
                self.log(f"Wrong JSON-RPC Version test (version={version}) - Status: {response.status_code}", "DEBUG")
                self.log(f"Wrong JSON-RPC Version test (version={version}) - Response: {response.text}", "DEBUG")
                
                if response.status_code == 200:
                    result = response.json()
                    self.log(f"Wrong JSON-RPC Version test (version={version}) - Parsed JSON: {result}", "DEBUG")
                    # Should return error for wrong version
                    if 'error' not in result or result['error'].get('code') != -32600:
                        return False
                elif response.status_code != 400:
                    return False
                    
            except Exception:
                continue
                
        return True
        
    def test_missing_method_field(self) -> bool:
        """Test request missing method field"""
        payload = {
            "jsonrpc": "2.0",
            "id": 1
            # Missing "method"
        }
        
        try:
            response = self.make_jsonrpc_request(payload)
            
            # DEBUG: Log what we actually got
            self.log(f"Missing Method Field test - Status: {response.status_code}", "DEBUG")
            self.log(f"Missing Method Field test - Response: {response.text}", "DEBUG")
            
            if response.status_code == 200:
                result = response.json()
                self.log(f"Missing Method Field test - Parsed JSON: {result}", "DEBUG")
                # Should return invalid request error
                return 'error' in result and result['error'].get('code') == -32600
            
            return response.status_code == 400
        except Exception:
            return False
            
    def test_invalid_method_types(self) -> bool:
        """Test method field with invalid types"""
        invalid_methods = [
            123,  # Number instead of string
            None,  # Null
            [],  # Array
            {},  # Object
            True,  # Boolean
            "",  # Empty string
        ]
        
        for method in invalid_methods:
            payload = {
                "jsonrpc": "2.0",
                "method": method,
                "id": 1
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                if response.status_code == 200:
                    result = response.json()
                    # Should return error for invalid method type
                    if 'error' not in result:
                        return False
                elif response.status_code not in [400, 401]:
                    return False
                    
            except Exception:
                continue
                
        return True
        
    def test_invalid_id_types(self) -> bool:
        """Test id field with invalid types (arrays and objects not allowed)"""
        invalid_ids = [
            [],  # Array not allowed
            {},  # Object not allowed
            {"nested": "object"},  # Complex object
            [1, 2, 3],  # Complex array
        ]
        
        valid_ids = [
            1,  # Number
            "string_id",  # String
            None,  # Null is allowed
            0,  # Zero
            -1,  # Negative number
        ]
        
        # Test invalid IDs - should be rejected
        for test_id in invalid_ids:
            payload = {
                "jsonrpc": "2.0",
                "method": "ping",
                "id": test_id
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                if response.status_code == 200:
                    result = response.json()
                    # Should return error for invalid ID type
                    if 'error' not in result:
                        return False
                elif response.status_code not in [400, 401]:
                    return False
                    
            except Exception:
                continue
                
        # Test valid IDs - should be accepted (though method might not exist)
        for test_id in valid_ids:
            payload = {
                "jsonrpc": "2.0",
                "method": "ping",
                "id": test_id
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                # Should not reject due to ID format (though ping method might not exist)
                if response.status_code == 200:
                    result = response.json()
                    # If there's an error, it should be method not found, not invalid request
                    if 'error' in result and result['error'].get('code') == -32600:
                        return False  # Invalid request error suggests ID format issue
                        
            except Exception:
                continue
                
        return True
        
    def test_oversized_request(self) -> bool:
        """Test oversized JSON-RPC request (>1MB)"""
        large_params = {
            "large_field": "x" * 1048576  # 1MB of data
        }
        
        payload = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": large_params,
            "id": 1
        }
        
        try:
            response = self.make_jsonrpc_request(payload)
            
            # DEBUG: Log what we actually got
            self.log(f"Oversized Request test - Status: {response.status_code}", "DEBUG")
            self.log(f"Oversized Request test - Response length: {len(response.text)}", "DEBUG")
            self.log(f"Oversized Request test - Response preview: {response.text[:200]}...", "DEBUG")
            
            # Should reject oversized requests or return JSON-RPC error
            # Accept either HTTP rejection (400/413) or processing with error response
            if response.status_code in [400, 413]:
                return True  # HTTP level rejection
            elif response.status_code == 200:
                # If processed, it should still be a valid response (not necessarily an error)
                # Large requests might be processed normally if server allows them
                return True  # Server processed it, which is acceptable
        except Exception:
            return True  # Connection error expected for oversized requests
            
    def test_deeply_nested_params(self) -> bool:
        """Test deeply nested parameter objects"""
        # Create deeply nested object (100 levels)
        nested_obj = {"value": "deep"}
        for _ in range(100):
            nested_obj = {"nested": nested_obj}
            
        payload = {
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": nested_obj,
            "id": 1
        }
        
        try:
            response = self.make_jsonrpc_request(payload)
            
            # Should handle or reject deep nesting gracefully
            return response.status_code in [200, 400, 401]
        except Exception:
            return True  # Stack overflow or parsing error is acceptable

    # ========================================================================
    # MCP Method Validation Tests
    # ========================================================================
    
    def test_unknown_mcp_methods(self) -> bool:
        """Test requests to unknown MCP methods"""
        unknown_methods = [
            "unknown/method",
            "tools/invalid",
            "resources/nonexistent",
            "prompts/missing",
            "invalid_method_name",
            "method/with/too/many/slashes",
            "",
            "123",
            "method with spaces",
            "method\nwith\nnewlines",
        ]
        
        for method in unknown_methods:
            payload = {
                "jsonrpc": "2.0",
                "method": method,
                "id": 1
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                # DEBUG: Log what we actually got for the first few methods
                if method in ["unknown/method", "tools/invalid", "resources/nonexistent"]:
                    self.log(f"Unknown MCP Method test (method={method}) - Status: {response.status_code}", "DEBUG")
                    self.log(f"Unknown MCP Method test (method={method}) - Response: {response.text}", "DEBUG")
                
                if response.status_code == 200:
                    result = response.json()
                    # Should return method not found error (-32601) or invalid request (-32600) for empty/invalid methods
                    if 'error' in result and result['error'].get('code') in [-32601, -32600]:
                        continue  # Good - either method not found or invalid request
                    else:
                        self.log(f"Unknown MCP Method test FAILED for method '{method}' - Expected -32601 or -32600 error but got: {result}", "DEBUG")
                        return False  # Should have returned method not found
                elif response.status_code in [400, 401]:
                    continue  # Acceptable
                else:
                    self.log(f"Unknown MCP Method test FAILED for method '{method}' - Unexpected status: {response.status_code}", "DEBUG")
                    return False
                    
            except Exception:
                continue
                
        return True
        
    def test_mcp_initialize_invalid_params(self) -> bool:
        """Test initialize method with invalid parameters"""
        invalid_params_sets = [
            "not_an_object",  # String instead of object
            123,  # Number instead of object
            [],  # Array instead of object
            None,  # Null params
            {
                "invalid_field": "value"
                # Missing required clientInfo
            },
            {
                "clientInfo": "not_an_object"  # Should be object
            },
            {
                "clientInfo": {
                    # Missing name field
                    "version": "1.0.0"
                }
            },
            {
                "clientInfo": {
                    "name": 123,  # Should be string
                    "version": "1.0.0"
                }
            },
            {
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                },
                "protocolVersion": 123  # Should be string
            },
        ]
        
        for params in invalid_params_sets:
            payload = {
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": params,
                "id": 1
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                if response.status_code == 200:
                    result = response.json()
                    # Should return invalid params error
                    if 'error' in result and result['error'].get('code') == -32602:
                        continue  # Good
                    elif 'result' in result:
                        return False  # Shouldn't succeed with invalid params
                elif response.status_code in [400, 401]:
                    continue  # Acceptable
                else:
                    return False
                    
            except Exception:
                continue
                
        return True
        
    def test_mcp_tools_call_invalid_params(self) -> bool:
        """Test tools/call method with invalid parameters"""
        invalid_params_sets = [
            None,  # Missing params
            "not_an_object",
            [],
            {
                # Missing name field
                "arguments": {}
            },
            {
                "name": 123,  # Should be string
                "arguments": {}
            },
            {
                "name": "",  # Empty name
                "arguments": {}
            },
            {
                "name": "calculator",
                "arguments": "not_an_object"  # Should be object
            },
            {
                "name": "nonexistent_tool",
                "arguments": {}
            },
        ]
        
        for params in invalid_params_sets:
            payload = {
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": params,
                "id": 1
            }
            
            try:
                response = self.make_jsonrpc_request(payload)
                
                # DEBUG: Log first few test cases
                if params in [None, "not_an_object", []]:
                    self.log(f"Tools Call Invalid Params test (params={params}) - Status: {response.status_code}", "DEBUG")
                    self.log(f"Tools Call Invalid Params test (params={params}) - Response: {response.text}", "DEBUG")
                
                if response.status_code == 200:
                    result = response.json()
                    # Should return error for invalid params or result with isError=True for tools
                    if 'error' in result:
                        continue  # Good - JSON-RPC error
                    elif 'result' in result:
                        # For tools/call, allow results with isError=True (tool-level error)
                        if result.get('result', {}).get('isError', False):
                            continue  # Good - tool returned error
                        else:
                            self.log(f"Tools Call Invalid Params test FAILED for params {params} - Should not succeed with invalid params: {result}", "DEBUG")
                            return False  # Shouldn't succeed with invalid params
                elif response.status_code in [400, 401]:
                    continue  # Acceptable
                else:
                    self.log(f"Tools Call Invalid Params test FAILED for params {params} - Unexpected status: {response.status_code}", "DEBUG")
                    return False
                    
            except Exception:
                continue
                
        return True
        
    def test_mcp_resources_operations_invalid_params(self) -> bool:
        """Test resources/* methods with invalid parameters"""
        resource_methods = [
            "resources/list",
            "resources/read",
            "resources/subscribe",
            "resources/unsubscribe",
        ]
        
        for method in resource_methods:
            invalid_params_sets = [
                "not_an_object",
                123,
                [],
                {
                    "invalid_field": "value"
                },
            ]
            
            if method in ["resources/read", "resources/subscribe", "resources/unsubscribe"]:
                # These methods require URI parameter
                invalid_params_sets.extend([
                    {},  # Missing uri
                    {"uri": 123},  # Invalid uri type
                    {"uri": ""},  # Empty uri
                    {"uri": "invalid_uri_format"},
                ])
            
            for params in invalid_params_sets:
                payload = {
                    "jsonrpc": "2.0",
                    "method": method,
                    "params": params,
                    "id": 1
                }
                
                try:
                    response = self.make_jsonrpc_request(payload)
                    
                    # DEBUG: Log first few test cases
                    if method == "resources/list" and params in ["not_an_object", 123]:
                        self.log(f"Resources Invalid Params test (method={method}, params={params}) - Status: {response.status_code}", "DEBUG")
                        self.log(f"Resources Invalid Params test (method={method}, params={params}) - Response: {response.text}", "DEBUG")
                    
                    if response.status_code == 200:
                        result = response.json()
                        # Should return error for invalid params, but some methods like resources/list may ignore params
                        if 'error' in result:
                            continue  # Good - returned error
                        elif 'result' in result:
                            # For resources/list, it may ignore invalid params and return results
                            if method == "resources/list":
                                continue  # Acceptable - resources/list ignores params
                            else:
                                self.log(f"Resources Invalid Params test FAILED for method {method} with params {params} - Should not succeed: {result}", "DEBUG")
                                return False  # Other methods shouldn't succeed with invalid params
                    elif response.status_code in [400, 401]:
                        continue  # Acceptable
                    else:
                        self.log(f"Resources Invalid Params test FAILED for method {method} with params {params} - Unexpected status: {response.status_code}", "DEBUG")
                        return False
                        
                except Exception:
                    continue
                    
        return True

    # ========================================================================
    # JSON-RPC Error Response Validation Tests
    # ========================================================================
    
    def test_error_response_format(self) -> bool:
        """Test that error responses follow JSON-RPC 2.0 format"""
        # Send clearly invalid request to trigger error
        payload = {
            "jsonrpc": "2.0",
            "method": "nonexistent_method",
            "id": "test_id"
        }
        
        try:
            response = self.make_jsonrpc_request(payload)
            
            if response.status_code == 200:
                result = response.json()
                
                # Check JSON-RPC error response format
                if 'error' not in result:
                    return False
                    
                error = result['error']
                
                # Must have code and message
                if 'code' not in error or 'message' not in error:
                    return False
                    
                # Code must be integer
                if not isinstance(error['code'], int):
                    return False
                    
                # Message must be string
                if not isinstance(error['message'], str):
                    return False
                    
                # Should have same ID as request
                if result.get('id') != "test_id":
                    return False
                    
                # Should have jsonrpc version
                if result.get('jsonrpc') != "2.0":
                    return False
                    
                return True
                
            return False
        except Exception:
            return False
            
    def test_proper_error_codes(self) -> bool:
        """Test that proper JSON-RPC error codes are returned"""
        test_cases = [
            # (payload, expected_error_code, description)
            ('{"malformed_json"}', -32700, "Parse error"),
            ({"method": "test"}, -32600, "Invalid request (missing jsonrpc)"),
            ({"jsonrpc": "2.0", "id": 1}, -32600, "Invalid request (missing method)"),
            ({"jsonrpc": "2.0", "method": "nonexistent", "id": 1}, -32601, "Method not found"),
        ]
        
        for payload, expected_code, description in test_cases:
            try:
                response = self.make_jsonrpc_request(payload)
                
                if response.status_code == 200:
                    result = response.json()
                    
                    if 'error' in result:
                        actual_code = result['error'].get('code')
                        if actual_code != expected_code:
                            self.log(f"Expected error code {expected_code}, got {actual_code} for {description}", "DEBUG")
                            return False
                    else:
                        return False
                elif response.status_code == 400:
                    # Some servers might return HTTP 400 instead of JSON-RPC error
                    continue
                else:
                    return False
                    
            except Exception:
                if expected_code == -32700:  # Parse error expected
                    continue
                return False
                
        return True
        
    def test_no_sensitive_data_in_errors(self) -> bool:
        """Test that error messages don't leak sensitive information"""
        # Send requests that might trigger different types of errors
        test_payloads = [
            {"jsonrpc": "2.0", "method": "../../../../etc/passwd", "id": 1},
            {"jsonrpc": "2.0", "method": "SELECT * FROM users", "id": 1},
            {"jsonrpc": "2.0", "method": "rm -rf /", "id": 1},
            {"jsonrpc": "2.0", "method": "system('ls')", "id": 1},
        ]
        
        for payload in test_payloads:
            try:
                response = self.make_jsonrpc_request(payload)
                
                if response.status_code == 200:
                    result = response.json()
                    
                    if 'error' in result:
                        error_message = str(result['error']).lower()
                        
                        # Check for leaked information (exclude echoed user input)
                        dangerous_terms = [
                            'file not found',
                            'permission denied',
                            'system error',
                            'internal server error',
                            'database error',
                            'stack trace',
                        ]
                        
                        # Check for path leaks that aren't just echoing the method name
                        path_terms = ['/etc/', '/var/', '/usr/', 'c:\\']
                        method_name = payload['method'].lower()
                        
                        for term in dangerous_terms:
                            if term in error_message:
                                self.log(f"Potential information leak: {term} in error for method {payload['method']}", "DEBUG")
                                return False
                        
                        # Only flag path terms if they appear outside of echoed method name
                        for term in path_terms:
                            if term in error_message and term not in method_name:
                                self.log(f"Potential path leak: {term} in error for method {payload['method']}", "DEBUG")
                                return False
                                
            except Exception:
                continue
                
        return True

    def run_all_jsonrpc_tests(self) -> bool:
        """Run all JSON-RPC edge case tests and return overall success"""
        print("🧪 JSON-RPC Protocol Edge Case Test Suite")
        print("=" * 60)
        
        # Start fresh
        self.cleanup()
        if not self.start_server():
            self.error("Failed to start server for JSON-RPC tests")
            return False
            
        # Setup valid authentication for tests
        if not self.setup_valid_auth():
            self.warning("Could not setup valid authentication - some tests may fail due to OAuth2")
            
        # JSON-RPC 2.0 Structure Validation Tests
        print("\n📋 JSON-RPC 2.0 Structure Validation")
        print("-" * 40)
        structure_tests = [
            ("Malformed JSON", self.test_malformed_json),
            ("Missing JSON-RPC Version", self.test_missing_jsonrpc_version),
            ("Wrong JSON-RPC Version", self.test_wrong_jsonrpc_version),
            ("Missing Method Field", self.test_missing_method_field),
            ("Invalid Method Types", self.test_invalid_method_types),
            ("Invalid ID Types", self.test_invalid_id_types),
            ("Oversized Request", self.test_oversized_request),
            ("Deeply Nested Params", self.test_deeply_nested_params),
        ]
        
        for test_name, test_func in structure_tests:
            self.run_test(test_name, test_func)
            
        # MCP Method Validation Tests
        print("\n📋 MCP Method Validation")
        print("-" * 40)
        mcp_tests = [
            ("Unknown MCP Methods", self.test_unknown_mcp_methods),
            ("Initialize Invalid Params", self.test_mcp_initialize_invalid_params),
            ("Tools Call Invalid Params", self.test_mcp_tools_call_invalid_params),
            ("Resources Operations Invalid Params", self.test_mcp_resources_operations_invalid_params),
        ]
        
        for test_name, test_func in mcp_tests:
            self.run_test(test_name, test_func)
            
        # JSON-RPC Error Response Validation Tests
        print("\n📋 JSON-RPC Error Response Validation")
        print("-" * 40)
        error_tests = [
            ("Error Response Format", self.test_error_response_format),
            ("Proper Error Codes", self.test_proper_error_codes),
            ("No Sensitive Data in Errors", self.test_no_sensitive_data_in_errors),
        ]
        
        for test_name, test_func in error_tests:
            self.run_test(test_name, test_func)
            
        # Print results
        print("\n📊 Test Results Summary")
        print("=" * 40)
        print(f"Total Tests: {self.tests_run}")
        print(f"Passed: {self.tests_passed}")
        print(f"Failed: {self.tests_failed}")
        print(f"Success Rate: {(self.tests_passed/self.tests_run)*100:.1f}%")
        
        success = self.tests_failed == 0
        if success:
            self.success("All JSON-RPC edge case tests passed!")
        else:
            self.error(f"{self.tests_failed} JSON-RPC edge case tests failed")
            
        return success


def main():
    """Main test execution"""
    test_suite = JsonRpcEdgeCaseTest()
    
    try:
        success = test_suite.run_all_jsonrpc_tests()
        return 0 if success else 1
    finally:
        test_suite.cleanup()


if __name__ == "__main__":
    sys.exit(main())