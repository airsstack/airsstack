# Active Context - AIRS-MCP

## Current Focus: TASK-031 Transport Builder Architectural Consistency

**Status**: Implementation plan complete and approved - ready for execution
**Priority**: CRITICAL - blocks TASK-029 and violates transport abstraction uniformity

### Key Architecture Discovery
During Task 029 Phase 2.2, discovered that HTTP transport uses completely different builder pattern than STDIO transport:
- **STDIO**: Correctly implements `TransportBuilder<()>` per ADR-011
- **HTTP**: Missing `TransportBuilder<HttpContext>` interface entirely
- **Impact**: Generic transport code cannot work with both transport types

### Comprehensive Analysis Completed (2025-09-13)
- **4-Layer Architecture Review**: Analyzed complete AIRS-MCP structure (Protocol, Transport, Integration, Providers)
- **HTTP Transport Deep Dive**: Discovered sophisticated framework abstraction with HttpEngine trait
- **Performance Analysis**: Identified and addressed serialization concerns with structured interface approach
- **Solution Design**: MessageHandlerAdapter bridge pattern to connect MessageHandler<HttpContext> with McpRequestHandler
- **Implementation Strategy**: Additive changes preserving existing HTTP architecture while adding STDIO-style interface

### Implementation Plan (Streamlined Core Focus)
1. **Enhanced McpRequestHandler Trait**: Add structured interface support
2. **HttpContext Methods**: Add response mode and auth context extraction
3. **MessageHandlerAdapter**: Bridge MessageHandler<HttpContext> to McpRequestHandler
4. **Enhanced MessageContext**: Add response collection via oneshot channels
5. **TransportBuilder<HttpContext>**: Implement missing interface for HttpTransportBuilder
6. **Updated AxumMcpRequestHandler**: Support new structured interface

### Critical Dependencies
- **Task 029 Phase 2.2**: BLOCKED until this is resolved
- **Generic Transport Code**: Cannot be implemented until both transports support same interface
- **Workspace Architecture**: Must maintain transport abstraction uniformity per ADR-011

## Recent Decisions
- **Scope**: Core interface consistency only - no performance optimizations or legacy support
- **Approach**: Bridge pattern with zero breaking changes to existing HTTP architecture
- **Priority**: CRITICAL implementation required before continuing other transport-related work

## Next Steps
1. Begin TASK-031 implementation using approved plan
2. Start with enhanced McpRequestHandler trait and MessageHandlerAdapter
3. Test interface consistency with both STDIO and HTTP transports
4. Resume Task 029 Phase 2.2 after transport interface is unified

## Recent Achievements
- **TASK-030**: ✅ Completed - Added comprehensive Cargo.toml documentation
- **TASK-029 Phase 2.1**: ✅ Completed - OAuth2 server modernization with TransportBuilder
- **Comprehensive Architecture Analysis**: ✅ Completed - Full documentation of AIRS-MCP structure

## Task Pipeline
1. **IMMEDIATE**: TASK-031 (Transport Builder Consistency) - Implementation ready
2. **NEXT**: TASK-029 Phase 2.2 (API Key Server Modernization) - Unblocked after TASK-031
3. **THEN**: TASK-029 Phase 2.3 (Zero-cost Auth Server Modernization)
4. **FUTURE**: Generic transport utilities leveraging unified interface