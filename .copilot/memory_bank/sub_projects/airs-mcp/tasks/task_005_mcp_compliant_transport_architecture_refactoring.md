# TASK-005: MCP-Compliant Transport Architecture Refactoring

**Status**: pending  
**Added**: 2025-09-01  
**Updated**: 2025-09-01

## Original Request
Refactor the Transport trait and HTTP transport implementation to align with the official MCP specification, eliminating architectural impedance mismatch and implementing event-driven message handling patterns.

## Thought Process
Research into official MCP specification and TypeScript/Python SDKs revealed that our current Transport trait design is fundamentally misaligned with MCP standards. The official specification uses event-driven message handling with clear separation between transport layer (message delivery) and protocol layer (MCP semantics). Our current sequential receive/send pattern forces artificial correlation mechanisms and creates unnecessary complexity, especially for HTTP transport.

Key insights:
1. **Event-Driven vs Sequential**: MCP uses `onmessage` callbacks, not blocking `receive()` calls
2. **Transport/Protocol Separation**: Transport handles delivery, MessageHandler handles MCP protocol logic
3. **Natural Correlation**: JSON-RPC message IDs provide correlation, no oneshot channels needed
4. **Session Management**: Transport-specific, not forced into common interface

This refactoring will eliminate HTTP transport complexity (oneshot channels, session tracking) and align with official SDK patterns.

## Implementation Plan

### Phase 1: Foundation Architecture (Week 1)
- Design and implement new MCP-compliant Transport trait interface
- Create JsonRpcMessage type matching MCP specification
- Implement MessageHandler trait for protocol logic separation
- Create MessageContext for session and metadata handling
- Design compatibility layer for migration period

### Phase 2: Core Components (Week 1-2)  
- Implement new transport trait with lifecycle management (start/close)
- Add event-driven message handling via MessageHandler callbacks
- Create session context management for multi-session transports
- Implement transport state tracking (connected/disconnected)
- Add transport type identification for debugging and metrics

### Phase 3: StdioTransport Adapter (Week 2)
- Create compatibility adapter for existing StdioTransport
- Implement event loop to convert blocking receive() to message events
- Ensure backward compatibility with existing stdio-based examples
- Test adapter with current McpServerBuilder integration
- Document migration path for stdio transport users

### Phase 4: HTTP Transport Redesign (Week 2-3)
- Complete rewrite of HttpServerTransport using new interface
- Eliminate oneshot channels and manual correlation mechanisms
- Implement natural HTTP request/response flow with message events
- Add proper session context management for concurrent HTTP requests
- Remove session tracking complexity and artificial correlation
- Integrate with AxumHttpServer using event-driven pattern

### Phase 5: McpServerBuilder Integration (Week 3)
- Implement McpServer as MessageHandler for protocol logic
- Update McpServerBuilder to work with new Transport interface
- Maintain backward compatibility during transition period
- Add support for pluggable MessageHandler implementations
- Update tool, resource, and prompt handling to use new pattern

### Phase 6: Testing and Validation (Week 3-4)
- Comprehensive unit tests for new Transport trait implementations
- Integration tests for HTTP and stdio transports with new interface
- Performance validation comparing old vs new architecture
- Stress testing for concurrent HTTP sessions
- Security testing for session isolation and message handling

### Phase 7: Migration and Documentation (Week 4)
- Create migration guides for existing Transport implementations
- Update all examples to use new transport interface
- Comprehensive documentation for MessageHandler pattern
- API documentation with usage examples and best practices
- Performance benchmarks and comparison with old implementation

### Phase 8: Cleanup and Optimization (Week 4)
- Remove deprecated Transport trait and compatibility adapters
- Performance optimization based on testing results
- Final security review and code audit
- Documentation review and developer guide updates
- Preparation for future transport implementations (WebSocket, SSE)

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 5.1 | Design MCP-compliant Transport trait interface | not_started | 2025-09-01 | Research official spec patterns and define Rust trait |
| 5.2 | Implement JsonRpcMessage and MessageContext types | not_started | 2025-09-01 | Core message types matching MCP specification |
| 5.3 | Create MessageHandler trait for protocol separation | not_started | 2025-09-01 | Event-driven message handling interface |
| 5.4 | Build compatibility adapter for StdioTransport | not_started | 2025-09-01 | Maintain backward compatibility during migration |
| 5.5 | Redesign HttpServerTransport with event-driven pattern | not_started | 2025-09-01 | Eliminate oneshot channels and correlation complexity |
| 5.6 | Implement McpServer as MessageHandler | not_started | 2025-09-01 | Protocol logic separation from transport layer |
| 5.7 | Update McpServerBuilder for new Transport interface | not_started | 2025-09-01 | Maintain API compatibility with new architecture |
| 5.8 | Comprehensive testing and validation | not_started | 2025-09-01 | Unit, integration, performance, and security testing |
| 5.9 | Documentation and migration guides | not_started | 2025-09-01 | Developer guides and migration documentation |
| 5.10 | Cleanup and optimization | not_started | 2025-09-01 | Remove deprecated code and optimize performance |

## Progress Log
### 2025-09-01
- Created comprehensive task plan based on MCP specification research
- Identified key architectural changes needed for specification compliance
- Documented 4-week implementation timeline with detailed phases
- Established subtask breakdown for tracking progress
- Linked to related architectural decisions and technical debt documentation
