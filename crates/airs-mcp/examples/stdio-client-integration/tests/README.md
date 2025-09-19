# STDIO MCP Client Integration Tests

This directory contains comprehensive tests for the STDIO MCP client integration example.

## Setup

### 1. Create Virtual Environment

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 2. Build Rust Binaries

```bash
cd ..
cargo build
cd tests
```

## Running Tests

### Quick Start

```bash
# Run all tests
./run_tests.sh

# Or run specific test suites
./run_tests.sh integration    # Integration tests only
./run_tests.sh transport      # Transport tests only  
./run_tests.sh error          # Error scenario tests only
./run_tests.sh pytest         # All tests with pytest
```

### Manual Execution

```bash
# Individual test files
python test_client_integration.py
python test_transport.py
python test_error_scenarios.py

# Using pytest
python -m pytest -v                    # All tests with verbose output
python -m pytest test_integration.py   # Specific test file
```

## Test Suites

### 1. Integration Tests (`test_client_integration.py`)
- **Purpose**: End-to-end functionality testing
- **Tests**: 3 test cases
- **Coverage**:
  - Client demo with mock server
  - Custom server command configuration  
  - Mock server standalone functionality

### 2. Transport Tests (`test_transport.py`)
- **Purpose**: Transport layer functionality
- **Tests**: 5 test cases
- **Coverage**:
  - Connection establishment
  - Request timeout handling
  - JSON-RPC protocol compliance
  - Concurrent request handling
  - Graceful shutdown

### 3. Error Scenario Tests (`test_error_scenarios.py`)
- **Purpose**: Error handling and edge cases
- **Tests**: 8 test cases
- **Coverage**:
  - Nonexistent server commands
  - Server immediate exit
  - Malformed JSON responses
  - Partial responses
  - Mock server error responses
  - Invalid JSON input
  - Configuration validation
  - Resource cleanup

## Dependencies

- `pytest>=7.4.0` - Test framework
- `pytest-asyncio>=0.21.0` - Async test support
- `pytest-timeout>=2.1.0` - Test timeout handling
- `requests>=2.31.0` - HTTP client (future use)
- `psutil>=5.9.0` - Process monitoring

## Test Output

All tests provide detailed output showing:
- ✅ Passed tests with descriptive messages
- 📊 Test execution summaries
- 🔍 Debug information for failures
- 📈 Coverage reports (when using pytest)

## Troubleshooting

### Common Issues

**Virtual environment not activated:**
```bash
source venv/bin/activate
```

**Rust binaries not built:**
```bash
cd .. && cargo build && cd tests
```

**Permission denied on run_tests.sh:**
```bash
chmod +x run_tests.sh
```

**Import errors:**
```bash
pip install -r requirements.txt
```

### Debug Mode

Enable debug logging for more detailed output:

```bash
RUST_LOG=debug ./run_tests.sh
```

## Integration with CI/CD

The test suite is designed to work in automated environments:

```bash
#!/bin/bash
# CI/CD script example
set -e

# Setup
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Build
cargo build

# Test
python -m pytest -v --timeout=30
```