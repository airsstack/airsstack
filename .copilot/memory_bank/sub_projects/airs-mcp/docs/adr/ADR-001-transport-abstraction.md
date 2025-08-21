# ADR-001: HTTP Transport Role-Specific Architecture

**Status**: Accepted  
**Date**: 2025-08-14  
**Deciders**: Core Development Team  
**Technical Story**: Phase 2 HTTP implementation revealed Transport trait architectural mismatch

## Context and Problem Statement
**What is the issue that we're seeing that is motivating this decision or change?**

During Phase 2 HTTP transport implementation, we discovered that a single `HttpStreamableTransport` type attempting to handle both client and server roles created semantic confusion and architectural complexity. The Transport trait abstraction assumed role-agnostic implementations, but HTTP transport has fundamentally different patterns for client (sending requests) vs server (receiving requests) operations.

The forces at play include:
- **Technical**: HTTP client and server have different lifecycle patterns, connection management, and error handling
- **Architectural**: Transport trait needs clear semantic boundaries for different operational roles  
- **Maintenance**: Mixed-role implementations create complex conditional logic and testing scenarios
- **Future Growth**: Additional transport types (WebSocket, TCP, etc.) will have similar role-specific patterns

## Decision Drivers
**What factors influenced this decision?**

- **Clarity and Maintainability**: Role-specific types provide clear semantic boundaries
- **Testing Strategy**: Separate implementations enable focused, role-specific test suites
- **Future Transport Types**: Establishes pattern for WebSocket, TCP, and other transport implementations
- **API Design**: Clear distinction between client and server usage patterns
- **Error Handling**: Different roles have different error handling and recovery strategies
- **Performance**: Role-specific optimizations become possible (connection pooling for clients, listener management for servers)

## Considered Options
**What are the ways we can solve this problem?**

1. **Option A: Role-Specific Transport Types** (CHOSEN)
   - Pros: Clear semantics, focused implementations, better testability, future-proof pattern
   - Cons: More types to maintain, slightly more complex API surface
   - Implementation effort: Medium

2. **Option B: Single Transport with Role Configuration**
   - Pros: Single type, simpler API surface
   - Cons: Complex conditional logic, unclear semantics, difficult testing, poor role separation
   - Implementation effort: Low

3. **Option C: Transport Factory Pattern**
   - Pros: Single entry point, role-specific instances
   - Cons: Additional abstraction layer, factory complexity, unclear ownership patterns
   - Implementation effort: High

## Decision Outcome
**Chosen option**: Option A - Role-Specific Transport Types, because it provides the clearest semantic boundaries, enables the most maintainable implementation patterns, and establishes a scalable architecture for future transport types.

### Positive Consequences
- Clear API semantics: `HttpClientTransport` for clients, `HttpServerTransport` for servers
- Focused test suites with role-specific test scenarios
- Enables role-specific optimizations (connection pooling, listener management)
- Establishes consistent pattern for future transport implementations
- Simplified error handling with role-appropriate error types

### Negative Consequences
- Increased API surface area with multiple transport types
- Potential for code duplication between similar role implementations
- More complex transport selection logic for applications supporting multiple roles

## Implementation Plan
**How will this decision be implemented?**

1. **Phase 1**: Rename `HttpStreamableTransport` to `HttpClientTransport` ✅
2. **Phase 2**: Create `HttpServerTransport` foundation for Phase 3 ✅  
3. **Phase 3**: Implement deprecation compatibility alias ✅
4. **Phase 4**: Update documentation and examples ✅
5. **Phase 5**: Remove deprecated alias in future release (planned)

**Migration Strategy**: Gradual migration with backward-compatible type alias
**Testing Approach**: Separate test suites for each transport role
**Rollback Plan**: Revert to single type if role separation proves problematic

## Validation Approach
**How will we know this decision was correct?**

- **Measurable Success Criteria**:
  - All existing tests pass with new architecture ✅
  - Clear documentation differentiating client vs server usage ✅
  - No regression in performance or functionality ✅
  - Simplified test maintenance (fewer conditional test scenarios) ✅

- **Timeline for Evaluation**: 3 months post-implementation
- **Key Metrics to Monitor**: Test maintenance effort, API adoption patterns, implementation complexity
- **Review Schedule**: Quarterly review to assess pattern success

## Compliance with Workspace Standards
**How does this align with workspace standards?**

- **Reference**: Aligns with workspace/shared_patterns.md §4.3 Module Architecture Patterns
- **Single Responsibility**: Each transport type has clear, focused responsibility
- **Clean Architecture**: Role separation maintains clean architectural boundaries
- **No Deviations**: Fully compliant with established workspace patterns

## Links and References
**Supporting information and context**

- **Related ADRs**: 
  - Enables: ADR-002 (Transport Abstraction Strategy)
  - Supports: ADR-003 (Single Responsibility Principle)
- **Implementation**: `crates/airs-mcp/src/transport/http/`
- **Technical Specs**: HTTP transport technical specifications
- **Test Results**: 258 unit tests + 6 integration tests + 129 doc tests passing

## Notes
**Additional context that doesn't fit elsewhere**

- **Alternative Names Considered**: `HttpClientConnector`, `HttpServerListener` (rejected: too specific)
- **Future Decision Points**: 
  - May need additional specialization for streaming vs request-response patterns
  - WebSocket transport implementation will validate this architectural pattern
- **Implementation Success**: Architecture refactoring completed within single day, maintaining full backward compatibility
