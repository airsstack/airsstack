# System Patterns

This document captures the architectural patterns, design decisions, and technical approaches that define the AIRS project structure and implementation strategy.

## Architectural Overview

### Multi-Crate Workspace Pattern
**Context**: Complex project requiring modular organization with shared dependencies  
**Pattern**: Cargo workspace with hierarchical crate structure  
**Implementation**: 
- Workspace root at `airs/Cargo.toml` manages all crates
- Individual crates in `crates/` directory (e.g., `crates/airs-mcp/`)
- Shared dependency management through workspace-level configuration
- Consistent versioning and metadata across all crates

**Benefits**:
- Modular development with clear separation of concerns
- Efficient dependency management and version consistency
- Independent testing and deployment of individual components

### Core-First Implementation Strategy
**Context**: Complex MCP implementation with multiple layers and advanced features  
**Pattern**: Build bulletproof foundation before adding sophistication  
**Implementation**:
- Phase 1: Core JSON-RPC 2.0 message types with trait-based abstractions
- Phase 2: Protocol layer with MCP-specific message handling
- Phase 3: Transport abstractions for different communication channels
- Phase 4: Advanced features (correlation, bidirectional communication, etc.)

**Benefits**:
- Solid foundation prevents architectural mistakes
- Early validation of core concepts
- Incremental complexity addition with stable base

## JSON-RPC Foundation Patterns

### Trait-Based Message Abstraction
**Context**: Multiple JSON-RPC message types requiring consistent serialization behavior  
**Pattern**: Common trait with blanket implementation for shared functionality  
**Implementation**:
```rust
pub trait JsonRpcMessage: Serialize + for<'de> Deserialize<'de> {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
```

**Benefits**:
- Eliminates code duplication across message types
- Consistent serialization behavior throughout the system
- Easy to extend with additional functionality
- Zero runtime cost through compile-time optimization

### Request ID Flexibility Pattern
**Context**: JSON-RPC 2.0 supports string, number, and null request IDs  
**Pattern**: Enum-based ID system with display formatting  
**Implementation**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

impl RequestId {
    pub fn new_string(id: impl Into<String>) -> Self {
        Self::String(id.into())
    }
    
    pub fn new_number(id: i64) -> Self {
        Self::Number(id)
    }
}
```

**Benefits**:
- Type-safe request ID handling
- Flexible ID generation strategies
- Proper JSON-RPC 2.0 specification compliance
- Efficient serialization with serde untagged enum

### Message Type Consistency Pattern
**Context**: Three distinct JSON-RPC message types with different field requirements  
**Pattern**: Consistent structure with automatic trait implementation  
**Implementation**:
- All message types include `jsonrpc: "2.0"` field
- Request messages: `method`, `params` (optional), `id`
- Response messages: `result` OR `error`, `id`
- Notification messages: `method`, `params` (optional), no `id`
- Automatic `JsonRpcMessage` trait implementation for all types

**Benefits**:
- Strict JSON-RPC 2.0 specification compliance
- Consistent API across all message types
- Compile-time verification of message structure
- Automatic serialization behavior inheritance

## Development Environment Patterns

### VS Code Integration Pattern
**Context**: Multi-crate Rust workspace requiring proper IDE integration  
**Pattern**: Explicit project linking with local configuration exclusion  
**Implementation**:
```json
{
    "rust-analyzer.linkedProjects": [
        "/ABSOLUTE/PATH/TO/airs/Cargo.toml"
    ]
}
```

**Benefits**:
- Prevents rust-analyzer workspace confusion
- Enables proper test discovery and CodeLens functionality
- Maintains development environment consistency across team
- Avoids performance degradation from multi-project analysis

### Git Repository Hygiene Pattern
**Context**: Development environment configurations containing local machine paths  
**Pattern**: Exclude local configs, provide templates, document setup  
**Implementation**:
- `.gitignore` excludes `.vscode/settings.json`
- `.vscode/settings.json.template` provides team template
- `.vscode/extensions.json` standardizes recommended extensions
- `docs/DEVELOPMENT_SETUP.md` documents configuration process

**Benefits**:
- Prevents local development environment pollution in shared repository
- Ensures consistent team development environment setup
- Maintains professional version control hygiene
- Enables new team members to quickly achieve optimal configuration

### Test-Driven Development Workflow Pattern
**Context**: Comprehensive test coverage requirements with efficient development workflow  
**Pattern**: In-editor test execution with CodeLens integration  
**Implementation**:
- Proper rust-analyzer configuration enables CodeLens test discovery
- Tests organized in modules with clear hierarchical structure
- Integration tests verify public API functionality
- Unit tests validate internal implementation details

**Benefits**:
- Efficient test execution during development
- Immediate feedback on code changes
- Comprehensive coverage verification
- Professional development workflow consistency

## Module Organization Patterns

### Hierarchical Module Structure
**Context**: Complex crate functionality requiring clear organization  
**Pattern**: Layer-based module hierarchy with re-exports  
**Implementation**:
```
src/
├── lib.rs           # Public API and re-exports
├── base/            # Foundation layer
│   ├── mod.rs       # Base layer re-exports
│   └── jsonrpc/     # JSON-RPC implementation
│       ├── mod.rs   # JSON-RPC module exports
│       └── message.rs # Core message types
└── (future layers)
```

**Benefits**:
- Clear separation of concerns across layers
- Logical organization that matches architectural design
- Easy navigation and maintenance
- Supports incremental development of additional layers

### Public API Design Pattern
**Context**: Crate users need convenient access to core functionality  
**Pattern**: Selective re-exports at crate root with comprehensive documentation  
**Implementation**:
```rust
// Re-export commonly used types for convenience
pub use base::jsonrpc::{
    JsonRpcMessage,
    JsonRpcRequest, 
    JsonRpcResponse, 
    JsonRpcNotification, 
    RequestId
};
```

**Benefits**:
- Convenient import paths for users (`use airs_mcp::JsonRpcRequest`)
- Hide internal module complexity from public API
- Maintain flexibility to reorganize internal structure
- Professional crate design consistent with Rust ecosystem standards

## Quality Assurance Patterns

### Comprehensive Testing Strategy
**Context**: Mission-critical JSON-RPC implementation requiring high reliability  
**Pattern**: Multi-level testing with specification compliance verification  
**Implementation**:
- Unit tests for individual message type functionality
- Integration tests for trait implementation consistency
- Specification compliance tests for JSON-RPC 2.0 adherence
- Round-trip serialization tests for data integrity
- Edge case tests for error conditions

**Benefits**:
- High confidence in implementation correctness
- Early detection of specification violations
- Prevents regression during future development
- Supports refactoring with confidence

### Documentation Excellence Pattern
**Context**: Professional crate requiring comprehensive documentation  
**Pattern**: Multi-level documentation with examples and architectural overview  
**Implementation**:
- Crate-level documentation with architecture overview
- Function-level documentation with usage examples
- Integration examples showing real-world usage patterns
- Error handling documentation with recovery strategies

**Benefits**:
- Professional appearance in Rust ecosystem
- Reduces support burden through clear documentation
- Enables efficient onboarding of new team members
- Demonstrates technical competence and attention to detail

## Error Handling Patterns

### Structured Error Strategy
**Context**: JSON-RPC operations that can fail in predictable ways  
**Pattern**: Use standard library Result types with serde_json::Error  
**Implementation**:
```rust
pub trait JsonRpcMessage {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
```

**Benefits**:
- Consistent error handling across all message operations
- Leverages standard library conventions
- Clear error propagation without custom error types
- Efficient error handling with zero-cost abstractions

## Performance Patterns

### Zero-Cost Abstractions
**Context**: High-performance JSON-RPC processing requirements  
**Pattern**: Trait-based abstractions that compile to efficient code  
**Implementation**:
- Generic trait implementations with compile-time specialization
- Serde integration for optimal serialization performance
- Enum-based request IDs with untagged serialization
- String interning avoided in favor of owned strings for simplicity

**Benefits**:
- Runtime performance equivalent to hand-optimized code
- Maintains abstraction benefits without performance penalty
- Efficient memory usage patterns
- Predictable performance characteristics

## Future Extension Patterns

### Layered Architecture Preparation
**Context**: Current core implementation will be extended with protocol and transport layers  
**Pattern**: Foundation designed for incremental layer addition  
**Implementation**:
- Clean separation between JSON-RPC foundation and MCP protocol
- Transport abstraction points identified but not implemented
- Message correlation hooks prepared but not activated
- Extension points documented for future development

**Benefits**:
- Smooth transition to advanced features
- Minimizes refactoring during feature addition
- Maintains architectural consistency across development phases
- Preserves core stability while adding complexity

## Decision Log

### 2025-07-28: Core-First Implementation Strategy
**Decision**: Implement JSON-RPC foundation before advanced MCP features  
**Context**: Complex MCP specification with multiple interconnected components  
**Rationale**: Solid foundation prevents architectural mistakes and enables incremental validation  
**Impact**: Delayed advanced features but higher confidence in core implementation  
**Review**: Evaluate after core completion for lessons learned

### 2025-07-28: Trait-Based Message Abstraction
**Decision**: Use common trait for all message types instead of code duplication  
**Context**: Three message types requiring identical serialization functionality  
**Rationale**: Eliminates duplication, ensures consistency, provides extension point  
**Impact**: Cleaner codebase with consistent behavior across all message types  
**Review**: Monitor for performance implications during high-throughput testing

### 2025-07-28: VS Code Explicit Project Linking
**Decision**: Use `rust-analyzer.linkedProjects` configuration for workspace focus  
**Context**: Multi-project development environment causing rust-analyzer confusion  
**Rationale**: Explicit configuration eliminates ambiguity and improves performance  
**Impact**: Enables proper test discovery and CodeLens functionality  
**Review**: Monitor team adoption and consider automation for new developer setup

### 2025-07-28: Git Repository Hygiene Implementation
**Decision**: Exclude local VS Code settings, provide template and documentation  
**Context**: Local development configurations with machine-specific paths  
**Rationale**: Prevents repository pollution while maintaining team consistency  
**Impact**: Professional repository hygiene with consistent team development environment  
**Review**: Evaluate effectiveness during team onboarding processes