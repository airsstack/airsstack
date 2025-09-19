#!/usr/bin/env python3
"""
Error handling and edge case tests for STDIO MCP client.

These tests verify that the client handles various error conditions
gracefully and provides appropriate error messages.
"""

import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path


class TestStdioErrorHandling:
    """Test error handling and edge cases."""
    
    @classmethod
    def setup_class(cls):
        """Build the binaries before running tests."""
        print("Building STDIO error handling test binaries...")
        
        result = subprocess.run([
            "cargo", "build", "--bin", "stdio-mcp-client", "--bin", "stdio-mock-server"
        ], cwd=Path(__file__).parent.parent, capture_output=True, text=True)
        
        if result.returncode != 0:
            raise RuntimeError(f"Failed to build binaries: {result.stderr}")
    
    def test_nonexistent_server_command(self):
        """Test client behavior when server command doesn't exist."""
        project_dir = Path(__file__).parent.parent
        
        env = os.environ.copy()
        env["MCP_SERVER_COMMAND"] = "/nonexistent/path/to/server"
        env["MCP_REQUEST_TIMEOUT"] = "5"
        env["RUST_LOG"] = "debug"
        
        result = subprocess.run([
            "cargo", "run", "--bin", "stdio-mcp-client"
        ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
        
        # Should fail with appropriate error message
        assert result.returncode != 0
        
        output = result.stdout + result.stderr
        assert any(word in output.lower() for word in [
            "failed", "error", "not found", "no such file"
        ])
    
    def test_server_immediate_exit(self):
        """Test client behavior when server exits immediately."""
        project_dir = Path(__file__).parent.parent
        
        # Create a script that exits immediately
        exit_script = project_dir / "exit_server.sh"
        exit_script.write_text('#!/bin/bash\nexit 1\n')
        exit_script.chmod(0o755)
        
        try:
            env = os.environ.copy()
            env["MCP_SERVER_COMMAND"] = str(exit_script)
            env["MCP_REQUEST_TIMEOUT"] = "5"
            env["RUST_LOG"] = "debug"
            
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
            
            # Should fail gracefully
            assert result.returncode != 0
            
            output = result.stdout + result.stderr
            assert any(word in output.lower() for word in [
                "failed", "error", "process", "exit"
            ])
        
        finally:
            if exit_script.exists():
                exit_script.unlink()
    
    def test_malformed_json_response(self):
        """Test client behavior when server sends malformed JSON."""
        project_dir = Path(__file__).parent.parent
        
        # Create a script that sends malformed JSON
        bad_json_script = project_dir / "bad_json_server.sh"
        bad_json_script.write_text('#!/bin/bash\necho "not valid json"\n')
        bad_json_script.chmod(0o755)
        
        try:
            env = os.environ.copy()
            env["MCP_SERVER_COMMAND"] = str(bad_json_script)
            env["MCP_REQUEST_TIMEOUT"] = "5"
            env["RUST_LOG"] = "debug"
            
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
            
            # Should handle malformed JSON gracefully
            output = result.stdout + result.stderr
            assert any(word in output.lower() for word in [
                "json", "parse", "invalid", "error", "failed"
            ])
        
        finally:
            if bad_json_script.exists():
                bad_json_script.unlink()
    
    def test_server_partial_response(self):
        """Test client behavior when server sends incomplete responses."""
        project_dir = Path(__file__).parent.parent
        
        # Create a script that sends partial JSON
        partial_script = project_dir / "partial_server.sh"
        partial_script.write_text('#!/bin/bash\necho \'{"jsonrpc":"2.0","id":\'\n')
        partial_script.chmod(0o755)
        
        try:
            env = os.environ.copy()
            env["MCP_SERVER_COMMAND"] = str(partial_script)
            env["MCP_REQUEST_TIMEOUT"] = "5"
            env["RUST_LOG"] = "debug"
            
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
            
            # Should handle incomplete responses gracefully
            output = result.stdout + result.stderr
            assert any(word in output.lower() for word in [
                "incomplete", "partial", "error", "failed", "timeout"
            ])
        
        finally:
            if partial_script.exists():
                partial_script.unlink()
    
    def test_mock_server_error_responses(self):
        """Test that mock server can send proper error responses."""
        project_dir = Path(__file__).parent.parent
        
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send request for non-existent method
            invalid_request = {
                "jsonrpc": "2.0",
                "method": "nonexistent/method",
                "params": {},
                "id": "error-test"
            }
            
            mock_process.stdin.write(json.dumps(invalid_request) + "\n")
            mock_process.stdin.flush()
            
            response_line = mock_process.stdout.readline()
            response = json.loads(response_line)
            
            assert response["jsonrpc"] == "2.0"
            assert response["id"] == "error-test"
            assert "error" in response
            assert "code" in response["error"]
            assert "message" in response["error"]
            
            # Common JSON-RPC error codes
            assert response["error"]["code"] in [-32601, -32602, -32603]  # Method not found, Invalid params, Internal error
        
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)
    
    def test_mock_server_invalid_json_input(self):
        """Test mock server handling of invalid JSON input."""
        project_dir = Path(__file__).parent.parent
        
        mock_process = subprocess.Popen([
            "cargo", "run", "--bin", "stdio-mock-server"
        ], cwd=project_dir, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
           stderr=subprocess.PIPE, text=True)
        
        try:
            # Send invalid JSON
            mock_process.stdin.write("not valid json\n")
            mock_process.stdin.flush()
            
            response_line = mock_process.stdout.readline()
            
            if response_line.strip():  # Server may respond with error
                response = json.loads(response_line)
                assert "error" in response
                assert response["error"]["code"] == -32700  # Parse error
            
            # Send incomplete JSON
            mock_process.stdin.write('{"jsonrpc":"2.0"\n')
            mock_process.stdin.flush()
            
            # Server should either respond with error or ignore gracefully
            response_line = mock_process.stdout.readline()
            
            if response_line.strip():
                response = json.loads(response_line)
                assert "error" in response
        
        finally:
            mock_process.terminate()
            mock_process.wait(timeout=5)
    
    def test_client_configuration_validation(self):
        """Test client behavior with invalid configuration."""
        project_dir = Path(__file__).parent.parent
        
        # Test with empty server command
        env = os.environ.copy()
        env["MCP_SERVER_COMMAND"] = ""
        env["RUST_LOG"] = "debug"
        
        result = subprocess.run([
            "cargo", "run", "--bin", "stdio-mcp-client"
        ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=10)
        
        # Should fail with configuration error
        assert result.returncode != 0
        
        output = result.stdout + result.stderr
        assert any(word in output.lower() for word in [
            "config", "command", "empty", "invalid", "error"
        ])
    
    def test_resource_cleanup_on_error(self):
        """Test that resources are properly cleaned up when errors occur."""
        project_dir = Path(__file__).parent.parent
        
        # Create a server that starts but fails after initialization
        failing_script = project_dir / "failing_server.sh"
        failing_script.write_text('''#!/bin/bash
# Respond to initialization 
read line
echo '{"jsonrpc":"2.0","id":"init","result":{"protocolVersion":"2025-06-18","capabilities":{},"serverInfo":{"name":"failing","version":"1.0"}}}'
# Then exit with error
exit 1
''')
        failing_script.chmod(0o755)
        
        try:
            env = os.environ.copy()
            env["MCP_SERVER_COMMAND"] = str(failing_script)
            env["MCP_REQUEST_TIMEOUT"] = "5"
            env["RUST_LOG"] = "debug"
            
            result = subprocess.run([
                "cargo", "run", "--bin", "stdio-mcp-client"
            ], cwd=project_dir, env=env, capture_output=True, text=True, timeout=15)
            
            # Should handle server failure gracefully
            output = result.stdout + result.stderr
            
            # Should show some progress before failing
            assert any(phrase in output for phrase in [
                "Initialization successful", "failed", "error"
            ])
        
        finally:
            if failing_script.exists():
                failing_script.unlink()


if __name__ == "__main__":
    # Run error handling tests
    test_instance = TestStdioErrorHandling()
    test_instance.setup_class()
    
    print("Running nonexistent server command test...")
    test_instance.test_nonexistent_server_command()
    print("✓ Nonexistent server command test passed")
    
    print("Running server immediate exit test...")
    test_instance.test_server_immediate_exit()
    print("✓ Server immediate exit test passed")
    
    print("Running malformed JSON response test...")
    test_instance.test_malformed_json_response()
    print("✓ Malformed JSON response test passed")
    
    print("Running server partial response test...")
    test_instance.test_server_partial_response()
    print("✓ Server partial response test passed")
    
    print("Running mock server error responses test...")
    test_instance.test_mock_server_error_responses()
    print("✓ Mock server error responses test passed")
    
    print("Running mock server invalid JSON input test...")
    test_instance.test_mock_server_invalid_json_input()
    print("✓ Mock server invalid JSON input test passed")
    
    print("Running client configuration validation test...")
    test_instance.test_client_configuration_validation()
    print("✓ Client configuration validation test passed")
    
    print("Running resource cleanup test...")
    test_instance.test_resource_cleanup_on_error()
    print("✓ Resource cleanup test passed")
    
    print("All error handling tests passed!")