# Tech Context - AIRS Root Documentation

**Updated**: 2025-01-27  
**Focus**: Documentation tooling, architecture constraints, and technical implementation  
**Critical Discovery**: mdbook cross-linking limitations and architectural solutions

## Core Technology Stack

### Documentation Generation
- **Primary Tool**: mdbook v0.4+ 
- **Source Format**: Markdown with frontmatter support
- **Build Integration**: Compatible with Cargo workspace structure
- **Deployment**: Static site generation for web hosting

### Development Environment
- **Platform**: macOS with zsh shell
- **Editor**: VS Code with Copilot integration
- **Package Management**: Cargo for mdbook installation
- **Version Control**: Git with structured commit workflow

## Critical Technical Discovery: mdbook Cross-Linking Constraints

### Problem Identified 🚨 **ARCHITECTURAL LIMITATION**
**Issue**: mdbook instances cannot reliably cross-link to each other
- **URL Namespace Conflicts**: Each mdbook controls its own URL structure
- **Deployment Fragility**: Links like `../../crates/airs-mcp/docs/book/usages/quick_start.html` fail in deployment
- **Maintenance Complexity**: Cross-references require constant coordination between independent systems

### Root Cause Analysis
**Technical Limitation**: mdbook generates static sites with isolated URL namespaces
- Each mdbook instance expects to control its own URL structure
- Relative links between different mdbook builds break in deployment scenarios  
- No built-in support for coordinated multi-book documentation systems

### Solution Implemented ✅ **INDEPENDENT DOCUMENTATION ARCHITECTURE**

#### Strategic Architecture Decision
**Approach**: Eliminate cross-linking in favor of independent documentation systems
- **Root Documentation**: Comprehensive strategic content providing 80%+ user value
- **Sub-Project Documentation**: Complete implementation details maintained independently
- **Navigation**: Clear instructions for accessing detailed documentation via `mdbook serve`

#### Current Architecture
```
Root Documentation (mdbook)
├── Strategic overviews and philosophy
├── Cross-project insights and patterns  
├── Documentation ecosystem guide
└── Navigation hub for detailed docs

Sub-Project Documentation (independent mdbook instances)
├── crates/airs-mcp/docs/ → mdbook serve (port 3000)
├── crates/airs-memspec/docs/ → mdbook serve (port 3000)
└── Future sub-projects maintain same pattern
```

## Development Workflow

### Local Development Setup
```bash
# Root documentation
cd docs/
mdbook serve --open -p 8000

# Sub-project documentation (independent)
cd crates/airs-mcp/docs/
mdbook serve --open -p 3000

cd crates/airs-memspec/docs/  
mdbook serve --open -p 3001
```

### Content Development Process
1. **Strategic Content**: Create comprehensive overviews in root documentation
2. **Navigation Guidance**: Provide clear instructions for accessing detailed docs
3. **Quality Assurance**: Ensure standalone value without cross-linking dependencies
4. **User Journey Validation**: Test that users can navigate effectively between systems

### Documentation Organization
```
docs/src/
├── foreword.md              # Project narrative and story
├── overview.md              # Technical landscape and achievements  
├── philosophy_principles.md # Design principles and methodology
├── projects/                # Sub-project strategic overviews
│   ├── airs_mcp.md         # AIRS-MCP comprehensive overview
│   └── airs_memspec.md     # AIRS-MemSpec comprehensive overview
├── technical/               # Cross-project technical knowledge
└── resources/               # Practical guides and ecosystem navigation
    └── documentation_guide.md # How to navigate AIRS documentation
```

## Technical Implementation Details

### mdbook Configuration
```toml
[book]
title = "AIRS: AI-Rust Infrastructure and Systems"
description = "Comprehensive documentation for the AIRS ecosystem"
src = "src"

[build]
build-dir = "book"

[output.html]
default-theme = "light"
```

### Content Standards
- **Standalone Value**: Each documentation system provides complete value for its purpose
- **Clear Navigation**: Explicit guidance for accessing detailed information
- **No Cross-Links**: Eliminates fragile dependencies between mdbook instances
- **Progressive Disclosure**: Natural information layering from strategic to tactical

### Quality Assurance
- **Link Validation**: Internal links within each mdbook instance only
- **Content Review**: Accuracy and clarity validation for all content
- **User Journey Testing**: Verify navigation patterns work effectively
- **Build Validation**: Ensure mdbook generates without errors

## Architecture Benefits

### Technical Benefits ✅
- **No Cross-Linking Fragility**: Eliminates broken links between mdbook instances
- **Independent Deployment**: Each documentation system deploys separately
- **Scalable Architecture**: Works for unlimited sub-projects without coordination
- **Maintenance Simplicity**: No complex cross-reference management required

### Development Benefits ✅
- **Independent Workflows**: Sub-projects evolve documentation without coordination
- **Documentation Autonomy**: Each component owns its detailed documentation
- **Reduced Complexity**: No cross-system coordination requirements
- **Contributor Freedom**: Work on documentation without affecting other systems

### User Experience Benefits ✅
- **Comprehensive Standalone Value**: Users get substantial value from root docs alone
- **Clear Navigation Paths**: Obvious progression from overview to implementation
- **No Broken Links**: Eliminates user frustration from navigation failures
- **Progressive Disclosure**: Natural information layering serves different user needs

## Constraints & Limitations

### Technical Constraints
- **mdbook Limitations**: Must work within mdbook's static generation capabilities
- **Markdown Constraints**: Complex content must fit markdown paradigms
- **No Cross-Linking**: Cannot link between different mdbook instances
- **Static Only**: No dynamic content or server-side processing

### Resource Constraints  
- **Single Maintainer**: Architecture must be sustainable for individual maintenance
- **Coordination Overhead**: Minimize synchronization between documentation systems
- **Quality Standards**: Maintain high content quality without complex processes

### Operational Constraints
- **Deployment Independence**: Each system must deploy without dependencies
- **User Education**: Users must understand how to navigate between systems
- **Content Duplication**: Avoid duplicating content while providing comprehensive value

## Success Metrics

### Technical Success ✅
- **Build Reliability**: All mdbook instances generate without errors
- **No Broken Links**: All internal references work correctly  
- **Independent Operation**: Each system works without external dependencies
- **Sustainable Maintenance**: Updates require minimal coordination

### User Experience Success ✅
- **Comprehensive Value**: Root docs provide 80%+ of discovery/evaluation needs
- **Clear Navigation**: Users understand how to access implementation details
- **Quality Consistency**: All documentation maintains high standards
- **Smooth Transitions**: Natural progression from strategic to tactical information
