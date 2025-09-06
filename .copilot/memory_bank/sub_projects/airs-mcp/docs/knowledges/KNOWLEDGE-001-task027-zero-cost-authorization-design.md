# KNOWLEDGE-001: Task 027 Zero-Cost Authorization Architecture Design

**Type**: Technical Architecture Knowledge  
**Created**: 2025-09-06  
**Context**: TASK-027 OAuth2 Bug Fix + ADR-009 Architecture Decision  
**Status**: Finalized - Ready for Implementation

## Key Architectural Insights

### Problem Discovery
Task 027 revealed a critical **layer violation** in OAuth2 HTTP authentication:
- HTTP transport layer was incorrectly extracting MCP methods from URL paths
- Should extract methods from JSON-RPC message payloads at the correct layer
- 100% authentication failure for JSON-RPC over HTTP requests

### Root Cause
```
WRONG: POST /mcp + {"method": "initialize"} → extracts "mcp" from path → requires mcp:mcp:* scope
RIGHT: POST /mcp + {"method": "initialize"} → extracts "initialize" from payload → requires mcp:* scope
```

## Finalized Architecture Decision

### Zero-Cost Generic Design
**Principle**: No runtime dispatch, pure compile-time optimization
- ✅ Generic traits: `AuthorizationPolicy<C>` 
- ✅ Concrete contexts: `OAuth2AuthContext`, `ApiKeyAuthContext`, `NoAuthContext`
- ❌ No `dyn` traits or vtable lookups
- ✅ Each server configuration = unique compile-time type

### Module Structure
```
src/
├── authentication/     # ✅ "Who are you?" - Identity verification
├── authorization/      # 🆕 "What can you do?" - Permission checking  
├── oauth2/            # ✅ OAuth2 protocol implementation
└── transport/.../auth/ # ✅ HTTP-specific token extraction only
```

### Layer Separation
```
HTTP Layer → JSON-RPC Layer → MCP Layer
Authentication → Method Extraction → Authorization
```

## Implementation Phases

### Phase 1: Authorization Framework (4 hours)
- Generic traits and concrete context types
- Zero-cost authorization policies  
- Generic method extraction abstractions

### Phase 2: Transport Cleanup (2 hours)
- Remove authorization from HTTP adapters
- Authentication-only pattern
- Concrete context return types (no heap allocation)

### Phase 3: Server Integration (3 hours)
- Generic server types: `McpServer<AuthAdapter, AuthzPolicy, AuthContext>`
- Type-safe builder pattern
- Compile-time specialization

### Phase 4: Testing & Documentation (1 hour)
- Performance validation (`cargo expand`)
- Integration tests
- Migration guides

## Key Design Decisions

### 1. Skip Quick Fixes
**Decision**: No band-aid solutions - implement proper architecture directly
**Reasoning**: Avoid technical debt, establish production-ready foundation

### 2. Pure Generics Only
**Decision**: No `dyn` patterns anywhere in the authorization architecture
**Reasoning**: Zero runtime overhead, compile-time optimization

### 3. Optional Authorization
**Decision**: Development/internal servers may not need authorization overhead
**Implementation**: `NoAuthorizationPolicy` optimizes to zero code

### 4. Type Safety
**Decision**: Impossible to mix incompatible auth/authz combinations at compile time
**Implementation**: Each configuration creates unique server type

## Performance Characteristics

### Zero-Cost Examples
```rust
// Development server - completely optimized away
type DevServer = McpServer<NoAuthAdapter, NoAuthorizationPolicy, NoAuthContext>;

// OAuth2 server - specific concrete type
type OAuth2Server = McpServer<OAuth2StrategyAdapter, ScopeBasedPolicy, OAuth2AuthContext>;
```

### Validation Methods
- `cargo expand` to verify inlining
- Performance benchmarks vs current implementation
- Memory allocation analysis (should be stack-only)

## Success Criteria

### Functional
- OAuth2 works with JSON-RPC over HTTP
- MCP Inspector integration successful
- All existing auth strategies preserved

### Performance  
- Zero runtime dispatch
- NoAuth mode has zero overhead
- No heap allocations for auth contexts

### Architecture
- Clean layer separation
- Type-safe configuration
- Protocol-agnostic design

## Implementation Readiness

**Status**: ✅ **Architecture Finalized - Ready for Implementation**

All design decisions have been made and documented in:
- **ADR-009**: Zero-Cost Generic Authorization Architecture
- **TASK-027**: Updated with finalized implementation plan
- **Memory Bank**: Complete architectural knowledge captured

The architecture provides a **production-ready, high-performance, type-safe** foundation that fixes the OAuth2 bug while enabling future authentication/authorization enhancements.
