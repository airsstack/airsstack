# AIRS Root Project - Product Context

**Last Updated**: 2025-08-11  
**Context**: Root documentation project for AIRS ecosystem

## Why This Project Exists

### The Problem

The AIRS ecosystem has grown to include multiple sophisticated sub-projects (airs-mcp, airs-memspec) with their own comprehensive documentation. However, there's a gap at the workspace level:

1. **Fragmented Knowledge**: Technical insights and architectural decisions are scattered across sub-projects
2. **Missing Narrative**: No unified story explaining the "why" behind AIRS and its design philosophy
3. **Onboarding Challenges**: New contributors lack a clear entry point to understand the ecosystem
4. **Knowledge Silos**: Cross-project learnings and research aren't centrally documented

### The Opportunity

Create a comprehensive documentation hub that:
- Tells the complete AIRS story from vision to implementation
- Serves as the definitive guide to the ecosystem's philosophy and principles
- Provides technical knowledge that spans multiple projects
- Offers clear pathways for different types of users (newcomers, contributors, researchers)

## What This Project Solves

### For New Users
- **Clear Introduction**: Understand what AIRS is and why it exists
- **Guided Discovery**: Find the right sub-project for their needs
- **Philosophy Understanding**: Grasp the human-AI collaboration approach

### For Contributors
- **Design Principles**: Understand the architectural philosophy and constraints
- **Development Methodology**: Learn the AI-assisted development workflow
- **Cross-Project Knowledge**: Access research and insights that span multiple projects

### For Researchers
- **Technical Deep Dives**: Access detailed technical knowledge and research
- **Architecture Insights**: Understand system design decisions and trade-offs
- **Future Directions**: See roadmap and planned research areas

## How It Should Work

### User Experience Goals

#### Information Architecture
```
Entry Points:
├── Curious → Foreword (storytelling)
├── Evaluating → Overview (technical landscape)  
├── Contributing → Philosophy & Principles
├── Implementing → Technical Knowledge
└── Researching → Deep Technical Sections
```

#### Navigation Flow
1. **Discovery**: Users find AIRS through root documentation
2. **Understanding**: Learn philosophy and approach through narrative content
3. **Exploration**: Discover relevant sub-projects through project overviews
4. **Deep Dive**: Access detailed documentation in respective sub-projects
5. **Cross-Reference**: Return to root for technical knowledge and research

#### Content Strategy
- **Layered Information**: Start high-level, provide paths to detailed content
- **Cross-Linking**: Seamless navigation between root and sub-project documentation
- **Living Document**: Regular updates as projects evolve and new insights emerge

### Key User Journeys

#### Journey 1: New Developer Discovery
```
README.md → Foreword → Overview → Getting Started → Sub-project docs
```

#### Journey 2: Contributor Onboarding  
```
Contributing Guide → Philosophy & Principles → Development Workflow → Active Projects
```

#### Journey 3: Technical Research
```
Technical Knowledge → Architecture docs → Sub-project deep dives → Research insights
```

#### Journey 4: AI-Rust Integration Learning
```
Overview → Philosophy → Technical Knowledge → Implementation examples
```

## Success Metrics

### Qualitative Indicators
- Users can quickly understand what AIRS is and why it exists
- Contributors can onboard effectively using the documentation
- Technical decisions and research are well-documented and accessible
- Documentation serves as a reliable reference for the ecosystem

### Quantitative Targets
- Complete documentation structure with all planned sections
- Comprehensive cross-references to sub-project documentation
- Sustainable maintenance workflow established
- Positive feedback from early users and contributors

## User Personas

### Primary: Technical Evaluator
- **Background**: Experienced developer evaluating AIRS for projects
- **Goals**: Understand capabilities, architecture, and production readiness
- **Needs**: Clear technical overview, real-world examples, decision criteria

### Secondary: AI-Rust Explorer  
- **Background**: Developer interested in AI-Rust integration patterns
- **Goals**: Learn from AIRS approach and apply insights to own projects
- **Needs**: Philosophy documentation, technical knowledge, research insights

### Tertiary: Open Source Contributor
- **Background**: Developer wanting to contribute to AIRS ecosystem
- **Goals**: Understand contribution process and project priorities
- **Needs**: Clear contribution guidelines, development workflow, project roadmap

## Context & Constraints

### Current State
- Sub-projects have comprehensive individual documentation
- Root documentation is minimal (basic mdbook setup)
- Project has established philosophy and working implementations
- Memory bank system provides structured development approach

### Future Vision
- Comprehensive documentation hub that rivals the best open source projects
- Clear technical leadership in AI-Rust integration space
- Knowledge base that accelerates ecosystem development
- Reference architecture for similar projects

### Success Dependencies
- Maintaining consistency with existing sub-project documentation
- Regular updates as projects evolve
- Clear ownership and maintenance responsibilities
- Integration with development workflow
