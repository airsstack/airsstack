# Usage Guide

This comprehensive usage guide covers all aspects of using airs-memspec for Multi-Project Memory Bank management and GitHub Copilot integration.

## Quick Navigation

- **[Installation & Setup](./usages/installation.md)** - Getting started with airs-memspec
- **[Essential Workflows](./usages/workflows.md)** - Daily development patterns and routines
- **[Command Reference](./usages/commands.md)** - Complete command documentation with examples
- **[Integration Patterns](./usages/integration.md)** - GitHub Copilot and VS Code integration
- **[Advanced Scenarios](./usages/advanced.md)** - Complex workflows and automation
- **[Best Practices](./usages/best-practices.md)** - Professional development recommendations
- **[Troubleshooting](./usages/troubleshooting.md)** - Common issues and solutions

## Overview

airs-memspec is designed to bridge the gap between structured development context and AI-assisted development workflows. It provides essential tools for:

### 🎯 Core Capabilities

- **Memory Bank Installation**: Deploy standardized memory bank structures with GitHub Copilot instructions
- **Context Analysis**: Analyze and display workspace context across multiple projects  
- **Project Monitoring**: Monitor progress and health across sub-projects
- **Task Management**: Track and manage development tasks with detailed progress logging
- **AI Integration**: Enable consistent AI guidance through embedded instruction templates

### 🚀 Getting Started

If you're new to airs-memspec, start with our [Installation & Setup](./usages/installation.md) guide, then follow the [Essential Workflows](./usages/workflows.md) to understand daily usage patterns.

### 📚 Learning Path

1. **Beginners**: [Installation](./usages/installation.md) → [Workflows](./usages/workflows.md) → [Commands](./usages/commands.md)
2. **Integrators**: [Integration Patterns](./usages/integration.md) → [Best Practices](./usages/best-practices.md)
3. **Advanced Users**: [Advanced Scenarios](./usages/advanced.md) → [Troubleshooting](./usages/troubleshooting.md)

### 🔧 Tool Philosophy

airs-memspec follows the principle of "intelligent assistance without interference" - providing powerful context management capabilities while maintaining simplicity and staying out of your way during development.

## Memory Bank Integration

The tool is designed to work seamlessly with the Multi-Project Memory Bank methodology:

```
Workspace Root
├── .copilot/
│   ├── instructions/          # ← airs-memspec installs here
│   └── memory_bank/           # ← airs-memspec reads from here
│       ├── current_context.md
│       ├── workspace/
│       └── sub_projects/
└── your-projects/
```

This structure enables AI assistants to maintain context awareness while airs-memspec provides the tooling to monitor and analyze that context.

## Professional Development Standards

airs-memspec enforces and supports professional development practices:

- **Zero-Warning Policy**: All operations maintain clean, warning-free code
- **Comprehensive Testing**: Every feature includes thorough test coverage
- **Rich Documentation**: All functionality includes examples and usage patterns
- **Error Handling**: Robust error reporting with actionable guidance
- **Performance**: Efficient operations optimized for large workspaces
