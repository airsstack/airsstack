# TASK-005: MCP-Compliant Transport Architecture Refactoring

**Status**: pending  
**Added**: 2025-09-01  
**Updated**: 2025-09-01

## Original Request
Refactor the Transport trait and HTTP transport implementation to align with the official MCP specification, eliminating architectural impedance mismatch and implementing event-driven message handling patterns.

## Thought Process
Research into official MCP specification and TypeScript/Python SDKs revealed that our current Transport trait design is fundamentally misaligned with MCP standards. The official specification uses event-driven message handling with clear separation between transport layer (message delivery) and protocol layer (MCP semantics). Our current sequential receive/send pattern forces artificial correlation mechanisms and creates unnecessary complexity, especially for HTTP transport.

Additionally, analysis of the official MCP documentation revealed that remote servers use diverse authentication methods (OAuth, API keys, username/password combinations), but our current implementation only supports OAuth2. This requires extending our existing `AuthContext` to support multiple authentication methods while maintaining backward compatibility.

Key insights:
1. **Event-Driven vs Sequential**: MCP uses `onmessage` callbacks, not blocking `receive()` calls
2. **Transport/Protocol Separation**: Transport handles delivery, MessageHandler handles MCP protocol logic
3. **Natural Correlation**: JSON-RPC message IDs provide correlation, no oneshot channels needed
4. **Session Management**: Transport-specific, not forced into common interface
5. **Multi-Method Authentication**: MCP servers require OAuth, API keys, and basic auth support
6. **AuthContext Evolution**: Extend existing OAuth2 AuthContext rather than creating new authentication system

This refactoring will eliminate HTTP transport complexity, align with official SDK patterns, and provide comprehensive authentication support for the MCP ecosystem.

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

### Phase 5: Multi-Method Authentication Enhancement (Week 3)
- Extend existing AuthContext to support multiple authentication methods (OAuth, API keys, username/password)
- Implement authentication strategy pattern for pluggable auth methods
- Create authentication manager for multi-strategy support and fallback chains
- Maintain 100% backward compatibility with existing OAuth2 AuthContext usage
- Add API key and basic authentication strategy implementations
- Update HTTP engines to use AuthenticationManager instead of single OAuth2 config

### Phase 6: McpServerBuilder Integration (Week 3-4)
- Implement McpServer as MessageHandler for protocol logic
- Update McpServerBuilder to work with new Transport interface
- Maintain backward compatibility during transition period
- Add support for pluggable MessageHandler implementations
- Update tool, resource, and prompt handling to use new pattern

### Phase 7: Testing and Validation (Week 4)
- Comprehensive unit tests for new Transport trait implementations
- Integration tests for HTTP and stdio transports with new interface
- Performance validation comparing old vs new architecture
- Stress testing for concurrent HTTP sessions and authentication methods
- Security testing for session isolation and multi-method authentication

### Phase 8: Migration and Documentation (Week 4)
- Create migration guides for existing Transport implementations
- Update all examples to use new transport interface and authentication
- Comprehensive documentation for MessageHandler and authentication patterns
- API documentation with usage examples and multi-auth best practices
- Performance benchmarks and comparison with old implementation

## Progress Tracking

**Overall Status:** in_progress - 15%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 5.1 | Design MCP-compliant Transport trait interface | complete | 2025-09-01 | ✅ New transport::mcp module with event-driven Transport trait matching MCP spec |
| 5.2 | Implement JsonRpcMessage and MessageContext types | complete | 2025-09-01 | ✅ Flat JsonRpcMessage structure aligned with official MCP specification |
| 5.3 | Create MessageHandler trait for protocol separation | complete | 2025-09-01 | ✅ Event-driven MessageHandler trait for clean transport/protocol separation |
| 5.3.1 | **REFACTOR: Module structure reorganization** | **pending** | **2025-09-01** | **🚨 CRITICAL: mcp.rs grown to 1000+ lines, requires module breakup before Phase 2** |
| 5.4 | Build compatibility adapter for StdioTransport | pending | 2025-09-01 | Blocked by module refactoring requirement |
| 5.5 | Redesign HttpServerTransport with event-driven pattern | not_started | 2025-09-01 | Will eliminate oneshot channels and correlation complexity |
| 5.6 | Extend AuthContext for multi-method authentication | not_started | 2025-09-01 | Support OAuth, API keys, username/password with backward compatibility |
| 5.7 | Implement authentication strategy pattern | not_started | 2025-09-01 | OAuth2, API key, basic auth, and custom authentication strategies |
| 5.8 | Create AuthenticationManager for multi-strategy support | not_started | 2025-09-01 | Strategy routing, fallback chains, and unified interface |
| 5.9 | Update HTTP engines for multi-method authentication | not_started | 2025-09-01 | Replace OAuth2-only config with AuthenticationManager |

## Progress Log
### 2025-09-01
- ✅ Completed Phase 1 foundation architecture (subtasks 5.1, 5.2, 5.3)
- ✅ All tests passing (419 unit + 32 integration + 188 doctests)
- ✅ Zero warnings across workspace, full compliance with workspace standards
- 🚨 **Critical Issue Identified**: `mcp.rs` file grown to 1000+ lines, violating Single Responsibility Principle
- 📋 **Next Action Required**: Module structure refactoring before Phase 2 implementation
- 🎯 **Refactoring Plan**: Break into focused modules (message.rs, transport.rs, context.rs, error.rs, compat.rs, tests/)
- 🔄 **Status Update**: Added subtask 5.3.1 for module reorganization, marked as critical blocker for Phase 2
| 5.10 | Implement McpServer as MessageHandler | not_started | 2025-09-01 | Protocol logic separation from transport layer |
| 5.11 | Update McpServerBuilder for new architecture | not_started | 2025-09-01 | Support new Transport interface and authentication |
| 5.12 | Comprehensive testing and validation | not_started | 2025-09-01 | Unit, integration, performance, and security testing |
| 5.13 | Documentation and migration guides | not_started | 2025-09-01 | Developer guides for transport and authentication migration |

## Progress Log
### 2025-09-01
- ✅ **PHASE 1 FOUNDATION COMPLETE**: Designed and implemented new MCP-compliant Transport trait interface
- ✅ **Core Types Implemented**: JsonRpcMessage, JsonRpcError, MessageContext, TransportError with full MCP specification alignment
- ✅ **Event-Driven Architecture**: Created MessageHandler trait for clean transport/protocol separation
- ✅ **Specification Compliance**: Flat JsonRpcMessage structure matches official TypeScript/Python SDK patterns
- ✅ **Compatibility Bridge**: Added conversion methods for gradual migration from legacy JsonRpcMessage trait
- ✅ **Comprehensive Testing**: 100% test coverage for new types and interfaces with mock implementations
- **NEXT**: Begin Phase 2 with StdioTransport compatibility adapter implementation
