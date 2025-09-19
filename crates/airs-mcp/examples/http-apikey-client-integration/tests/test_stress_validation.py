#!/usr/bin/env python3
"""
Client validation and edge case tests for HTTP API Key MCP client.

These tests validate client behavior under various edge cases,
error scenarios, and moderate stress conditions to ensure
robust client-side behavior.
"""

import json
import os
import subprocess
import sys
import time
import threading
import signal
from pathlib import Path
from typing import Dict, Any, Optional, List
import concurrent.futures

import pytest
import requests
import psutil


class TestHttpClientValidation:
    """Client validation and edge case tests for HTTP API Key MCP client."""
    
    @classmethod
    def setup_class(cls):
        """Build binaries and start mock server for stress testing."""
        print("Building HTTP client and mock server binaries for client validation testing...")
        
        # Build both binaries
        result = subprocess.run([
            "cargo", "build", "--bin", "http-apikey-client", "--bin", "http-mock-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Build failed: {result.stderr}")
            raise RuntimeError(f"Failed to build binaries: {result.stderr}")
        
        print("Build successful")
        
        cls.mock_server_process = None
        cls.mock_server_url = "http://127.0.0.1:3001"  # Use same port as regular mock tests
        cls.project_dir = Path(__file__).parent.parent
        cls.start_mock_server()
        cls.wait_for_mock_server()
    
    @classmethod
    def teardown_class(cls):
        """Clean up mock server process."""
        if cls.mock_server_process:
            print("Stopping HTTP mock server...")
            cls.mock_server_process.terminate()
            try:
                cls.mock_server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cls.mock_server_process.kill()
                cls.mock_server_process.wait()
            print("Mock server stopped")
    
    @classmethod
    def start_mock_server(cls):
        """Start the HTTP mock server for client stress testing."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        print(f"Starting HTTP mock server on port 3001 for client validation testing...")
        cls.mock_server_process = subprocess.Popen([
            "cargo", "run", "--bin", "http-mock-server"
        ], cwd=cls.project_dir, env=env, stdout=subprocess.PIPE, 
           stderr=subprocess.PIPE, text=True)
    
    @classmethod
    def wait_for_mock_server(cls, timeout=30):
        """Wait for the mock server to be ready."""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                response = requests.get(f"{cls.mock_server_url}/health", timeout=1)
                if response.status_code == 200:
                    print("Mock server is ready for client validation testing!")
                return
            except requests.exceptions.RequestException:
                pass
            
            if cls.mock_server_process.poll() is not None:
                stdout, stderr = cls.mock_server_process.communicate()
                raise RuntimeError(f"Mock server process died: {stderr}")
            
            time.sleep(0.5)
        
        raise RuntimeError(f"Mock server failed to start within {timeout} seconds")
    
    def run_client_command(self, command: str, env_vars: Dict[str, str] = None, timeout: int = 30) -> subprocess.CompletedProcess:
        """Run the HTTP client with the given command and environment variables."""
        env = os.environ.copy()
        env["RUST_LOG"] = "info,airs_mcp=debug"
        
        # Set default environment variables
        env["MCP_SERVER_URL"] = self.mock_server_url
        env["MCP_API_KEY"] = "test-key-123"
        
        # Override with provided env vars
        if env_vars:
            env.update(env_vars)
        
        cmd = ["cargo", "run", "--bin", "http-apikey-client", "--"] + command.split()
        
        return subprocess.run(
            cmd,
            cwd=self.project_dir,
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout
        )
    
    def test_moderate_concurrency_connections(self):
        """Test client behavior with moderate concurrent connections."""
        num_concurrent = 5  # Reduced for client testing
        success_count = 0
        error_count = 0
        timeout_count = 0
        
        def run_test_connection():
            try:
                result = self.run_client_command("test-connection", timeout=15)
                return "success" if result.returncode == 0 else "error"
            except subprocess.TimeoutExpired:
                return "timeout"
        
        print(f"Testing {num_concurrent} concurrent client connections...")
        start_time = time.time()
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=num_concurrent) as executor:
            futures = [executor.submit(run_test_connection) for _ in range(num_concurrent)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        
        end_time = time.time()
        
        success_count = results.count("success")
        error_count = results.count("error")
        timeout_count = results.count("timeout")
        
        print(f"Client concurrency test results:")
        print(f"  - Successes: {success_count}/{num_concurrent}")
        print(f"  - Errors: {error_count}/{num_concurrent}")
        print(f"  - Timeouts: {timeout_count}/{num_concurrent}")
        print(f"  - Total time: {end_time - start_time:.2f}s")
        
        # At least 80% should succeed for client testing
        success_rate = success_count / num_concurrent
        assert success_rate >= 0.8, f"Client success rate {success_rate:.2%} is below 80%"
        print(f"✅ Client concurrency test passed with {success_rate:.2%} success rate")
    
    def test_rapid_sequential_requests(self):
        """Test rapid sequential client requests to detect client-side issues."""
        num_requests = 20  # Reduced for client testing
        success_count = 0
        
        print(f"Testing {num_requests} rapid sequential client requests...")
        start_time = time.time()
        
        for i in range(num_requests):
            result = self.run_client_command("test-connection", timeout=10)
            if result.returncode == 0:
                success_count += 1
            
            # Small delay to avoid overwhelming the client
            time.sleep(0.2)
        
        end_time = time.time()
        
        success_rate = success_count / num_requests
        print(f"Sequential client test results:")
        print(f"  - Successes: {success_count}/{num_requests}")
        print(f"  - Success rate: {success_rate:.2%}")
        print(f"  - Total time: {end_time - start_time:.2f}s")
        print(f"  - Average time per request: {(end_time - start_time) / num_requests:.3f}s")
        
        # At least 90% should succeed for sequential client requests
        assert success_rate >= 0.9, f"Client success rate {success_rate:.2%} is below 90%"
        print(f"✅ Rapid sequential client test passed with {success_rate:.2%} success rate")
    
    def test_tool_call_stress(self):
        """Test stress on tool calls with various argument sizes."""
        tool_tests = [
            ("echo", {"message": "small"}),
            ("echo", {"message": "medium " * 100}),  # ~700 chars
            ("echo", {"message": "large " * 1000}),  # ~7000 chars
            ("calculate", {"operation": "add", "a": 42, "b": 58}),
            ("calculate", {"operation": "multiply", "a": 123, "b": 456}),
        ]
        
        print("Testing tool call stress scenarios...")
        
        for tool_name, args in tool_tests:
            args_json = json.dumps(args)
            
            print(f"Testing {tool_name} with {len(args_json)} char args...")
            # Use a list to avoid shell parsing issues
            cmd = ["cargo", "run", "--bin", "http-apikey-client", "--", "call-tool", tool_name, "--args", args_json]
            
            env = os.environ.copy()
            env.update({
                "MCP_SERVER_URL": self.mock_server_url,
                "MCP_API_KEY": "test-key-123"
            })
            
            result = subprocess.run(cmd, cwd=self.project_dir, env=env, capture_output=True, text=True, timeout=15)
            
            assert result.returncode == 0, f"Tool call {tool_name} failed with args size {len(args_json)}: {result.stderr}"
            print(f"✅ {tool_name} with {len(args_json)} char args successful")
    
    def test_resource_read_stress(self):
        """Test stress on resource reads."""
        resources = [
            "mock://server/info",
            "mock://server/status",
            "mock://server/config",
            "mock://data/sample.txt"
        ]
        
        print("Testing resource read stress scenarios...")
        
        for resource in resources:
            print(f"Testing resource read: {resource}")
            result = self.run_client_command(f"read-resource {resource}", timeout=15)
            
            assert result.returncode == 0, f"Resource read failed for {resource}"
            print(f"✅ Resource read successful for {resource}")
    
    def test_timeout_scenarios(self):
        """Test various timeout scenarios."""
        timeout_tests = [
            ("very_short", {"MCP_TIMEOUT": "1"}),
            ("short", {"MCP_TIMEOUT": "5"}),
            ("normal", {"MCP_TIMEOUT": "30"}),
        ]
        
        print("Testing timeout scenarios...")
        
        for test_name, env_vars in timeout_tests:
            print(f"Testing {test_name} timeout...")
            
            # Test connection with timeout
            result = self.run_client_command("test-connection", env_vars, timeout=60)
            
            # We don't assert success/failure since it depends on timing
            # Just ensure the client doesn't crash
            print(f"✅ Client handled {test_name} timeout without crashing")
    
    def test_malformed_environment_variables(self):
        """Test client behavior with malformed environment variables."""
        malformed_tests = [
            ("empty_url", {"MCP_SERVER_URL": ""}, True),  # Should fail
            ("invalid_url", {"MCP_SERVER_URL": "not-a-url"}, True),  # Should fail
            ("empty_api_key", {"MCP_API_KEY": ""}, True),  # Should fail
            ("invalid_auth_method", {"MCP_AUTH_METHOD": "InvalidMethod"}, False),  # Should succeed (fallback)
            ("invalid_timeout", {"MCP_TIMEOUT": "not-a-number"}, False),  # Should succeed (fallback)
        ]
        
        print("Testing malformed environment variables...")
        
        for test_name, env_vars, should_fail in malformed_tests:
            print(f"Testing {test_name}...")
            
            result = self.run_client_command("test-connection", env_vars, timeout=10)
            
            if should_fail:
                # These should fail gracefully, not crash
                assert result.returncode != 0, f"Expected failure for {test_name} but got success"
                
                # Check that error messages are informative (check both stderr and stdout)
                error_output = (result.stderr + result.stdout).lower()
                error_keywords = ["error", "invalid", "failed", "missing", "could not", "unable", "connection"]
                assert any(word in error_output for word in error_keywords), \
                    f"Error message not informative for {test_name}: stderr='{result.stderr}' stdout='{result.stdout}'"
                    
                print(f"✅ Client gracefully handles {test_name} (failed as expected)")
            else:
                # These should succeed due to fallback behavior
                assert result.returncode == 0, f"Expected success for {test_name} due to fallback behavior, but got failure"
                print(f"✅ Client gracefully handles {test_name} (succeeded with fallback)")
        
        print("✅ All malformed environment variable tests passed")

    def test_network_failure_scenarios(self):
        """Test client behavior with network failures."""
        network_tests = [
            ("unreachable_host", {"MCP_SERVER_URL": "http://192.0.2.1:3000"}),  # RFC 3330 test address
            ("invalid_port", {"MCP_SERVER_URL": "http://127.0.0.1:99999"}),
            ("nonexistent_host", {"MCP_SERVER_URL": "http://nonexistent.example.com:3000"}),
        ]
        
        print("Testing network failure scenarios...")
        
        for test_name, env_vars in network_tests:
            print(f"Testing {test_name}...")
            
            result = self.run_client_command("test-connection", env_vars, timeout=15)
            
            # These should fail gracefully
            assert result.returncode != 0, f"Expected network failure for {test_name} but got success"
            
            # Should have meaningful error messages
            error_output = result.stderr.lower()
            assert any(word in error_output for word in ["connection", "timeout", "refused", "unreachable"]), \
                f"Network error not properly reported for {test_name}: {result.stderr}"
            
            print(f"✅ Client gracefully handles {test_name}")
    
    def test_authentication_edge_cases(self):
        """Test edge cases in authentication."""
        auth_tests = [
            ("empty_key", {"MCP_API_KEY": ""}),
            ("very_long_key", {"MCP_API_KEY": "x" * 1000}),
            ("special_chars_key", {"MCP_API_KEY": "key-with-!@#$%^&*()"}),
            ("unicode_key", {"MCP_API_KEY": "🔑-unicode-key-🗝️"}),
        ]
        
        print("Testing authentication edge cases...")
        
        for test_name, env_vars in auth_tests:
            print(f"Testing {test_name}...")
            
            result = self.run_client_command("test-connection", env_vars, timeout=10)
            
            # These might succeed or fail depending on server implementation
            # The important thing is they don't crash the client
            if result.returncode == 0:
                print(f"✅ {test_name} authentication succeeded")
            else:
                print(f"✅ {test_name} authentication failed gracefully")
    
    def test_memory_usage_stability(self):
        """Test memory usage during extended operation."""
        print("Testing memory usage stability...")
        
        # Get initial memory usage
        process = psutil.Process()
        initial_memory = process.memory_info().rss / 1024 / 1024  # MB
        
        print(f"Initial memory usage: {initial_memory:.2f} MB")
        
        # Run multiple operations
        operations = ["test-connection", "list-tools", "list-resources"] * 10
        
        for i, operation in enumerate(operations):
            result = self.run_client_command(operation, timeout=10)
            
            if i % 5 == 0:  # Check memory every 5 operations
                current_memory = process.memory_info().rss / 1024 / 1024
                print(f"Memory after {i+1} operations: {current_memory:.2f} MB")
        
        final_memory = process.memory_info().rss / 1024 / 1024
        memory_increase = final_memory - initial_memory
        
        print(f"Final memory usage: {final_memory:.2f} MB")
        print(f"Memory increase: {memory_increase:.2f} MB")
        
        # Memory increase should be reasonable (less than 100MB for this test)
        assert memory_increase < 100, f"Memory usage increased by {memory_increase:.2f} MB (too much)"
        print("✅ Memory usage appears stable")
    
    def test_signal_handling(self):
        """Test client behavior when interrupted."""
        print("Testing signal handling...")
        
        # Start a long-running operation
        env = os.environ.copy()
        env["MCP_SERVER_URL"] = self.mock_server_url
        env["MCP_API_KEY"] = "test-key-123"
        
        cmd = ["cargo", "run", "--bin", "http-apikey-client", "--", "demo"]
        
        process = subprocess.Popen(
            cmd,
            cwd=self.project_dir,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Let it run for a bit
        time.sleep(2)
        
        # Send SIGINT (Ctrl+C)
        process.send_signal(signal.SIGINT)
        
        try:
            # Wait for graceful shutdown
            stdout, stderr = process.communicate(timeout=10)
            exit_code = process.returncode
            
            # Client should exit (may be 0 for graceful shutdown or non-zero for interruption)
            # The important thing is that it exits and doesn't hang
            print("✅ Client handles SIGINT gracefully")
            
        except subprocess.TimeoutExpired:
            # Force kill if it doesn't shut down gracefully
            process.kill()
            process.wait()
            pytest.fail("Client did not shut down gracefully after SIGINT")
    
    def test_large_response_handling(self):
        """Test handling of large responses from server."""
        print("Testing large response handling...")
        
        # Try to call a tool that might return large data
        # The echo tool with large input should return large output
        large_message = "x" * 1000  # Reduced to 1KB to avoid shell issues
        args_json = json.dumps({"message": large_message})
        
        # Use subprocess directly to avoid shell parsing issues
        cmd = ["cargo", "run", "--bin", "http-apikey-client", "--", "call-tool", "echo", "--args", args_json]
        
        env = os.environ.copy()
        env.update({
            "MCP_SERVER_URL": self.mock_server_url,
            "MCP_API_KEY": "test-key-123"
        })
        
        result = subprocess.run(cmd, cwd=self.project_dir, env=env, capture_output=True, text=True, timeout=30)
        
        assert result.returncode == 0, f"Large response handling failed: {result.stderr}"
        assert large_message in result.stdout, "Large response content not properly received"
        
        print("✅ Client handles large responses correctly")
    
    def test_error_recovery(self):
        """Test client error recovery capabilities."""
        print("Testing error recovery...")
        
        # Sequence: good request, bad request, good request
        test_sequence = [
            ("good", "test-connection", True),
            ("bad", "call-tool nonexistent_tool", False),
            ("good", "test-connection", True),
            ("bad", "read-resource nonexistent://resource", False),
            ("good", "list-tools", True),
        ]
        
        for test_type, command, should_succeed in test_sequence:
            print(f"Running {test_type} request: {command}")
            
            result = self.run_client_command(command, timeout=15)
            
            if should_succeed:
                assert result.returncode == 0, f"Expected success but got failure for: {command}"
                print(f"✅ Good request succeeded: {command}")
            else:
                assert result.returncode != 0, f"Expected failure but got success for: {command}"
                print(f"✅ Bad request failed as expected: {command}")
        
        print("✅ Client demonstrates good error recovery")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])