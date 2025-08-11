# Contributing to AIRS

*A comprehensive guide to contributing to the AI-Rust Integration System ecosystem*

---

## Welcome Contributors!

AIRS thrives on community contributions, whether you're fixing a bug, adding a feature, improving documentation, or sharing insights from your AI-Rust integration journey. This guide will help you contribute effectively and align with our development philosophy.

## Before You Begin

### Understanding AIRS Philosophy

AIRS is built on these core principles:

**🎯 Pragmatic Excellence**: Balance engineering craft with delivery needs  
**🤖 AI-Human Synergy**: Systematic collaboration between AI and human intelligence  
**📚 Knowledge Persistence**: Capture and share learning through memory bank architecture  
**🔧 Production Quality**: Real-world applicability over academic perfection  
**🌱 Incremental Growth**: Build understanding and capability systematically  

### Development Methodology

We use a **6-phase specification-driven workflow**:

1. **ANALYZE**: Understand requirements and assess confidence
2. **DESIGN**: Create comprehensive technical design and implementation plan
3. **IMPLEMENT**: Write production-quality code following the design
4. **VALIDATE**: Verify implementation meets requirements and quality standards
5. **REFLECT**: Improve codebase and update documentation
6. **HANDOFF**: Package work for review and transition to next task

**Memory Bank Integration**: All work is documented and captured in our memory bank structure for future reference and learning.

## Types of Contributions

### 🐛 Bug Reports and Fixes

**Reporting Bugs**:
- Use clear, descriptive titles
- Include steps to reproduce
- Provide environment details (Rust version, OS, etc.)
- Include relevant logs or error messages
- Reference related documentation if applicable

**Bug Fix Process**:
1. **ANALYZE**: Understand the bug and its impact
2. **DESIGN**: Plan the fix approach and test strategy
3. **IMPLEMENT**: Fix the issue with comprehensive tests
4. **VALIDATE**: Verify fix resolves issue without side effects
5. **REFLECT**: Update documentation and identify prevention measures
6. **HANDOFF**: Submit PR with detailed description and validation evidence

### ✨ New Features

**Feature Proposal Process**:
1. **Create Issue**: Describe the feature, use cases, and expected benefits
2. **Discussion**: Engage with maintainers and community for feedback
3. **Design**: Create detailed technical design following our patterns
4. **Implementation**: Follow the 6-phase workflow for development
5. **Review**: Submit PR with comprehensive documentation and tests

**Feature Development Requirements**:
- **Memory Bank Documentation**: Update relevant memory bank files
- **Comprehensive Tests**: Unit, integration, and performance tests as appropriate
- **Documentation**: Update all relevant documentation including examples
- **Backwards Compatibility**: Maintain API stability unless major version change

### 📚 Documentation Improvements

**Documentation Contributions**:
- **Technical Accuracy**: Ensure all examples are tested and current
- **Clarity**: Write for developers at various experience levels
- **Completeness**: Include real-world examples and common pitfalls
- **Memory Bank Updates**: Keep memory bank documentation current with changes

**Documentation Types**:
- **API Documentation**: Code comments and generated docs
- **User Guides**: Practical implementation guidance
- **Examples**: Working code examples with explanations
- **Memory Bank**: Project knowledge and learning capture

### 🎯 Performance Improvements

**Performance Contribution Guidelines**:
- **Benchmarking**: Include before/after performance measurements
- **Analysis**: Document the performance issue and solution approach
- **Testing**: Verify improvements don't introduce regressions
- **Documentation**: Update performance guidance and best practices

## Development Setup

### Environment Preparation

**Required Tools**:
```bash
# Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Additional tools
cargo install mdbook          # Documentation
cargo install cargo-watch    # Development workflow
cargo install cargo-nextest  # Advanced testing
```

**Repository Setup**:
```bash
# Fork and clone the repository
git clone https://github.com/your-username/airs
cd airs

# Set up development environment
cargo build --workspace
cargo test --workspace
```

### Project Structure Understanding

**Workspace Layout**:
```
airs/
├── .copilot/memory_bank/           # Project knowledge and context
├── crates/
│   ├── airs-mcp/                   # Model Context Protocol implementation
│   └── airs-memspec/               # Memory bank toolkit
├── docs/                           # Root documentation
└── examples/                       # Cross-project examples
```

**Memory Bank Structure**:
```
.copilot/memory_bank/
├── current_context.md              # Active project context
├── workspace/                      # Workspace-level documentation
│   ├── project_brief.md
│   ├── shared_patterns.md
│   └── workspace_architecture.md
└── sub_projects/                   # Individual project documentation
    └── airs/
        ├── active_context.md
        ├── progress.md
        └── tasks/
```

### Memory Bank Integration

**Reading Project Context**:
Always start by reading the memory bank to understand current context:

1. **Workspace Context**: Read `workspace/` files for overall project understanding
2. **Active Context**: Check `current_context.md` for current focus areas
3. **Project Context**: Review relevant sub-project documentation
4. **Tasks**: Check `tasks/` folder for current work items

**Updating Memory Bank**:
When making significant contributions:

1. **Document Decisions**: Add decision records for significant choices
2. **Update Progress**: Reflect your contributions in progress tracking
3. **Capture Patterns**: Document new patterns or improvements discovered
4. **Update Context**: Keep active context current with your work

## Development Workflow

### Using the 6-Phase Process

**Phase 1: ANALYZE**
```
🎯 Understand the contribution scope
📋 Read existing code, docs, and tests
📝 Define requirements in EARS notation
🔍 Identify dependencies and constraints
📊 Assess confidence level (0-100%)
```

**Phase 2: DESIGN**
```
🏗️ Create technical design
📋 Plan implementation approach
🧪 Define testing strategy  
📚 Plan documentation updates
✅ Create task breakdown
```

**Phase 3: IMPLEMENT**
```
⚡ Code in small, testable increments
🧪 Write tests alongside implementation
📝 Add meaningful code comments
🔄 Update task status regularly
✅ Follow project coding standards
```

**Phase 4: VALIDATE**
```
🧪 Run all tests (unit, integration, performance)
🔍 Verify edge cases and error handling
📊 Check performance impact
🔗 Validate documentation accuracy
✅ Ensure no regressions
```

**Phase 5: REFLECT**
```
🔧 Refactor for maintainability
📚 Update all relevant documentation
💡 Identify potential improvements
🎯 Document lessons learned
🏗️ Address any technical debt
```

**Phase 6: HANDOFF**
```
📋 Create comprehensive PR description
🔗 Link to validation artifacts
📚 Update memory bank documentation
🎯 Prepare for review process
✅ Ensure complete handoff package
```

### Coding Standards

**Rust Code Quality**:
```rust
// Use meaningful names and clear intent
fn handle_memory_bank_validation(
    request: ValidationRequest,
) -> Result<ValidationResponse, ValidationError> {
    // Document complex logic and decisions
    // Handle errors explicitly and meaningfully
    // Write code that tells a story
}
```

**Documentation Standards**:
- **Code Comments**: Focus on "why" not "what"
- **API Docs**: Include examples and common use cases
- **Guides**: Provide practical, tested examples
- **Memory Bank**: Capture decision context and learning

**Testing Requirements**:
- **Unit Tests**: Test individual components thoroughly
- **Integration Tests**: Verify component interactions
- **Performance Tests**: Benchmark critical paths
- **Documentation Tests**: Ensure examples compile and run

## Contribution Workflow

### Making Your Contribution

**1. Preparation Phase**
```bash
# Create feature branch
git checkout -b feature/your-contribution-name

# Read memory bank for context
# Start with ANALYZE phase
```

**2. Development Phase**
```bash
# Implement using 6-phase workflow
cargo watch -x "check --workspace"  # Continuous validation
cargo test --workspace              # Regular testing
```

**3. Documentation Phase**
```bash
# Update documentation
mdbook build docs                    # Validate documentation
mdbook serve docs --open            # Review locally
```

**4. Validation Phase**
```bash
# Comprehensive validation
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
cargo fmt --all

# Performance testing (if applicable)
cargo bench
```

**5. Memory Bank Update**
```bash
# Update memory bank with your contribution
# Document decisions, patterns, and learning
# Update progress and context as needed
```

### Pull Request Process

**PR Preparation Checklist**:
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code follows style guidelines (`cargo clippy`, `cargo fmt`)
- [ ] Documentation is updated and builds (`mdbook build`)
- [ ] Memory bank is updated with relevant context
- [ ] Performance impact is assessed (if applicable)
- [ ] Backwards compatibility is maintained (or explicitly noted)

**PR Description Template**:
```markdown
## Contribution Summary
Brief description of what this PR accomplishes

## Changes Made
- Detailed list of changes
- Link to design documents if applicable
- Reference to memory bank updates

## Validation
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Documentation builds successfully
- [ ] Performance impact assessed
- [ ] Memory bank updated

## Related Issues
Fixes #issue-number

## Review Notes
Any specific areas for reviewer attention
```

### Review Process

**What to Expect**:
1. **Automated Checks**: CI will run tests, lints, and validation
2. **Technical Review**: Maintainers will review code quality and design
3. **Documentation Review**: Ensure documentation is clear and complete
4. **Memory Bank Review**: Verify memory bank updates are appropriate
5. **Integration Testing**: Verify changes work well with existing codebase

**Addressing Feedback**:
- Respond promptly to review comments
- Make requested changes in additional commits (don't squash during review)
- Update tests and documentation as needed
- Continue updating memory bank with review insights

## Advanced Contribution Patterns

### Cross-Project Integration

**When Contributing Across Multiple Crates**:
1. **Understand Dependencies**: Map relationships between crates
2. **Coordinate Changes**: Plan changes across dependent crates
3. **Test Integration**: Verify cross-crate functionality
4. **Update Documentation**: Keep all affected documentation current

### Performance-Critical Contributions

**For Performance-Sensitive Changes**:
1. **Baseline Measurement**: Establish current performance benchmarks
2. **Implementation with Monitoring**: Track performance impact during development
3. **Comprehensive Benchmarking**: Test various scenarios and edge cases
4. **Documentation**: Update performance guidance and best practices

### Memory Bank Methodology Improvements

**Contributing to Memory Bank Patterns**:
1. **Document Current Patterns**: Understand existing memory bank methodology
2. **Identify Improvements**: Find gaps or enhancement opportunities
3. **Prototype Changes**: Test improvements in real scenarios
4. **Update Guidance**: Improve memory bank documentation and examples

## Community Guidelines

### Communication

**Be Respectful and Constructive**:
- Focus on the code and ideas, not individuals
- Provide specific, actionable feedback
- Assume positive intent in all interactions
- Help others learn and improve

**Technical Discussions**:
- Support arguments with data and examples
- Reference existing patterns and documentation
- Consider long-term maintenance implications
- Think about impact on different user types

### Knowledge Sharing

**Share Your Learning**:
- Document insights in memory bank structure
- Contribute examples from your experience
- Help improve documentation clarity
- Mentor other contributors

**Collaborate Effectively**:
- Communicate your plans early for large changes
- Ask questions when uncertain
- Share progress and blockers promptly
- Coordinate with maintainers on architectural decisions

## Recognition and Growth

### Contribution Recognition

**How We Recognize Contributors**:
- **Code Contributions**: Credit in commit messages and changelogs
- **Documentation**: Recognition in documentation improvements
- **Memory Bank**: Insights captured in project knowledge base
- **Community**: Acknowledgment of helpful community participation

### Growth Opportunities

**Becoming a Core Contributor**:
1. **Consistent Quality**: Demonstrate reliable, high-quality contributions
2. **Community Engagement**: Help other contributors and users
3. **Technical Leadership**: Take ownership of features or areas
4. **Memory Bank Stewardship**: Help maintain and improve project knowledge

**Maintainer Path**:
- Sustained high-quality contributions
- Deep understanding of project architecture
- Strong communication and mentoring skills
- Commitment to project values and methodology

## Getting Help

### Resources

**Documentation**:
- **[Getting Started Guide](./getting_started.md)** - Introduction to AIRS ecosystem
- **[Technical Documentation](../technical/)** - Deep implementation guidance
- **Memory Bank** - Project knowledge and patterns

**Community Support**:
- **GitHub Issues**: For bugs, features, and technical questions
- **Pull Request Reviews**: Direct feedback on contributions
- **Documentation**: Comprehensive guides and examples

### Common Questions

**Q: How do I understand the current project context?**
A: Always start by reading the memory bank structure, particularly `current_context.md`, workspace files, and relevant sub-project documentation.

**Q: What if I'm not sure about the best approach?**
A: Create an issue to discuss your approach before implementation. Include your confidence assessment and proposed design.

**Q: How do I handle memory bank updates?**
A: Document your decisions, learning, and patterns in the relevant memory bank files. Focus on capturing context that will help future contributors.

**Q: What's the difference between AIRS-MCP and AIRS-MemSpec?**
A: AIRS-MCP handles Model Context Protocol integration for AI systems. AIRS-MemSpec provides memory bank validation and management tools.

---

## Ready to Contribute?

1. **Start Small**: Begin with documentation improvements or simple bug fixes
2. **Follow the Workflow**: Use our 6-phase development process
3. **Engage the Community**: Ask questions and share your progress
4. **Update Memory Bank**: Capture your learning and decisions
5. **Build Iteratively**: Grow your contributions over time

**Thank you for contributing to AIRS!** Your participation helps build better AI-Rust integration tools and methodologies for the entire community.

For technical questions, start with our **[Getting Started Guide](./getting_started.md)**.
