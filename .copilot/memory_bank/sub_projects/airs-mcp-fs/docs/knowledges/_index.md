# Knowledge Documentation Index - airs-mcp-fs

**Last Updated**: 2025-08-29  
**Total Knowledge Docs**: 5  
**Categories**: 3 (Architecture, Integration, Security)

## Knowledge Categories

### Architecture
**Documentation Count**: 2  
**Complexity Level**: Medium  
**Maintenance Priority**: High

#### Active Documents
- **[Dependency Injection Architecture](./dependency-injection-architecture.md)**
  - **Focus**: Trait-based dependency injection patterns, constructor refactoring, testability improvements
  - **Complexity**: Medium - Design pattern implementation and migration strategy
  - **Updated**: 2025-08-25
  - **Related**: FilesystemMcpServer refactoring, SOLID principles compliance

- **[Permissions Module Refactoring Decision](./architecture/permissions-refactoring-decision.md)** ⭐ **NEW**
  - **Focus**: Large module refactoring strategy, sub-module architecture, comprehensive documentation approach
  - **Complexity**: Medium - Architectural refactoring and developer experience improvement
  - **Updated**: 2025-08-29
  - **Related**: Security framework maintainability, workspace standards compliance (§4.3)

### Integration
**Documentation Count**: 2  
**Complexity Level**: High  
**Maintenance Priority**: High

#### Active Documents
- **[MCP Integration Patterns](./integration/mcp-integration-patterns.md)** 
  - **Focus**: MCP ecosystem integration, tool registration, Claude Desktop compatibility
  - **Complexity**: High - Protocol integration and message handling
  - **Updated**: 2025-08-22
  - **Related**: ADR-001 (Foundation patterns)

- **[MCP Server Foundation Patterns](./integration/mcp-server-foundation-patterns.md)** 
  - **Focus**: AIRS MCP server implementation patterns, STDIO transport, ToolProvider architecture
  - **Complexity**: High - Core server architecture and Claude Desktop integration
  - **Updated**: 2025-08-25  
  - **Related**: ADR-002 (MCP server architecture), task_002

### Security
**Documentation Count**: 1  
**Complexity Level**: High  
**Maintenance Priority**: High

#### Active Documents
- **[Security Framework Architecture](./security/security-framework-architecture.md)**
  - **Focus**: Human-in-the-loop approval workflows, access control, audit logging
  - **Complexity**: High - Security validation and approval systems
  - **Updated**: 2025-08-22
  - **Related**: ADR-001 (Foundation patterns)

## Documentation by Complexity

### High Complexity (3 documents)
- MCP Integration Patterns
- MCP Server Foundation Patterns  
- Security Framework Architecture

### Medium Complexity (1 document)
- Dependency Injection Architecture

### Low Complexity (0 documents)
*No low complexity documents yet*

## Recent Updates

### 2025-08-25
- **Created**: Dependency Injection Architecture - Trait-based dependency injection patterns for FilesystemMcpServer
- **Content**: Constructor refactoring strategy, SOLID principles compliance, testability improvements
- **Impact**: High - Architectural improvement for server design and testing capabilities
- **Created**: MCP Server Foundation Patterns - Comprehensive technical implementation patterns for MCP servers
- **Content**: AIRS MCP foundation integration, STDIO transport patterns, ToolProvider implementation
- **Impact**: High - Primary reference for task_002 implementation and future MCP server development

### 2025-08-22  
- **Created**: MCP Integration Patterns - MCP ecosystem integration and tool registration
- **Created**: Security Framework Architecture - Security validation and approval workflows
- **Status**: Foundation knowledge documentation established

## Planned Knowledge Documentation

### Implementation Phase (Weeks 1-3)
1. **Binary Processing Patterns** (integration/) - Format detection, image/PDF processing patterns
2. **Error Handling Strategies** (patterns/) - FilesystemError to MCP error mapping patterns
3. **Performance Optimization Techniques** (performance/) - Streaming, caching, memory management

### Advanced Features Phase (Weeks 4-6)
4. **Advanced Security Patterns** (security/) - Threat detection, rate limiting, audit correlation
5. **Multi-Transport Architecture** (integration/) - HTTP and STDIO transport abstraction patterns
6. **Testing Strategies** (testing/) - Integration testing, Claude Desktop compatibility validation

## Maintenance Schedule

### Monthly Review (Next: 2025-09-25)
**Focus**: Content accuracy, code example validation, links verification

**Review Checklist**:
- [ ] Verify all code examples compile and run
- [ ] Update any API changes from airs-mcp foundation
- [ ] Validate integration patterns against latest implementations
- [ ] Check for broken cross-references

### Quarterly Review (Next: 2025-11-25)
**Focus**: Strategic value assessment and reorganization

**Review Checklist**:
- [ ] Assess documentation usefulness for development team
- [ ] Identify knowledge gaps in current documentation
- [ ] Plan new documentation based on implementation learnings
- [ ] Archive or update outdated patterns

## Cross-References

### Related ADRs
- **ADR-001**: Foundation Architecture Patterns → Security Framework Architecture
- **ADR-002**: MCP Server Architecture Decisions → MCP Server Foundation Patterns, MCP Integration Patterns

### Related Technical Debt
- **DEBT-002**: MCP Server Implementation Scope → MCP Server Foundation Patterns (remediation guidance)

### Related Tasks
- **task_002**: MCP Server Foundation → MCP Server Foundation Patterns (implementation guide)
- **task_003**: Core File Operations → Will reference integration and security patterns

---

**Note**: This index is automatically maintained. When creating new knowledge documentation, ensure the index is updated with proper categorization and cross-references.
