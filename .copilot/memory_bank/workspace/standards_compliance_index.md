# Standards Compliance Index - Quick Reference

This document provides quick links to all standards compliance documentation across the workspace, following the pattern: **Workspace Standards (Rules) → Project Compliance (Applied Rules)**.

## Workspace-Level Standards (The Rules)

### Core Technical Standards
**File**: `workspace/shared_patterns.md`

**Contains**:
- **§2.1**: 3-Layer Import Organization (std → third-party → internal)
- **§3.2**: chrono DateTime<Utc> Standard (mandatory for all time operations)
- **§4.3**: Module Architecture Patterns (mod.rs organization principles)
- **§5.1**: Dependency Management (workspace centralization patterns)

**Usage**: Universal standards that ALL sub-projects must follow

### Code Quality Standards
**File**: `workspace/zero_warning_policy.md`

**Contains**:
- **Zero Warning Policy**: All projects must compile with zero warnings
- **Warning Resolution Strategies**: Dead code, unused imports, documentation tests
- **Enforcement**: CI/CD pipeline requirements

**Usage**: Quality gate for all implementations

### Technical Debt Management
**File**: `workspace/technical_debt_management.md`

**Contains**:
- **Debt Classification**: Architectural, code quality, documentation, testing, performance
- **Identification Process**: Inline documentation patterns and GitHub issue tracking
- **Remediation Planning**: Prioritization and resolution strategies

**Usage**: Framework for managing technical debt across workspace

## airs-mcp Project Compliance (Applied Rules)

### OAuth Module Standards Compliance
**File**: `sub_projects/airs-mcp/tasks/task_022_oauth_technical_standards.md`

**Contains**:
- **Standards Compliance Checklist**: Verification against workspace/shared_patterns.md
- **Evidence Documentation**: Proof of standards application with code examples
- **Compliance Results**: 328 tests passing, zero warnings, clean compilation

**Usage**: Example of how to document workspace standards application

### OAuth 2.1 Authentication Standards
**File**: `sub_projects/airs-mcp/oauth2_rfc_specifications.md`

**Contains**:
- **RFC 9728**: OAuth 2.0 Protected Resource Metadata (complete implementation guide)
- **RFC 7636**: Proof Key for Code Exchange (PKCE) with S256 method requirements
- **RFC 8707**: Resource Indicators for OAuth 2.0 (prevents confused deputy attacks)  
- **RFC 6749**: OAuth 2.0 Authorization Framework (core authorization flows)

**Usage**: Protocol-specific standards for OAuth 2.1 implementation (TASK014)

### MCP Protocol Standards
**File**: `sub_projects/airs-mcp/mcp_official_specification.md`

**Contains**:
- **MCP 2025-06-18**: Current specification with OAuth 2.1 integration requirements
- **JSON-RPC 2.0**: Base protocol for MCP message format
- **Security Architecture**: Client-host-server isolation boundaries
- **OAuth Integration**: Mandatory HTTP transport authentication requirements

**Usage**: Protocol-specific standards for MCP integration (TASK014)

### Implementation Integration Documentation
**File**: `sub_projects/airs-mcp/tasks/task_014_oauth2_1_enterprise_authentication.md`

**Contains**:
- **Standards Convergence**: OAuth 2.1 + MCP + Workspace requirements mapping
- **Workspace Standards Reference**: Links to applicable workspace standards
- **Implementation Architecture**: 3-phase middleware-based design following workspace patterns
- **Compliance Foundation**: References TASK022 workspace standards application

**Usage**: Complete implementation roadmap with workspace standards integration

### Module Architecture Documentation
**File**: `sub_projects/airs-mcp/oauth2_module_architecture.md`

**Contains**:
- **7-Module Structure**: OAuth 2.1 implementation following workspace patterns
- **Integration Patterns**: Axum middleware with zero HTTP transport modifications
- **Dependencies**: Complete crate dependencies following workspace dependency management
- **Testing Strategy**: Unit and integration testing patterns
- **Workspace Compliance**: Architecture designed for workspace standards adherence

**Usage**: Technical architecture following workspace standards for OAuth 2.1 implementation

## Standards Application Pattern

### The Architecture: Rules → Applied Rules

```
Workspace Standards (Universal Rules)
    ↓ Reference
Project-Specific Standards (Protocol Rules)  
    ↓ Reference + Apply
Implementation Tasks (Applied Rules + Evidence)
    ↓ Verify
Compliance Documentation (Proof of Application)
```

### Example: OAuth Standards Application

1. **Workspace Rules**: `workspace/shared_patterns.md` §3.2 - chrono DateTime<Utc> mandatory
2. **Protocol Rules**: `oauth2_rfc_specifications.md` - JWT token validation requirements  
3. **Applied Rules**: `task_014_oauth2_1_enterprise_authentication.md` - Implementation plan
4. **Evidence**: `task_022_oauth_technical_standards.md` - Compliance verification

## Reference Locations in Memory Bank

### Core Memory Bank References
- **tech_context.md**: Standards compliance section with document links
- **active_context.md**: Current standards compliance status with workspace references
- **system_patterns.md**: Standards compliance architecture patterns

### Workspace-Level References  
- **workspace/shared_patterns.md**: Universal technical standards (THE RULES)
- **workspace/standards_compliance_index.md**: This quick reference document
- **workspace/zero_warning_policy.md**: Code quality standards
- **workspace/technical_debt_management.md**: Debt management framework

## Quick Access Commands

```bash
# View workspace standards (THE RULES)
cat .copilot/memory_bank/workspace/shared_patterns.md

# View OAuth compliance evidence (APPLIED RULES)
cat .copilot/memory_bank/sub_projects/airs-mcp/tasks/task_022_oauth_technical_standards.md

# View OAuth implementation plan (RULES APPLICATION)
cat .copilot/memory_bank/sub_projects/airs-mcp/tasks/task_014_oauth2_1_enterprise_authentication.md
```

## Development Usage

### For New Project Development
1. **Start with workspace standards**: Review `workspace/shared_patterns.md` for applicable rules
2. **Document protocol standards**: Create protocol-specific standards documentation
3. **Plan compliance**: Reference workspace standards in implementation tasks
4. **Track evidence**: Document compliance verification with examples

### For TASK014 OAuth Implementation
1. **Workspace Foundation**: ✅ Complete - OAuth module workspace-compliant (TASK022)
2. **Protocol Standards**: Review `oauth2_rfc_specifications.md` + `mcp_official_specification.md`
3. **Implementation Plan**: Follow `task_014_oauth2_1_enterprise_authentication.md`
4. **Maintain Compliance**: Apply workspace standards throughout implementation

This architecture ensures workspace standards are the single source of truth while project implementations provide evidence of proper application.
