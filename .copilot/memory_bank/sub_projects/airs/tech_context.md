# AIRS Root Project - Tech Context

**Last Updated**: 2025-08-11  
**Context**: Technical environment and tooling for root documentation project

## Technology Stack

### Core Technologies
- **Documentation Engine**: mdbook 0.4+ 
- **Source Format**: Markdown (CommonMark)
- **Build System**: Cargo workspace integration
- **Version Control**: Git with structured branching
- **Hosting**: Static site hosting (GitHub Pages compatible)

### Development Environment
- **Primary Platform**: macOS (development environment)
- **Shell**: zsh (default shell for command execution)
- **Editor**: VS Code with Copilot integration
- **Package Manager**: Cargo (for mdbook and dependencies)

### Documentation Toolchain
```
Source (Markdown) → mdbook → Static HTML → Hosting
├── Input: .md files in docs/src/
├── Processing: mdbook build with custom themes/config
├── Output: Static HTML site in docs/book/
└── Deployment: Static file hosting
```

## Development Setup

### Required Dependencies
```bash
# Core requirement
cargo install mdbook

# Optional enhancements (if needed)
cargo install mdbook-linkcheck     # Link validation
cargo install mdbook-toc           # Table of contents generation
cargo install mdbook-mermaid       # Diagram support
```

### Project Structure
```
/Users/hiraq/Projects/rstlix0x0/airs/docs/
├── book.toml                    # mdbook configuration
├── src/                         # Source markdown files
│   ├── SUMMARY.md              # Table of contents
│   ├── *.md                    # Content files
│   ├── projects/               # Sub-project overviews
│   ├── technical/              # Technical knowledge
│   └── resources/              # Guides and references
├── book/                       # Generated static site (git-ignored)
└── .gitignore                  # Ignore build artifacts
```

### Build Commands
```bash
# Development server with live reload
mdbook serve docs

# Production build
mdbook build docs

# Test and validate
mdbook test docs

# Clean build artifacts
mdbook clean docs
```

## Technical Constraints

### mdbook Limitations
- **Static Generation Only**: No dynamic content or server-side processing
- **Markdown Restrictions**: Limited to supported markdown extensions
- **Theme Customization**: Limited CSS/JS customization options
- **Navigation Structure**: Table of contents defined in SUMMARY.md

### Performance Requirements
- **Build Time**: Documentation should build quickly (< 30 seconds)
- **Site Performance**: Fast loading and navigation
- **Mobile Compatibility**: Responsive design for various devices
- **Search Performance**: Efficient client-side search

### Integration Requirements
- **Cargo Workspace**: Integration with existing workspace structure
- **Memory Bank**: Compatible with memory bank development workflow
- **Sub-Project Docs**: Coordinate with existing sub-project documentation
- **Git Workflow**: Standard git branching and merging

## Configuration Details

### mdbook Configuration (book.toml)
```toml
[book]
authors = ["rstlix0x0"]
language = "en"
src = "src"
title = "AIRS - AI in Rust"
description = "Comprehensive documentation for the AIRS AI & Rust technology stack"

[build]
build-dir = "book"
create-missing = true

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/rstlix0x0/airs"
edit-url-template = "https://github.com/rstlix0x0/airs/edit/main/docs/src/{path}"

[output.html.search]
enable = true
limit-results = 20
teaser-word-count = 30
use-boolean-and = true

[output.html.print]
enable = true
```

### File Organization Strategy
- **Naming Convention**: snake_case for all files and directories
- **Content Structure**: Logical hierarchy with clear relationships
- **Cross-References**: Relative links for portability
- **Asset Management**: Images and resources in appropriate subdirectories

## Development Workflow

### Content Development Process
```
1. Plan → Memory Bank (task creation)
2. Write → Markdown files in docs/src/
3. Test → mdbook serve for local validation
4. Review → Content quality and link validation
5. Build → mdbook build for production
6. Deploy → Static site hosting
7. Update → Memory bank with progress
```

### Quality Assurance
- **Link Validation**: Regular checking of internal and external links
- **Content Review**: Systematic review for accuracy and clarity
- **Build Testing**: Verify builds work across different environments
- **User Testing**: Validate navigation and user experience

### Maintenance Workflow
- **Regular Updates**: Keep content current with project evolution
- **Link Maintenance**: Update links as sub-projects evolve
- **Structure Evolution**: Adapt organization as content grows
- **Performance Monitoring**: Ensure site performance remains optimal

## Integration Patterns

### Workspace Integration
```bash
# From workspace root
cd docs && mdbook serve     # Development
cd docs && mdbook build     # Production

# Integrated with cargo (potential future)
cargo doc --workspace      # Code documentation
cd docs && mdbook build    # Project documentation
```

### Memory Bank Integration
- **Task Management**: Use memory bank for tracking documentation tasks
- **Progress Tracking**: Document development progress in memory bank
- **Decision Recording**: Capture architectural decisions
- **Context Preservation**: Maintain development context across sessions

### Sub-Project Coordination
- **Shared Standards**: Consistent style and structure across all documentation
- **Cross-References**: Maintain links between root and sub-project documentation
- **Version Coordination**: Ensure compatibility across documentation versions
- **Build Coordination**: Potential for unified documentation builds

## Security & Privacy

### Content Security
- **No Sensitive Data**: Documentation contains only public information
- **Link Safety**: All external links verified for safety and relevance
- **Static Security**: Benefits from static site security model

### Privacy Considerations
- **No Analytics**: No user tracking in documentation (unless explicitly added)
- **No Dynamic Content**: No user data collection or processing
- **Open Source**: All content publicly available

## Performance Optimization

### Build Performance
- **Incremental Builds**: mdbook supports incremental building
- **Asset Optimization**: Optimize images and other assets for web delivery
- **Content Organization**: Structure content for efficient processing

### Runtime Performance
- **Static Assets**: All content served as static files
- **Client-Side Search**: Efficient search without server requirements
- **Responsive Design**: Optimized for various screen sizes and devices

## Future Technical Considerations

### Potential Enhancements
- **Custom Themes**: Develop AIRS-specific visual themes
- **Enhanced Search**: Integrate more sophisticated search capabilities
- **Diagram Support**: Add mermaid or other diagram rendering
- **PDF Generation**: Enable PDF export for offline reading

### Scalability Planning
- **Content Growth**: Architecture should scale with increasing content
- **Multi-Language**: Structure should accommodate future internationalization
- **Advanced Features**: Framework should support additional mdbook plugins
- **Integration Expansion**: Prepare for integration with additional tools

### Technology Evolution
- **mdbook Updates**: Stay current with mdbook feature development
- **Markdown Standards**: Adapt to evolving markdown standards
- **Web Standards**: Ensure compatibility with web technology evolution
- **Accessibility Standards**: Maintain compliance with accessibility requirements
