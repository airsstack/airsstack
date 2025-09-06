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

**ARCHITECTURAL DECISION**: Based on ADR-009 Zero-Cost Generic Authorization Architecture, we will implement a comprehensive refactoring that fixes the layer violation while establishing a production-ready authentication/authorization foundation.

**Key Decision**: Skip quick fixes to avoid technical debt - implement proper architecture directly.

### Architecture Overview

**Objective**: Implement Zero-Cost Generic Authorization Architecture (ADR-009)

**Module Structure**:
```
src/
├── authentication/     # ✅ "Who are you?" - Identity verification
├── authorization/      # 🆕 "What can you do?" - Permission checking  
├── oauth2/            # ✅ OAuth2 protocol implementation
└── transport/.../auth/ # ✅ HTTP-specific token extraction only
```

**Zero-Cost Architecture**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP Layer    │    │  JSON-RPC Layer │    │   MCP Layer     │
│                 │    │                 │    │                 │
│ • Bearer Token  │───▶│ • Parse Message │───▶│ • Method Auth   │
│ • Authentication│    │ • Extract Method│    │ • Scope Check   │
│                 │    │ (Generic)       │    │ (Zero-Cost)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Core Principles**:
- **Zero Runtime Dispatch**: Pure generics, no `dyn` traits
- **Compile-Time Specialization**: Each auth/authz combo creates unique server type
- **Optional Authorization**: Development/internal use needs no authorization overhead
- **Type Safety**: Impossible to mix incompatible combinations

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

**Reference**: ADR-009 Zero-Cost Generic Authorization Architecture

### Phase 1: Authorization Framework (4 hours)
- **Priority**: CRITICAL
- **Goal**: Create zero-cost generic authorization abstractions

**Actions**:
- [ ] Create `src/authorization/` module with generic traits
- [ ] Implement concrete context types (OAuth2AuthContext, ApiKeyAuthContext, NoAuthContext)
- [ ] Create authorization policies (NoAuthorizationPolicy, ScopeBasedPolicy, BinaryAuthorizationPolicy)
- [ ] Build generic authorization middleware with `MethodExtractor` trait
- [ ] Ensure all abstractions use pure generics (no `dyn` traits)

### Phase 2: Transport Layer Cleanup (2 hours)
- **Priority**: HIGH
- **Goal**: Remove authorization logic from HTTP transport layer

**Actions**:
- [ ] Remove method extraction from HTTP OAuth2 adapter
- [ ] Focus HTTP auth adapters on token extraction and authentication only
- [ ] Return concrete authentication context types (no heap allocation)
- [ ] Deprecate incorrect `HttpExtractor::extract_method()` pattern
- [ ] Update all HTTP auth adapters to follow authentication-only pattern

### Phase 3: Server Integration (3 hours)
- **Priority**: HIGH
- **Goal**: Integrate generic auth/authz with server architecture

**Actions**:
- [ ] Create generic server types with compile-time specialization
- [ ] Implement builder pattern for type-safe auth/authz configuration
- [ ] Update `McpServer<AuthAdapter, AuthzPolicy, AuthContext>` generic structure
- [ ] Ensure each configuration creates unique server type (zero runtime dispatch)
- [ ] Update example servers to use new builder pattern

### Phase 4: Testing & Documentation (1 hour)
- **Priority**: MEDIUM
- **Goal**: Validate zero-cost abstractions and provide migration guides

**Actions**:
- [ ] Integration tests for JSON-RPC over HTTP OAuth2 authentication
- [ ] Performance benchmarks to verify zero-cost abstractions
- [ ] Validate with `cargo expand` that NoAuth compiles to zero code
- [ ] Update documentation with new authentication/authorization patterns
- [ ] Create migration guide from current OAuth2 usage

## Acceptance Criteria

### Functional Requirements
- [ ] OAuth2 authentication works correctly with JSON-RPC over HTTP
- [ ] Method extraction happens at the correct protocol layer (JSON-RPC payload, not URL path)
- [ ] Authorization is optional and configurable per server
- [ ] All existing authentication strategies continue to work
- [ ] MCP Inspector successfully authenticates with OAuth2 server
- [ ] `initialize` method calls succeed with `mcp:*` scope tokens

### Performance Requirements (Zero-Cost Abstractions)
- [ ] Zero runtime dispatch - all calls inlined at compile time
- [ ] No heap allocations for authentication contexts
- [ ] Development mode (NoAuth) has zero authorization overhead
- [ ] Performance benchmarks show no regression vs current implementation
- [ ] `cargo expand` verification: NoAuth authorization compiles to zero code

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

### Authentication Success
- OAuth2 authentication success rate: 0% → 100% for JSON-RPC requests
- MCP Inspector integration: Non-functional → Fully functional
- Example server usability: Broken → Working with clean builder pattern

### Architecture Quality (ADR-009 Compliance)
- Layer separation: Clean authentication vs authorization boundaries
- Zero-cost abstractions: All authorization logic inlined at compile time  
- Type safety: Compile-time verification of auth/authz combinations
- Performance: NoAuth development mode has literally zero overhead
- Flexibility: Optional authorization works with any authentication strategy

### Developer Experience
- Simple configuration: Builder pattern for all auth/authz combinations
- Clear documentation: Migration guides and usage examples
- Test coverage: Comprehensive integration testing for real-world scenarios

## Related Work

### Architecture Decision Records
- **ADR-009**: Zero-Cost Generic Authorization Architecture (accepted 2025-09-06)
  - Establishes the architectural foundation for this implementation
  - Defines pure generic design without `dyn` patterns
  - Specifies optional authorization with compile-time optimization

### Technical Debt
- **DEBT-ARCH-003**: OAuth2 HTTP JSON-RPC Method Extraction Bug (this task resolves)

### Dependencies
- **TASK005**: Zero-Cost Authentication Architecture (completed)
- **OAuth2 Infrastructure**: Existing OAuth2 validator and scope checking
- **ADR-009**: Architecture decisions and implementation guidelines

### Follow-up Tasks
- Performance benchmarking to validate zero-cost abstractions
- Enhanced OAuth2 configuration options and patterns
- Advanced scope mapping and method authorization patterns
- WebSocket and STDIO transport authorization support

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
