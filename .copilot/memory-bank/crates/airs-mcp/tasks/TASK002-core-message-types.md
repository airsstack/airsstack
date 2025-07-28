# [TASK002] - Core JSON-RPC Message Types

**Status:** In Progress - Ready for Implementation  
**Added:** 2025-07-28  
**Updated:** 2025-07-28  

## Original Request
Auto-generated from TASK001 strategic pivot: Implement core JSON-RPC 2.0 message types (JsonRpcRequest, JsonRpcResponse, JsonRpcNotification) with serde serialization.

## Thought Process
Following the core-first implementation strategy, this task focuses exclusively on the fundamental JSON-RPC 2.0 message structures without advanced features like correlation or transport. This provides the solid foundation that all other features will build upon.

Key implementation decisions:
- **Pure JSON-RPC 2.0 Compliance**: Strict adherence to specification
- **Serde Integration**: Complete serialization/deserialization support
- **Type Safety**: Leverage Rust's type system for compile-time correctness
- **Minimal Dependencies**: Only essential crates (serde, serde_json)
- **Comprehensive Testing**: Unit tests for all message variants

## Implementation Plan

### Core Message Structures
```rust
// Target implementation in src/base/jsonrpc/message.rs

pub struct JsonRpcRequest {
    pub jsonrpc: String,        // Always "2.0"
    pub method: String,         // Method name
    pub params: Option<Value>,  // Optional parameters
    pub id: RequestId,          // Request identifier
}

pub struct JsonRpcResponse {
    pub jsonrpc: String,              // Always "2.0"
    pub result: Option<Value>,        // Success result
    pub error: Option<JsonRpcError>,  // Error information  
    pub id: Option<RequestId>,        // Request ID (null for parse errors)
}

pub struct JsonRpcNotification {
    pub jsonrpc: String,        // Always "2.0"
    pub method: String,         // Method name
    pub params: Option<Value>,  // Optional parameters
    // No ID field - notifications don't expect responses
}
```

### RequestId Implementation
```rust
// Support both string and numeric IDs per JSON-RPC 2.0 spec
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}
```

### Module Organization
```
src/base/jsonrpc/
├── mod.rs      # Public API exports
├── message.rs  # Core message types (THIS TASK)
├── error.rs    # JSON-RPC error handling (TASK003)
├── id.rs       # Request ID implementation (TASK004)  
└── validation.rs # Message validation (TASK005)
```

## Progress Tracking

**Overall Status:** Ready for Implementation - 0%

### Implementation Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 2.1 | Create module structure in src/base/jsonrpc/ | Not Started | 2025-07-28 | Directory and mod.rs |
| 2.2 | Implement JsonRpcRequest structure | Not Started | 2025-07-28 | With serde derive macros |
| 2.3 | Implement JsonRpcResponse structure | Not Started | 2025-07-28 | Result/error mutual exclusion |
| 2.4 | Implement JsonRpcNotification structure | Not Started | 2025-07-28 | No ID field variant |
| 2.5 | Add RequestId enum with string/number variants | Not Started | 2025-07-28 | Serde untagged enum |
| 2.6 | Create message construction helpers | Not Started | 2025-07-28 | Builder methods for convenience |
| 2.7 | Add comprehensive unit tests | Not Started | 2025-07-28 | Serialization/deserialization |
| 2.8 | Validate JSON-RPC 2.0 compliance | Not Started | 2025-07-28 | Test against spec examples |
| 2.9 | Document public API | Not Started | 2025-07-28 | Rustdoc with usage examples |
| 2.10 | Integration with Cargo.toml dependencies | Not Started | 2025-07-28 | Ensure serde features enabled |

## Progress Log

### 2025-07-28 - Task Creation and Planning
- ✅ **Task Created**: Auto-generated from TASK001 strategic pivot
- ✅ **Scope Defined**: Core JSON-RPC message types only
- ✅ **Dependencies Identified**: serde, serde_json for serialization
- ✅ **Module Structure Planned**: Clear organization in src/base/jsonrpc/
- ✅ **Testing Strategy**: Unit tests for all message variants
- 🎯 **NEXT**: Begin implementation with module structure creation

### JSON-RPC 2.0 Specification Requirements
- **Request Messages**: Must contain jsonrpc="2.0", method, optional params, id
- **Response Messages**: Must contain jsonrpc="2.0", result XOR error, id
- **Notification Messages**: Must contain jsonrpc="2.0", method, optional params (no id)
- **Request IDs**: Support string, number, or null values
- **Parameter Format**: null, array, or object values

### Success Criteria
- ✅ All message types serialize/deserialize correctly with serde
- ✅ JSON output matches JSON-RPC 2.0 specification format
- ✅ RequestId supports both string and numeric variants
- ✅ Response messages enforce result/error mutual exclusion
- ✅ Notification messages correctly omit ID field
- ✅ Unit tests achieve >95% code coverage
- ✅ Public API documented with usage examples

### Integration Points (Future Tasks)
- **TASK003**: JsonRpcError integration for response error field
- **TASK004**: Enhanced RequestId with validation and generation
- **TASK005**: Message validation using these core types
- **TASK006**: Comprehensive testing framework
- **Future**: Correlation manager will use these message types

### Quality Standards
- **Type Safety**: Leverage Rust's type system for correctness
- **JSON-RPC Compliance**: 100% specification adherence
- **Error Handling**: Proper serde error propagation
- **Documentation**: Complete rustdoc for all public APIs
- **Testing**: Property-based testing for edge cases

This task provides the foundational message types that all JSON-RPC functionality will build upon, ensuring a solid base for future advanced features.