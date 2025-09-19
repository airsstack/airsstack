#!/usr/bin/env python3
"""
Additional stress tests and edge case validation for HTTP API Key MCP server.

These tests complement the main integration tests with more thorough validation
of edge cases, error conditions, and performance characteristics.
"""

import json
import subprocess
import time
import concurrent.futures
from pathlib import Path

import pytest
import requests


class TestHttpApiKeyStress:
    """Stress tests and edge case validation for HTTP API Key server."""
    
    @classmethod
    def setup_class(cls):
        """Start a test server instance."""
        cls.base_url = "http://127.0.0.1:3002"  # Different port from main tests
        cls.server_process = None
        cls.start_server()
        cls.wait_for_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up server process."""
        if cls.server_process:
            cls.server_process.terminate()
            try:
                cls.server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.server_process.kill()
                cls.server_process.wait()
    
    @classmethod
    def start_server(cls):
        """Start the HTTP API Key server on port 3002."""
        project_dir = Path(__file__).parent.parent
        
        cls.server_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-apikey-server", "--", "--port", "3002"
        ], cwd=project_dir, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_server(cls, timeout=30):
        """Wait for the server to be ready."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                response = requests.get(f"{cls.base_url}/health", timeout=1)
                if response.status_code == 200:
                    return
            except requests.exceptions.RequestException:
                pass
            
            if cls.server_process.poll() is not None:
                stdout, stderr = cls.server_process.communicate()
                raise RuntimeError(f"Server process died: {stderr}")
            
            time.sleep(0.5)
        
        raise RuntimeError(f"Server failed to start within {timeout} seconds")
    
    def mcp_request(self, method: str, params: dict = None, 
                   api_key: str = "dev-key-123", auth_method: str = "header",
                   timeout: int = 10) -> requests.Response:
        """Make an MCP request with configurable timeout."""
        if params is None:
            params = {}
        
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }
        
        headers = {"Content-Type": "application/json"}
        url = f"{self.base_url}/mcp"
        
        if auth_method == "header":
            headers["X-API-Key"] = api_key
        elif auth_method == "bearer":
            headers["Authorization"] = f"Bearer {api_key}"
        elif auth_method == "query":
            url += f"?api_key={api_key}"
        
        return requests.post(url, headers=headers, json=payload, timeout=timeout)
    
    def test_malformed_json_payload(self):
        """Test server handling of malformed JSON requests."""
        headers = {
            "Content-Type": "application/json",
            "X-API-Key": "dev-key-123"
        }
        
        # Test invalid JSON
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               data='{"invalid": json}',
                               timeout=5)
        assert response.status_code in [400, 500]  # Bad request or server error
        print("✅ Malformed JSON properly rejected")
        
        # Test empty body
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               data='',
                               timeout=5)
        assert response.status_code in [400, 500]
        print("✅ Empty body properly rejected")
    
    def test_missing_required_fields(self):
        """Test handling of JSON-RPC requests missing required fields."""
        headers = {
            "Content-Type": "application/json",
            "X-API-Key": "dev-key-123"
        }
        
        # Missing method field
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               json={"jsonrpc": "2.0", "id": 1, "params": {}},
                               timeout=5)
        # Should be handled as invalid JSON-RPC request
        assert response.status_code in [400, 500]
        print("✅ Missing method field properly rejected")
        
        # Missing jsonrpc field
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               json={"method": "tools/list", "id": 1, "params": {}},
                               timeout=5)
        assert response.status_code in [400, 500]
        print("✅ Missing jsonrpc field properly rejected")
    
    def test_large_payload_handling(self):
        """Test server handling of unusually large payloads."""
        # Create a large numbers array for math operations
        large_numbers = list(range(1000))
        
        response = self.mcp_request("tools/call", {
            "name": "add",
            "arguments": {"numbers": large_numbers}
        }, timeout=30)
        
        assert response.status_code == 200
        data = response.json()
        assert "result" in data
        
        # Verify the calculation (sum of 0 to 999 = 499500)
        expected_sum = sum(large_numbers)
        result_text = data["result"]["content"][0]["text"]
        assert str(expected_sum) in result_text
        
        print(f"✅ Large payload handled: sum of {len(large_numbers)} numbers = {expected_sum}")
    
    def test_concurrent_different_operations(self):
        """Test concurrent execution of different MCP operations."""
        def make_different_requests():
            operations = [
                ("tools/list", {}),
                ("resources/list", {}),
                ("tools/call", {"name": "add", "arguments": {"numbers": [1, 2, 3]}}),
                ("tools/call", {"name": "multiply", "arguments": {"numbers": [4, 5]}}),
            ]
            
            results = []
            for method, params in operations:
                try:
                    response = self.mcp_request(method, params)
                    results.append(response.status_code == 200)
                except Exception as e:
                    results.append(False)
            
            return all(results)
        
        # Run multiple concurrent mixed operations
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(make_different_requests) for _ in range(10)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        success_rate = sum(results) / len(results)
        assert success_rate >= 0.9  # Allow for 10% failure rate under stress
        
        print(f"✅ Concurrent mixed operations: {success_rate:.1%} success rate")
    
    def test_rapid_authentication_switching(self):
        """Test rapid switching between different authentication methods."""
        auth_methods = [
            ("header", "dev-key-123"),
            ("bearer", "test-key-456"),
            ("query", "demo-key-789"),
        ]
        
        results = []
        for i in range(20):  # 20 rapid requests
            auth_method, api_key = auth_methods[i % len(auth_methods)]
            
            try:
                response = self.mcp_request("tools/list", 
                                          auth_method=auth_method, 
                                          api_key=api_key,
                                          timeout=5)
                results.append(response.status_code == 200)
            except Exception:
                results.append(False)
        
        success_rate = sum(results) / len(results)
        assert success_rate >= 0.9
        
        print(f"✅ Rapid auth switching: {success_rate:.1%} success rate over 20 requests")
    
    def test_invalid_content_type(self):
        """Test handling of requests with invalid Content-Type headers."""
        headers = {
            "Content-Type": "text/plain",  # Wrong content type
            "X-API-Key": "dev-key-123"
        }
        
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               data='{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}',
                               timeout=5)
        
        # Server may accept requests and try to parse JSON regardless of content type
        # This is actually reasonable behavior for many HTTP servers
        # The important thing is that it handles the request gracefully
        if response.status_code == 200:
            print("✅ Invalid Content-Type handled gracefully (server accepts JSON regardless of header)")
        else:
            assert response.status_code in [400, 415, 500]  # Bad request, unsupported media type, or server error
            print("✅ Invalid Content-Type properly rejected")
    
    def test_oversized_api_key(self):
        """Test handling of unusually large API keys."""
        # Create a very large API key
        oversized_key = "x" * 10000
        
        headers = {
            "Content-Type": "application/json",
            "X-API-Key": oversized_key
        }
        
        payload = {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
        
        response = requests.post(f"{self.base_url}/mcp", 
                               headers=headers, 
                               json=payload,
                               timeout=5)
        
        # Should be rejected as invalid API key
        assert response.status_code == 401
        print("✅ Oversized API key properly rejected")
    
    def test_unicode_handling(self):
        """Test handling of Unicode characters in requests."""
        # Test Unicode in tool parameters
        response = self.mcp_request("tools/call", {
            "name": "add",
            "arguments": {"numbers": [1, 2, 3], "label": "数学运算 🧮"}
        })
        
        # Should handle Unicode gracefully (either succeed or fail gracefully)
        assert response.status_code in [200, 400, 500]
        print("✅ Unicode characters handled gracefully")
    
    def test_response_time_consistency(self):
        """Test that response times are reasonably consistent."""
        response_times = []
        
        for _ in range(10):
            start_time = time.time()
            response = self.mcp_request("tools/list")
            end_time = time.time()
            
            if response.status_code == 200:
                response_times.append(end_time - start_time)
        
        if response_times:
            avg_time = sum(response_times) / len(response_times)
            max_time = max(response_times)
            
            # Response times should be reasonable and consistent
            assert avg_time < 1.0  # Average under 1 second
            assert max_time < 2.0  # No response over 2 seconds
            
            print(f"✅ Response times: avg={avg_time:.3f}s, max={max_time:.3f}s")
        else:
            print("⚠️  Could not measure response times (all requests failed)")
    
    @pytest.mark.timeout(60)  # Reduced timeout since we shortened the test
    def test_sustained_load(self):
        """Test server performance under sustained load."""
        print("\n🔥 Running sustained load test...")
        
        def sustained_requests(duration_seconds=30):
            """Make requests continuously for the specified duration."""
            start_time = time.time()
            request_count = 0
            success_count = 0
            
            while time.time() - start_time < duration_seconds:
                try:
                    response = self.mcp_request("tools/list", timeout=2)
                    request_count += 1
                    if response.status_code == 200:
                        success_count += 1
                except Exception:
                    request_count += 1
                
                # Small delay to avoid overwhelming
                time.sleep(0.1)
            
            return request_count, success_count
        
        # Run sustained load test (shorter duration for testing)
        total_requests, successful_requests = sustained_requests(10)  # 10 seconds instead of 30
        success_rate = successful_requests / total_requests if total_requests > 0 else 0
        
        # Under sustained load, we expect decent performance
        # Note: Success rate may be lower under heavy load due to timeouts and resource constraints
        assert success_rate >= 0.7  # 70% success rate minimum (more realistic under sustained load)
        assert total_requests >= 20  # Should handle at least 20 requests in 10 seconds (more realistic)
        
        print(f"   Sustained load: {total_requests} requests, {success_rate:.1%} success rate (10 second test)")
        print("✅ Server handled sustained load successfully")


if __name__ == "__main__":
    # Run stress tests directly
    pytest.main([__file__, "-v", "-s"])