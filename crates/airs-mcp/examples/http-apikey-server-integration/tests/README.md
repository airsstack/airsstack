# HTTP API Key Integration Tests

This directory contains comprehensive integration tests for the HTTP API Key MCP server example.

## Overview

The test suite validates:
- ✅ All three API key authentication methods (X-API-Key header, Authorization Bearer, query parameter)
- ✅ Complete MCP protocol operations (tools/list, tools/call, resources/list, resources/read)
- ✅ Error handling and edge cases
- ✅ Concurrent request handling
- ✅ End-to-end workflow testing

## Quick Start

Run all tests with the provided script:

```bash
./tests/run_tests.sh
```

This script will:
1. Create a Python virtual environment (`tests/venv/`)
2. Install required dependencies
3. Build the HTTP server binary
4. Run the comprehensive test suite

## Manual Testing Setup

If you prefer manual setup:

```bash
# Create virtual environment
python3 -m venv tests/venv
source tests/venv/bin/activate

# Install dependencies
pip install -r tests/requirements.txt

# Run tests
pytest tests/test_http_apikey_integration.py -v
```

## Test Structure

### `test_http_apikey_integration.py`
Main integration test file with comprehensive test coverage:

- **Authentication Tests**: Validates all three API key methods
- **MCP Protocol Tests**: Tests core MCP operations
- **Error Handling Tests**: Validates proper error responses
- **Concurrency Tests**: Ensures server handles multiple requests
- **Full Workflow Test**: End-to-end scenario testing

### Key Test Features

1. **Automatic Server Management**: Tests automatically build and start the HTTP server
2. **Multiple Authentication Methods**: Validates X-API-Key header, Authorization Bearer, and query parameters
3. **Complete MCP Coverage**: Tests all implemented MCP methods
4. **Robust Error Testing**: Validates authentication failures and invalid requests
5. **Concurrency Testing**: Ensures thread-safe operation
6. **Comprehensive Logging**: Detailed output for debugging

## Test Configuration

The tests use the following configuration:
- **Server Port**: 3001 (to avoid conflicts with development instances)
- **API Keys**: `dev-key-123`, `test-key-456`, `demo-key-789`
- **Timeout**: 30 seconds for server startup, 10 seconds for requests
- **Logging**: `RUST_LOG=info,airs_mcp=debug`

## Expected Outputs

Successful test run includes:
```
✅ X-API-Key header authentication: 4 tools
✅ Authorization Bearer authentication: 4 tools  
✅ Query parameter authentication: 4 tools
✅ Missing API key properly rejected
✅ Invalid API key properly rejected
✅ tools/list: Found 4 tools including ['add', 'subtract', 'multiply', 'divide']
✅ tools/call add: Addition result: 10
✅ tools/call multiply: Multiplication result: 12
✅ resources/list: Found 3 resources including ['api-info.txt', 'server-config.json', 'README.md']
✅ resources/read: Successfully read 156 characters
✅ Invalid method properly rejected with JSON-RPC error
✅ Server handled 5 concurrent requests successfully
✅ Full workflow completed successfully
```

## Dependencies

The test suite requires:
- **Python 3.8+**
- **pytest>=7.4.0**: Test framework
- **requests>=2.31.0**: HTTP client library
- **pytest-asyncio>=0.21.0**: Async test support
- **pytest-timeout>=2.1.0**: Test timeout management
- **psutil>=5.9.0**: Process management
- **aiohttp>=3.8.0**: Alternative HTTP client

## Troubleshooting

### Common Issues

1. **Server Won't Start**: 
   - Check if port 3001 is available
   - Ensure the binary builds successfully with `cargo build --bin http-apikey-server`

2. **Authentication Failures**:
   - Verify API keys match the configured values in `config.rs`
   - Check request headers are properly formatted

3. **Test Timeouts**:
   - Increase timeout values if running on slower systems
   - Check server logs for startup issues

### Debug Mode

Run tests with extra debugging:

```bash
RUST_LOG=debug pytest tests/test_http_apikey_integration.py -v -s
```

### Manual Server Testing

Start the server manually for debugging:

```bash
cargo run --bin http-apikey-server -- --port 3001
```

Then test with curl:

```bash
# Test authentication
curl -H "X-API-Key: dev-key-123" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
     http://127.0.0.1:3001/mcp
```

## Integration with CI/CD

The test suite is designed to be CI/CD friendly:
- All dependencies are pinned in `requirements.txt`
- Tests clean up server processes automatically
- Exit codes properly indicate success/failure
- Comprehensive logging for debugging failures

Add to your CI pipeline:

```yaml
- name: Run HTTP API Key Integration Tests
  run: |
    cd crates/airs-mcp/examples/http-apikey-server-integration
    ./tests/run_tests.sh
```