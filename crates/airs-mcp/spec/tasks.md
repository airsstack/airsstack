# [TASK001] - JSON-RPC Foundation Implementation

**Status:** In Progress  
**Added:** 2025-07-28  
**Updated:** 2025-07-28  

## Original Request
Implement base JSON-RPC 2.0 foundation for MCP implementation, focusing on message types, correlation management, and STDIO transport.

## Thought Process
Following the documented architecture in `crates/airs-mcp/docs/`, the JSON-RPC foundation belongs in `src/base/jsonrpc/` module. This is the foundational layer that all MCP functionality will build upon.

Key architectural decisions:
- Bidirectional communication support (both client and server can initiate requests)
- Type-safe message handling using serde
- Efficient correlation management using dashmap
- STDIO transport as primary interface for Claude Desktop integration

## Implementation Plan
1. Core message types (JsonRpcRequest, JsonRpcResponse, JsonRpcNotification)
2. Request ID management and correlation
3. STDIO transport with tokio-util framing
4. Error handling with structured types
5. Performance benchmarks for sub-millisecond claims

## Progress Tracking

**Overall Status:** In Progress - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Define core JSON-RPC message types | Not Started | 2025-07-28 | Using serde for serialization |
| 1.2 | Implement request correlation manager | Not Started | 2025-07-28 | Dashmap for thread-safe correlation |
| 1.3 | Create STDIO transport implementation | Not Started | 2025-07-28 | Tokio-util codec for framing |
| 1.4 | Add structured error handling | Not Started | 2025-07-28 | Thiserror for JSON-RPC errors |
| 1.5 | Implement performance benchmarks | Not Started | 2025-07-28 | Criterion for sub-millisecond validation |

## Progress Log
### 2025-07-28
- Created task structure following memory bank methodology
- Established spec-driven workflow integration
- Ready to begin ANALYZE phase with EARS notation requirements