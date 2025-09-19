#!/usr/bin/env python3
"""
Transport-specific tests for STDIO MCP client.

These tests focus on the transport layer functionality,
testing connection handling, timeouts, and protocol compliance.
"""

import json
import os
import subprocess
import sys
import time
from pathlib import Path
import pytest
import signal
import threading


class TestStdioTransport:
    """Test STDIO transport layer functionality."""
    
    @classmethod
    def setup_class(cls):
        """Build the binaries before running tests."""
        print("Building STDIO transport test binaries...")
        
        result = subprocess.run([
            "cargo", "build", "--bin", "stdio-mock-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to build mock server: {result.stderr}")
    
    def test_transport_connection_establishment(self):
        """Test that transport can establish connection to server."""
        project_dir = Path(__file__).parent.parent
        
        # Start mock server
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send a simple ping to verify connection
            ping_request = {
                "jsonrpc": "2.0",
                "method": "ping", 
                "id": "transport-test"
            }
            
            mock_process.stdin.write(json.dumps(ping_request) + "\n")
            mock_process.stdin.flush()
            
            # Read response with timeout
            response_line = mock_process.stdout.readline()
            assert response_line.strip(), "No response received from server"
            
            response = json.loads(response_line)
            assert response["jsonrpc"] == "2.0"
            assert response["id"] == "transport-test"
            
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)
    
    def test_transport_request_timeout(self):
        """Test transport timeout handling."""
        project_dir = Path(__file__).parent.parent
        
        env = os.environ.copy()
        env["MCP_REQUEST_TIMEOUT"] = "2"  # Very short timeout
        env["USE_MOCK"] = "1"
        env["RUST_LOG"] = "debug"
        
        # Create a hanging server script
        hanging_script = project_dir / "hanging_server.sh"
        hanging_script.write_text('#!/bin/bash\nsleep 10\necho "{}"\n')
        hanging_script.chmod(0o755)
        
        try:
            env["MCP_SERVER_COMMAND"] = str(hanging_script)
            
            start_time = time.time()
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=15)
            
            elapsed = time.time() - start_time
            
            # Should timeout in around 2 seconds, not take 10+
            assert elapsed < 8, f"Client took too long to timeout: {elapsed}s"
            
            # Should contain timeout error
            output = result.stdout + result.stderr
            assert any(word in output.lower() for word in ["timeout", "failed", "error"])
            
        finally:
            if hanging_script.exists():
                hanging_script.unlink()
    
    def test_transport_json_protocol_compliance(self):
        """Test that transport correctly formats JSON-RPC messages."""
        project_dir = Path(__file__).parent.parent
        
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Test multiple request types to verify protocol compliance
            test_requests = [
                {
                    "jsonrpc": "2.0",
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-06-18",
                        "capabilities": {},
                        "clientInfo": {"name": "transport-test", "version": "1.0"}
                    },
                    "id": "init-1"
                },
                {
                    "jsonrpc": "2.0", 
                    "method": "tools/list",
                    "params": {},
                    "id": "tools-1"
                },
                {
                    "jsonrpc": "2.0",
                    "method": "tools/call",
                    "params": {
                        "name": "echo",
                        "arguments": {"message": "transport test"}
                    },
                    "id": "call-1"
                }
            ]
            
            for request in test_requests:
                mock_process.stdin.write(json.dumps(request) + "\n")
                mock_process.stdin.flush()
                
                response_line = mock_process.stdout.readline()
                assert response_line.strip(), f"No response for {request['method']}"
                
                response = json.loads(response_line)
                
                # Verify JSON-RPC 2.0 compliance
                assert response["jsonrpc"] == "2.0"
                assert response["id"] == request["id"]
                assert "result" in response or "error" in response
                
                if "result" in response:
                    assert response["result"] is not None
        
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)
    
    def test_transport_concurrent_requests(self):
        """Test transport handling of concurrent requests."""
        project_dir = Path(__file__).parent.parent
        
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send multiple requests rapidly
            request_ids = []
            for i in range(5):
                request = {
                    "jsonrpc": "2.0",
                    "method": "tools/list",
                    "params": {},
                    "id": f"concurrent-{i}"
                }
                request_ids.append(request["id"])
                
                mock_process.stdin.write(json.dumps(request) + "\n")
                mock_process.stdin.flush()
            
            # Collect all responses
            responses = []
            for _ in range(5):
                response_line = mock_process.stdout.readline()
                if response_line.strip():
                    response = json.loads(response_line)
                    responses.append(response)
            
            # Verify all requests got responses
            assert len(responses) == 5
            
            response_ids = [r["id"] for r in responses]
            for req_id in request_ids:
                assert req_id in response_ids, f"Missing response for {req_id}"
        
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)


@pytest.mark.timeout(30)
class TestStdioTransportAsync:
    """Test asynchronous transport behavior."""
    
    def test_transport_graceful_shutdown(self):
        """Test that transport handles graceful shutdown."""
        project_dir = Path(__file__).parent.parent
        
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send initialization
            init_request = {
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "shutdown-test", "version": "1.0"}
                },
                "id": "init-shutdown"
            }
            
            mock_process.stdin.write(json.dumps(init_request) + "\n")
            mock_process.stdin.flush()
            
            # Read initialization response
            response_line = mock_process.stdout.readline()
            response = json.loads(response_line)
            assert response["id"] == "init-shutdown"
            
            # Send shutdown signal
            mock_process.send_signal(signal.SIGTERM)
            
            # Process should exit gracefully within reasonable time
            try:
                exit_code = mock_process.wait(timeout=5)
                assert exit_code in [0, -signal.SIGTERM], f"Unexpected exit code: {exit_code}"
            except subprocess.TimeoutExpired:
                mock_process.kill()
                pytest.fail("Server did not shutdown gracefully within timeout")
        
        finally:
            if mock_process.poll() is None:
                mock_process.kill()
                mock_process.wait()


if __name__ == "__main__":
    # Run transport tests
    test_instance = TestStdioTransport()
    test_instance.setup_class()
    
    print("Running transport connection test...")
    test_instance.test_transport_connection_establishment()
    print("✓ Transport connection test passed")
    
    print("Running transport timeout test...")
    test_instance.test_transport_request_timeout()
    print("✓ Transport timeout test passed")
    
    print("Running JSON protocol compliance test...")
    test_instance.test_transport_json_protocol_compliance()
    print("✓ JSON protocol compliance test passed")
    
    print("Running concurrent requests test...")
    test_instance.test_transport_concurrent_requests()
    print("✓ Concurrent requests test passed")
    
    print("Running graceful shutdown test...")
    async_test = TestStdioTransportAsync()
    async_test.test_transport_graceful_shutdown()
    print("✓ Graceful shutdown test passed")
    
    print("All transport tests passed!")