# AIRS Workspace Shared Patterns

**Last Updated**: 2025-07-28  
**Pattern Status**: Core implementation patterns established, advanced patterns preserved

## Development Methodology Patterns

### Core-First Implementation Strategy ✅
**Pattern**: Build solid foundation before adding architectural complexity  
**Usage**: Implement core functionality first, then layer advanced features  
**Benefits**: Bulletproof foundation, focused testing, incremental complexity  
**Implementation**:
- **Phase 1**: Core JSON-RPC message types (JsonRpcRequest, JsonRpcResponse, JsonRpcNotification)
- **Phase 2**: Advanced features (correlation, transport) on proven foundation
- **Knowledge Preservation**: Document advanced concepts during core implementation

### Spec-Driven Workflow Integration ✅
**Pattern**: Systematic 6-phase development cycle  
**Usage**: All crates follow ANALYZE → DESIGN → IMPLEMENT → VALIDATE → REFLECT → HANDOFF  
**Benefits**: Consistent quality, comprehensive documentation, predictable outcomes  
**Current Status**:
- **ANALYZE**: ✅ Completed (26 EARS notation requirements)
- **DESIGN**: ✅ Completed (technical architecture + strategic pivot)
- **IMPLEMENT**: 🎯 Active (core JSON-RPC message types)

### Memory Bank Architecture ✅  
**Pattern**: Workspace-aware persistent project intelligence  
**Usage**: Hierarchical organization with workspace/crate separation  
**Benefits**: Context preservation across memory resets, scalable organization  
**Structure**:
```
.copilot/memory-bank/
├── workspace/           # Cross-crate intelligence
├── crates/             # Crate-specific intelligence  
└── current_focus.md    # Active work indicator
```

### Knowledge Preservation Strategy ✅
**Pattern**: Document advanced concepts while focusing on core implementation  
**Usage**: Preserve architectural intelligence for future phases  
**Benefits**: No lost knowledge, clear future integration path  
**Implementation**: Advanced concepts in `.agent_work/research/` directory

### Gilfoyle Code Review Standards ✅
**Pattern**: Technical excellence with sardonic precision  
**Usage**: All code subjected to high standards review  
**Benefits**: Superior code quality, architectural consistency  
**Standards**: SOLID principles, performance optimization, clean architecture

## Architecture Patterns

### Foundation-First Development ✅
**Pattern**: Build solid core before adding sophisticated features  
**Usage**: JSON-RPC message types before correlation and transport  
**Benefits**: Stable architecture, testable components, performance optimization  
**Current Implementation**: `src/base/jsonrpc/` contains core message types

### Domain-Driven Module Organization ✅
**Pattern**: Modules organized by domain boundaries  
**Usage**: Clear separation of concerns with well-defined interfaces  
**Benefits**: Maintainable code, clear dependencies, testable units  
**Structure**:
```
src/base/jsonrpc/
├── mod.rs           # Public API exports
├── message.rs       # Core message types (CURRENT)
├── error.rs         # JSON-RPC error handling (NEXT)
├── id.rs            # Request ID implementation (PENDING)
└── validation.rs    # Message validation logic (PENDING)
```

### Incremental Feature Addition (Implementation Strategy)
**Pattern**: Add features on proven foundation incrementally  
**Usage**: Core → Error Handling → IDs → Validation → Advanced Features  
**Benefits**: Each layer builds on tested foundation, clear integration points  
**Current Status**: Message types implementation in progress

## Dependency Management Patterns

### Centralized Workspace Dependencies ✅
**Pattern**: All third-party dependencies managed at workspace level  
**Usage**: Sub-crates extend from workspace dependencies  
**Benefits**: Version consistency, conflict prevention, security management  
**Implementation**:
```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }
thiserror = { version = "1.0" }
```

### Minimal Dependency Strategy ✅
**Pattern**: Only essential, proven dependencies for core implementation  
**Usage**: Rigorous evaluation before adding new dependencies  
**Benefits**: Reduced attack surface, faster builds, simpler maintenance  
**Current Core Set**: serde, serde_json, thiserror (for JSON-RPC foundation)

## Implementation Patterns

### Type-Safe Message Handling ✅
**Pattern**: Leverage Rust's type system for compile-time correctness  
**Usage**: Structured message types with serde serialization  
**Benefits**: Compile-time protocol compliance, zero-cost abstractions  
**Implementation**:
```rust
// Type-safe JSON-RPC message structures
pub struct JsonRpcRequest {
    pub jsonrpc: String,        // Always "2.0"
    pub method: String,         // Method name
    pub params: Option<Value>,  // Optional parameters
    pub id: RequestId,          // Request identifier
}
```

### Serde Integration Pattern ✅
**Pattern**: Complete serialization/deserialization with serde  
**Usage**: All message types derive Serialize/Deserialize  
**Benefits**: Automatic JSON handling, specification compliance  
**Implementation**: Derive macros with custom serialization where needed

### RequestId Flexibility ✅
**Pattern**: Support both string and numeric request IDs  
**Usage**: Untagged enum for JSON-RPC 2.0 compliance  
**Benefits**: Maximum compatibility with different JSON-RPC implementations  
**Implementation**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}
```

## Testing Patterns

### Comprehensive Unit Testing ✅
**Pattern**: >95% test coverage with unit + integration tests  
**Usage**: All public APIs and critical paths covered  
**Benefits**: Quality assurance, refactoring confidence  
**Strategy**: Unit tests + integration tests + property tests for core types

### JSON-RPC 2.0 Compliance Testing ✅
**Pattern**: Test against official JSON-RPC specification examples  
**Usage**: Validate serialization/deserialization with spec examples  
**Benefits**: Guaranteed specification compliance, interoperability  
**Implementation**: Test cases using official JSON-RPC examples

### Property-Based Testing ✅
**Pattern**: Use `proptest` for edge case discovery  
**Usage**: Generate test cases for message parsing and generation  
**Benefits**: Comprehensive edge case coverage, specification compliance  
**Focus**: JSON-RPC message serialization and validation

## Error Handling Patterns

### Structured Error Types ✅
**Pattern**: Use `thiserror` for type-safe error handling  
**Usage**: Domain-specific error types with proper context  
**Benefits**: Compile-time error safety, clear error propagation  
**Implementation**: JSON-RPC 2.0 compliant error codes and messages

### Error Context Preservation (Planned)
**Pattern**: Maintain error context across async boundaries  
**Usage**: Error chaining through serialization and validation layers  
**Benefits**: Debuggable error traces, operational visibility  
**Status**: Documented for future implementation

## Quality Assurance Patterns

### JSON-RPC 2.0 Specification Compliance ✅
**Pattern**: 100% adherence to JSON-RPC 2.0 specification  
**Usage**: All message types conform to official specification  
**Benefits**: Interoperability, standard compliance, predictable behavior  
**Validation**: Test against specification examples and edge cases

### Zero Technical Debt Policy (Core Phase) ✅
**Pattern**: Address technical debt immediately during core implementation  
**Usage**: Technical debt tracking and immediate remediation  
**Benefits**: Clean foundation for advanced features  
**Process**: Decision records for all technical decisions

### Gilfoyle Code Review Integration ✅
**Pattern**: Technical excellence review for all implementations  
**Usage**: Sardonic precision in code quality assessment  
**Benefits**: Superior code quality, architectural consistency  
**Standards**: SOLID principles, performance optimization, clean architecture

## Documentation Patterns

### API Documentation Standards ✅
**Pattern**: Complete rustdoc with usage examples  
**Usage**: All public APIs documented with examples  
**Benefits**: Developer experience, adoption facilitation  
**Standard**: Examples for all public functions and types

### Living Documentation ✅
**Pattern**: Documentation updated with code changes  
**Usage**: Memory bank and spec files maintained continuously  
**Benefits**: Always current documentation, development context preservation  
**Tools**: Memory bank system + spec-driven workflow artifacts

## Advanced Patterns (Preserved for Future Implementation)

### Request Correlation Architecture (Documented)
**Pattern**: Thread-safe bidirectional request/response matching  
**Usage**: DashMap for lock-free concurrent operations  
**Benefits**: Scalable concurrent processing without blocking  
**Status**: Documented in `.agent_work/research/advanced-jsonrpc-architecture.md`

### Transport Abstraction (Documented)
**Pattern**: Pluggable transport implementations  
**Usage**: STDIO, HTTP, WebSocket transports behind common interface  
**Benefits**: Flexibility, testability, protocol independence  
**Status**: Architecture preserved for future implementation

### Zero-Copy Message Processing (Documented)
**Pattern**: Minimize allocations during message handling  
**Usage**: Use `Bytes` type and buffer pooling  
**Benefits**: Sub-millisecond latency, high throughput  
**Status**: Performance strategies documented for future phases

## Pattern Evolution Strategy

### Current Phase: Core Implementation
- **Active Patterns**: Type-safe messages, serde integration, comprehensive testing
- **Quality Focus**: JSON-RPC 2.0 compliance, technical excellence
- **Foundation Building**: Solid base for advanced features

### Future Phase: Advanced Features
- **Integration Patterns**: Correlation manager, transport abstraction
- **Performance Patterns**: Zero-copy processing, concurrent handling
- **Scalability Patterns**: Connection pooling, resource management

### Knowledge Preservation
- **Research Documentation**: Advanced patterns preserved in research files
- **Integration Points**: Clear boundaries defined for feature addition
- **Architectural Evolution**: Patterns evolve with implementation phases

These patterns provide the foundation for consistent, high-quality development across all AIRS workspace crates, with particular focus on the core-first implementation strategy that ensures solid foundations before architectural sophistication.