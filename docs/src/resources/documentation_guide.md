# AIRS Documentation Guide

*Navigate the AIRS documentation ecosystem effectively*

---

## Documentation Architecture

The AIRS ecosystem uses a **layered documentation approach** designed to serve different user needs while maintaining independent development workflows for each component.

### Root Documentation (This Site)
**Purpose**: Strategic overview, philosophy, and cross-project insights
**Best For**: Discovery, evaluation, and understanding AIRS ecosystem value
**Content**: 
- Project philosophy and principles
- High-level technical overviews
- Cross-project learning and patterns
- Getting started guidance

### Sub-Project Documentation
**Purpose**: Detailed implementation guidance, API references, and tutorials
**Best For**: Implementation, troubleshooting, and advanced usage
**Technology**: Each sub-project uses `mdbook` for comprehensive technical documentation

## Accessing Sub-Project Documentation

AIRS uses `mdbook` for all detailed technical documentation. Each sub-project maintains its own comprehensive documentation with step-by-step guides, API references, and advanced patterns.

### AIRS-MCP Documentation
**Focus**: Model Context Protocol implementation, server/client development, performance optimization

**Access Instructions:**
```bash
# Navigate to the sub-project
cd crates/airs-mcp/docs/

# Start the documentation server
mdbook serve

# Browse at http://localhost:3000
```

**Documentation Includes:**
- Quick start guides with complete examples
- Protocol implementation deep dives
- Performance optimization and benchmarking
- Security configuration for production
- Advanced patterns and custom transports
- Troubleshooting and migration guides

### AIRS-MemSpec Documentation  
**Focus**: Memory bank methodology, document processing, team collaboration patterns

**Access Instructions:**
```bash
# Navigate to the sub-project
cd crates/airs-memspec/docs/

# Start the documentation server  
mdbook serve

# Browse at http://localhost:3000
```

**Documentation Includes:**
- Installation and setup for different environments
- Essential workflows and command reference
- Architecture and system design details
- Integration patterns for teams and enterprises
- Advanced scenarios and troubleshooting
- Research and development methodology

## Documentation Development Workflow

### For Contributors
Each sub-project's documentation is maintained alongside the code implementation, ensuring accuracy and completeness:

1. **Documentation is Code**: All docs are version-controlled with the implementation
2. **Parallel Development**: Documentation updates happen with feature development
3. **Quality Assurance**: Documentation is reviewed as part of the development process
4. **Independent Deployment**: Each sub-project can publish documentation independently

### For Users
The layered approach provides optimal user experience:

1. **Start with Root Docs**: Get comprehensive overview and strategic understanding
2. **Identify Relevant Sub-Projects**: Determine which components meet your needs
3. **Deep Dive with Sub-Project Docs**: Access detailed implementation guidance
4. **Cross-Reference**: Use root docs for context and sub-project docs for implementation

## Documentation Standards

### Content Quality
- **Accuracy**: All technical information verified against implementation
- **Completeness**: Comprehensive coverage of features and use cases
- **Clarity**: Accessible to intended audience with clear examples
- **Currency**: Regular updates to reflect implementation changes

### User Experience
- **Progressive Disclosure**: Information layered from overview to detail
- **Multiple Entry Points**: Support different user goals and experience levels
- **Clear Navigation**: Obvious paths between overview and detailed content
- **Practical Focus**: Emphasis on actionable guidance and real-world examples

## Getting Help

### Documentation Issues
- **Sub-Project Issues**: Report documentation issues in the relevant sub-project repository
- **Root Documentation Issues**: Report issues with overview content in the main AIRS repository
- **Suggestions**: Contribute improvements through the standard GitHub workflow

### Technical Support
- **Implementation Questions**: Consult sub-project documentation first, then community forums
- **Architecture Questions**: Root documentation provides strategic context
- **Contribution Questions**: Follow contribution guidelines in relevant sub-project documentation

---

**The AIRS documentation ecosystem is designed to scale with the project while serving users effectively. Whether you're evaluating, implementing, or contributing, there's a clear path to the information you need.**
