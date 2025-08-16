# Standards Compliance Index - Quick Reference

This document provides quick links to all standards compliance documentation across the workspace.

## airs-mcp Standards Documentation

### OAuth 2.1 Authentication Standards
**File**: `sub_projects/airs-mcp/oauth2_rfc_specifications.md`

**Contains**:
- **RFC 9728**: OAuth 2.0 Protected Resource Metadata (complete implementation guide)
- **RFC 7636**: Proof Key for Code Exchange (PKCE) with S256 method requirements
- **RFC 8707**: Resource Indicators for OAuth 2.0 (prevents confused deputy attacks)  
- **RFC 6749**: OAuth 2.0 Authorization Framework (core authorization flows)

**Usage**: Complete technical reference for TASK014 OAuth 2.1 middleware implementation

### MCP Protocol Standards
**File**: `sub_projects/airs-mcp/mcp_official_specification.md`

**Contains**:
- **MCP 2025-06-18**: Current specification with OAuth 2.1 integration requirements
- **JSON-RPC 2.0**: Base protocol for MCP message format
- **Security Architecture**: Client-host-server isolation boundaries
- **OAuth Integration**: Mandatory HTTP transport authentication requirements
- **Implementation Patterns**: Token audience validation, scope mapping, PKCE integration

**Usage**: Official MCP protocol requirements for enterprise authentication implementation

### Integration Documentation
**File**: `sub_projects/airs-mcp/tasks/task_014_oauth2_1_enterprise_authentication.md`

**Contains**:
- **Standards Convergence**: OAuth 2.1 + MCP requirements mapping
- **Implementation Architecture**: 3-phase middleware-based design
- **Scope Mapping**: MCP methods to OAuth scopes (`mcp:tools:execute`, etc.)
- **Security Requirements**: Token audience validation, PKCE S256, resource indicators

**Usage**: Complete implementation roadmap for OAuth 2.1 + MCP integration

## Reference Locations in Memory Bank

### Core Memory Bank References
- **tech_context.md**: Standards compliance section with document links
- **active_context.md**: Current OAuth 2.1 preparation status with reference documents
- **system_patterns.md**: Standards compliance architecture patterns

### Workspace-Level References  
- **workspace/shared_patterns.md**: Protocol standards documentation requirements
- **workspace/standards_compliance_index.md**: This quick reference document

## Quick Access Commands

```bash
# View OAuth 2.1 standards
cat .copilot/memory_bank/sub_projects/airs-mcp/oauth2_rfc_specifications.md

# View MCP protocol standards  
cat .copilot/memory_bank/sub_projects/airs-mcp/mcp_official_specification.md

# View implementation plan
cat .copilot/memory_bank/sub_projects/airs-mcp/tasks/task_014_oauth2_1_enterprise_authentication.md
```

## Development Usage

**For TASK014 Implementation**:
1. Start with `oauth2_rfc_specifications.md` for OAuth 2.1 technical requirements
2. Reference `mcp_official_specification.md` for MCP integration requirements  
3. Follow `task_014_oauth2_1_enterprise_authentication.md` for implementation roadmap
4. Check memory bank references in `tech_context.md`, `active_context.md`, `system_patterns.md`

**For Future Protocol Implementations**:
- Follow the standards compliance pattern established for OAuth 2.1 + MCP
- Create complete specification documentation before implementation
- Document standards convergence for multiple protocol integrations
- Update memory bank references for easy discovery

This index ensures that critical standards compliance documentation is never lost and can be easily referenced during development sessions.
