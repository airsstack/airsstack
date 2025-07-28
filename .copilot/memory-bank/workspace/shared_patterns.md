# AIRS Workspace Shared Patterns

**Last Updated**: 2025-07-28  
**Pattern Status**: Foundation patterns established, implementation patterns pending

## Development Methodology Patterns

### Spec-Driven Workflow Integration ✅
**Pattern**: Systematic 6-phase development cycle  
**Usage**: All crates follow ANALYZE → DESIGN → IMPLEMENT → VALIDATE → REFLECT → HANDOFF  
**Benefits**: Consistent quality, comprehensive documentation, predictable outcomes  
**Implementation**: 
- Requirements in EARS notation with confidence scoring
- Technical architecture before implementation
- Comprehensive validation and reflection phases

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

### Gilfoyle Code Review Standards ✅
**Pattern**: Technical excellence with sardonic precision  
**Usage**: All code subjected to high standards review  
**Benefits**: Superior code quality, architectural consistency  
**Standards**: SOLID principles, performance optimization, clean architecture

## Dependency Management Patterns

### Centralized Workspace Dependencies ✅
**Pattern**: All third-party dependencies managed at workspace level  
**Usage**: Sub-crates extend from workspace dependencies  
**Benefits**: Version consistency, conflict prevention, security management  
**Implementation**:
```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Minimal Dependency Strategy ✅
**Pattern**: Only essential, proven dependencies  
**Usage**: Rigorous evaluation before adding new dependencies  
**Benefits**: Reduced attack surface, faster builds, simpler maintenance  
**Current Set**: tokio, serde, dashmap, thiserror, uuid, bytes, tokio-util, criterion

## Architecture Patterns

### Foundation-First Development ✅
**Pattern**: Build solid foundation before adding features  
**Usage**: JSON-RPC foundation complete before MCP protocol layer  
**Benefits**: Stable architecture, testable components, performance optimization  
**Implementation**: `src/base/` contains foundational layers

### Domain-Driven Module Organization ✅
**Pattern**: Modules organized by domain boundaries  
**Usage**: Clear separation of concerns with well-defined interfaces  
**Benefits**: Maintainable code, clear dependencies, testable units  
**Structure**:
```
src/
├── base/          # Foundation layers (JSON-RPC, transport)
├── protocol/      # Protocol implementations (MCP)
├── security/      # Authentication and authorization
└── utils/         # Shared utilities
```

## Performance Patterns

### Zero-Copy Message Processing (Pending Implementation)
**Pattern**: Minimize allocations during message handling  
**Usage**: Use `Bytes` type and buffer pooling  
**Benefits**: Sub-millisecond latency, high throughput  
**Target**: <1ms processing (99th percentile), >10,000 msg/sec

### Concurrent Request Correlation (Pending Implementation)
**Pattern**: Thread-safe bidirectional request/response matching  
**Usage**: DashMap for lock-free concurrent operations  
**Benefits**: Scalable concurrent processing without blocking  
**Implementation**: Separate correlation spaces for each direction

## Error Handling Patterns

### Structured Error Types ✅
**Pattern**: Use `thiserror` for type-safe error handling  
**Usage**: Domain-specific error types with proper context  
**Benefits**: Compile-time error safety, clear error propagation  
**Standard**: JSON-RPC 2.0 compliant error codes and messages

### Error Context Preservation (Pending Implementation)
**Pattern**: Maintain error context across async boundaries  
**Usage**: Error chaining through correlation and transport layers  
**Benefits**: Debuggable error traces, operational visibility  
**Implementation**: Include correlation IDs in error context

## Testing Patterns

### Property-Based Testing ✅
**Pattern**: Use `proptest` for edge case discovery  
**Usage**: Generate test cases for protocol compliance  
**Benefits**: Comprehensive edge case coverage, specification compliance  
**Focus**: JSON-RPC message parsing and generation

### Performance Benchmarking ✅
**Pattern**: Use `criterion` for performance validation  
**Usage**: Continuous performance monitoring and regression detection  
**Benefits**: Measurable performance improvements, target validation  
**Targets**: Sub-millisecond latency, high throughput validation

### Comprehensive Test Coverage (Pending Implementation)
**Pattern**: >95% test coverage with unit + integration tests  
**Usage**: All public APIs and critical paths covered  
**Benefits**: Quality assurance, refactoring confidence  
**Strategy**: Unit tests + integration tests + property tests + benchmarks

## Documentation Patterns

### API Documentation Standards (Pending Implementation)
**Pattern**: 100% public API documentation coverage  
**Usage**: Comprehensive rustdoc with examples  
**Benefits**: Developer experience, adoption facilitation  
**Standard**: Examples for all public functions and types

### Living Documentation ✅
**Pattern**: Documentation updated with code changes  
**Usage**: Memory bank and spec files maintained continuously  
**Benefits**: Always current documentation, development context preservation  
**Tools**: Memory bank system + spec-driven workflow artifacts

## Quality Assurance Patterns

### Zero Technical Debt Policy ✅
**Pattern**: Address technical debt immediately or explicitly defer  
**Usage**: Technical debt tracking and remediation planning  
**Benefits**: Maintainable codebase, predictable development velocity  
**Process**: Decision records for all technical debt decisions

### Continuous Quality Monitoring (Pending Implementation)
**Pattern**: Automated quality checks in development workflow  
**Usage**: Static analysis, performance regression detection  
**Benefits**: Early issue detection, consistent quality standards  
**Tools**: Clippy, rustfmt, criterion benchmarks

## Future Pattern Evolution

### Transport Abstraction (Planned)
**Pattern**: Pluggable transport implementations  
**Usage**: STDIO, HTTP, WebSocket transports behind common interface  
**Benefits**: Flexibility, testability, protocol independence

### Configuration Management (Planned)
**Pattern**: Hierarchical configuration with environment overrides  
**Usage**: Default → file → environment → CLI argument precedence  
**Benefits**: Flexible deployment, environment-specific configuration

### Observability Integration (Planned)
**Pattern**: Structured logging and metrics collection  
**Usage**: OpenTelemetry for distributed tracing and metrics  
**Benefits**: Production debugging, performance monitoring

These patterns provide the foundation for consistent, high-quality development across all AIRS workspace crates. They will evolve as implementation proceeds and new patterns emerge.