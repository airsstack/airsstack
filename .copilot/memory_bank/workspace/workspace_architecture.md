# Workspace Architecture

This document describes the high-level structure, relationships between sub-projects, shared resources, and architectural patterns that govern the AIRS workspace ecosystem.

## Multi-Crate Architecture

### Workspace Structure
```
airs/                           # Root workspace
├── Cargo.toml                  # Workspace configuration with shared dependencies
├── crates/
│   ├── airs-mcp/              # Model Context Protocol client implementation
│   └── airs-memspec/          # Memory specification and navigation system
└── .copilot/
    └── memory_bank/           # Workspace-level governance and standards
```

### Crate Responsibilities

#### airs-mcp: JSON-RPC MCP Client
**Purpose:** Production-ready Model Context Protocol client with comprehensive correlation system

**Architecture Layers:**
- `base/`: Core JSON-RPC types and message primitives
- `correlation/`: Request/response correlation with timeout management  
- `transport/`: Abstract transport layer for various communication channels
- `integration/`: High-level client API integrating all layers

**Key Components:**
- **JsonRpcClient:** Main integration layer with call/notify/shutdown methods
- **CorrelationManager:** Request correlation with background message processing
- **ConcurrentProcessor:** Enterprise-grade concurrent processing pipeline ✅ NEW
- **Transport Abstraction:** Pluggable transport implementations with zero-copy optimization
- **Message Types:** Strongly typed JSON-RPC 2.0 message system

#### airs-memspec: Memory Specification System  
**Purpose:** Intelligent workspace navigation and memory management

**Architecture Layers:**
- `parser/`: Markdown parsing and content analysis
- `models/`: Domain types for workspace representation
- `cli/`: Command-line interface and user interaction
- `embedded/`: Embedded system interfaces and integrations

**Key Components:**
- **ContextCorrelator:** Workspace context discovery and aggregation
- **MemoryBankNavigator:** File system navigation and analysis
- **MarkdownParser:** Structured content parsing and extraction
- **WorkspaceContext:** Complete workspace state representation

## Architectural Patterns

### Layered Architecture
Each crate follows clean architecture principles with clear layer separation:

1. **Domain Layer:** Core business logic and types
   - Pure Rust types with no external dependencies
   - Business rules and domain constraints
   - Example: `JsonRpcMessage`, `WorkspaceContext`

2. **Application Layer:** Use cases and orchestration
   - Coordinates domain objects and infrastructure
   - Implements business workflows
   - Example: `CorrelationManager`, `ContextCorrelator`

3. **Infrastructure Layer:** External integrations
   - I/O operations, networking, file system
   - Framework-specific implementations
   - Example: `StdioTransport`, `MemoryBankNavigator`

4. **Interface Layer:** User-facing APIs
   - Public APIs and integration points
   - Adapters for external systems
   - Example: `JsonRpcClient`, CLI commands

### Dependency Management
- **Workspace Inheritance:** All dependencies centralized in root `Cargo.toml`
- **Version Consistency:** Single source of truth for dependency versions
- **Feature Flags:** Conditional compilation for different environments

### Error Handling Strategy
- **Structured Errors:** All errors use `thiserror` with contextual information
- **Error Propagation:** Consistent use of `Result` types and `?` operator
- **Error Context:** Rich error context for debugging and monitoring
- **Recovery Patterns:** Graceful degradation where possible

### Async Programming Model
- **Tokio Runtime:** Consistent async runtime across all crates
- **Channel Communication:** `tokio::sync::mpsc` for inter-component messaging
- **Concurrent Processing:** Worker pool architecture with deadlock-free design ✅ NEW
- **Backpressure Management:** Non-blocking semaphore-based overload protection ✅ NEW
- **Async Traits:** `async-trait` for trait definitions requiring async methods
- **Cancellation Support:** Graceful shutdown and cancellation handling

## Integration Patterns

### Inter-Crate Communication
- **Shared Types:** Common types defined in dedicated modules
- **Event-Driven:** Loose coupling through event/message passing
- **Interface Contracts:** Well-defined APIs with documented contracts

### External System Integration
- **Transport Abstraction:** Pluggable transport layer for different protocols
- **Adapter Pattern:** Adapters for external APIs and systems
- **Configuration Management:** Structured configuration with validation

### Testing Architecture
- **Unit Tests:** Comprehensive testing of individual components
- **Integration Tests:** End-to-end testing of complete workflows
- **Mock Implementations:** Test doubles for external dependencies
- **Property-Based Testing:** Where appropriate for complex logic

## Quality Assurance

### Code Quality Gates
- **Compilation:** Zero warnings with strict linting
- **Testing:** Minimum 80% test coverage across all crates
- **Documentation:** Comprehensive documentation with working examples
- **Performance:** Benchmarking for critical paths

### Architectural Governance
- **Design Reviews:** Architectural changes require explicit approval
- **Technical Debt Tracking:** All technical debt documented with remediation plans
- **Pattern Consistency:** Enforcement of workspace-wide patterns and standards
- **Security Reviews:** Security considerations for all external integrations

## Context Inheritance Model

### Memory Bank Hierarchy
```
.copilot/memory_bank/
├── workspace/                 # Workspace-level governance
│   ├── shared_patterns.md     # Technical standards and patterns
│   ├── workspace_architecture.md  # This document
│   └── project_brief.md       # Overall vision and objectives
├── sub_projects/              # Project-specific context
│   ├── airs-mcp/             # MCP client context
│   └── airs-memspec/         # Memory system context
└── context_snapshots/         # Point-in-time context captures
```

### Context Switching
- **Current Context:** Active project tracking with automatic updates
- **Context Correlation:** Cross-project dependency and interaction tracking
- **Progress Aggregation:** Workspace-wide progress and health monitoring

### Shared Resource Management
- **Documentation Standards:** Consistent documentation patterns across projects
- **Build Configuration:** Shared build tools and configuration
- **Development Environment:** Consistent tooling and development practices

## Evolution Strategy

### Extensibility Points
- **Transport Layer:** New transport implementations for different protocols
- **Parser Extensions:** Additional content parsers for different formats
- **Integration APIs:** Extension points for external tool integration

### Migration Patterns
- **Backward Compatibility:** Careful consideration of breaking changes
- **Feature Flags:** Gradual rollout of new functionality
- **Documentation Updates:** Architecture documentation kept current with changes

### Performance Considerations
- **Resource Management:** Efficient memory and CPU usage patterns
- **Concurrency:** Safe concurrent access to shared resources
- **Monitoring:** Performance monitoring and alerting capabilities
