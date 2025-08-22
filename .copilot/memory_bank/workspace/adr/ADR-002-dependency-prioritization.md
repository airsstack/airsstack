# ADR-002: AIRS Foundation Crate Dependency Prioritization

**Date:** 2025-08-22  
**Status:** Accepted  
**Tags:** workspace, dependencies, standards  

## Context

The AIRS workspace contains multiple foundation crates (`airs-mcp`, `airs-mcp-fs`, `airs-memspec`) that serve as the architectural foundation for all external projects. The organization of dependencies in the root `Cargo.toml` communicates dependency hierarchy and affects developer understanding of the codebase structure.

## Decision

**AIRS foundation crates MUST be prioritized and organized at the top of workspace dependencies in root Cargo.toml.**

### Required Organization Pattern
```toml
[workspace.dependencies]
# Layer 1: AIRS Foundation Crates (MUST be at top)
airs-mcp = { path = "crates/airs-mcp" }
airs-mcp-fs = { path = "crates/airs-mcp-fs" }  
airs-memspec = { path = "crates/airs-memspec" }

# Layer 2: Core Runtime Dependencies  
tokio = { version = "1.47", features = ["full"] }
futures = { version = "0.3" }

# Layer 3: External Dependencies (organized by category)
# ... rest of external dependencies
```

## Rationale

### Technical Benefits
- **Dependency Hierarchy Clarity**: Internal AIRS crates represent the foundation layer that everything else builds upon
- **Version Management**: Foundation crates use path dependencies and require different update strategies than external crates
- **Build System Optimization**: Foundation crates are processed first in dependency resolution

### Development Workflow Benefits  
- **Maintenance Visibility**: Changes to foundation crates have the highest impact and should be immediately visible
- **Developer Experience**: Developers see internal dependencies first when reviewing workspace configuration
- **Onboarding**: New developers immediately understand the foundational architecture

### Architectural Communication
- **Clear Layering**: Visual separation communicates that AIRS crates are the foundation layer
- **Dependency Direction**: Establishes that external dependencies support AIRS functionality, not vice versa
- **Design Intent**: Makes it clear that AIRS crates define the core abstractions

## Implementation

### Current State (Post-Refactor)
✅ `airs-mcp` moved to top of dependencies section  
✅ Clear separation between foundation and external dependencies  
✅ Documented as workspace standard §5.1  

### Enforcement
- Added to workspace standards enforcement instructions
- Included in compliance checklists for all future development
- Automated validation in development workflows

## Consequences

### Positive
- Clear visual hierarchy in dependency management
- Improved developer understanding of architectural layers
- Consistent workspace organization across all AIRS projects
- Better maintenance workflow for foundation crate updates

### Negative
- Minor disruption to existing dependency organization patterns
- Requires updating existing development practices and documentation

### Mitigation
- All changes documented in workspace standards
- Clear migration patterns provided for future updates
- Integration with existing compliance checking workflows

## Review Schedule

- **Next Review**: 2025-11-22 (3 months)
- **Trigger Conditions**: Addition of new AIRS foundation crates, major dependency reorganization needs
- **Success Metrics**: Consistent application across all sub-projects, improved developer onboarding feedback
