# OAuth2 MCP Integration Example

This directory contains a comprehensive OAuth2 authentication and authorization integration example for the AirsStack MCP HTTP transport server.

## Features

- **Complete OAuth2 Flow**: JWT token validation with JWKS endpoint
- **Scope-Based Authorization**: Method-level access control with configurable scopes
- **Mock JWKS Server**: Testing endpoint for JWT validation
- **Test Token Generation**: Multiple token scenarios for different access levels
- **MCP Inspector Compatible**: Ready-to-use with MCP Inspector tools
- **Modular Architecture**: Well-organized codebase with separated concerns
- **Python Test Suite**: Comprehensive automated testing with detailed reporting

## Architecture

```
oauth2-integration/
├── Cargo.toml                      # Project configuration and dependencies
├── README.md                       # This comprehensive usage guide
├── IMPLEMENTATION_SUMMARY.md       # Complete implementation overview
├── tests/                          # Test suite directory
│   ├── README.md                  # Detailed testing documentation
│   ├── run_tests.py               # Unified test runner script
│   ├── test_oauth2_basic.py       # Basic functionality test (start here!)
│   ├── test_oauth2_comprehensive.py # Full integration test suite
│   └── test_oauth2_integration.py   # Advanced test with all features
├── src/
│   ├── main.rs                    # Main entry point and server orchestration
│   ├── auth.rs                    # OAuth2 message handler implementation
│   ├── config.rs                  # OAuth2 configuration management
│   ├── jwks.rs                    # Mock JWKS server implementation
│   ├── server.rs                  # Test environment and MCP handlers setup
│   └── tokens.rs                  # JWT token generation and test configurations
├── keys/
│   └── test_rsa_key.pem           # RSA private key for JWT signing (testing only)
├── config/                        # Configuration files (future use)
└── docs/                          # Documentation (future use)
```

## Quick Start

### 1. Automated Testing (Recommended)

The quickest way to verify OAuth2 MCP integration:

```bash
# Navigate to the project directory
cd crates/airs-mcp/examples/oauth2-integration

# Quick setup with virtual environment
cd tests/
./setup.sh

# Activate environment and run basic test
source venv/bin/activate
python run_tests.py basic
```

**Alternative manual setup:**
```bash
# Navigate to tests directory
cd tests/

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Run tests
python run_tests.py basic
```

# Install dependencies (first time only)
pip3 install requests

# Run basic functionality test (30 seconds)
python3 tests/run_tests.py

# Or run comprehensive test (45 seconds)
python3 tests/run_tests.py comprehensive

# With debug output for troubleshooting
python3 tests/run_tests.py basic --debug
```

#### **Basic Functionality Test** (Start Here!)
```bash
# Quick verification that everything works
python3 tests/test_oauth2_basic.py

# With debug output
python3 tests/test_oauth2_basic.py --debug

# Keep server running for manual testing
python3 tests/test_oauth2_basic.py --no-cleanup
```

#### **Comprehensive Integration Test**
```bash
# Run full test suite with all features
python3 tests/test_oauth2_comprehensive.py

# With detailed debug information
python3 tests/test_oauth2_comprehensive.py --debug
```

#### **Advanced Integration Test**
```bash
# Run most comprehensive test with retry logic
python3 tests/test_oauth2_integration.py --debug
```

### Expected Output

A successful test run will show:
```
🧪 OAuth2 MCP Basic Integration Test
==================================================
ℹ️ 🧹 Cleaning up...
ℹ️ 🚀 Starting OAuth2 MCP Server...
✅ OAuth2 MCP Server ready! (HTTP 401)
✅ OAuth2 JWKS Server ready!
ℹ️ 🔐 Testing Token Generation...
✅ Token extracted: eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6InRlc3...

📋 Available Token Types:
  • full: 1 scopes, expires in 60 minutes
    - mcp:*
  • readonly: 3 scopes, expires in 15 minutes
    - mcp:resources:list
    - mcp:tools:list
    - mcp:prompts:list
  • resources: 2 scopes, expires in 30 minutes
    - mcp:resources:list
    - mcp:resources:read
  • tools: 2 scopes, expires in 30 minutes
    - mcp:tools:list
    - mcp:tools:execute

ℹ️ 🔧 Testing MCP Initialize...
✅ MCP Initialize successful!
  • Protocol Version: 2024-11-05
  • Server Name: airs-mcp-server
  • Server Capabilities: ['experimental', 'logging', 'prompts', 'resources', 'tools']

🎉 Basic OAuth2 MCP integration is working correctly!
```

### 2. Manual Server Startup

For manual testing and development:

```bash
# From the oauth2-integration directory
RUST_LOG=info cargo run

# Or from the workspace root
cd crates/airs-mcp/examples/oauth2-integration && RUST_LOG=info cargo run
```

The server will start:
- 🔑 **Mock JWKS Server**: `http://localhost:3002` (OAuth2 token validation)
- 📡 **MCP Server**: `http://localhost:3001/mcp` (OAuth2 protected endpoint)

### 3. Get Test Tokens

```bash
curl http://localhost:3002/auth/tokens | python3 -m json.tool
```

This returns test tokens for different access levels (full, tools, resources, readonly) with ready-to-use curl commands and MCP Inspector commands.

### 4. Test with MCP Inspector

```bash
# Example with full access token (copy from token response)
npx @modelcontextprotocol/inspector-cli \
  --transport http \
  --server-url http://localhost:3001/mcp \
  --header "Authorization: Bearer YOUR_FULL_ACCESS_TOKEN_HERE"

# Example with readonly token
npx @modelcontextprotocol/inspector-cli \
  --transport http \
  --server-url http://localhost:3001/mcp \
  --header "Authorization: Bearer YOUR_READONLY_TOKEN_HERE"
```

### 5. Manual MCP Testing

```bash
# Initialize MCP session (required protocolVersion field)
curl -H 'Authorization: Bearer <token>' \
     -H 'Content-Type: application/json' \
     -X POST \
     -d '{
       "jsonrpc": "2.0",
       "id": "init",
       "method": "initialize",
       "params": {
         "protocolVersion": "2024-11-05",
         "capabilities": {},
         "clientInfo": {"name": "test-client", "version": "1.0.0"}
       }
     }' \
     http://localhost:3001/mcp

# List available tools
curl -H 'Authorization: Bearer <token>' \
     -H 'Content-Type: application/json' \
     -X POST \
     -d '{"jsonrpc":"2.0","id":"tools","method":"tools/list","params":{}}' \
     http://localhost:3001/mcp

# Execute a calculation tool
curl -H 'Authorization: Bearer <token>' \
     -H 'Content-Type: application/json' \
     -X POST \
     -d '{"jsonrpc":"2.0","id":"calc","method":"tools/call","params":{"name":"calculate","arguments":{"expression":"2+2*3"}}}' \
     http://localhost:3001/mcp
```

## Server Endpoints

### MCP Server (Port 3001)
- **`/mcp`** - Main MCP JSON-RPC endpoint (OAuth2 protected)
- **`/health`** - Health check endpoint
- **`/status`** - Server status endpoint
- **`/metrics`** - Server metrics endpoint

### JWKS Server (Port 3002)
- **`/.well-known/jwks.json`** - JWKS endpoint for JWT validation
- **`/auth/tokens`** - Generate test OAuth2 tokens
- **`/info`** - Server information endpoint

## OAuth2 Configuration

- **Audience**: `mcp-server`
- **Issuer**: `https://example.com`
- **Algorithm**: RS256 (RSA signatures)
- **Token Validation**: JWKS-based with configurable caching
- **Scope Format**: Hierarchical (e.g., `mcp:tools:*`, `mcp:resources:list`)

## Test Token Scenarios

### Full Access (`full`)
- **Scopes**: `mcp:*`
- **Access**: Complete access to all MCP operations
- **Duration**: 60 minutes

### Tools Only (`tools`)
- **Scopes**: `mcp:tools:list`, `mcp:tools:execute`
- **Access**: Tool operations only
- **Duration**: 30 minutes

### Resources Only (`resources`)
- **Scopes**: `mcp:resources:list`, `mcp:resources:read`
- **Access**: Resource operations only
- **Duration**: 30 minutes

### Read Only (`readonly`)
- **Scopes**: `mcp:resources:list`, `mcp:tools:list`, `mcp:prompts:list`
- **Access**: Read-only access to listings
- **Duration**: 15 minutes

## Edge Case Testing Strategy

### Server-Side OAuth2 Edge Case Testing (Primary Focus)

This example is the **primary testing ground for server-side OAuth2 edge cases** because it uses the actual AIRS MCP `HttpTransport` with real OAuth2 middleware validation.

#### **Why This Example for Server-Side Testing?**
- ✅ **Real Production Code**: Tests actual AIRS MCP OAuth2 middleware
- ✅ **Security Critical**: Server-side validation is the primary security boundary
- ✅ **Middleware Testing**: Validates how OAuth2 middleware handles malformed requests
- ✅ **Error Response Validation**: Tests proper HTTP status codes and error formats

#### **Edge Case Coverage Areas**

**1. JWT Token Validation Edge Cases**
- Malformed JWT structure (missing parts, invalid encoding)
- Expired token handling and error responses
- Invalid signature verification (tampered tokens)
- Incorrect audience claims and issuer validation
- Token without required scopes for specific operations

**2. Authorization Middleware Edge Cases**
- Missing Authorization header handling
- Malformed Bearer token format
- Invalid token encoding (non-base64, corrupted)
- Authorization header injection attempts
- Scope validation with edge case permissions

**3. HTTP Request Edge Cases**
- Oversized Authorization headers
- Malformed JSON-RPC payloads with valid tokens
- Concurrent request handling with invalid tokens
- Network timeout scenarios during token validation
- JWKS endpoint failure simulation

**4. Security Attack Scenarios**
- Token replay attacks (expired tokens)
- Authorization bypass attempts
- Scope privilege escalation attempts
- JWT bombing (oversized tokens)
- Invalid token format fuzzing

#### **Test Implementation Location**
- **Primary**: `tests/test_oauth2_edge_cases.py` (to be implemented)
- **Integration**: Extends existing `test_oauth2_authorization_flow.py`
- **Coverage**: ~25-30 additional edge case tests focusing on server middleware

For **client-side OAuth2 edge cases** (flow interruption, network failures, client resilience), see the `http-oauth2-client-integration` example which focuses on end-to-end flow robustness.

## Scope-Based Authorization

The server enforces method-level authorization based on JWT scopes:

| MCP Method | Required Scope |
|------------|----------------|
| `initialize` | *(no scope required)* |
| `resources/list` | `mcp:resources:*` or `mcp:*` |
| `resources/read` | `mcp:resources:*` or `mcp:*` |
| `tools/list` | `mcp:tools:*` or `mcp:*` |
| `tools/call` | `mcp:tools:*` or `mcp:*` |
| `prompts/list` | `mcp:prompts:*` or `mcp:*` |
| `prompts/get` | `mcp:prompts:*` or `mcp:*` |

## Security Notes

⚠️ **Important**: This example is for testing and development only.

- The RSA private key is included for testing convenience
- In production, use proper key management and rotation
- The issuer `https://example.com` is a placeholder for testing
- JWKS server runs on localhost without HTTPS (testing only)

## Development

### Adding New Token Scenarios

1. Edit `src/tokens.rs`
2. Add new `TokenConfig` implementation
3. Include in `TokenConfig::all_configs()`

### Modifying OAuth2 Configuration

1. Edit `src/config.rs`
2. Update `create_oauth2_config()` function
3. Adjust constants as needed

### Extending MCP Capabilities

1. Edit `src/server.rs`
2. Add new providers to `create_test_environment()`
3. Update the handler configuration

## Integration Testing

### Automated Test Suite

The `test_integration.sh` script provides comprehensive automated testing:

```bash
# Run all integration tests
./test_integration.sh

# View test help and details
./test_integration.sh --help
```

**Test Coverage:**
- 🔐 **OAuth2 Authentication**: JWT validation with different token types
- 🚀 **MCP Protocol**: Proper initialization with protocolVersion 2024-11-05
- 📁 **Resource Operations**: List and read operations with scope validation
- 🔧 **Tool Operations**: List and execute tools with authorization checks
- 💭 **Prompt Operations**: List and retrieve prompts with access control
- 🛡️ **Authorization**: Scope-based access control enforcement
- ⚠️ **Error Handling**: Invalid tokens, unauthorized access, malformed requests

**Test Output:**
- ✅ Real-time pass/fail status with colored output
- 📊 Comprehensive test summary with statistics
- 📝 Detailed logs in `integration_test.log`
- 🧹 Automatic cleanup of all processes

### Manual Testing Scenarios

After running `./test_integration.sh` or starting the server manually, you can test specific scenarios:

#### Test Different Access Levels

```bash
# Get all test tokens
curl -s http://localhost:3002/auth/tokens | python3 -m json.tool

# Test full access (can do everything)
FULL_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6InRlc3Qta2V5LW9hdXRoMi1tY3AifQ..."
curl -H "Authorization: Bearer $FULL_TOKEN" \
     -H "Content-Type: application/json" \
     -X POST \
     -d '{"jsonrpc":"2.0","id":"test","method":"tools/call","params":{"name":"calculate","arguments":{"expression":"10*5"}}}' \
     http://localhost:3001/mcp

# Test readonly token (should fail on tool execution)
READONLY_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6InRlc3Qta2V5LW9hdXRoMi1tY3AifQ..."
curl -H "Authorization: Bearer $READONLY_TOKEN" \
     -H "Content-Type: application/json" \
     -X POST \
     -d '{"jsonrpc":"2.0","id":"test","method":"tools/call","params":{"name":"calculate","arguments":{"expression":"10*5"}}}' \
     http://localhost:3001/mcp
```

#### Test MCP Protocol Compliance

```bash
# Proper MCP initialization (required for protocol compliance)
curl -H "Authorization: Bearer $FULL_TOKEN" \
     -H "Content-Type: application/json" \
     -X POST \
     -d '{
       "jsonrpc": "2.0",
       "id": "init",
       "method": "initialize",
       "params": {
         "protocolVersion": "2024-11-05",
         "capabilities": {},
         "clientInfo": {"name": "test-client", "version": "1.0.0"}
       }
     }' \
     http://localhost:3001/mcp

# Test without protocolVersion (should fail)
curl -H "Authorization: Bearer $FULL_TOKEN" \
     -H "Content-Type: application/json" \
     -X POST \
     -d '{"jsonrpc":"2.0","id":"test","method":"initialize","params":{}}' \
     http://localhost:3001/mcp
```

## Troubleshooting

### Running Integration Tests

**Port Already in Use**
```bash
# Kill any existing servers
pkill -f "cargo run"
pkill -f "oauth2-mcp-server"

# Check what's using the ports
lsof -i :3001
lsof -i :3002
```

**Test Script Hangs or Fails to Start**
```bash
# Check if ports are already in use
lsof -i :3001
lsof -i :3002

# Kill any existing servers
pkill -f "cargo run"
pkill -f "oauth2-mcp-server"

# Run with debug logging to see detailed startup
./test_integration.sh --debug

# Check server startup logs
cat server.log
```

**Server Startup Issues**
```bash
# Verify cargo can build the project
cargo check

# Check for compilation errors
cargo build

# Manually start server to see immediate errors
RUST_LOG=debug cargo run
```

### Manual Testing Issues

**JWT Signature Validation Errors**
- Verify JWKS endpoint is accessible: `curl http://localhost:3002/.well-known/jwks.json`
- Check that private key matches JWKS public key
- Ensure token was generated with correct key ID (`test-key-oauth2-mcp`)

**Authorization Failures**
- Verify token contains required scopes: Decode JWT at jwt.io
- Check scope format matches expected patterns (`mcp:*`, `mcp:tools:*`, etc.)
- Ensure token hasn't expired (check `exp` claim)

**MCP Protocol Errors**
- Always include `protocolVersion: "2024-11-05"` in initialize requests
- Include proper `clientInfo` object with name and version
- Use correct JSON-RPC 2.0 format with `jsonrpc`, `id`, `method`, `params`

**Connection Issues**
- Verify server is running: `curl http://localhost:3001/health`
- Check JWKS server: `curl http://localhost:3002/info`
- Ensure no firewall blocking localhost ports 3001/3002

## Integration with Other Examples

This OAuth2 integration can be adapted for use with other MCP examples by:

1. Copying the OAuth2 modules (`auth.rs`, `config.rs`, `jwks.rs`, `tokens.rs`)
2. Adding OAuth2 dependencies to `Cargo.toml`
3. Integrating with the existing example's main function
4. Configuring appropriate scopes for the example's capabilities

## License

This example is part of the AirsStack project and follows the same licensing terms (MIT OR Apache-2.0).