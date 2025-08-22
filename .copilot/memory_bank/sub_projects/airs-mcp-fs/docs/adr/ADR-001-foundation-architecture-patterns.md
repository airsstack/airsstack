# ADR-001: Foundation Architecture Patterns

**Status**: Accepted  
**Date**: 2025-08-22  
**Deciders**: [@hiraq, GitHub Copilot]  
**Technical Story**: task_001_project_foundation_setup - Establishing architectural foundation for airs-mcp-fs

## Context and Problem Statement

The airs-mcp-fs project requires fundamental architectural decisions that will determine development velocity, maintainability, and consistency with the broader AIRS workspace ecosystem. Multiple approaches exist for dependency management, module organization, and testing strategies, each with different trade-offs for long-term project success.

Key forces at play:
- **Workspace Consistency**: Need alignment with existing AIRS ecosystem patterns
- **Development Velocity**: Architecture must enable rapid, iterative development
- **Technical Debt Prevention**: Early decisions have compounding effects on code quality
- **Standard Compliance**: Must follow Rust community conventions while respecting workspace standards
- **Future Maintainability**: Architecture must support extension and modification

## Decision Drivers

- **Workspace Standards Compliance**: Must align with documented patterns in `workspace/shared_patterns.md`
- **Development Team Efficiency**: Minimize cognitive overhead for developers familiar with Rust conventions
- **Dependency Management Complexity**: Large projects require centralized dependency control
- **Module Coupling Concerns**: Prevent tight coupling between architectural layers
- **Testing Strategy Requirements**: Enable comprehensive testing without architectural overhead
- **Build Performance**: Compilation time and dependency resolution efficiency

## Considered Options

### Option 1: Centralized Workspace Dependency Management
- **Approach**: All dependencies defined in root Cargo.toml, inherited via `.workspace = true`
- **Pros**: 
  - Single source of truth for versions across entire workspace
  - Simplified dependency updates and security patching
  - Consistent versions prevent diamond dependency issues
  - Reduces individual Cargo.toml complexity
- **Cons**: 
  - Root Cargo.toml becomes large and complex
  - Less flexibility for crate-specific dependency optimization
- **Implementation effort**: Low

### Option 2: Decentralized Crate-Specific Dependencies
- **Approach**: Each crate manages its own dependencies independently
- **Pros**: 
  - Maximum flexibility for per-crate optimization
  - Smaller, focused Cargo.toml files
  - Independent dependency evolution
- **Cons**: 
  - Version conflicts across workspace
  - Duplicate dependency specifications
  - Increased maintenance overhead
  - Inconsistent security patch application
- **Implementation effort**: Medium

### Option 3: lib.rs as Pure Module Coordinator
- **Approach**: lib.rs contains only module declarations and re-exports (like mod.rs)
- **Pros**: 
  - Clear separation of concerns
  - Follows established mod.rs patterns
  - Prevents lib.rs from becoming monolithic
  - Easier to locate type definitions
- **Cons**: 
  - Slight indirection for finding type definitions
  - More files to navigate initially
- **Implementation effort**: Low

### Option 4: lib.rs as Type Definition Hub
- **Approach**: lib.rs contains core types and business logic
- **Pros**: 
  - Single location for core types
  - Fewer files to navigate
  - Traditional library pattern
- **Cons**: 
  - lib.rs becomes monolithic over time
  - Mixing of concerns (coordination + implementation)
  - Harder to maintain as project grows
- **Implementation effort**: Low

### Option 5: Inline Unit Tests (Standard Rust)
- **Approach**: `#[cfg(test)]` modules within each source file
- **Pros**: 
  - Standard Rust convention
  - Tests stay close to implementation
  - Easy to find relevant tests
  - Compiler optimizes out test code in release builds
- **Cons**: 
  - Source files become longer
  - Slight cognitive overhead when reading implementation
- **Implementation effort**: Low

### Option 6: Separate Unit Test Files
- **Approach**: Dedicated test files for each module
- **Pros**: 
  - Clean separation of test and implementation code
  - Shorter source files
- **Cons**: 
  - Non-standard Rust pattern
  - Tests can become disconnected from implementation
  - Additional file navigation overhead
- **Implementation effort**: Medium

## Decision Outcome

**Chosen options**: 
- **Option 1**: Centralized Workspace Dependency Management
- **Option 3**: lib.rs as Pure Module Coordinator  
- **Option 5**: Inline Unit Tests (Standard Rust)

### Rationale
1. **Centralized Dependencies**: Workspace-level management aligns with AIRS ecosystem patterns and prevents version conflicts that could impede integration
2. **Pure lib.rs**: Maintains clean architectural boundaries and follows established workspace patterns from airs-mcp
3. **Inline Testing**: Standard Rust conventions ensure maintainability and developer familiarity

### Positive Consequences
- **Consistency**: All AIRS sub-projects follow identical dependency management patterns
- **Maintainability**: Clear separation of concerns prevents architectural drift
- **Developer Experience**: Standard Rust patterns reduce onboarding time
- **Build Performance**: Centralized dependencies enable better optimization
- **Security**: Unified dependency updates across entire workspace

### Negative Consequences
- **Root Cargo.toml Complexity**: Will require careful organization as workspace grows
- **Initial Setup Overhead**: More upfront configuration compared to isolated approach
- **Slight Indirection**: Type definitions require navigation to specific modules

## Implementation Plan

### Phase 1: Dependency Infrastructure (Subtasks 1.1-1.2)
1. Add airs-mcp-fs specific dependencies to root Cargo.toml with latest stable versions:
   - `image = "0.25"` (binary processing)
   - `infer = "0.16"` (format detection)
   - `config = "0.14"` (configuration management)
   - `path-clean = "1.0"` (path utilities)
   - `glob = "0.3"` (pattern matching)
   - `assert_fs = "1.1"` (testing utilities)
2. Configure airs-mcp-fs Cargo.toml to inherit all dependencies via `.workspace = true`

### Phase 2: Module Foundation (Subtasks 1.3-1.4)
1. Create directory structure: `mcp/`, `security/`, `filesystem/`, `binary/`, `config/`
2. Implement lib.rs as pure coordinator with module declarations and re-exports only
3. Create main.rs binary entry point following workspace patterns

### Phase 3: Testing Framework (Subtasks 1.5-1.6)
1. Implement inline unit tests in each module using `#[cfg(test)]` blocks
2. Set up integration tests in `tests/` directory
3. Validate workspace standards compliance

## Validation Approach

### Success Criteria
- `cargo check --workspace` returns zero warnings
- `cargo clippy --workspace --all-targets --all-features` passes without issues
- All unit tests pass with `cargo test --workspace`
- No dependency version conflicts across workspace
- lib.rs contains only module declarations and re-exports

### Timeline for Evaluation
- **Immediate**: Build system validation during implementation
- **1 week**: Developer experience assessment during task_002 implementation
- **1 month**: Architectural pattern effectiveness during Phase 2 development

### Key Metrics to Monitor
- Build time performance
- Dependency resolution time
- Developer onboarding speed
- Architectural pattern consistency across modules

## Compliance with Workspace Standards

### Direct Alignment
- **§5.1 Dependency Management**: Centralized workspace dependency control
- **§4.3 Module Architecture**: lib.rs follows mod.rs patterns (declarations only)
- **§2.1 Import Organization**: 3-layer import structure enforced in all files
- **§3.2 chrono Standard**: DateTime<Utc> for all time operations

### New Patterns Established
- **Pure lib.rs Coordinator**: Establishes precedent for other AIRS sub-projects
- **Inline Unit Testing**: Standard Rust conventions adopted workspace-wide
- **Latest Stable Dependencies**: Consistent policy for security and feature updates

## Links and References

### Related Documentation
- **Workspace Standards**: `workspace/shared_patterns.md` (§4.3, §5.1)
- **Task Context**: `tasks/task_001_project_foundation_setup.md`
- **Technical Context**: `tech_context.md` (dependency specifications)

### Implementation Tracking
- **GitHub Issue**: [TBD - will be created during implementation]
- **Memory Bank**: All decisions documented in task_001 progress log

### External References
- [Rust Book - Workspace Dependencies](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust Testing Conventions](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)

## Notes

### Future Decision Points
- **Binary Processing Libraries**: Detailed selection of image/PDF processing crates
- **Security Framework Architecture**: Human-in-the-loop approval system design
- **MCP Integration Patterns**: Tool registration and capability discovery

### Assumptions Made
- AIRS workspace will continue to grow with additional sub-projects
- Rust ecosystem stability for chosen dependency versions
- Development team familiarity with standard Rust conventions
- Workspace standards will remain stable during foundation implementation

### Alternative Names Considered
- "Project Foundation Architecture"
- "Workspace Integration Patterns"
- "Dependency and Module Organization Strategy"
