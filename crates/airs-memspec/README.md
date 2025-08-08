# airs-memspec

A CLI tool for Multi-Project Memory Bank management and GitHub Copilot custom instructions integration.

## Overview

`airs-memspec` is a lightweight command-line tool designed to streamline AI-assisted development workflows by managing Multi-Project Memory Bank structures and GitHub Copilot custom instructions. The tool enables seamless context preservation and intelligent project state management across multiple sub-projects within a workspace.

## Key Features

- **📦 Custom Instructions Management**: Install and manage GitHub Copilot custom instructions
- **🏗️ Memory Bank Awareness**: Parse and display Multi-Project Memory Bank structures
- **📊 Project State Visualization**: Quick overview of workspace and sub-project states
- **🎯 Context-Aware Reading**: Understand workspace hierarchy and sub-project relationships
- **⚡ Lightweight & Fast**: Simple tool focused on essential memory bank operations

## Installation

### From Source
```bash
git clone https://github.com/rstlix0x0/airs.git
cd airs
cargo build --release --bin airs-memspec
```

### Using Cargo
```bash
cargo install --path crates/airs-memspec
```

## Comprehensive Usage Guide

### Installation & Setup

#### Option 1: From Cargo (Recommended)
```bash
# Install from crates.io (when published)
cargo install airs-memspec

# Verify installation
airs-memspec --version
```

#### Option 2: From Source
```bash
# Clone the AIRS workspace
git clone https://github.com/rstlix0x0/airs.git
cd airs

# Build and install
cargo build --release --bin airs-memspec
cargo install --path crates/airs-memspec

# Verify installation
airs-memspec --help
```

### Essential Workflows

#### 🚀 New Project Setup
```bash
# 1. Navigate to your workspace root
cd /path/to/your/workspace

# 2. Install GitHub Copilot instructions
airs-memspec install --path .copilot --force

# 3. Verify installation
airs-memspec status

# 4. Expected output if no memory bank exists yet:
# ❌ No memory bank structure found
# 💡 Use GitHub Copilot with custom instructions to create it
```

#### 📊 Daily Development Workflow
```bash
# Morning routine - check current state
airs-memspec status --workspace

# Check what you were working on
airs-memspec context

# Review active tasks
airs-memspec tasks --filter active

# After making progress - refresh context
airs-memspec context --project your-project
```

#### 🔄 Multi-Project Context Switching
```bash
# Check all projects
airs-memspec status

# Switch focus to specific project
airs-memspec context --project backend-api

# Check progress across all projects
airs-memspec status --workspace

# View specific project tasks
airs-memspec tasks --project frontend-app --filter pending
```

### Command Reference with Examples

#### `airs-memspec install`
Deploy GitHub Copilot custom instructions to enable AI-assisted memory bank management.

```bash
# Basic installation to .copilot directory
airs-memspec install

# Install to custom path
airs-memspec install --path /path/to/copilot/config

# Force overwrite existing files
airs-memspec install --force

# Install specific template
airs-memspec install --template multi-project
```

**When to use**: First-time setup, updating instruction templates, or setting up new workspaces.

#### `airs-memspec status`
Display comprehensive workspace and project status overview.

```bash
# Default: current active project status
airs-memspec status

# Workspace-wide overview
airs-memspec status --workspace

# Specific project status
airs-memspec status --project backend-api

# Quiet mode (essential info only)
airs-memspec status --quiet

# Verbose mode (detailed analysis)
airs-memspec status --verbose
```

**When to use**: Daily development start, progress reviews, project health checks.

#### `airs-memspec context`
Analyze and display current development context across projects.

```bash
# Show current active context
airs-memspec context

# Workspace-level context and integration
airs-memspec context --workspace

# Specific project context
airs-memspec context --project frontend-app

# Context with custom workspace path
airs-memspec context --path /path/to/workspace
```

**When to use**: Understanding current work focus, context switching, architectural review.

#### `airs-memspec tasks`
Task viewing and progress tracking (read-only).

```bash
# List all tasks with smart filtering (default: 15 most relevant)
airs-memspec tasks list

# Show all tasks (disable smart filtering)
airs-memspec tasks list --all

# Filter by status
airs-memspec tasks list --status active
airs-memspec tasks list --status pending
airs-memspec tasks list --status completed

# Filter by project
airs-memspec tasks list --project backend-api

# Include completed tasks in smart view
airs-memspec tasks list --completed

# Show detailed task information
airs-memspec tasks show task_001

# Show recently updated tasks
airs-memspec tasks list --filter recent

# Project-specific tasks
airs-memspec tasks list --project backend-api --status active
```

**When to use**: Sprint planning, progress tracking, task prioritization, team coordination.

### Integration Patterns

#### 🤖 GitHub Copilot Workflow Integration

**Step 1: Install Instructions**
```bash
# Setup instructions for Copilot
airs-memspec install --path .copilot
```

**Step 2: Configure GitHub Copilot**
1. Open VS Code settings
2. Navigate to GitHub Copilot settings
3. Set custom instructions path to `.copilot/instructions/`
4. Restart VS Code to apply changes

**Step 3: Create Memory Bank Structure**
```bash
# Chat with Copilot to create memory bank
# Use commands like: "create memory bank structure"
# Copilot will use the installed instructions automatically
```

**Step 4: Monitor and Analyze**
```bash
# Regular status checks
airs-memspec status --workspace

# Context analysis before major changes
airs-memspec context --project your-project

# Task progress monitoring
airs-memspec tasks list --filter active
```

#### 🏗️ VS Code Workspace Setup

**1. Workspace Configuration**
```json
// .vscode/settings.json
{
  "github.copilot.chat.localeOverride": "en",
  "github.copilot.enable": {
    "*": true,
    "yaml": true,
    "plaintext": true,
    "markdown": true
  }
}
```

**2. Integrated Terminal Workflow**
```bash
# Add to your shell profile (.bashrc, .zshrc)
alias mbs="airs-memspec status"
alias mbc="airs-memspec context"
alias mbt="airs-memspec tasks list --filter active"

# Quick status in terminal
mbs && mbc
```

**3. Development Task Integration**
```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Memory Bank Status",
      "type": "shell",
      "command": "airs-memspec",
      "args": ["status", "--workspace"],
      "group": "test",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    }
  ]
}
```

#### 🔄 Multi-Project Development Patterns

**Pattern 1: Shared Library + Multiple Consumers**
```bash
# Monitor shared library changes impact
airs-memspec status --workspace

# Check integration points
airs-memspec context --workspace

# Track dependent project tasks
airs-memspec tasks list --filter blocked
```

**Pattern 2: Microservices Architecture**
```bash
# Service-by-service status
for service in auth user payment notification; do
  echo "=== $service ==="
  airs-memspec context --project $service
done

# Cross-service dependency tracking
airs-memspec tasks list --filter active
```

**Pattern 3: Frontend + Backend Development**
```bash
# Full-stack development workflow
airs-memspec context --project backend-api
airs-memspec context --project frontend-app
airs-memspec tasks list --project backend-api --status in_progress
airs-memspec tasks list --project frontend-app --status pending
```

### Advanced Usage Scenarios

#### 🔍 Project Health Monitoring
```bash
# Daily health check script
#!/bin/bash
echo "=== Daily Workspace Health Check ==="
airs-memspec status --workspace --verbose

echo -e "\n=== Active Development Focus ==="
airs-memspec context

echo -e "\n=== Blocked or High Priority Tasks ==="
airs-memspec tasks list --priority high
airs-memspec tasks list --filter blocked

echo -e "\n=== Recent Progress ==="
airs-memspec tasks list --filter recent
```

#### 📈 Progress Reporting
```bash
# Weekly progress report
#!/bin/bash
echo "=== Weekly Progress Report ==="
echo "Date: $(date)"
echo ""

for project in $(airs-memspec status --workspace --quiet | grep "📁" | cut -d' ' -f2); do
  echo "=== $project ==="
  airs-memspec tasks list --project $project --status completed --filter recent
  airs-memspec tasks list --project $project --status in_progress
  echo ""
done
```

#### 🧹 Workspace Maintenance
```bash
# Workspace cleanup and validation
#!/bin/bash
echo "=== Workspace Maintenance ==="

# Check for orphaned tasks
airs-memspec tasks list --filter stale

# Validate memory bank structure integrity
airs-memspec status --workspace --verbose

# Check for missing documentation
airs-memspec context --workspace | grep "⚠️"
```

### Best Practices

#### 📋 Development Workflow Best Practices

1. **Start Each Day with Status Check**
   ```bash
   airs-memspec status --workspace && airs-memspec context
   ```

2. **Use Context Switching for Focus**
   ```bash
   # Before starting work on a different project
   airs-memspec context --project new-project-focus
   ```

3. **Regular Task Progress Updates**
   ```bash
   # Check active tasks before standups
   airs-memspec tasks list --filter active
   ```

4. **Monitor Cross-Project Dependencies**
   ```bash
   # Weekly dependency review
   airs-memspec status --workspace --verbose
   ```

#### 🎯 AI-Assisted Development Best Practices

1. **Keep Instructions Updated**
   ```bash
   # Periodically update Copilot instructions
   airs-memspec install --force
   ```

2. **Use Context for Better AI Responses**
   - Always run `airs-memspec context` before asking complex questions to Copilot
   - Reference current project state in AI conversations
   - Use task status to inform AI about current priorities

3. **Leverage Memory Bank Structure**
   - Organize work according to memory bank patterns
   - Use standardized task tracking for consistency
   - Maintain context documentation for AI understanding

4. **Regular Memory Bank Validation**
   ```bash
   # Ensure memory bank structure is healthy
   airs-memspec status --workspace --verbose
   ```
airs-memspec status
```

### 3. View Context Information
```bash
# Show workspace context
airs-memspec context --workspace

# Show sub-project active context
airs-memspec context --project airs-mcp
```

### 4. View Task Management
```bash
# Show all tasks for a sub-project
airs-memspec tasks --project airs-mcp

# Show tasks with specific filter
airs-memspec tasks --project airs-mcp --filter active
```

## Command Reference

### Installation Commands
- `install --path <PATH>` - Install custom instructions to specified directory

### Status Commands
- `status [--workspace] [--project <name>]` - Show development status and progress
- `context [--workspace] [--project <name>]` - Show active context and focus areas
- `tasks --project <name> [--filter <type>]` - Show task lists and progress

### Global Options
- `--path <PATH>` - Path to memory bank root [default: .copilot]
- `--verbose, -v` - Enable verbose output
- `--quiet, -q` - Minimal output for scripting
- `--no-color` - Disable colored output
- `--help, -h` - Print help information
- `--version, -V` - Print version information

## Memory Bank Structure

`airs-memspec` understands and parses the Multi-Project Memory Bank structure:

```
.copilot/memory_bank/
├── current_context.md              # Active sub-project tracker
├── workspace/                      # Workspace-level shared knowledge
│   ├── project_brief.md
│   ├── shared_patterns.md
│   ├── workspace_architecture.md
│   └── workspace_progress.md
├── context_snapshots/              # Historical state snapshots
└── sub_projects/                   # Individual sub-projects
    ├── airs-mcp/
    │   ├── project_brief.md
    │   ├── product_context.md
    │   ├── active_context.md
    │   ├── system_patterns.md
    │   ├── tech_context.md
    │   ├── progress.md
    │   └── tasks/
    │       ├── _index.md
    │       └── task_*.md
    └── airs-memspec/
        └── ...
```

## Integration with GitHub Copilot

1. **Install Custom Instructions**: Use `airs-memspec install` to set up custom instructions
2. **Apply to Copilot**: Configure GitHub Copilot to use the installed custom instructions
3. **Create Memory Bank**: Chat with Copilot to create the memory bank structure
4. **Monitor State**: Use `airs-memspec` commands to view current project state
5. **Develop with Context**: GitHub Copilot automatically maintains context awareness

## Example Workflow

```bash
# 1. Setup custom instructions
airs-memspec install --path .copilot

# 2. Check if memory bank exists and current state
airs-memspec status

# 3. View active context for current work
airs-memspec context --project airs-mcp

# 4. Check task progress
airs-memspec tasks --project airs-mcp --filter active

# 5. Get workspace overview
airs-memspec status --workspace
```

## Output Examples

### Workspace Status
```
🏢 AIRS Workspace
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Status        Active Development - Foundation Phase
Focus         MCP Protocol Implementation & Tooling  
Updated       2 hours ago

Projects      2 active, 0 paused
├─ airs-mcp      🟢 Week 1/14 - JSON-RPC Foundation
└─ airs-memspec  🟡 Planning - CLI Development

Next Milestone   JSON-RPC 2.0 Core Complete (3 days)
Blockers         None
```

### Project Context
```
🎯 airs-mcp Active Context
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current Focus
  JSON-RPC 2.0 Foundation & Transport Layer Implementation

Active Work
  🔧 Implementing MCP error code extensions
  📝 Serde integration and serialization testing
  ⏱️  Started 4 hours ago

Integration Points
  • Transport abstraction for STDIO and HTTP
  • State machine for protocol lifecycle management
  • Security layer for OAuth 2.1 + PKCE

Constraints
  • Must follow JSON-RPC 2.0 specification exactly
  • MCP protocol compliance required for Claude Desktop
  • Performance targets: <1ms message processing
```

## Development

### Building from Source
```bash
# Clone the AIRS workspace
git clone https://github.com/rstlix0x0/airs.git
cd airs

# Build the memspec tool
cargo build --release --bin airs-memspec

# Run tests
cargo test -p airs-memspec

# Install locally
cargo install --path crates/airs-memspec
```

### Contributing

This project is part of the AIRS (AI & Rust) workspace. See the main [AIRS README](../../README.md) for contribution guidelines and development setup.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- **airs-mcp**: Model Context Protocol implementation in Rust
- **AIRS Workspace**: Complete AI & Rust technology stack

---

**Note**: This tool is designed to work in conjunction with GitHub Copilot and the Multi-Project Memory Bank custom instructions. The memory bank structure itself is created and maintained through AI collaboration, while `airs-memspec` provides installation and state reading capabilities.