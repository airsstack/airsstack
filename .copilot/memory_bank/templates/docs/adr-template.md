# Architecture Decision Record (ADR) Template

## File Name Convention
`ADR-{ID}-{brief-description}.md`
- Example: `ADR-001-transport-abstraction.md`
- Example: `ADR-002-correlation-id-strategy.md`
- Sequential numbering across all ADRs in the sub-project

## Template Structure

```markdown
# ADR-{ID}: {Decision Title}

**Status**: [Proposed/Accepted/Deprecated/Superseded]  
**Date**: [YYYY-MM-DD]  
**Deciders**: [List of people involved in the decision]  
**Technical Story**: [Link to GitHub Issue/Task/Epic that prompted this decision]

## Context and Problem Statement
**What is the issue that we're seeing that is motivating this decision or change?**
- Describe the forces at play (technical, political, social, and project local)
- This section describes the forces at play, including technological, political, social, and project local
- It is value-neutral -- the problem statement
- These forces are probably in tension, and should be called out as such

## Decision Drivers
**What factors influenced this decision?**
- Performance requirements
- Scalability needs  
- Maintainability concerns
- Team expertise and preferences
- Technology ecosystem constraints
- Timeline and resource constraints
- Compliance or regulatory requirements

## Considered Options
**What are the ways we can solve this problem?**
1. **Option 1**: [Brief description]
   - Pros: [List advantages]
   - Cons: [List disadvantages]
   - Implementation effort: [High/Medium/Low]

2. **Option 2**: [Brief description]
   - Pros: [List advantages]  
   - Cons: [List disadvantages]
   - Implementation effort: [High/Medium/Low]

3. **Option 3**: [Brief description]
   - Pros: [List advantages]
   - Cons: [List disadvantages] 
   - Implementation effort: [High/Medium/Low]

## Decision Outcome
**Chosen option**: [Option X], because [justification]

### Positive Consequences
- [Benefit 1]
- [Benefit 2]
- [Benefit 3]

### Negative Consequences
- [Downside 1]
- [Downside 2]
- [Risk that needs mitigation]

## Implementation Plan
**How will this decision be implemented?**
- Migration strategy (if applicable)
- Implementation phases
- Testing approach
- Rollback plan
- Success metrics

## Validation Approach
**How will we know this decision was correct?**
- Measurable success criteria
- Timeline for evaluation
- Key metrics to monitor
- Review schedule

## Compliance with Workspace Standards
**How does this align with workspace standards?**
- Reference relevant workspace standards (e.g., `workspace/shared_patterns.md`)
- Document any deviations and their justification
- Note any new patterns this decision establishes

## Links and References
**Supporting information and context**
- Related ADRs (supersedes, superseded by, relates to)
- External documentation
- Research papers or articles that influenced the decision
- Code examples or prototypes
- Benchmarks or performance analysis

## Notes
**Additional context that doesn't fit elsewhere**
- Alternative names considered
- Assumptions made during the decision process
- Future decision points that may revisit this choice
```

## Usage Guidelines

### When to Create an ADR
1. **Architectural Changes**: Any decision that affects the structure of the system
2. **Technology Selection**: Choosing between frameworks, libraries, or tools
3. **Design Patterns**: Establishing patterns that will be used across the codebase
4. **Performance Strategies**: Decisions about optimization approaches
5. **Security Approaches**: Decisions about authentication, authorization, or data protection
6. **Integration Strategies**: How to connect with external systems
7. **API Design**: Public interface design decisions

### Decision Criteria Threshold
Create an ADR when the decision:
- Has long-term impact (>6 months)
- Affects multiple modules or teams
- Involves significant trade-offs
- Sets precedent for future decisions
- Requires explanation to future maintainers
- Costs significant time/money to reverse

### ADR Lifecycle
1. **Proposed**: Initial draft, seeking feedback
2. **Accepted**: Decision finalized and implementation begins
3. **Deprecated**: Decision no longer recommended but not yet replaced
4. **Superseded**: Replaced by a newer ADR (link to replacement)

### Maintenance Process
- Review ADRs annually for continued relevance
- Update status when circumstances change
- Link related decisions to maintain decision history
- Archive superseded ADRs with clear references to replacements

### Integration with Other Documentation
- Reference relevant knowledge documentation for technical details
- Create technical debt records for implementation shortcuts
- Link to GitHub Issues for tracking implementation
- Cross-reference workspace standards compliance

### Quality Standards
- Decisions must be justified with concrete reasoning
- Consider at least 2-3 viable alternatives
- Include quantitative analysis where possible
- Be specific about success criteria and review timeline
- Maintain clear, executive-level language
