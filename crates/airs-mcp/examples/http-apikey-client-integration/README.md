# HTTP API Key Client Integration Example

This example demonstrates comprehensive integration testing for HTTP API Key MCP (Model Context Protocol) client implementations.

## Overview

This integration example provides:

- 🔧 **Complete Test Infrastructure**: Python-based test suite with virtual environment management
- 🎯 **Multi-Target Testing**: Mock server and production server integration tests  
- 🚀 **Stress Testing**: High-concurrency, edge cases, and performance validation
- 📊 **Advanced Reporting**: Comprehensive test results with metrics and analysis
- 🔐 **Authentication Testing**: All API key authentication methods (header, bearer, query parameter)
- 🔄 **MCP Protocol Validation**: Complete protocol operation testing (tools, resources, connections)

## Quick Start

Run the complete test suite:

```bash
cd crates/airs-mcp/examples/http-apikey-client-integration/tests
./run_tests.sh
```

Or with comprehensive reporting:

```bash
./run_tests.sh --comprehensive
```

## Test Architecture

### Core Components

1. **HTTP Client** (`http-apikey-client`): Rust binary for MCP client operations
2. **Mock Server** (`http-mock-server`): Lightweight test server for controlled testing
3. **Production Integration**: Tests against Phase 4.4 HTTP server implementation
4. **Stress Testing**: Performance and edge case validation

### Test Suites

- **Mock Server Integration** (`test_http_client_mock_integration.py`)
  - Automated server lifecycle management
  - All authentication methods testing
  - Complete MCP protocol validation
  - Concurrent connection testing

- **Production Server Integration** (`test_http_client_production_integration.py`)
  - Real-world server compatibility
  - Phase 4.4 feature validation
  - Performance benchmarking
  - Production environment testing

- **Stress & Validation Testing** (`test_stress_validation.py`)
  - High concurrency (20+ simultaneous connections)
  - Memory usage monitoring
  - Network failure scenarios
  - Authentication edge cases
  - Signal handling and graceful shutdown

## Key Features

### 🔧 Easy Setup
- Automated virtual environment creation
- Dependency management
- Binary compilation
- One-command test execution

### 🎯 Comprehensive Coverage
- All authentication methods (X-API-Key, Bearer token, Query parameter)
- Complete MCP protocol operations
- Error handling and edge cases
- Performance and stress testing

### 📊 Advanced Reporting
- Detailed pass/fail analysis
- Performance metrics
- Error categorization
- JSON report generation
- Success rate tracking

### 🚀 Production Ready
- CI/CD compatible
- Environment-based configuration
- Graceful error handling
- Comprehensive logging

## Usage Examples

### Basic Testing
```bash
# Run all tests
./tests/run_tests.sh

# Run specific test suite
./tests/run_tests.sh --suite mock
./tests/run_tests.sh --suite production
./tests/run_tests.sh --suite stress
```

### Advanced Testing
```bash
# Comprehensive reporting
./tests/run_tests.sh --comprehensive

# Environment validation
./tests/validate_environment.py

# Manual test execution
cd tests
source venv/bin/activate
pytest test_http_client_mock_integration.py -v
```

### Environment Configuration
```bash
# Custom server settings
export PRODUCTION_SERVER_URL="http://custom-server:3000"
export PRODUCTION_API_KEY="your-api-key"
./tests/run_tests.sh --suite production

# Authentication method testing
export AUTH_METHOD="Bearer"
pytest test_http_client_mock_integration.py::TestHttpClientMockIntegration::test_client_test_connection_bearer -v
```

## Integration with Development Workflow

This test suite is designed to integrate seamlessly with your development process:

1. **Development Testing**: Quick mock server tests for rapid iteration
2. **Integration Validation**: Production server tests for compatibility
3. **Performance Validation**: Stress tests for production readiness
4. **CI/CD Integration**: Automated testing in build pipelines

## Files Structure

```
tests/
├── run_tests.sh                              # Main test runner script
├── run_comprehensive_tests.py                # Advanced test runner with reporting
├── validate_environment.py                   # Environment validation script
├── requirements.txt                          # Python dependencies
├── README.md                                 # Test documentation
├── test_http_client_mock_integration.py      # Mock server integration tests
├── test_http_client_production_integration.py # Production server tests
├── test_stress_validation.py                 # Stress and edge case tests
└── venv/                                     # Python virtual environment (auto-created)
```

## Contributing

When adding new tests:

1. Follow the established test patterns in existing files
2. Add both positive and negative test cases
3. Include stress testing for new features
4. Update documentation and examples
5. Ensure tests work in both CI and local environments

## Requirements

- **Python 3.8+** with `venv` support
- **Rust** with Cargo (for binary compilation)
- **Network access** for HTTP server testing
- **Unix-like environment** (tested on macOS and Linux)

## Integration Pattern

This example demonstrates the recommended pattern for HTTP client integration testing:

1. **Controlled Environment Testing** (Mock Server)
2. **Real-world Validation** (Production Server)  
3. **Performance & Edge Cases** (Stress Testing)
4. **Comprehensive Reporting** (Metrics & Analysis)

Use this pattern as a template for implementing integration tests in your own MCP client projects.