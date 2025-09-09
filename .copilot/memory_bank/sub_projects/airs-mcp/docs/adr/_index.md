# Architecture Decision Record Registry - airs-mcp

**Last Updated**: 2025-09-09  
**Total ADRs**: 11  
**Active ADRs**: 10
**Proposed ADRs**: 1
**Superseded ADRs**: 0

## Decision Categories

### System Architecture
- **Active**: 8 ADRs
- **Proposed**: 1 ADRs
- **Superseded**: 0 ADRs

### Technology Selection  
- **Active**: 0 ADRs
- **Superseded**: 0 ADRs

### Design Patterns
- **Active**: 1 ADRs
- **Superseded**: 0 ADRs

### Performance Strategy
- **Active**: 1 ADRs
- **Superseded**: 0 ADRs

### Security & Compliance
- **Active**: 0 ADRs
- **Superseded**: 0 ADRs

## Active Decisions (Current Architecture)

### System Architecture
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-001 | MCP-Compliant Transport Redesign | 2025-09-01 | Proposed | Critical | TBD |
| ADR-002 | Transport Role-Specific Architecture | 2025-08-14 | Accepted | High | 2026-02-14 |
| ADR-003 | HTTP Transport Architecture Strategy | 2025-08-14 | Accepted | High | 2026-02-14 |
| ADR-004 | Axum Modular Architecture Refactor | 2025-08-14 | Accepted | High | 2026-02-14 |
| ADR-006 | MCP Protocol Field Naming Compliance | 2025-08-14 | Accepted | Medium | 2025-11-14 |
| ADR-008 | MCP Protocol Architecture | 2025-08-14 | Accepted | High | 2026-02-14 |
| ADR-009 | Zero-Cost Generic Authorization Architecture | 2025-09-06 | Accepted | Critical | 2026-03-06 |

### Design Patterns
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-005 | Single Responsibility Principle Standard | 2025-08-14 | Accepted | Medium | 2025-11-14 |

### Performance Strategy
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-007 | Benchmarking Environment Constraints | 2025-08-14 | Accepted | Medium | 2025-11-14 |

## Proposed Decisions (Under Review)

| ID | Title | Proposed Date | Deciders | Target Decision Date | Discussion |
|----|-------|---------------|----------|-------------------|------------|
| ADR-001 | MCP-Compliant Transport Redesign | 2025-09-01 | Core Team | 2025-09-02 | Critical architecture refactoring |
| ADR-011 | Transport Configuration Separation Architecture | 2025-09-09 | @hiraq + GitHub Copilot | 2025-09-15 | Clean separation of MCP core vs transport-specific configuration |

## Recently Superseded (Last 6 Months)

| ID | Title | Superseded Date | Superseded By | Reason |
|----|-------|----------------|---------------|--------|
| - | No superseded decisions | - | - | - |

## Deprecated Decisions

| ID | Title | Deprecated Date | Reason | Migration Status |
|----|-------|----------------|--------|------------------|
| - | No deprecated decisions | - | - | - |

## Decision Timeline (Chronological)

### Recent Decisions (Last 3 Months)
- **2025-09-06**: ADR-009 - Zero-Cost Generic Authorization Architecture (Accepted)
- **2025-09-01**: ADR-001 - MCP-Compliant Transport Redesign (Proposed)
- **2025-08-14**: ADR-008 - MCP Protocol Architecture (Accepted)
- **2025-08-14**: ADR-007 - Benchmarking Environment Constraints (Accepted)
- **2025-08-14**: ADR-006 - MCP Protocol Field Naming Compliance (Accepted)
- **2025-08-14**: ADR-005 - Single Responsibility Principle Standard (Accepted)
- **2025-08-14**: ADR-004 - Axum Modular Architecture Refactor (Accepted)
- **2025-08-14**: ADR-003 - HTTP Transport Architecture Strategy (Accepted)  
- **2025-08-14**: ADR-002 - Transport Role-Specific Architecture (Accepted)

### Historical Decisions (Older than 3 Months)
- No historical decisions (project started August 2025)

## Decision Relationships

### Dependency Chains
- **ADR-001** → enables → **ADR-002** (role-specific transports enable HTTP architecture)
- **ADR-004** → supports → **ADR-001, ADR-003** (SRP enables clean architectural separation)
- **ADR-005** → supports → **ADR-007** (field naming supports protocol architecture)
- **ADR-006** → supports → **performance optimization** (benchmarking enables optimization)
- **ADR-009** → resolves → **TASK-027, DEBT-ARCH-003** (fixes OAuth2 authorization architecture)

### Conflict Resolution
- No conflicts identified between current decisions

## Impact Analysis

### Critical Impact Decisions (Production Blocking)
- **ADR-009**: Zero-Cost Generic Authorization Architecture - Fixes OAuth2 authentication layer violation, enables production OAuth2 deployment

### High Impact Decisions (Affect Multiple Modules)
- **ADR-001**: Transport Role-Specific Architecture - Affects all transport usage, establishes pattern for future transports
- **ADR-002**: HTTP Transport Architecture Strategy - Affects HTTP transport implementation, client/server separation
- **ADR-003**: Axum Modular Architecture - Affects server framework structure and organization
- **ADR-007**: MCP Protocol Architecture - Affects protocol compliance and implementation strategy

### Technology Decisions
- **Current Stack**: HTTP-based transport with Axum server framework, role-specific transport separation
- **Performance Strategy**: Zero-cost generic abstractions with compile-time optimization (ADR-009)
- **Security Approach**: Generic authentication/authorization architecture with OAuth2 support (ADR-009)

## Review Schedule

### Upcoming Reviews
| ADR ID | Title | Review Date | Review Type | Owner |
|--------|-------|-------------|-------------|-------|
| ADR-004 | Single Responsibility Principle | 2025-11-14 | Quarterly | Core Team |
| ADR-005 | MCP Protocol Field Naming | 2025-11-14 | Quarterly | Core Team |
| ADR-006 | Benchmarking Environment | 2025-11-14 | Quarterly | Core Team |
| ADR-009 | Zero-Cost Authorization Architecture | 2026-03-06 | Annual | Core Team |

### Overdue Reviews (Requires Attention)
| ADR ID | Title | Original Review Date | Days Overdue | Action Required |
|--------|-------|-------------------|--------------|-----------------|
| - | No overdue reviews | - | - | - |

## Success Metrics

### Decision Quality
- **Implementation Success Rate**: 100% (8/8 decisions successfully implemented or in progress)
- **Average Time to Implementation**: Same-day implementation for most decisions
- **Reversal Rate**: 0% (no decisions reversed within 6 months)

### Process Efficiency  
- **Average Discussion Time**: Same-day (decisions made and implemented rapidly during intensive development)
- **Stakeholder Engagement**: Core team consensus on all decisions
- **Documentation Completeness**: 100% of decisions have complete implementation tracking

## Action Items
- [ ] Schedule quarterly reviews for all decisions approaching review dates
- [x] Create ADR for OAuth2 security architecture decisions (ADR-009 completed)
- [ ] Document performance optimization strategy decisions
- [ ] Plan ADR for Phase 3 server implementation architecture
- [ ] Implement ADR-009 authorization architecture (TASK-027)
