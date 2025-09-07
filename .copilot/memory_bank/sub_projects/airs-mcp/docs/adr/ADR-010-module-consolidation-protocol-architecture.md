# ADR-010: Module Consolidation - Protocol Architecture Unification

**Status:** Accepted  
**Date:** 2025-09-07  
**Author:** Development Team  
**Tags:** architecture, refactoring, modules, technical-debt

## Context

### Module Overlap Analysis Discovery

During architecture review of the `airs-mcp` crate, we discovered significant functional overlap between three modules:

1. **`src/base/jsonrpc`** - JSON-RPC 2.0 foundation implementation
2. **`src/shared/protocol`** - MCP protocol layer implementation 
3. **`src/transport/mcp`** - MCP-compliant transport layer

### Evidence of Code Duplication

#### **Serialization Methods Duplication**

**In `base/jsonrpc/message.rs` (lines 52-54, 99-101):**
```rust
fn to_json(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string(self)
}
fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    serde_json::from_str(json)
}
```

**In `transport/mcp/message.rs` (lines 250-257) - IDENTICAL:**
```rust
pub fn to_json(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string(self)
}
pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
    serde_json::from_str(json)
}
```

#### **Conceptual Overlap Evidence**

- **`shared/protocol`**: Claims to be "MCP protocol layer implementation" built "on top of the existing JSON-RPC 2.0 foundation"
- **`transport/mcp`**: Claims to implement "MCP-Compliant Transport Layer" with "JsonRpcMessage" that matches "MCP specification exactly"
- **Both modules**: Claim to build on JSON-RPC but redefine message structures

#### **API Confusion Evidence**

From `lib.rs` lines 209-232, the public API exposes types from ALL THREE modules:
```rust
// From base/jsonrpc
pub use base::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};
// From shared/protocol  
pub use shared::protocol::{Base64Data, ClientInfo, Content, /*...*/};
// transport/mcp types are also used internally
```

This creates user confusion - nearly identical functionality accessible through different import paths.

#### **Compatibility Layer Code Smell**

The existence of `transport/mcp/compat.rs` indicates design problems:
```rust
// Lines 6, 42-48 show the problem
use crate::base::jsonrpc::message::JsonRpcMessage as LegacyJsonRpcMessage;

impl JsonRpcMessage {
    pub fn from_legacy<T>(legacy_message: &T) -> Result<Self, serde_json::Error>
    where T: LegacyJsonRpcMessage,
```

The need for a "legacy" compatibility bridge suggests `transport/mcp` was created to replace `base/jsonrpc`, but both were kept, creating maintenance burden.

### Usage Pattern Analysis

**Examples show confusion** - both `simple-mcp-server` and `simple-mcp-client` import from multiple modules for essentially the same functionality.

### Workspace Standards Violation

- **Zero Warning Policy**: Code duplication violates maintenance efficiency
- **Minimal Dependencies**: Three overlapping modules violate "minimal, well-maintained dependencies"
- **Clear Architecture**: Overlapping responsibilities violate clean architecture principles

## Decision

### **Consolidate into Single `src/protocol/` Module**

We will merge the three overlapping modules into a single, well-structured `src/protocol/` module:

```
src/protocol/  (NEW unified module)
├── mod.rs              # Single entry point with all re-exports
├── message.rs          # Unified JSON-RPC + MCP message types
├── types.rs            # MCP-specific types (consolidated)
├── error.rs            # Consolidated error handling
├── transport.rs        # Transport abstractions
└── providers.rs        # MCP providers (tools, resources, prompts)
```

### **Migration Strategy**

#### **From `base/jsonrpc` → `src/protocol/message.rs`**
- **PRESERVE** the trait-based design (well-architected)
- **PRESERVE** `JsonRpcMessage` trait, `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcNotification`
- **PRESERVE** `RequestId` enum and all serialization methods
- **PRESERVE** zero-copy optimizations

#### **From `shared/protocol` → `src/protocol/types.rs` + `src/protocol/message.rs`**
- **MIGRATE** MCP-specific types (Uri, ProtocolVersion, ClientInfo, etc.) to `types.rs`
- **MIGRATE** MCP message structures (InitializeRequest, etc.) to `message.rs` as extensions
- **PRESERVE** type safety and validation patterns

#### **From `transport/mcp` → `src/protocol/transport.rs`**
- **MIGRATE** transport abstractions (`Transport` trait, `MessageHandler`, etc.)
- **DISCARD** the duplicate JsonRpcMessage struct (flat one that duplicates base/jsonrpc)
- **REMOVE** the compatibility layer (no longer needed)

#### **DELETE** the three original modules
- Remove `src/base/jsonrpc/`
- Remove `src/shared/protocol/`
- Remove `src/transport/mcp/`

### **Updated Public API in `lib.rs`**

```rust
// Single import path instead of three overlapping ones
pub use protocol::{
    // JSON-RPC types (from former base/jsonrpc)
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
    
    // MCP types (from former shared/protocol)
    Base64Data, ClientInfo, Content, Uri, ProtocolVersion, /* etc */
    
    // Transport types (from former transport/mcp)
    Transport, MessageHandler, MessageContext, TransportError,
};
```

## Benefits

### **Eliminate Code Duplication**
- Remove identical serialization methods across modules
- Single set of message construction logic
- Unified error handling patterns

### **Simplify API**
- Single import path instead of multiple overlapping ones
- Clear user guidance - one place for all protocol functionality
- Elimination of "which module should I use?" confusion

### **Reduce Maintenance Burden**
- One set of tests, documentation, and bug fixes instead of three
- Single place to implement protocol updates
- Simplified dependency management

### **Better Performance**
- No conversion overhead between "legacy" and "modern" formats
- Direct usage of optimized implementations
- Eliminated trait object overhead in some cases

### **Clearer Architecture**
- Single responsibility - protocol handling
- Clear layering - JSON-RPC foundation + MCP extensions + transport abstractions
- Elimination of circular dependencies and compatibility layers

## Risks & Mitigation

### **Risk: Breaking Changes**
**Mitigation**: Maintain public API compatibility through careful re-exports in `lib.rs`

### **Risk: Large Refactoring**
**Mitigation**: Phase-by-phase migration with continuous testing

### **Risk: Import Path Changes**
**Mitigation**: Provide clear migration guide and deprecation warnings

## Compliance

### **Workspace Standards Adherence**
- ✅ **Zero Warning Policy**: Eliminates code duplication warnings
- ✅ **Minimal Dependencies**: Reduces internal module complexity
- ✅ **Clear Architecture**: Single responsibility for protocol handling
- ✅ **Technical Standards**: Maintains existing code quality and testing

### **User Preference Compliance**
- ✅ **Generic Types Over `dyn`**: Preserves trait-based design from `base/jsonrpc`
- ✅ **Inline Code**: Eliminates vtable lookups through consolidation

## Implementation Notes

### **Preservation Requirements**
- Maintain all existing functionality
- Preserve performance characteristics  
- Keep comprehensive test coverage
- Maintain API compatibility where possible

### **Quality Gates**
- All tests must pass during migration
- Zero compilation warnings maintained
- Documentation updated to reflect new structure
- Examples updated to use new import paths

## Alternatives Considered

### **Alternative 1: Keep All Three Modules**
- ❌ Rejected: Maintenance burden too high
- ❌ Rejected: User confusion continues
- ❌ Rejected: Code duplication violations

### **Alternative 2: Gradual Deprecation**
- ❌ Rejected: Extends maintenance burden
- ❌ Rejected: Doesn't address core duplication issues

### **Alternative 3: Minimal Refactoring**
- ❌ Rejected: Doesn't solve architectural problems
- ❌ Rejected: Compatibility layer remains necessary

## Success Criteria

1. ✅ Single `src/protocol/` module handles all JSON-RPC and MCP functionality
2. ✅ Zero code duplication in serialization methods
3. ✅ Simplified public API with single import path
4. ✅ All existing tests continue to pass
5. ✅ Examples and documentation updated
6. ✅ Workspace standards compliance maintained
7. ✅ Performance characteristics preserved or improved

## References

- **User Preference Rule**: "User prefers to avoid using `dyn` patterns in Rust code and instead use generic types to produce inline code rather than vtable lookups."
- **Workspace Standards**: WARP.md - Zero Warning Policy, Minimal Dependencies, Clear Architecture
- **Code Evidence**: Line references documented in Context section
- **Usage Patterns**: Examples showing import confusion and multiple pathways for same functionality

---

**Next Action**: Begin implementation of unified `src/protocol/` module following the migration strategy outlined above.
