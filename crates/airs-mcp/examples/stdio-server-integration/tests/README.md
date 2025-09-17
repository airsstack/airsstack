# STDIO MCP Integration Tests

This directory contains Python test suites for the STDIO MCP server integration example.

## Test Files

### `test_stdio_basic.py`
Basic functionality test that verifies core MCP operations:
- Ping/pong communication
- Initialize handshake
- Tools list retrieval

**Usage:**
```bash
python3 test_stdio_basic.py
```

### `test_stdio_integration.py`
Comprehensive integration test suite covering:
- All MCP protocol methods (ping, initialize, tools, resources, prompts)
- JSON-RPC validation
- Error handling
- Edge cases

**Usage:**
```bash
python3 test_stdio_integration.py [--debug] [--server-path PATH]
```

**Options:**
- `--debug`: Enable verbose debug logging
- `--server-path PATH`: Specify custom path to stdio-server binary

### `test_stdio_comprehensive.py`
Advanced test suite with performance testing and comprehensive validation:
- Multiple ID types testing
- Complete workflows (list → call, list → read)
- Error condition testing
- Performance benchmarking
- Invalid request handling

**Usage:**
```bash
python3 test_stdio_comprehensive.py [--verbose] [--server-path PATH]
```

**Options:**
- `--verbose`: Enable detailed operation logging
- `--server-path PATH`: Specify custom path to stdio-server binary

## Running Tests

### Prerequisites

1. **Build the STDIO server:**
   ```bash
   cargo build --example stdio-server-integration
   ```

2. **Ensure Python 3.6+ is available**

### Quick Test
```bash
# Run basic functionality test
python3 test_stdio_basic.py
```

### Full Test Suite
```bash
# Run comprehensive integration tests
python3 test_stdio_integration.py --debug

# Run advanced comprehensive tests with performance metrics
python3 test_stdio_comprehensive.py --verbose
```

### Custom Server Path
If the server binary is in a custom location:
```bash
python3 test_stdio_integration.py --server-path /path/to/stdio-server
```

## Test Architecture

### Transport Method
The tests use **subprocess communication** with the STDIO server:
1. **Request**: JSON-RPC formatted request sent to stdin
2. **Processing**: Server processes request and generates response
3. **Response**: JSON-RPC formatted response received from stdout
4. **Validation**: Response structure and content validated

### Key Differences from HTTP Tests
- **One-shot communication**: Each test spawns a new server process
- **No persistent connection**: Server exits after each request
- **Direct STDIO**: No HTTP layer, pure JSON-RPC over stdin/stdout
- **Process management**: Tests handle server lifecycle automatically

### Error Handling
Tests validate both success and error scenarios:
- **Valid requests**: Proper JSON-RPC responses
- **Invalid methods**: Method not found errors (-32601)
- **Malformed params**: Invalid params errors (-32602)
- **Server failures**: Process exit codes and stderr capture

## Expected Test Results

### Basic Test (`test_stdio_basic.py`)
```
🚀 Running Basic STDIO MCP Tests
📍 Server: target/debug/stdio-server
Testing ping...
✅ Ping test passed
Testing initialize...
✅ Initialize test passed
Testing tools/list...
✅ Tools list test passed (10 tools found)

🎉 All basic tests passed!
```

### Integration Test (`test_stdio_integration.py`)
```
🚀 Starting STDIO MCP Integration Test Suite
📍 Server binary: target/debug/stdio-server
ℹ️ 🧪 Test 1: Ping
✅ Ping test passed
ℹ️ 🧪 Test 2: Initialize
✅ Initialize test passed
...
📊 Test Summary:
   Total tests: 8
   Passed: 8
   Failed: 0

🎉 All tests passed! STDIO MCP server is working correctly.
```

### Comprehensive Test (`test_stdio_comprehensive.py`)
```
🚀 Running Comprehensive STDIO MCP Test Suite
📍 Server: target/debug/stdio-server

🧪 Ping Basic...
✅ Ping Basic passed (0.045s)
...

📊 COMPREHENSIVE TEST RESULTS
Total Tests: 11
Passed: 11
Failed: 0

🎉 ALL COMPREHENSIVE TESTS PASSED!
The STDIO MCP server is fully functional and robust.

Performance Summary:
  Total time: 1.234s
  Average per test: 0.112s
```

## Troubleshooting

### Server Binary Not Found
```
❌ Test setup failed: Could not find stdio-server binary
```
**Solution:** Build the server or specify the correct path:
```bash
cargo build --example stdio-server-integration
# or
python3 test_stdio_integration.py --server-path /custom/path/stdio-server
```

### Server Process Failures
```
❌ Server process failed with exit code 1
```
**Solution:** Check server logs and ensure proper request format. Use `--debug` flag for details.

### JSON Parse Errors
```
❌ Failed to parse JSON response
```
**Solution:** Server may be outputting logs to stdout. Set `STDIO_LOG_LEVEL=error` environment variable.

## Integration with CI/CD

These tests are designed for automated testing in CI/CD pipelines:

```bash
# CI script example
set -e

# Build server
cargo build --example stdio-server-integration

# Run tests
python3 crates/airs-mcp/examples/stdio-server-integration/tests/test_stdio_basic.py
python3 crates/airs-mcp/examples/stdio-server-integration/tests/test_stdio_integration.py

echo "All STDIO MCP tests passed!"
```

## Test Coverage

The test suite covers:
- ✅ **Core Protocol**: ping, initialize methods
- ✅ **Tools**: list, call operations with math providers
- ✅ **Resources**: list, read operations with filesystem provider
- ✅ **Prompts**: list operations with code review provider
- ✅ **Error Handling**: Invalid methods, malformed requests
- ✅ **JSON-RPC Compliance**: Proper request/response format
- ✅ **Performance**: Basic timing and throughput testing
- ✅ **Edge Cases**: Different ID types, empty responses

This comprehensive coverage ensures the STDIO MCP server implementation is robust and production-ready.