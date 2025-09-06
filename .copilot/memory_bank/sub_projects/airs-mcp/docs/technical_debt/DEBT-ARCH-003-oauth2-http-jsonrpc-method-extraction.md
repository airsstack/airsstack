# TECHNICAL DEBT: DEBT-ARCH-003 - OAuth2 HTTP JSON-RPC Method Extraction Bug

**Document Type**: Technical Debt Analysis  
**Category**: DEBT-ARCH (Architectural Debt)  
**Created**: 2025-09-06T02:40:00Z  
**Priority**: CRITICAL  
**Status**: IDENTIFIED - Requires immediate fix  
**Impact**: PRODUCTION-BLOCKING - Affects all JSON-RPC over HTTP authentication

## Problem Summary

**Critical Architectural Bug**: OAuth2 HTTP authentication incorrectly extracts MCP method names from URL paths instead of JSON-RPC message payloads, causing scope validation to fail for all JSON-RPC requests.

## Bug Discovery Process

### Testing Context
While testing OAuth2 integration with MCP Inspector using the example `mcp-remote-server-oauth2`, we discovered that authentication consistently fails with the error:

```
OAuth2 authentication failed: Invalid credentials: OAuth2 validation failed: 
Insufficient scope: required 'mcp:mcp:*', provided 'mcp:*'
```

### Root Cause Analysis

**The Issue**: HTTP OAuth2 adapter incorrectly extracts method names from URL paths rather than JSON-RPC message content.

#### Expected Flow (Correct)
```json
POST /mcp
Content-Type: application/json

{"jsonrpc": "2.0", "method": "initialize", "id": "1", "params": {}}
```
- **URL Path**: `/mcp` (transport endpoint)
- **Actual MCP Method**: `"initialize"` (from JSON-RPC payload)
- **Expected Scope Check**: `initialize` method → `mcp:*` should be sufficient

#### Actual Flow (Bug)
```rust
// File: transport/adapters/http/auth/oauth2/extractor.rs:73-92
pub fn extract_method(path: &str) -> Option<String> {
    // Handle MCP-style paths: /mcp/tools/call -> tools/call
    if let Some(mcp_path) = path.strip_prefix("/mcp/") {
        return Some(mcp_path.to_string());
    }
    // ... other path patterns ...
    
    // Handle root-level paths: /tools/call -> tools/call
    if let Some(root_path) = path.strip_prefix('/') {
        if !root_path.is_empty() {
            return Some(root_path.to_string());  // BUG: Returns "mcp"!
        }
    }
}
```

- **URL Path**: `/mcp` → extracted as method `"mcp"`
- **JSON-RPC Method**: `"initialize"` (ignored)
- **Incorrect Scope Check**: `mcp` method → requires `mcp:mcp:*` scope
- **Result**: Authentication fails because token has `mcp:*` but needs `mcp:mcp:*`

### Technical Details

#### Bug Location
```rust
// File: transport/adapters/http/auth/oauth2/adapter.rs:115-116
let method = HttpExtractor::extract_method(&request.path);  // BUG: Wrong source!
```

#### Scope Validator Logic
```rust
// File: oauth2/validator/scope.rs:218-221
Err(OAuth2Error::InsufficientScope {
    required: format!("mcp:{}:*", method.split('/').next().unwrap_or(method)),
    provided: scopes.join(" "),
})
```

With `method = "mcp"`:
- `method.split('/').next()` → `"mcp"`
- Required scope becomes `"mcp:mcp:*"`

## Impact Assessment

### Severity: CRITICAL
- **Production Impact**: 100% authentication failure for JSON-RPC over HTTP
- **Scope**: All OAuth2-protected MCP servers using HTTP transport
- **User Experience**: Complete inability to use OAuth2 authentication with MCP Inspector and similar tools

### Affected Components
1. **OAuth2StrategyAdapter**: Core authentication logic
2. **HttpExtractor**: Method extraction utility
3. **All JSON-RPC over HTTP Endpoints**: `/mcp` endpoint patterns
4. **Scope Validation**: Incorrect method-to-scope mapping

### Test Coverage Gap
- **Unit Tests**: Pass because they use contrived method extraction scenarios
- **Integration Tests**: Don't test real JSON-RPC over HTTP authentication flows
- **Example Servers**: Work in isolation but fail with real MCP clients

## Architecture Analysis

### The Fundamental Issue

**JSON-RPC vs REST Confusion**: The OAuth2 adapter assumes REST-style URL patterns where method names are in paths:
- ✅ REST Pattern: `POST /mcp/tools/call` → method = `"tools/call"`  
- ❌ JSON-RPC Pattern: `POST /mcp` + `{"method": "initialize"}` → incorrectly extracts `"mcp"`

### Why This Is Architectural Debt

1. **Layer Violation**: HTTP transport layer making protocol-level decisions
2. **Assumption Mismatch**: Code assumes REST when JSON-RPC is being used
3. **Authentication vs Authorization Confusion**: Method-level authorization should happen at MCP protocol layer, not HTTP transport layer

### Correct Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Layer    │    │  JSON-RPC Layer │    │   MCP Layer     │
│                 │    │                 │    │                 │
│ • Transport     │    │ • Message Parse │    │ • Method Auth   │
│ • Bearer Token  │───▶│ • Method Extract│───▶│ • Scope Check   │
│ • General Auth  │    │ • Request Route │    │ • Handler Call  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Current (Wrong)**:
- HTTP layer extracts method from URL path
- Performs method-level authorization at wrong layer

**Correct**:
- HTTP layer only validates bearer tokens
- JSON-RPC layer extracts method from message payload  
- MCP layer performs method-level authorization

## Solution Strategies

### Option A: Quick Fix (Band-aid)
Update scope mappings to not require method-level authorization for JSON-RPC endpoints.

```rust
// Don't extract method from path for JSON-RPC endpoints
if path == "/mcp" {
    return None;  // Skip method extraction
}
```

**Pros**: Minimal code change, unblocks testing  
**Cons**: Doesn't fix architectural issue

### Option B: Architectural Fix (Proper)
Move method-level authorization to the correct layer.

1. **HTTP Layer**: Only validate bearer tokens (authentication)
2. **JSON-RPC Layer**: Extract method from message payload
3. **MCP Layer**: Perform method-level authorization with proper scope checking

**Pros**: Correct architecture, fixes root cause  
**Cons**: Requires significant refactoring

### Option C: Hybrid Approach
Support both patterns with explicit configuration.

```rust
pub enum AuthorizationMode {
    TokenOnly,      // HTTP layer only validates tokens
    MethodLevel,    // HTTP layer validates method permissions
}
```

**Pros**: Flexible, backward compatible  
**Cons**: Complexity, maintains architectural confusion

## Recommended Solution

**Recommendation**: **Option B - Architectural Fix**

### Reasoning
1. **Correctness**: Aligns with proper layered architecture
2. **JSON-RPC Compliance**: Respects JSON-RPC protocol semantics
3. **Future-Proofing**: Supports multiple transport patterns correctly
4. **Test Coverage**: Enables proper integration testing

### Implementation Plan

#### Phase 1: Immediate Fix (Band-aid)
For urgent unblocking, implement Option A quick fix.

#### Phase 2: Architectural Refactoring  
1. **Update HTTP OAuth2 Adapter**: Remove method extraction from path
2. **Create MCP OAuth2 Middleware**: Handle method-level authorization at MCP layer
3. **Update Integration Points**: Fix all callers and tests
4. **Add Integration Tests**: Real JSON-RPC over HTTP test scenarios

## Evidence and References

### Bug Discovery Session
- **Date**: 2025-09-06T01:48:00Z - 2025-09-06T02:39:00Z
- **Context**: OAuth2 MCP remote server example testing with MCP Inspector
- **Tool**: `npx @modelcontextprotocol/inspector-cli`
- **Error Pattern**: Consistent `mcp:mcp:*` vs `mcp:*` scope mismatch

### Code References
- **Bug Location**: `transport/adapters/http/auth/oauth2/extractor.rs:85-89`
- **Adapter Usage**: `transport/adapters/http/auth/oauth2/adapter.rs:116`
- **Scope Validation**: `oauth2/validator/scope.rs:218-221`
- **Test Coverage Gap**: No integration tests for JSON-RPC over HTTP OAuth2 flow

### Test Case for Reproduction
```rust
#[tokio::test]
async fn test_json_rpc_over_http_oauth2_authentication() {
    // Setup OAuth2 server with mcp:* scope token
    let server = setup_oauth2_mcp_server().await;
    
    // Send JSON-RPC request to /mcp endpoint
    let request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": "test",
        "params": {}
    });
    
    let response = http_client
        .post("http://localhost:3001/mcp")
        .header("Authorization", "Bearer <token-with-mcp-star-scope>")
        .json(&request)
        .send()
        .await;
    
    // Should succeed but currently fails with scope mismatch
    assert!(response.status().is_success());
}
```

## Next Actions

### Immediate (CRITICAL Priority)
1. **Create TASK**: Create formal task for fixing this architectural issue
2. **Quick Fix**: Implement band-aid solution to unblock testing
3. **Update Memory Bank**: Document architectural findings and solution plan

### Short Term (HIGH Priority)
1. **Integration Tests**: Add comprehensive JSON-RPC over HTTP OAuth2 tests
2. **Architecture Review**: Review all method extraction patterns for similar issues
3. **Documentation**: Update OAuth2 integration guides with correct patterns

### Long Term (MEDIUM Priority)
1. **Architectural Refactoring**: Implement proper layered authorization
2. **Standard Compliance**: Ensure alignment with JSON-RPC and OAuth2 specifications
3. **Performance Review**: Analyze impact of architectural changes on performance

## Workspace Standards Compliance

- ✅ **§2.1 Import Organization**: 3-layer structure maintained in analysis
- ✅ **§3.2 Time Management**: Using `chrono::DateTime<Utc>` for timestamps  
- ✅ **§4.3 Module Architecture**: Clear architectural layer analysis provided
- ✅ **§5.1 Zero Warnings**: Technical debt properly categorized as DEBT-ARCH-003

## Conclusion

This architectural bug represents a fundamental misunderstanding of JSON-RPC over HTTP authentication patterns. The immediate impact is complete OAuth2 authentication failure for all JSON-RPC requests, making the OAuth2 example server non-functional with real MCP clients.

The root cause is a layer violation where HTTP transport logic attempts to perform MCP protocol-level authorization. The correct fix involves separating authentication (HTTP layer) from authorization (MCP layer) concerns.

This issue must be addressed immediately to enable OAuth2 testing and validation with real MCP clients like MCP Inspector.
