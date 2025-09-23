# Architecture Decision Record Registry - airs-mcpserver-fs

**Last Updated**: 2025-09-23  
**Total ADRs**: 2 (Migrated from Legacy)  
**Active ADRs**: 2  
**Superseded ADRs**: 0  
**Migration Status**: Complete - All Legacy ADRs Migrated

## Migration Notice

**Source**: All ADRs migrated from `airs-mcp-fs` project  
**Status**: Review required for applicability to new architecture  
**Action**: Assess if decisions still apply to airs-mcpserver-fs

## Decision Categories

### System Architecture
- **Active**: 1 ADRs (Migrated)
  - [ADR-001: Foundation Architecture Patterns](./ADR-001-foundation-architecture-patterns.md) *(Assessment Required)*
- **Superseded**: 0 ADRs

### Integration Patterns
- **Active**: 1 ADRs (Migrated)
  - [ADR-002: MCP Server Architecture Decisions](./ADR-002-mcp-server-architecture-decisions.md) *(Assessment Required)*
- **Superseded**: 0 ADRs

### Technology Selection  
- **Active**: 0 ADRs
- **Superseded**: 0 ADRs

### Security & Compliance
- **Active**: 0 ADRs
- **Superseded**: 0 ADRs

### Performance Strategy
- **Active**: 0 ADRs
- **Superseded**: 0 ADRs

## Chronological Decision History

### 2025-09-23 (Migration)
- **Migration Complete**: All legacy ADRs transferred to airs-mcpserver-fs
- **Assessment Required**: Review applicability of each decision to new architecture

### 2025-08-25 (Legacy Origin)
- **[ADR-002: MCP Server Architecture Decisions](./ADR-002-mcp-server-architecture-decisions.md)** - **ACCEPTED** *(MIGRATED)*
  - **Impact**: High - Establishes MCP server foundation architecture
  - **Scope**: airs-mcp foundation leverage, STDIO transport, ToolProvider patterns
  - **Drivers**: Claude Desktop compatibility, development velocity, ecosystem alignment
  - **Migration Status**: May still apply - assessment required

### 2025-08-22 (Legacy Origin)
- **[ADR-001: Foundation Architecture Patterns](./ADR-001-foundation-architecture-patterns.md)** - **ACCEPTED** *(MIGRATED)*
  - **Impact**: High - Establishes fundamental project architecture
  - **Scope**: Workspace dependency management, lib.rs patterns, testing strategy
  - **Drivers**: Workspace consistency, development velocity, standard compliance
  - **Migration Status**: Likely still applies - assessment required

## Assessment Required

### Decision Applicability Review
1. **ADR-001**: Do foundation patterns still apply to airs-mcpserver-fs?
2. **ADR-002**: Is MCP server architecture consistent between old and new?
3. **New Decisions**: Are there architectural decisions unique to airs-mcpserver-fs?

### Action Items
- [ ] Review ADR-001 for continued applicability
- [ ] Review ADR-002 for architecture consistency  
- [ ] Document any new architectural decisions made during migration
- [ ] Update decision status based on new architecture

## Pending Decisions for Current Architecture

### High Priority Decisions Needed
1. **CLI Architecture Design** - Module structure and command handling patterns (Task 003)
2. **Migration Compatibility** - Backward compatibility strategies with legacy tools
3. **Security Framework Adaptation** - How security decisions apply to new architecture