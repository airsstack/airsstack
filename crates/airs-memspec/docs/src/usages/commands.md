# Command Reference

Complete reference documentation for all airs-memspec commands with detailed examples and use cases.

## Global Options

These options are available for all commands:

```bash
--path <PATH>     # Path to workspace root [default: current directory]
--verbose, -v     # Enable verbose output with detailed information
--quiet, -q       # Minimal output suitable for scripting
--no-color        # Disable colored output
--help, -h        # Show help information
--version, -V     # Show version information
```

### Global Options Examples

```bash
# Use custom workspace path
airs-memspec status --path /path/to/workspace

# Enable verbose mode for debugging
airs-memspec status --verbose

# Quiet mode for scripts
airs-memspec status --quiet

# Disable colors for plain text output
airs-memspec status --no-color
```

## Commands Overview

| Command | Purpose | Common Use Cases |
|---------|---------|------------------|
| [`install`](#install-command) | Deploy instruction templates | First-time setup, template updates |
| [`status`](#status-command) | Show workspace/project status | Daily health checks, progress reviews |
| [`context`](#context-command) | Display development context | Focus switching, architectural review |
| [`tasks`](#tasks-command) | Task management operations | Sprint planning, progress tracking |

---

## `install` Command

Deploy GitHub Copilot custom instructions to enable AI-assisted memory bank management.

### Syntax

```bash
airs-memspec install [OPTIONS]
```

### Options

```bash
--path <PATH>            # Target directory for installation [default: .copilot]
--force                  # Force overwrite existing files
--template <TEMPLATE>    # Specific template to install [default: multi-project]
```

### Usage Examples

#### Basic Installation

```bash
# Install to default .copilot directory
airs-memspec install

# Expected output:
# ✅ Instructions installed to .copilot/instructions/
# 📁 Created: multi_project_memory_bank.instructions.md
# 💡 Configure GitHub Copilot to use this directory
```

#### Custom Path Installation

```bash
# Install to custom directory
airs-memspec install --path /path/to/copilot/config

# Install to VS Code user settings directory (macOS example)
airs-memspec install --path ~/Library/Application\ Support/Code/User/copilot
```

#### Force Overwrite

```bash
# Overwrite existing installation
airs-memspec install --force

# Useful when updating instruction templates
airs-memspec install --force --path .copilot
```

#### Template Selection

```bash
# Install specific template (currently only multi-project available)
airs-memspec install --template multi-project

# Future templates might include:
# airs-memspec install --template single-project
# airs-memspec install --template enterprise
```

### When to Use

- **First-time setup**: Initial installation in new workspace
- **Template updates**: When airs-memspec releases new instruction versions  
- **Multi-workspace**: Setting up instructions in multiple workspaces
- **Team setup**: Ensuring consistent instruction deployment across team

### Error Handling

```bash
# Permission denied
$ airs-memspec install --path /restricted/path
❌ Error: Permission denied to write to /restricted/path
💡 Try: sudo airs-memspec install --path /restricted/path

# Directory doesn't exist
$ airs-memspec install --path /nonexistent/path
❌ Error: Directory /nonexistent does not exist
💡 Try: mkdir -p /nonexistent/path && airs-memspec install --path /nonexistent/path
```

---

## `status` Command

Display comprehensive workspace and project status overview with health metrics and progress indicators.

### Syntax

```bash
airs-memspec status [OPTIONS]
```

### Options

```bash
--workspace              # Show workspace-level overview
--project <PROJECT>      # Show specific project status
```

### Usage Examples

#### Default Status

```bash
# Show current active project status
airs-memspec status

# Example output:
# 🎯 airs-mcp Active Context
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 
# Current Focus    JSON-RPC 2.0 Foundation & Transport Layer
# Active Work      🔧 Implementing MCP error code extensions
# Status           🟢 In Progress - Week 1/14
# Last Updated     2 hours ago
```

#### Workspace Overview

```bash
# Show complete workspace status
airs-memspec status --workspace

# Example output:
# 🏢 AIRS Workspace
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 
# Status           Active Development - Foundation Phase
# Focus            MCP Protocol Implementation & Tooling  
# Updated          2 hours ago
# 
# Projects         2 active, 0 paused
# ├─ airs-mcp      🟢 Week 1/14 - JSON-RPC Foundation
# └─ airs-memspec  🟡 Planning - CLI Development
# 
# Next Milestone   JSON-RPC 2.0 Core Complete (3 days)
# Blockers         None
```

#### Specific Project Status

```bash
# Show status for specific project
airs-memspec status --project backend-api

# Show status with verbose details
airs-memspec status --project frontend-app --verbose
```

#### Quiet Mode for Scripts

```bash
# Machine-readable output
airs-memspec status --quiet

# Example output:
# STATUS=active
# PROJECT_COUNT=2
# ACTIVE_PROJECT=airs-mcp
# HEALTH=green
# BLOCKERS=0
```

### When to Use

- **Daily startup**: Check workspace health at beginning of work
- **Progress reviews**: Weekly/monthly status assessment
- **Team standups**: Quick project health overview
- **CI/CD pipelines**: Automated workspace health checks
- **Debugging**: Understanding current workspace state

### Status Indicators

| Indicator | Meaning | Action Required |
|-----------|---------|-----------------|
| 🟢 Green | Healthy, on track | Continue current work |
| 🟡 Yellow | Attention needed | Review progress, address issues |
| 🔴 Red | Blocked or critical | Immediate attention required |
| 🟤 Brown | Planning/setup phase | Define next steps |
| ⚪ Gray | Paused/inactive | Resume when ready |

---

## `context` Command

Analyze and display current development context with architectural decisions, focus areas, and integration points.

### Syntax

```bash
airs-memspec context [OPTIONS]
```

### Options

```bash
--workspace              # Show workspace-level context and architecture
--project <PROJECT>      # Show specific project context
```

### Usage Examples

#### Current Active Context

```bash
# Show current active project context
airs-memspec context

# Example output:
# 🎯 airs-mcp Active Context
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 
# Current Focus
#   JSON-RPC 2.0 Foundation & Transport Layer Implementation
# 
# Active Work
#   🔧 Implementing MCP error code extensions
#   📝 Serde integration and serialization testing
#   ⏱️  Started 4 hours ago
# 
# Integration Points
#   • Transport abstraction for STDIO and HTTP
#   • State machine for protocol lifecycle management
#   • Security layer for OAuth 2.1 + PKCE
# 
# Constraints
#   • Must follow JSON-RPC 2.0 specification exactly
#   • MCP protocol compliance required for Claude Desktop
#   • Performance targets: <1ms message processing
```

#### Workspace-Level Context

```bash
# Show workspace architecture and integration
airs-memspec context --workspace

# Example output:
# 🏢 AIRS Workspace Architecture
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 
# Vision
#   AI-assisted development workspace with unified tooling
# 
# Architecture
#   📦 airs-mcp       Core MCP protocol implementation
#   🔧 airs-memspec   Development tooling and memory bank management
#   🔄 Shared         Common types, utilities, testing frameworks
# 
# Integration Strategy
#   • Cargo workspace with shared dependencies
#   • Common development patterns and standards
#   • Unified documentation and testing approach
# 
# Current Phase
#   Foundation Development - Core Protocol + Tooling
```

#### Project-Specific Context

```bash
# Show specific project context
airs-memspec context --project backend-api

# Show context with additional verbosity
airs-memspec context --project frontend-app --verbose
```

### When to Use

- **Context switching**: Before switching between projects
- **Architecture review**: Understanding system design decisions
- **New team member onboarding**: Explaining current development state
- **Code review preparation**: Understanding architectural context
- **Planning sessions**: Review current constraints and focus areas

### Context Information Types

| Section | Contains | Use For |
|---------|----------|---------|
| Current Focus | Active development areas | Understanding immediate priorities |
| Active Work | Specific tasks in progress | Coordination and status updates |
| Integration Points | System interfaces and connections | Architecture and dependency analysis |
| Constraints | Technical and business limitations | Decision making and planning |
| Recent Changes | Latest modifications and decisions | Understanding recent evolution |

---

## `tasks` Command

Comprehensive task management with filtering, progress tracking, and detailed task information.

### Syntax

```bash
airs-memspec tasks <SUBCOMMAND> [OPTIONS]
```

### Subcommands

- `list` - List tasks with optional filtering

### `tasks list` Options

```bash
--project <PROJECT>      # Filter by specific project
--status <STATUS>        # Filter by task status
--priority <PRIORITY>    # Filter by priority level
--filter <FILTER>        # Apply predefined filters
```

### Status Values

- `pending` - Tasks not yet started
- `in_progress` - Currently active tasks
- `completed` - Finished tasks
- `blocked` - Tasks waiting on dependencies
- `abandoned` - Cancelled or obsolete tasks

### Priority Values

- `low` - Low priority tasks
- `medium` - Medium priority tasks (default)
- `high` - High priority tasks
- `critical` - Critical priority tasks

### Filter Values

- `active` - In progress tasks
- `recent` - Recently updated tasks
- `blocked` - Blocked tasks needing attention
- `stale` - Tasks not updated recently
- `all` - All tasks regardless of status

### Usage Examples

#### Basic Task Listing

```bash
# Smart default: shows 15 most relevant tasks
airs-memspec tasks list

# List all tasks (disable smart filtering)
airs-memspec tasks list --all

# Include completed tasks in smart view
airs-memspec tasks list --completed

# Example smart output:
# 📋 Task Overview
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 
# Active Tasks (2) - Always shown
# ├─ [PROJECT] TASK001 Current work item         � In Progress  85%
# └─ [PROJECT] TASK002 Another active item       � In Progress  60%
# 
# Pending Tasks (13) - From active project only
# ├─ [active-project] TASK003 Next priority      ⚪ Pending      0%
# └─ [active-project] TASK004 Following task     ⚪ Pending      0%
#
# 🧠 Smart filtering active: showing 15 most relevant tasks
# 📋 Focusing on active project: active-project
# 💡 Use --all to see all tasks or --status/--project for custom filtering
```

#### Status-Based Filtering

```bash
# Show only in-progress tasks
airs-memspec tasks list --status in_progress

# Show completed tasks
airs-memspec tasks list --status completed

# Show blocked tasks needing attention
airs-memspec tasks list --status blocked
```

#### Priority-Based Filtering

```bash
# Show high priority tasks
airs-memspec tasks list --priority high

# Show critical tasks requiring immediate attention
airs-memspec tasks list --priority critical

# Show all high and critical priority tasks
airs-memspec tasks list --priority high --priority critical
```

#### Project-Specific Tasks

```bash
# Show all tasks for specific project
airs-memspec tasks list --project backend-api

# Show active tasks for frontend project
airs-memspec tasks list --project frontend-app --status in_progress

# Show blocked tasks across all projects
airs-memspec tasks list --filter blocked
```

#### Advanced Filtering

```bash
# Recently updated tasks (last 7 days)
airs-memspec tasks list --filter recent

# Stale tasks (not updated in 30+ days)
airs-memspec tasks list --filter stale

# High priority active tasks
airs-memspec tasks list --priority high --status in_progress

# Project-specific blocked tasks
airs-memspec tasks list --project backend-api --filter blocked
```

#### Combined Filters

```bash
# Multiple criteria
airs-memspec tasks list --project frontend-app --priority high --status pending

# Comprehensive project review
airs-memspec tasks list --project backend-api --filter all --verbose
```

### When to Use

- **Sprint planning**: Review pending and blocked tasks
- **Daily standups**: Check active tasks and progress
- **Progress tracking**: Monitor completion percentages
- **Bottleneck identification**: Find blocked tasks
- **Project coordination**: Understand cross-project dependencies
- **Workload balancing**: Distribute tasks based on priority and status

### Task Information Format

```bash
# Standard task entry format:
[TASK###] Task Name                    Status Icon  Status Text   Progress%

# Example:
[TASK013] Integration testing framework    🟢       In Progress   85%

# Detailed view (with --verbose):
[TASK013] Integration testing framework
  Status: In Progress (85% complete)
  Priority: High
  Started: 2025-08-06
  Updated: 2025-08-08 (2 hours ago)
  
  Current Phase: Test implementation and validation
  
  Subtasks:
  ✅ 13.1 Framework setup and configuration
  ✅ 13.2 Basic test structure implementation  
  🔄 13.3 Advanced test scenarios
  ⏸️ 13.4 Performance test integration
  
  Next Actions:
  • Complete advanced test scenarios
  • Begin performance test integration
  • Validate error handling edge cases
```

### Output Formatting

#### Compact Mode (Default)

```bash
# Compact format for quick overview
airs-memspec tasks list --quiet

# Output:
# TASK013:in_progress:85%:high
# TASK014:in_progress:60%:medium
# TASK016:in_progress:40%:medium
```

#### Verbose Mode

```bash
# Detailed task information
airs-memspec tasks list --verbose

# Includes:
# - Full task descriptions
# - Subtask breakdowns
# - Progress details
# - Recent updates
# - Next actions
```

## Command Combinations

### Powerful Workflow Combinations

```bash
# Morning routine
airs-memspec status --workspace && airs-memspec context && airs-memspec tasks list --filter active

# Project health check
airs-memspec status --project myproject && airs-memspec tasks list --project myproject --filter blocked

# Sprint planning preparation
airs-memspec tasks list --priority high && airs-memspec tasks list --filter blocked

# End-of-day review
airs-memspec tasks list --filter recent && airs-memspec status --workspace --quiet
```

### Scripting Examples

```bash
#!/bin/bash
# Daily status report
echo "=== Daily Status Report ==="
echo "Date: $(date)"
echo ""

echo "=== Workspace Status ==="
airs-memspec status --workspace

echo ""
echo "=== Active Tasks ==="
airs-memspec tasks list --filter active

echo ""
echo "=== Blocked Items ==="
airs-memspec tasks list --filter blocked

echo ""
echo "=== High Priority ==="
airs-memspec tasks list --priority high
```

## Error Handling and Troubleshooting

### Common Error Scenarios

```bash
# No memory bank found
$ airs-memspec status
❌ Error: No memory bank structure found in current directory
💡 Solution: Run 'airs-memspec install' and create memory bank with GitHub Copilot

# Invalid project name
$ airs-memspec context --project invalid-name
❌ Error: Project 'invalid-name' not found
💡 Available projects: airs-mcp, airs-memspec
💡 Use: airs-memspec status to see all projects

# Permission issues
$ airs-memspec install --path /system/path
❌ Error: Permission denied
💡 Try: sudo airs-memspec install --path /system/path
💡 Or: Use a user-writable directory
```

### Validation Commands

```bash
# Verify command functionality
airs-memspec --version          # Check installation
airs-memspec --help            # Verify command access
airs-memspec status --quiet    # Test basic functionality
```

## Next Steps

Continue exploring:

- **[Integration Patterns](./integration.md)** - GitHub Copilot and IDE integration
- **[Advanced Scenarios](./advanced.md)** - Complex automation and workflows  
- **[Best Practices](./best-practices.md)** - Professional development recommendations
- **[Troubleshooting](./troubleshooting.md)** - Problem resolution and debugging
