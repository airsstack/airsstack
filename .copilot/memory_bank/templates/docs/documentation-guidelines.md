# Technical Documentation Guidelines

## Overview
This document establishes clear criteria and processes for maintaining technical knowledge in the AIRS workspace. Each sub-project maintains three types of technical documentation:

- **Technical Debt Records** (`docs/debts/`): Track compromises and shortcuts that need future attention
- **Knowledge Documentation** (`docs/knowledges/`): Capture architectural patterns, implementation details, and domain expertise  
- **Architecture Decision Records** (`docs/adr/`): Document significant technical decisions and their rationale

## Documentation Triggers

### Technical Debt Records
**REQUIRED when:**
- Adding any `TODO(DEBT)` comment to code
- Violating workspace standards that cannot be immediately resolved
- Taking architectural shortcuts due to time/resource constraints
- Implementing temporary solutions or workarounds
- Deferring refactoring during feature development

**OPTIONAL when:**
- Identifying potential improvements during code review
- Discovering legacy code that needs modernization
- Planning future optimization opportunities

### Knowledge Documentation  
**REQUIRED when:**
- Implementing complex algorithms or data structures
- Creating reusable patterns that will be used across modules
- Integrating with external systems or protocols
- Implementing performance-critical optimizations
- Establishing security-critical implementations

**OPTIONAL when:**
- Documenting useful debugging techniques
- Capturing domain-specific business rules
- Recording performance benchmarking results
- Explaining non-obvious implementation choices

### Architecture Decision Records
**REQUIRED when:**
- Making technology selection decisions (frameworks, libraries, tools)
- Establishing architectural patterns or design principles
- Choosing between significantly different implementation approaches
- Making decisions that affect system scalability or performance
- Establishing security or compliance approaches

**OPTIONAL when:**
- Documenting smaller design pattern choices
- Recording lessons learned from implementation experiments
- Capturing rationale for API design decisions

## Documentation Workflow Integration

### During Development
1. **Task Planning**: Identify documentation needs during task breakdown
2. **Implementation**: Create debt records for any shortcuts taken
3. **Code Review**: Verify documentation completeness before approval
4. **Task Completion**: Update relevant documentation with outcomes

### During Architecture Work
1. **Decision Process**: Draft ADR during architectural discussion
2. **Implementation**: Create knowledge docs for complex implementations
3. **Review**: Finalize ADR status based on implementation results
4. **Knowledge Transfer**: Ensure patterns are documented for reuse

### During Maintenance
1. **Quarterly Review**: Update all documentation for accuracy
2. **Annual Review**: Archive obsolete documentation
3. **Cross-Reference Check**: Ensure links between documents remain valid
4. **Migration Planning**: Update documentation when deprecating approaches

## Quality Standards

### Content Requirements
- **Clarity**: Written for future maintainers who weren't involved in original decisions
- **Completeness**: Include all information needed to understand or reproduce the work
- **Accuracy**: All code examples must compile and run correctly
- **Currency**: Regular updates when underlying systems change

### Technical Standards
- Follow workspace coding standards in all code examples
- Reference workspace standards documents rather than duplicating content
- Include benchmarks and performance data where relevant
- Provide working code examples that demonstrate key concepts

### Cross-Reference Requirements
- Link related documentation (ADRs reference knowledge docs, etc.)
- Reference GitHub Issues for tracking implementation
- Connect to task management for completion tracking
- Link to workspace standards for compliance verification

## Directory Structure Standards

### Per Sub-Project Structure
```
{sub-project}/
├── docs/
│   ├── debts/
│   │   ├── _index.md              # Debt registry with priorities
│   │   ├── DEBT-001-*.md          # Individual debt records
│   │   └── DEBT-002-*.md
│   ├── knowledges/
│   │   ├── architecture/          # System design patterns
│   │   ├── patterns/             # Code patterns and practices
│   │   ├── performance/          # Performance analysis
│   │   ├── integration/          # External system integration
│   │   ├── security/            # Security implementations
│   │   └── domain/              # Business domain knowledge
│   └── adr/
│       ├── _index.md              # ADR registry with status
│       ├── ADR-001-*.md           # Individual decision records
│       └── ADR-002-*.md
```

### Index File Requirements
Each documentation type must maintain an index file:

**`_index.md` for debts/**
- List all debt records with current status and priority
- Group by category (Architecture, Performance, Security, etc.)
- Track resolution progress and responsible parties

**`_index.md` for adr/**
- List all ADRs with current status
- Maintain chronological decision history
- Track superseded decisions and their replacements

## Maintenance Responsibilities

### Development Team
- Create documentation during development according to triggers
- Update documentation when making related code changes
- Review documentation accuracy during code reviews
- Maintain quality standards for all technical writing

### Tech Lead/Architect
- Review ADRs for technical soundness and completeness
- Ensure architectural decisions align with workspace standards
- Coordinate cross-project documentation consistency
- Approve deprecation and archival of obsolete documentation

### Project Maintenance
- Quarterly documentation review for accuracy and relevance
- Annual archival of obsolete documentation
- Cross-reference validation and repair
- Template updates based on evolving needs

## Success Metrics

### Coverage Metrics
- All major architectural decisions documented in ADRs
- All technical shortcuts tracked in debt records
- All complex implementations explained in knowledge docs
- Zero `TODO(DEBT)` comments without corresponding debt records

### Quality Metrics
- All code examples compile and run correctly
- Documentation updated within 30 days of related code changes
- Cross-references remain valid and current
- New team members can understand system from documentation alone

### Process Metrics
- Documentation created within same sprint as related code
- Debt records resolved within planned timeframes
- ADR review and approval cycle completed within 1 week
- Knowledge docs referenced during onboarding and troubleshooting

This guidelines document ensures consistent, high-quality technical documentation that supports long-term maintainability and knowledge transfer across the AIRS workspace.
