# Architecture Decision Record Index Template

## Purpose
This index maintains a chronological registry of all architectural decisions for this sub-project. It provides visibility into decision history, current status, and evolution of architectural thinking.

## Template Structure

```markdown
# Architecture Decision Record Registry

**Last Updated**: [YYYY-MM-DD]  
**Total ADRs**: [N]  
**Active ADRs**: [N]  
**Superseded ADRs**: [N]

## Decision Categories

### System Architecture
- **Active**: [N ADRs]
- **Superseded**: [N ADRs]

### Technology Selection  
- **Active**: [N ADRs]
- **Superseded**: [N ADRs]

### Design Patterns
- **Active**: [N ADRs]
- **Superseded**: [N ADRs]

### Performance Strategy
- **Active**: [N ADRs]
- **Superseded**: [N ADRs]

### Security & Compliance
- **Active**: [N ADRs]
- **Superseded**: [N ADRs]

## Active Decisions (Current Architecture)

### System Architecture
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-001 | [Decision title] | [YYYY-MM-DD] | [Accepted] | [High/Medium/Low] | [YYYY-MM-DD] |

### Technology Selection
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-002 | [Decision title] | [YYYY-MM-DD] | [Accepted] | [High/Medium/Low] | [YYYY-MM-DD] |

### Design Patterns
| ID | Title | Date | Status | Impact | Next Review |
|----|-------|------|--------|--------|-------------|
| ADR-003 | [Decision title] | [YYYY-MM-DD] | [Accepted] | [High/Medium/Low] | [YYYY-MM-DD] |

## Proposed Decisions (Under Review)

| ID | Title | Proposed Date | Deciders | Target Decision Date | Discussion |
|----|-------|---------------|----------|-------------------|------------|
| ADR-004 | [Decision title] | [YYYY-MM-DD] | [Names] | [YYYY-MM-DD] | [GitHub Issue #] |

## Recently Superseded (Last 6 Months)

| ID | Title | Superseded Date | Superseded By | Reason |
|----|-------|----------------|---------------|--------|
| ADR-005 | [Decision title] | [YYYY-MM-DD] | [ADR-009] | [Brief reason] |

## Deprecated Decisions

| ID | Title | Deprecated Date | Reason | Migration Status |
|----|-------|----------------|--------|------------------|
| ADR-006 | [Decision title] | [YYYY-MM-DD] | [Brief reason] | [Complete/In Progress/Planned] |

## Decision Timeline (Chronological)

### Recent Decisions (Last 3 Months)
- **[YYYY-MM-DD]**: ADR-003 - [Decision title] (Accepted)
- **[YYYY-MM-DD]**: ADR-002 - [Decision title] (Accepted)  
- **[YYYY-MM-DD]**: ADR-001 - [Decision title] (Accepted)

### Historical Decisions (Older than 3 Months)
- **[YYYY-MM-DD]**: ADR-000 - [Decision title] (Superseded by ADR-001)

## Decision Relationships

### Dependency Chains
- **ADR-001** → influences → **ADR-003** (design patterns follow architecture)
- **ADR-002** → enables → **ADR-004** (technology choice enables new patterns)

### Conflict Resolution
- **ADR-005** vs **ADR-007**: Resolved by **ADR-009** (consolidated approach)

## Impact Analysis

### High Impact Decisions (Affect Multiple Modules)
- **ADR-001**: [Brief description of system-wide impact]
- **ADR-002**: [Brief description of cross-cutting impact]

### Technology Decisions
- **Current Stack**: [Based on ADR-002, ADR-005, etc.]
- **Performance Strategy**: [Based on ADR-003, ADR-007, etc.]
- **Security Approach**: [Based on ADR-004, ADR-008, etc.]

## Review Schedule

### Upcoming Reviews
| ADR ID | Title | Review Date | Review Type | Owner |
|--------|-------|-------------|-------------|-------|
| ADR-001 | [Title] | [YYYY-MM-DD] | [Annual/Impact-driven] | [Name] |

### Overdue Reviews (Requires Attention)
| ADR ID | Title | Original Review Date | Days Overdue | Action Required |
|--------|-------|-------------------|--------------|-----------------|
| ADR-002 | [Title] | [YYYY-MM-DD] | [N days] | [Review/Update/Supersede] |

## Success Metrics

### Decision Quality
- **Implementation Success Rate**: [X%] (decisions that achieved intended outcomes)
- **Average Time to Implementation**: [X days] from acceptance to completion
- **Reversal Rate**: [X%] (decisions that were superseded within 6 months)

### Process Efficiency  
- **Average Discussion Time**: [X days] from proposal to acceptance
- **Stakeholder Engagement**: [X people] average per decision
- **Documentation Completeness**: [X%] of decisions with full implementation tracking

## Action Items
- [ ] Review overdue ADRs and update status
- [ ] Schedule reviews for decisions approaching review dates
- [ ] Analyze recent decision outcomes for process improvements
- [ ] Update superseded decisions with links to replacements
- [ ] Create GitHub Issues for complex decision implementations
```

## Maintenance Guidelines

### Weekly Updates
- Update status of proposed decisions under review
- Track implementation progress of recently accepted decisions
- Add new proposed decisions to the registry

### Monthly Reviews
- Review decisions scheduled for monthly evaluation
- Update impact assessments based on implementation experience
- Analyze decision outcomes and implementation success

### Quarterly Analysis
- Comprehensive review of all active decisions for continued relevance
- Update decision relationships and dependency analysis
- Assess decision quality metrics and process improvements
- Plan major architectural evolution based on decision history

### Annual Assessment
- Deep evaluation of all architectural decisions for strategic alignment
- Comprehensive review of deprecated and superseded decisions
- Long-term architectural roadmap planning based on decision trends
- Process refinement based on annual decision effectiveness analysis

This index should serve as the authoritative source for understanding the architectural evolution of the sub-project and should be referenced during all major technical planning and design discussions.
