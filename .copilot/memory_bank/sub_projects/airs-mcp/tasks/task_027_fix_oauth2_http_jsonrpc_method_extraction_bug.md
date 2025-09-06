# TASK-027: Fix OAuth2 HTTP JSON-RPC Method Extraction Bug

**Status**: pending  
**Priority**: CRITICAL  
**Created**: 2025-09-06T02:40:00Z  
**Updated**: 2025-09-06T02:40:00Z  
**Category**: Architecture Fix  
**Impact**: PRODUCTION-BLOCKING  
**Related**: DEBT-ARCH-003  

## Problem Statement

**Critical Architectural Bug**: OAuth2 HTTP authentication incorrectly extracts MCP method names from URL paths instead of JSON-RPC message payloads, causing 100% authentication failure for all JSON-RPC over HTTP requests.

### Issue Description

The OAuth2 HTTP adapter assumes REST-style API patterns where method names appear in URL paths (e.g., `/mcp/tools/call`), but MCP uses JSON-RPC over HTTP where the actual method is in the JSON payload (e.g., `{"method": "initialize"}` sent to `/mcp`).

**Error Pattern**:
```
OAuth2 authentication failed: Invalid credentials: OAuth2 validation failed: 
Insufficient scope: required 'mcp:mcp:*', provided 'mcp:*'
```

**Root Cause**: URL path `/mcp` is incorrectly extracted as method `"mcp"`, requiring `mcp:mcp:*` scope instead of checking the actual JSON-RPC method `"initialize"`.

## Technical Analysis

### Bug Location
```rust
// File: transport/adapters/http/auth/oauth2/extractor.rs:85-89
if let Some(root_path) = path.strip_prefix('/') {
    if !root_path.is_empty() {
        return Some(root_path.to_string());  // BUG: Returns "mcp" for "/mcp"!
    }
}
```

### Architecture Issue
```
WRONG: HTTP Layer extracts "mcp" from URL → OAuth2 checks mcp:mcp:* scope
RIGHT: JSON-RPC Layer extracts "initialize" from payload → OAuth2 checks initialize scope
```

### Impact Assessment
- **Severity**: CRITICAL - Blocks all OAuth2 authentication for JSON-RPC
- **Scope**: All MCP servers using OAuth2 HTTP authentication  
- **Tools Affected**: MCP Inspector, Claude Desktop, custom MCP clients
- **Examples Affected**: `mcp-remote-server-oauth2` completely non-functional

## Solution Strategy

### Phase 1: Immediate Fix (Quick Deployment)

**Objective**: Unblock OAuth2 testing immediately with minimal code changes.

**Approach**: Skip method extraction for JSON-RPC endpoints.

```rust
// In HttpExtractor::extract_method()
pub fn extract_method(path: &str) -> Option<String> {
    // Skip method extraction for JSON-RPC endpoints
    if path == "/mcp" || path.starts_with("/mcp?") {
        return None;  // Let MCP layer handle method authorization
    }
    
    // Keep existing REST-style extraction for other endpoints
    // ... existing code ...
}
```

**Benefits**:
- ✅ Immediate fix - OAuth2 authentication will work
- ✅ Minimal code change - low risk of regression
- ✅ Unblocks testing and development
- ✅ Preserves REST-style method extraction for other use cases

### Phase 2: Architectural Fix (Proper Implementation)

**Objective**: Implement correct layered architecture separating authentication from authorization.

**Architecture Redesign**:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Layer    │    │  JSON-RPC Layer │    │   MCP Layer     │
│                 │    │                 │    │                 │
│ • Bearer Token  │───▶│ • Parse Message │───▶│ • Method Auth   │
│ • Authentication│    │ • Extract Method│    │ • Scope Check   │
│                 │    │ • Route Request │    │ • Execute       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Implementation Tasks**:

1. **Update OAuth2 HTTP Adapter** (`transport/adapters/http/auth/oauth2/adapter.rs`)
   - Remove method extraction from HTTP path
   - Focus on bearer token validation only
   - Add configuration for method-level vs token-only authentication

2. **Create MCP OAuth2 Middleware** (new file: `transport/adapters/http/mcp_oauth2.rs`)
   - Parse JSON-RPC message to extract method
   - Perform method-level scope validation
   - Integrate with existing MCP handlers

3. **Update Integration Points**
   - Fix AxumHttpServer OAuth2 integration
   - Update example servers to use new architecture
   - Fix all OAuth2 authentication flows

4. **Comprehensive Testing**
   - Add JSON-RPC over HTTP integration tests
   - Test with real MCP Inspector flows
   - Validate both REST and JSON-RPC patterns

## Implementation Plan

### Subtask 27.1: Immediate Quick Fix
- **Priority**: CRITICAL
- **Effort**: 30 minutes
- **Goal**: Unblock OAuth2 testing immediately

**Actions**:
- [ ] Update `HttpExtractor::extract_method()` to skip `/mcp` endpoints
- [ ] Test with MCP Inspector to verify authentication works
- [ ] Update OAuth2 example server documentation

### Subtask 27.2: Integration Test Coverage
- **Priority**: HIGH  
- **Effort**: 2 hours
- **Goal**: Prevent regression and validate fix

**Actions**:
- [ ] Add JSON-RPC over HTTP OAuth2 integration tests
- [ ] Test complete authentication flow with real JSON-RPC messages
- [ ] Validate both successful and failure authentication scenarios
- [ ] Add continuous integration for OAuth2 authentication testing

### Subtask 27.3: Architectural Refactoring
- **Priority**: HIGH
- **Effort**: 8-12 hours  
- **Goal**: Implement proper layered architecture

**Actions**:
- [ ] Design new OAuth2 middleware architecture
- [ ] Create MCP-level OAuth2 authorization middleware
- [ ] Update HTTP OAuth2 adapter to token-only validation
- [ ] Add configuration options for different authorization modes
- [ ] Migration guide for existing OAuth2 integrations

### Subtask 27.4: Documentation and Examples
- **Priority**: MEDIUM
- **Effort**: 2 hours
- **Goal**: Update all OAuth2 documentation

**Actions**:
- [ ] Update OAuth2 integration guides
- [ ] Fix example server documentation
- [ ] Add architectural decision records
- [ ] Create troubleshooting guides for OAuth2 authentication

## Acceptance Criteria

### Phase 1 Success Criteria
- [ ] MCP Inspector successfully authenticates with OAuth2 server
- [ ] `initialize` method calls succeed with `mcp:*` scope tokens
- [ ] All existing OAuth2 unit tests continue to pass
- [ ] REST-style method extraction still works for non-JSON-RPC endpoints

### Phase 2 Success Criteria  
- [ ] Clear separation between HTTP authentication and MCP authorization
- [ ] Support for both REST and JSON-RPC authentication patterns
- [ ] Comprehensive integration test coverage
- [ ] Zero regression in existing OAuth2 functionality
- [ ] Performance impact analysis completed and acceptable

### Quality Gates
- [ ] All tests pass (unit + integration)
- [ ] Zero compilation warnings
- [ ] Code review approval from architecture team
- [ ] Documentation updated and reviewed
- [ ] Memory bank updated with architectural decisions

## Risk Assessment

### High Risk Areas
- **Integration Impact**: Changes affect core authentication flow
- **Backward Compatibility**: Must not break existing REST-style OAuth2 usage
- **Performance**: Additional JSON parsing in authentication path

### Mitigation Strategies
- **Phased Approach**: Quick fix first, then architectural improvements
- **Comprehensive Testing**: Both unit and integration test coverage
- **Feature Flags**: Configuration options for different authentication modes
- **Rollback Plan**: Quick revert capability if issues discovered

## Success Metrics

### Immediate Success (Phase 1)
- OAuth2 authentication success rate: 0% → 100% for JSON-RPC requests
- MCP Inspector integration: Non-functional → Fully functional
- Example server usability: Broken → Working

### Long-term Success (Phase 2)
- Architecture clarity: Clear separation of authentication vs authorization layers
- Test coverage: Comprehensive JSON-RPC over HTTP authentication testing
- Developer experience: Clear patterns for OAuth2 integration

## Related Work

### Technical Debt
- **DEBT-ARCH-003**: OAuth2 HTTP JSON-RPC Method Extraction Bug (this task resolves)

### Dependencies
- **TASK005**: Zero-Cost Authentication Architecture (completed)
- **OAuth2 Infrastructure**: Existing OAuth2 validator and scope checking

### Follow-up Tasks
- Performance optimization for JSON-RPC parsing in authentication path
- Enhanced OAuth2 configuration options and patterns
- Advanced scope mapping and method authorization patterns

## Notes

### Discovery Context
This bug was discovered during OAuth2 integration testing with MCP Inspector on 2025-09-06. The issue manifested as 100% authentication failure despite valid tokens and correct scope permissions.

### Architecture Insight
The bug revealed a fundamental misunderstanding in the OAuth2 HTTP adapter about JSON-RPC vs REST communication patterns. This highlights the importance of proper layer separation in authentication architectures.

### Testing Gap
The bug existed because unit tests used contrived scenarios that didn't match real-world JSON-RPC over HTTP usage patterns. This emphasizes the need for integration testing with actual MCP clients.

## Workspace Standards Compliance

- ✅ **§2.1 Import Organization**: All code changes will follow 3-layer import structure
- ✅ **§3.2 Time Management**: Using `chrono::DateTime<Utc>` for all timestamps
- ✅ **§4.3 Module Architecture**: Proper separation of concerns in authentication layers
- ✅ **§5.1 Zero Warnings**: All code changes must compile with zero warnings
