# OAuth2 MCP Integration Tests

This directory contains comprehensive test suites for the OAuth2 MCP integration example.

## Environment Setup

### 🐍 Python Virtual Environment (Recommended)

For isolation and reproducibility, use a virtual environment:

```bash
# Create virtual environment
python3 -m venv venv

# Activate virtual environment
source venv/bin/activate  # On macOS/Linux
# or
venv\Scripts\activate     # On Windows

# Install dependencies
pip install -r requirements.txt
```

### 📦 Dependencies

The tests require the following Python packages:
- `requests` - HTTP client for API calls
- `PyJWT` - JWT token handling
- `cryptography` - Cryptographic operations

```bash
# Install all dependencies
pip install -r requirements.txt

# Or install individually
pip install requests PyJWT cryptography
```

## Test Scripts

### 🚀 Quick Start
```bash
# Run the recommended basic test
python3 run_tests.py basic

# Or run directly
python3 test_oauth2_basic.py
```

### 📋 Available Tests

#### 1. **Basic Functionality Test** (`test_oauth2_basic.py`)
**Recommended starting point** - Quick verification that core OAuth2 MCP functionality is working.

```bash
python3 test_oauth2_basic.py
python3 test_oauth2_basic.py --debug
python3 test_oauth2_basic.py --no-cleanup
```

**What it tests:**
- Server startup and connectivity
- Token generation (all 4 types)
- MCP initialize operation
- Basic protocol compliance

**Duration:** ~30 seconds  
**Use case:** Quick verification, CI/CD, development workflow

#### 2. **Comprehensive Integration Test** (`test_oauth2_comprehensive.py`)
Full feature validation with detailed reporting.

```bash
python3 test_oauth2_comprehensive.py
python3 test_oauth2_comprehensive.py --debug
```

**What it tests:**
- All basic functionality
- Scope-based authorization validation
- Resources, tools, and prompts access
- Error handling and edge cases
- Detailed capability reporting

**Duration:** ~45 seconds  
**Use case:** Release validation, feature verification

#### 3. **Advanced Integration Test** (`test_oauth2_integration.py`)
Most comprehensive test with retry logic and multiple token validation.

```bash
python3 test_oauth2_integration.py --debug
```

**What it tests:**
- Multiple token type validation
- Retry mechanisms
- Comprehensive error scenarios
- Advanced protocol compliance

**Duration:** ~60+ seconds  
**Use case:** Full system validation, debugging

### 🎛️ Test Runner

Use the unified test runner for convenience:

```bash
# Run basic test (default)
python3 run_tests.py

# Run specific test type
python3 run_tests.py basic
python3 run_tests.py comprehensive  
python3 run_tests.py advanced

# Run all tests in sequence
python3 run_tests.py all

# Options
python3 run_tests.py basic --debug      # Enable debug output
python3 run_tests.py basic --no-cleanup # Keep server running
python3 run_tests.py --help            # Show help
```

### 📊 Expected Results

#### Successful Basic Test Output:
```
🧪 OAuth2 MCP Basic Integration Test
==================================================
✅ OAuth2 MCP Server ready! (HTTP 401)
✅ OAuth2 JWKS Server ready!
✅ Token extracted: eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIs...

📋 Available Token Types:
  • full: 1 scopes, expires in 60 minutes
  • readonly: 3 scopes, expires in 15 minutes  
  • resources: 2 scopes, expires in 30 minutes
  • tools: 2 scopes, expires in 30 minutes

✅ MCP Initialize successful!
  • Protocol Version: 2024-11-05
  • Server Name: airs-mcp-server

🎉 Basic OAuth2 MCP integration is working correctly!
```

#### Successful Comprehensive Test Output:
```
📊 Comprehensive Test Results Summary
============================================================
✅ PASS: MCP Endpoint - Accessible
✅ PASS: JWKS Server - Accessible
✅ PASS: Token Fetch - Successfully extracted 4 tokens
✅ PASS: Initialize (full) - Success
✅ PASS: Scope Validation - Token scope validation working
✅ PASS: Resources List (full) - Success - Found 3 resources
✅ PASS: Tools List (full) - Success - Found 10 tools
✅ PASS: Prompts List (full) - Success - Found 4 prompts
------------------------------------------------------------
Results: 8/8 tests passed
🎉 All tests passed! OAuth2 MCP integration is fully functional.
```

## 🛠️ Prerequisites

### Dependencies
```bash
pip3 install requests
```

### Server Requirements
The tests automatically:
- Build the OAuth2 MCP server (`cargo build --bin oauth2-mcp-server`)
- Start the server on ports 3001 (MCP) and 3002 (JWKS)
- Clean up processes after testing

### Manual Server Control
If you need to manually manage the server:

```bash
# Kill any existing servers
pkill -f oauth2-mcp-server

# Start server manually (from oauth2-integration directory)
cargo run

# Check if ports are in use
lsof -ti :3001 :3002
```

## 🐛 Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Kill existing processes
pkill -f oauth2-mcp-server
# Or kill specific PIDs
kill $(lsof -ti :3001 :3002)
```

#### Build Failures
```bash
# Clean and rebuild
cargo clean
cargo build --bin oauth2-mcp-server
```

#### Python Dependencies
```bash
# Install required packages
pip3 install requests

# Or use virtual environment
python3 -m venv venv
source venv/bin/activate
pip install requests
```

#### Test Timeout Issues
- Use `--debug` flag for verbose output
- Check server logs in the oauth2-integration directory
- Ensure no firewall blocking localhost connections

### Debug Mode

Enable debug output for detailed troubleshooting:

```bash
python3 run_tests.py basic --debug
```

Debug mode provides:
- Detailed HTTP request/response logging
- Server startup process monitoring
- Token extraction step-by-step details
- Timing information for each operation

### Manual Verification

If automated tests fail, verify manually:

```bash
# 1. Start server
cargo run

# 2. Check endpoints (in another terminal)
curl http://localhost:3001/mcp          # Should return 401
curl http://localhost:3002/auth/tokens  # Should return JSON

# 3. Test with token
curl -H "Authorization: Bearer TOKEN_HERE" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":"test","method":"initialize","params":{}}' \
     http://localhost:3001/mcp
```

## 🎯 Integration with Development Workflow

### Quick Development Check
```bash
# After making changes to OAuth2 implementation
python3 run_tests.py basic
```

### Pre-commit Validation
```bash
# Full validation before committing
python3 run_tests.py comprehensive
```

### CI/CD Integration
```bash
# For automated pipelines
python3 run_tests.py basic --no-cleanup
# Allows further testing or manual verification
```

This test suite ensures the OAuth2 MCP integration remains functional and provides confidence when making changes to the authentication and authorization system.