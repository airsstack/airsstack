# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common Development Commands

### Building and Testing
```bash
# Build entire workspace
cargo build --workspace

# Build with release optimizations
cargo build --release --workspace

# Check code without building
cargo check --workspace

# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p airs-mcp

# Run tests with output visible
cargo test -- --nocapture
```

### Code Quality
```bash
# Format all code (mandatory before commits)
cargo fmt --all

# Lint with Clippy - must pass with zero warnings
cargo clippy --workspace --all-targets --all-features

# Run security audit
cargo audit

# Generate documentation and open in browser  
cargo doc --open --workspace
```

### Working with Examples
```bash
# Test MCP server (Claude Desktop integration)
cd crates/airs-mcp/examples/simple-mcp-server
cargo run

# Test MCP client (automatic server interaction)  
cd crates/airs-mcp/examples/simple-mcp-client
cargo run

# Test production filesystem MCP server
cd crates/airs-mcp-fs
cargo build --release
./target/release/airs-mcp-fs generate-config
```

### Documentation
```bash
# Serve workspace documentation locally
cd docs && mdbook serve --open
# Access at: http://localhost:3000

# Build workspace documentation
cd docs && mdbook build

# Serve individual crate documentation
cd crates/airs-mcp/docs && mdbook serve --open --port 3001
cd crates/airs-mcp-fs/docs && mdbook serve --open --port 3002
cd crates/airs-memspec/docs && mdbook serve --open --port 3003

# Build individual crate documentation
cd crates/airs-mcp/docs && mdbook build
cd crates/airs-mcp-fs/docs && mdbook build
cd crates/airs-memspec/docs && mdbook build

# Build all crate documentation at once
for crate in crates/*/docs; do
  if [ -f "$crate/book.toml" ]; then
    echo "Building documentation for $crate"
    (cd "$crate" && mdbook build)
  fi
done
```

### Benchmarking
```bash
# Run performance benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench -p airs-mcp --bench message_processing
```

## Architecture Overview

AirsStack is a **Cargo Workspace** containing composable AI building blocks with a focus on privacy, performance, and sovereignty. The architecture follows a layered, modular design:

### Workspace Structure
```
airsstack/
├── crates/                 # Core building blocks
│   ├── airs-mcp/          # MCP implementation (✅ Production Ready)
│   ├── airs-mcp-fs/       # Filesystem MCP tools (✅ Production Ready)  
│   └── airs-memspec/      # Memory bank specification
├── .copilot/              # AI-assisted development configuration
├── docs/                  # Comprehensive documentation ecosystem
└── target/                # Build artifacts
```

### Core Components

**airs-mcp**: Complete Model Context Protocol implementation
- JSON-RPC 2.0 foundation with full MCP 2024-11-05 specification compliance
- High-level type-safe APIs for both server and client
- Advanced transport layer with SubprocessTransport
- Claude Desktop integration verified with working examples
- OAuth 2.1 authentication support

**airs-mcp-fs**: Production-ready filesystem bridge  
- Enterprise-grade security (97.5/100 security audit score)
- Human-in-the-loop approval workflows
- Advanced binary processing (images, PDFs, archives)
- Comprehensive path traversal protection

**airs-memspec**: Memory bank specification and tooling
- Structured AI-assisted development workflow management
- Context preservation and snapshot functionality
- Multi-project workspace support

### Key Architectural Patterns

**Layered Dependencies** (mandatory workspace standard):
1. **Layer 1**: AIRS foundation crates (top priority)
2. **Layer 2**: Core runtime (Tokio, Futures)  
3. **Layer 3**: External dependencies (categorized)

**Import Organization** (mandatory in all Rust files):
```rust
// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports  
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::shared::protocol::core::McpMethod;
```

**Time Management Standard**:
- ALL time operations must use `chrono::DateTime<Utc>`
- Never use `std::time::SystemTime` in business logic

**Module Architecture**:
- `mod.rs` files contain ONLY module declarations and re-exports
- No implementation code in `mod.rs` files
- Clear separation of concerns across modules

## AI-Assisted Development

This project uses sophisticated AI-assisted development with specific patterns:

### Specification-Driven Workflow
The project follows a 6-phase development cycle:
1. **ANALYZE**: Understand requirements (EARS notation)
2. **DESIGN**: Create technical design and implementation plan
3. **IMPLEMENT**: Write production-quality code
4. **VALIDATE**: Verify implementation meets requirements
5. **REFLECT**: Improve codebase and update documentation
6. **HANDOFF**: Package work for review and deployment

### Memory Bank System
- **Workspace-Level Context**: Shared vision, architecture, standards
- **Component Memory Banks**: Per-component requirements, design, progress
- **Task Management**: Detailed tracking with status and subtasks
- **Context Snapshots**: Operational state preservation

### Human-AI Collaboration Model
- **Strategic Human Control**: Architecture decisions remain human-driven
- **AI-Accelerated Implementation**: Code generation and routine tasks enhanced by AI
- **Transparent Process**: Every action and decision logged
- **Specification-Driven**: Clear requirements, validated designs, thorough testing

## Memory Bank Integration (Mandatory for Warp Agents)

This project uses an advanced Multi-Project Memory Bank system for context-aware AI assistance. **CRITICAL**: Every Warp agent MUST consult both this `WARP.md` file AND the memory bank context before any development work.

### Memory Bank Custom Instructions
**Reference**: `.copilot/instructions/multi_project_memory_bank.instructions.md`

This file contains comprehensive instructions for:
- Multi-project context management
- Workspace-aware development workflows
- Technical documentation frameworks
- Task management and progress tracking
- Context snapshots and operational continuity

### Integration Requirements for Warp Agents

**MANDATORY WORKFLOW**: When `WARP.md` is parsed by any Warp agent, the agent MUST:

1. **Read Memory Bank Context**: Always consult `.copilot/instructions/multi_project_memory_bank.instructions.md`
2. **Check Current Context**: Read `.copilot/memory_bank/current_context.md` for active sub-project
3. **Load Workspace Context**: Review all files in `.copilot/memory_bank/workspace/`
4. **Load Sub-Project Context**: Review all relevant files in `.copilot/memory_bank/sub_projects/[active_project]/`
5. **Verify Standards Compliance**: Ensure all work follows workspace standards defined in both documents

### Memory Bank Structure Integration

The memory bank operates alongside this `WARP.md` file to provide:
- **Context Preservation**: Maintains development state across sessions
- **Task Management**: Structured tracking with progress logs and status
- **Technical Documentation**: ADRs, technical debt records, and knowledge capture
- **Workspace Standards Enforcement**: Compliance verification and evidence tracking
- **Multi-Project Coordination**: Shared patterns and cross-crate dependencies

### Workspace Standards Compliance

Both `WARP.md` and the memory bank system enforce:
- **Zero Warning Policy**: All code must compile cleanly
- **Mandatory Code Quality**: Clippy compliance, proper error handling
- **Documentation Requirements**: Comprehensive rustdoc and technical docs
- **Technical Debt Management**: Structured tracking and remediation
- **Testing Standards**: Unit, integration, and property-based testing

**Agent Responsibility**: Verify compliance evidence is documented in both the memory bank AND follows the standards defined in this `WARP.md` file.

## Development Standards (Mandatory)

### Zero Warning Policy
- `cargo check --workspace` must return zero warnings
- `cargo clippy --workspace` must pass completely
- All tests must pass before any code submission

### Code Quality Requirements
- Comprehensive rustdoc comments for public APIs
- Proper `Result` and `Option` usage for error handling
- Minimal, well-maintained dependencies
- Unit tests alongside code, integration tests in `tests/` directories

### Technical Debt Management
When introducing technical debt, document with:
```rust
// TODO(DEBT): [Category] - [Description]
// Impact: [Performance/Maintainability impact]
// Remediation: [Specific fix needed]
// Reference: [GitHub issue if created]
// Workspace Standard: [Which standard is violated, if any]
```

Categories: `DEBT-ARCH`, `DEBT-QUALITY`, `DEBT-DOCS`, `DEBT-TEST`, `DEBT-PERF`

### Error Handling Patterns
- Never use `unwrap()`, `expect()`, or `panic!` in production code (enforced by Clippy)
- Allowed in test code only
- Use proper error types with `thiserror` crate
- Comprehensive error context and recovery strategies

## Testing Strategy

### Test Organization
- **Unit Tests**: In-module tests using `#[cfg(test)]`
- **Integration Tests**: Separate `tests/` directory per crate
- **Documentation Tests**: Embedded examples in rustdoc comments
- **Property-Based Testing**: Using `proptest` for complex scenarios
- **Performance Testing**: Benchmarks for critical paths

### Test Data and Fixtures
- Use `tempfile` for temporary test directories
- Use `assert_cmd` for CLI testing
- Use `wiremock` for HTTP mocking in tests

## MCP Integration Patterns

### Server Development
- Extend `examples/simple-mcp-server` as starting point
- Implement `McpServer` trait with proper error handling
- Use structured logging with `tracing` crate
- Configure for Claude Desktop integration

### Client Development  
- Use high-level `McpClient` API with automatic subprocess management
- Handle transport failures gracefully
- Implement proper correlation for request/response matching
- Use connection pooling for HTTP transports

### Security Considerations
- All file operations go through `airs-mcp-fs` security layer
- Implement human-in-the-loop approval for sensitive operations
- Use comprehensive path traversal protection
- Log all filesystem operations for audit trail

## Performance Optimization

### Key Performance Areas
- Message processing and correlation latency
- Transport layer throughput and connection management  
- Filesystem operation security overhead
- Memory allocation patterns in async contexts

### Benchmarking
- Run benchmarks before/after changes: `cargo bench --workspace`
- Focus on correlation manager performance under load
- Monitor HTTP server performance with concurrent connections
- Profile memory usage in long-running operations

## Documentation Requirements

### Required Documentation Files
- **README.md**: Per-crate overview and quick start
- **CHANGELOG.md**: Semantic versioning with detailed change logs
- **API Documentation**: Comprehensive rustdoc for all public items
- **Examples**: Working examples with comprehensive comments

### Documentation Architecture

**Workspace-Level Documentation** (`docs/`):
- Comprehensive project overview and architecture guides
- Cross-crate integration patterns and workflows
- High-level user journeys and getting started guides
- Workspace standards and development processes

**Per-Crate Documentation** (`crates/<project>/docs/`):
- Each crate maintains its own MDBook documentation
- Crate-specific technical guides, API references, and tutorials
- Implementation details, performance characteristics, and usage patterns
- Examples, testing strategies, and troubleshooting guides

### MDBook Documentation Standards

**Structure Requirements**:
- Each crate MUST maintain a `docs/` directory with `book.toml`
- Documentation structure: `src/SUMMARY.md`, organized chapters
- Cross-references between workspace and crate documentation
- Consistent styling and navigation patterns

**Content Standards**:
- Use MDBook for comprehensive guides (both workspace and per-crate)
- Maintain separate technical guides, project overviews, and resource guides
- Include real-world examples from actual development
- Provide multiple user paths (AI teams, Rust+AI projects, documentation teams)
- Ensure documentation is buildable and serves correctly on different ports

**Per-Crate Documentation Responsibilities**:
- **Engineers**: Maintain crate-specific documentation alongside code changes
- **Users**: Can access individual crate documentation at different ports (3001, 3002, 3003, etc.)
- **Integration**: Cross-link between workspace docs and relevant crate docs
- **Maintenance**: Regular review and updates as part of development workflow

**Documentation Commands**:
```bash
# Access workspace documentation
http://localhost:3000

# Access individual crate documentation
http://localhost:3001  # airs-mcp
http://localhost:3002  # airs-mcp-fs  
http://localhost:3003  # airs-memspec
```

## Privacy and Security

### Privacy-First Design
- No telemetry or data collection
- Local processing prioritized over cloud services
- Transparent data handling policies
- User control over all AI operations

### Security Standards
- Comprehensive security audit framework (see `airs-mcp-fs`)
- Regular vulnerability testing and remediation
- Path traversal protection in all file operations
- OAuth 2.1 implementation for secure authentication
