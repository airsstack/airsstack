# Knowledge Documentation Template

## Directory Structure
```
docs/knowledges/
├── architecture/          # System design and architectural patterns
├── patterns/             # Reusable code and design patterns
├── performance/          # Performance analysis and optimization
├── integration/          # External system integration knowledge
├── security/            # Security considerations and implementations
└── domain/              # Business domain knowledge and rules
```

## File Name Convention
`{category}/{descriptive-name}.md`
- Example: `architecture/transport-layer-design.md`
- Example: `patterns/async-error-handling.md`
- Example: `performance/streaming-benchmarks.md`

## Template Structure

```markdown
# {Knowledge Title}

**Category**: [Architecture/Patterns/Performance/Integration/Security/Domain]  
**Complexity**: [Low/Medium/High]  
**Last Updated**: [YYYY-MM-DD]  
**Maintainer**: [Role/Team responsible for keeping this current]

## Overview
**What is this knowledge about?**
- Brief description of the topic
- Why this knowledge is important for the project
- Who should read this documentation

## Context & Background
**When and why was this approach chosen?**
- Historical context of the decision
- Problems this approach solves
- Alternative approaches that were considered
- Reference to related ADRs if applicable

## Technical Details
**How does this work?**
- Detailed technical explanation
- Key algorithms, data structures, or patterns used
- Important implementation details
- Configuration parameters and their effects

## Code Examples
**Practical implementation examples**
```rust
// Provide concrete, working code examples
// Include common usage patterns
// Show both simple and complex scenarios
```

## Performance Characteristics
**How does this perform?**
- Time complexity analysis
- Memory usage patterns
- Throughput and latency characteristics
- Benchmarking results (reference to `performance/` docs if detailed)

## Trade-offs & Limitations
**What are the constraints and compromises?**
- Known limitations of this approach
- Performance trade-offs
- Scalability considerations
- Maintenance overhead

## Dependencies
**What does this rely on?**
- External crates and libraries
- Internal modules and components
- Configuration requirements
- Runtime dependencies

## Testing Strategy
**How is this tested?**
- Unit testing approach
- Integration testing considerations
- Performance testing methodology
- Edge cases and error conditions

## Common Pitfalls
**What should developers watch out for?**
- Common mistakes when implementing
- Debugging tips and techniques
- Performance gotchas
- Security considerations

## Related Knowledge
**What else should I read?**
- Related architecture documents
- Relevant patterns and practices
- Performance analysis documents
- Related ADRs and technical decisions

## Evolution History
**How has this changed over time?**
- Major revisions and their reasons
- Deprecated approaches
- Future evolution plans
- Migration guides for major changes

## Examples in Codebase
**Where can I see this in action?**
- File paths to reference implementations
- Test files demonstrating usage
- Example applications or benchmarks
- Documentation or tutorial code
```

## Usage Guidelines

### When to Create Knowledge Documentation
1. **Complex Architectural Decisions**: When implementing significant system components
2. **Reusable Patterns**: When developing patterns that will be used across modules
3. **Performance Critical Code**: When implementing optimizations or addressing bottlenecks
4. **Integration Points**: When connecting to external systems or protocols
5. **Domain Knowledge**: When implementing business rules or domain-specific logic

### Documentation Triggers
- Completing major architectural work
- Discovering non-obvious implementation patterns
- Solving complex performance or scalability challenges
- Integrating with external systems or protocols
- Implementing security-critical functionality

### Maintenance Process
- Review knowledge docs quarterly for accuracy
- Update when related code undergoes major changes
- Archive obsolete knowledge with clear deprecation notes
- Cross-reference with ADRs and technical debt records

### Quality Standards
- All code examples must compile and run
- Performance claims must be backed by benchmarks
- Include both simple and complex usage examples
- Maintain clear, jargon-free explanations
- Reference external resources for deeper learning
