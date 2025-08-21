# Technical Debt Index Template

## Purpose
This index maintains a registry of all technical debt records for this sub-project. It provides quick visibility into debt status, priorities, and resolution progress.

## Template Structure

```markdown
# Technical Debt Registry

**Last Updated**: [YYYY-MM-DD]  
**Total Debt Items**: [N]  
**High Priority Items**: [N]  
**In Progress**: [N]

## Summary by Category

### Architecture Debt
- **Critical**: [N items]
- **High**: [N items]  
- **Medium**: [N items]
- **Low**: [N items]

### Performance Debt
- **Critical**: [N items]
- **High**: [N items]
- **Medium**: [N items]
- **Low**: [N items]

### Security Debt
- **Critical**: [N items]
- **High**: [N items]
- **Medium**: [N items]
- **Low**: [N items]

### Code Quality Debt
- **Critical**: [N items]
- **High**: [N items]
- **Medium**: [N items] 
- **Low**: [N items]

## Active Debt (Requires Attention)

### Critical Priority
| ID | Description | Category | Created | Estimated Effort | Owner | GitHub Issue |
|----|-------------|----------|---------|------------------|-------|--------------|
| DEBT-001 | [Brief description] | [Category] | [YYYY-MM-DD] | [Hours/Days] | [Team/Person] | [#123] |

### High Priority  
| ID | Description | Category | Created | Estimated Effort | Owner | GitHub Issue |
|----|-------------|----------|---------|------------------|-------|--------------|
| DEBT-002 | [Brief description] | [Category] | [YYYY-MM-DD] | [Hours/Days] | [Team/Person] | [#124] |

### Medium Priority
| ID | Description | Category | Created | Estimated Effort | Owner | GitHub Issue |  
|----|-------------|----------|---------|------------------|-------|--------------|
| DEBT-003 | [Brief description] | [Category] | [YYYY-MM-DD] | [Hours/Days] | [Team/Person] | [#125] |

## In Progress

| ID | Description | Category | Started | Progress | Expected Completion | Owner |
|----|-------------|----------|---------|----------|-------------------|-------|
| DEBT-004 | [Brief description] | [Category] | [YYYY-MM-DD] | [%] | [YYYY-MM-DD] | [Team/Person] |

## Recently Resolved (Last 30 Days)

| ID | Description | Category | Resolved | Resolution Summary | 
|----|-------------|----------|----------|-------------------|
| DEBT-005 | [Brief description] | [Category] | [YYYY-MM-DD] | [How it was resolved] |

## Low Priority / Backlog

| ID | Description | Category | Created | Estimated Effort | Notes |
|----|-------------|----------|---------|------------------|-------|
| DEBT-006 | [Brief description] | [Category] | [YYYY-MM-DD] | [Hours/Days] | [Additional context] |

## Abandoned/Won't Fix

| ID | Description | Category | Abandoned | Reason |
|----|-------------|----------|-----------|--------|
| DEBT-007 | [Brief description] | [Category] | [YYYY-MM-DD] | [Why abandoned] |

## Debt Trends

### Monthly Debt Creation/Resolution
- **This Month**: [N created] / [N resolved]
- **Last Month**: [N created] / [N resolved]  
- **Trend**: [Increasing/Decreasing/Stable]

### Average Resolution Time
- **Critical**: [X days average]
- **High**: [X days average]
- **Medium**: [X days average]

## Action Items
- [ ] Review stale debt items (>90 days with no activity)
- [ ] Escalate critical items blocking development
- [ ] Plan remediation for high-priority architectural debt
- [ ] Update GitHub Issues for debt requiring cross-team coordination
```

## Maintenance Guidelines

### Weekly Updates
- Update progress on "In Progress" items
- Move completed items to "Recently Resolved"  
- Review and re-prioritize based on current project needs

### Monthly Reviews
- Calculate and update debt trends
- Review "Low Priority" items for potential priority changes
- Identify stale items that need attention or abandonment
- Plan remediation efforts for upcoming sprint cycles

### Quarterly Analysis
- Analyze debt patterns and root causes
- Identify process improvements to reduce debt creation
- Review abandoned items for potential re-evaluation
- Update estimation accuracy based on completed work

This index should be maintained as the single source of truth for technical debt status and should be referenced during sprint planning and architectural decision-making processes.
