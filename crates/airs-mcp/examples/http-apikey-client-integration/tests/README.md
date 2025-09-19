# HTTP API Key Client Integration Tests

This directory contains comprehensive integration tests for the HTTP API Key MCP client example.

## Overview

The test suite validates:
- ✅ HTTP MCP client with mock server integration
- ✅ All three API key authentication methods (X-API-Key header, Authorization Bearer, query parameter)
- ✅ Complete MCP protocol operations (initialize, tools/list, tools/call, resources/list, resources/read)
- ✅ Client-server communication with Phase 4.4 production server
- ✅ Error handling and timeout scenarios
- ✅ Stress testing and concurrent request handling

## Quick Start

Run all tests with the provided script:

```bash
./tests/run_tests.sh
```

This script will:
1. Create a Python virtual environment (`tests/venv/`)
2. Install required dependencies
3. Build the HTTP client and mock server binaries
4. Run the comprehensive test suite

## Advanced Test Options

The test suite now includes enhanced capabilities for comprehensive testing:

### Environment Validation
Check if your test environment is properly configured:

```bash
./tests/validate_environment.py
```

### Test Suite Selection
Run specific test suites:

```bash
# Run only mock server tests
./tests/run_tests.sh --suite mock

# Run only production server tests
./tests/run_tests.sh --suite production

# Run only stress tests
./tests/run_tests.sh --suite stress

# Show all options
./tests/run_tests.sh --help
```

### Comprehensive Test Reporting
Get detailed test reports with metrics and analysis:

```bash
# Run with comprehensive reporting
./tests/run_tests.sh --comprehensive

# Run specific suite with reporting
./tests/run_tests.sh --suite stress --comprehensive
```

The comprehensive runner provides:
- ✅ **Detailed Results**: Pass/fail status with timing for each test
- 📊 **Success Rate Metrics**: Overall and per-suite success percentages
- 🔍 **Error Analysis**: Detailed error messages and debugging information
- 📋 **JSON Reports**: Machine-readable test results saved to `test_report.json`
- 🚀 **Authentication Variations**: Tests all auth methods automatically

## Manual Testing Setup

If you prefer manual setup:

```bash
# Create virtual environment
python3 -m venv tests/venv
source tests/venv/bin/activate

# Install dependencies
pip install -r tests/requirements.txt

# Run tests against mock server
pytest tests/test_http_client_mock_integration.py -v

# Run tests against production server (Phase 4.4)
pytest tests/test_http_client_production_integration.py -v

# Run stress tests
pytest tests/test_stress_validation.py -v
```

## Test Structure

### `test_http_client_mock_integration.py`
Main integration test file for client-mock server interaction:

- **Mock Server Lifecycle**: Automated mock server startup/shutdown
- **Authentication Tests**: Validates all three API key methods with mock server
- **MCP Protocol Tests**: Tests client MCP operations against mock responses
- **Error Handling**: Tests timeout, authentication failures, and protocol errors

### `test_http_client_production_integration.py`
Integration tests against the Phase 4.4 production server:

- **Production Server Tests**: Validates client against real HTTP server
- **Cross-Authentication**: Tests authentication compatibility
- **Real Tool Execution**: Tests actual tool calls and resource access

### `test_stress_validation.py`
Stress testing and performance validation:

- **Concurrent Requests**: Multiple simultaneous client connections
- **Authentication Load**: High-volume authentication testing
- **Timeout Scenarios**: Network delay and timeout handling
- **Resource Limits**: Memory and connection limit testing

## Environment Variables

The tests support these environment variables:

- `MCP_API_KEY`: API key for authentication (default: `test-key-123`)
- `MCP_SERVER_URL`: Server URL for production tests (default: `http://127.0.0.1:3000`)
- `MOCK_SERVER_URL`: Mock server URL (default: `http://127.0.0.1:3001`)
- `PRODUCTION_SERVER_URL`: Production server URL for Phase 4.4 tests
- `TEST_TIMEOUT`: Individual test timeout in seconds (default: 30)
- `STRESS_TEST_REQUESTS`: Number of requests for stress testing (default: 100)

## Test Configuration

### Mock Server Tests
- **Target**: Lightweight Axum mock server (port 3001)
- **Purpose**: Fast, reliable testing of client functionality
- **Coverage**: Complete MCP protocol simulation

### Production Server Tests  
- **Target**: Phase 4.4 HTTP API Key server (port 3000)
- **Purpose**: Real-world integration validation
- **Coverage**: Actual MCP server interaction

## Running Specific Test Categories

```bash
# Mock server integration only
pytest tests/test_http_client_mock_integration.py -v

# Production server integration only
pytest tests/test_http_client_production_integration.py -v

# Authentication tests only
pytest -k "auth" -v

# Stress tests only
pytest tests/test_stress_validation.py -v

# With timeout control
pytest --timeout=60 tests/ -v
```

## Test Results

All tests generate detailed reports:
- ✅ **Pass/Fail Status**: Clear test outcome indicators
- 📊 **Performance Metrics**: Response times and throughput
- 🔍 **Debug Information**: Detailed logs for troubleshooting
- 📈 **Coverage Reports**: Test coverage analysis

## Integration with CI/CD

These tests are designed to work in automated environments:
- Automated server lifecycle management
- Environment-based configuration
- JUnit XML output for CI systems
- Comprehensive error reporting