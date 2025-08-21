# Technical Debt Record Template

## File Name Convention
`DEBT-{ID}-{brief-description}.md`
- Example: `DEBT-001-correlation-error-handling.md`
- Example: `DEBT-002-oauth2-token-refresh.md`

## Template Structure

```markdown
# DEBT-{ID}: {Brief Description}

**Status**: [Active/In Progress/Resolved/Abandoned]  
**Priority**: [Critical/High/Medium/Low]  
**Category**: [Architecture/Code Quality/Performance/Security/Documentation/Testing]  
**Created**: [YYYY-MM-DD]  
**Updated**: [YYYY-MM-DD]  
**Estimated Effort**: [Hours/Days/Weeks]

## Problem Description
**What is the technical debt?**
- Clear description of what was compromised or deferred
- Impact on current development velocity
- Impact on code maintainability

## Context & Reason
**Why was this debt incurred?**
- Business pressure/deadline constraints
- Lack of information at implementation time
- Technology limitations
- Resource constraints

## Current Impact
**How does this debt affect the project today?**
- Development velocity impact
- Code complexity increase
- Testing difficulty
- Performance implications
- Security concerns

## Future Risk
**What happens if this debt is not addressed?**
- Projected impact on future development
- Risk of becoming impossible to fix
- Potential for cascading problems

## Remediation Plan
**How should this debt be resolved?**
- Specific steps required for resolution
- Dependencies and prerequisites
- Breaking changes or migration requirements
- Testing strategy for the fix

## Code References
**Where is this debt located?**
- File paths and line numbers
- Related modules or components
- Test files that need updates

## Related Issues
**Links to related tracking**
- GitHub Issues
- Related technical debt items
- Task management references

## Notes
**Additional context**
- Previous remediation attempts
- Alternative approaches considered
- Stakeholder discussions
```

## Usage Guidelines

### When to Create Technical Debt Records
1. **During Code Review**: When compromises are identified but accepted for delivery
2. **During Refactoring**: When existing debt is discovered
3. **During Architecture Changes**: When temporary solutions are implemented
4. **During Bug Fixes**: When root cause indicates systematic issues

### Documentation Triggers
- Any `TODO(DEBT)` comment in code MUST have corresponding debt record
- Any architectural shortcuts taken under time pressure
- Any violation of workspace standards that cannot be immediately resolved
- Any performance or security compromises

### Maintenance Process
- Review all debt records monthly
- Update status and priority based on current project needs
- Archive resolved debt with summary of resolution approach
- Escalate critical debt that remains unaddressed for >3 months

### Integration with GitHub Issues
- Create GitHub Issues for debt requiring cross-team coordination
- Reference GitHub Issue number in debt record
- Use debt records as detailed documentation for GitHub Issues
- Close GitHub Issues when debt record status becomes "Resolved"
