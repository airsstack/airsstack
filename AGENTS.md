# AGENTS.md

## Project Overview

**AirsStack** is a privacy-first, Rust-based AI infrastructure workspace providing composable building blocks for AI application development. The workspace implements Model Context Protocol (MCP) client/server architecture with advanced memory specification systems.

**Core Components:**
- `airs-mcp`: Production-ready JSON-RPC MCP client with correlation management
- `airs-memspec`: Workspace memory and context management system
- `airs-mcpserver-fs`: MCP filesystem server (production verified with Claude Desktop)

**Architecture:** Multi-crate Rust workspace with clean separation of concerns, async-first design, and enterprise-grade quality standards.

## Setup Commands

```bash
# Install dependencies and build workspace
cargo build --workspace

# Run development checks (mandatory before any changes)
cargo check --workspace
cargo clippy --workspace --all-targets --all-features  
cargo test --workspace

# Run specific crate tests
cargo test --package airs-mcp
cargo test --package airs-mcp-fs
cargo test --package airs-memspec

# Documentation generation
cargo doc --workspace --open

# Performance benchmarks
cargo bench
```

## Build and Test Workflow

### Pre-Development Setup
1. **Always run workspace validation first:**
   ```bash
   cargo check --workspace
   cargo clippy --workspace --all-targets --all-features
   cargo test --workspace
   ```

2. **Ensure zero warnings policy compliance** - ALL code must compile with zero warnings

### Development Workflow
1. **Check memory bank context** - Read relevant `.copilot/memory_bank/` files for current project state
2. **Apply workspace standards** - Follow patterns in `.copilot/memory_bank/workspace/shared_patterns.md`
3. **Run incremental tests** - Test specific components during development
4. **Validate before commit** - Complete workspace validation required

### VS Code Tasks Available
- `cargo check` - Workspace compilation check
- `cargo clippy` - Linting and best practices
- `cargo test` - Full test suite 
- `cargo test airs-mcp` - Specific crate testing

## Code Style Guidelines

### Rust Standards (Mandatory)
- **Import Organization** (§2.1): 3-layer structure (std → third-party → internal)
- **Time Management** (§3.2): Always use `chrono::DateTime<Utc>`, never `std::time::SystemTime`
- **Module Architecture** (§4.3): `mod.rs` files contain ONLY declarations and re-exports
- **Generic Types** (§1): Prefer zero-cost generics over trait objects for performance
- **Error Handling**: Use `Result<T, E>` consistently, avoid `unwrap()`/`expect()` in production code

### Example Code Patterns
```rust
// ✅ Correct import organization (§2.1)
use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::shared::protocol::core::McpMethod;
use crate::transport::http::config::HttpConfig;

// ✅ Correct time handling (§3.2)
use chrono::{DateTime, Utc};
let timestamp = Utc::now();

// ✅ Correct generic usage (§1)
pub struct Validator<J, S> 
where
    J: JwtValidator,
    S: ScopeValidator,
{
    jwt: J,
    scope: S,
}
```

### Quality Requirements
- **Zero Warnings**: All code must compile with zero warnings (`cargo clippy`)
- **Test Coverage**: Comprehensive unit and integration tests required
- **Documentation**: All public APIs must have doc comments with examples
- **Performance**: Sub-millisecond response times for core operations

## Memory Bank Management System

### Overview
This workspace uses a sophisticated memory bank system for context management and task tracking. AI agents should integrate with this system for optimal workflow.

### Memory Bank Structure
```
.copilot/memory_bank/
├── current_context.md              # Active sub-project tracking
├── workspace/                      # Workspace-level standards
│   ├── project_brief.md           # Overall vision and architecture
│   ├── shared_patterns.md         # Technical standards (MANDATORY)
│   ├── workspace_architecture.md  # High-level structure
│   └── technical_debt_management.md
├── sub_projects/                   # Individual project contexts
│   ├── airs-mcp/                  # MCP client project
│   ├── airs-memspec/             # Memory specification project
│   └── airs-mcpserver-fs/        # MCP server project
└── context_snapshots/             # Historical state preservation
```

### Memory Bank Workflow (MANDATORY for AI Agents)

#### Before Starting ANY Task:
1. **Read Current Context**: Check `.copilot/memory_bank/current_context.md` for active sub-project
2. **Load Workspace Standards**: Review `.copilot/memory_bank/workspace/shared_patterns.md`
3. **Read Sub-Project Context**: Load all core files for the active sub-project:
   - `project_brief.md` - Project foundation and scope
   - `active_context.md` - Current work focus and recent changes
   - `system_patterns.md` - Architecture and technical decisions
   - `tech_context.md` - Technology stack and constraints
   - `progress.md` - Current status and known issues
   - `tasks/_index.md` - Task management state

#### During Development:
1. **Apply Workspace Standards**: Follow all patterns from shared_patterns.md
2. **Update Documentation**: Keep memory bank files current with changes
3. **Track Technical Debt**: Document any shortcuts or compromises
4. **Maintain Task Status**: Update relevant task files with progress

#### Memory Bank Commands
Use these patterns when working with the memory bank:

- **Switch Context**: Update `current_context.md` when changing sub-projects
- **Update Memory Bank**: Review and update all relevant files after significant changes
- **Save Context Snapshot**: Preserve important project states for future reference
- **Task Management**: Create/update task files in the active sub-project's `tasks/` folder

### Technical Documentation Framework
Each sub-project maintains structured technical documentation:

- **`docs/debts/`**: Technical debt tracking with remediation plans
- **`docs/knowledges/`**: Architectural patterns and implementation knowledge  
- **`docs/adr/`**: Architecture Decision Records for significant technical choices

## Testing Instructions

### Test Strategy
- **Unit Tests**: Component isolation with comprehensive coverage
- **Integration Tests**: Cross-component interaction validation
- **Performance Tests**: Benchmarks for critical path operations
- **Compliance Tests**: MCP schema validation and protocol compliance

### Running Tests
```bash
# Full workspace test suite
cargo test --workspace

# Specific component testing
cargo test --package airs-mcp --lib
cargo test --package airs-mcp --test integration_tests

# Performance benchmarks  
cargo bench --package airs-mcp

# With output for debugging
cargo test --package airs-mcp -- --nocapture
```

### Test Requirements
- **All public APIs** must have unit tests with >95% coverage
- **Integration tests** required for cross-component interactions
- **Performance tests** for operations expected to be sub-millisecond
- **Error path testing** for all `Result` returning functions

## Security Considerations

### File System Operations
- **airs-mcp-fs** implements security-first filesystem access
- All file operations go through permission validation
- No direct filesystem access outside of designated secure interfaces

### MCP Protocol Security
- JSON-RPC message validation for all protocol interactions
- Type-safe deserialization prevents injection attacks
- Authenticated transport channels for production deployments

### Dependencies
- Regular security audits with `cargo audit`
- Minimal dependency surface area
- Prefer well-maintained crates with security track records

## Deployment

### Development Environment
```bash
# Clone and setup
git clone https://github.com/airsstack/airsstack.git
cd airsstack
cargo build --workspace

# Verify setup
cargo test --workspace
```

### Production Deployment
- **MCP Servers**: Deploy as standalone binaries or Docker containers
- **Integration**: Compatible with Claude Desktop and other MCP clients
- **Monitoring**: Structured logging with `tracing` crate
- **Configuration**: Environment-based configuration management

## Chat Modes Available

The workspace includes specialized chat modes for different development contexts:

- **`principal-software-engineer`**: Expert-level engineering guidance and technical leadership
- **`gilfoyle`**: Code quality review with focus on performance and best practices  
- **`software-engineer-agent-v1`**: Structured development workflow with spec-driven approach
- **`rust-gpt-4.1-beast-mode`**: Advanced Rust-specific development assistance

## Workspace Standards Enforcement

### Mandatory Compliance Checks
Before ANY code implementation, verify:

- **§2.1**: 3-Layer Import Organization (std → third-party → internal)
- **§3.2**: chrono DateTime<Utc> Standard (mandatory for all time operations)  
- **§4.3**: Module Architecture Patterns (mod.rs organization principles)
- **§5.1**: Dependency Management (AIRS foundation crates prioritization)

### Quality Gates (HARD Requirements)
- ✅ Zero compiler warnings across workspace
- ✅ All workspace standards applied to new code
- ✅ Task documentation follows reference patterns
- ✅ Technical debt properly categorized and tracked
- ✅ Evidence documentation for all compliance claims

### Violation Response Protocol
1. **Stop current work** and address violations immediately
2. **Reference specific workspace standard** being violated  
3. **Provide corrected implementation** following workspace patterns
4. **Document the fix** as compliance evidence
5. **Update relevant documentation** with resolution

## Contributing

### Pull Request Requirements
- **Title Format**: `[crate-name] Brief description`
- **Standards Compliance**: All workspace standards must be followed
- **Testing**: Full test suite must pass (`cargo test --workspace`)
- **Documentation**: Update relevant memory bank files and API docs
- **Zero Warnings**: `cargo clippy --workspace` must pass without warnings

### Memory Bank Updates
When making significant changes:
1. Update relevant memory bank files with new context
2. Document compliance evidence for workspace standards  
3. Update task files with progress and decisions made
4. Consider creating context snapshot for major milestones

This `AGENTS.md` provides AI coding agents with comprehensive context for working effectively within the AirsStack workspace while maintaining the high standards of engineering excellence established in the project.