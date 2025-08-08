# Essential Workflows

This guide covers the core daily workflows and usage patterns for airs-memspec in professional development environments.

## Daily Development Routine

### Morning Startup Workflow

```bash
# 1. Navigate to your workspace
cd /path/to/your/workspace

# 2. Check workspace health and status
airs-memspec status --workspace

# 3. Review what you were working on
airs-memspec context

# 4. Check active tasks across all projects
airs-memspec tasks list --filter active

# 5. Focus on specific project if needed
airs-memspec context --project your-current-project
```

**Expected Output Example:**
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

### Project Context Switching

```bash
# Check all available projects
airs-memspec status

# Switch focus to backend work
airs-memspec context --project backend-api

# Review backend-specific tasks
airs-memspec tasks list --project backend-api --status in_progress

# Switch to frontend work
airs-memspec context --project frontend-app

# Check frontend dependencies
airs-memspec tasks list --project frontend-app --filter blocked
```

### End-of-Day Review

```bash
# Review daily progress
airs-memspec tasks list --filter recent

# Check workspace status before shutdown
airs-memspec status --workspace

# Quick context check for tomorrow
airs-memspec context --workspace
```

## New Project Setup Workflow

### 🚀 Starting a New Workspace

```bash
# 1. Create and navigate to new workspace
mkdir my-new-workspace && cd my-new-workspace

# 2. Initialize Git (recommended)
git init

# 3. Install airs-memspec instructions
airs-memspec install --force

# 4. Verify installation
airs-memspec status

# Expected: "No memory bank structure found"
# This is normal for new workspaces
```

### 🏗️ Creating Memory Bank Structure

Use GitHub Copilot to create the memory bank:

```bash
# Open VS Code or your preferred editor
code .

# Chat with Copilot:
# "Create a memory bank structure for this workspace"
# "Set up multi-project memory bank for [describe your projects]"
```

### 📝 Validating New Setup

```bash
# Verify memory bank creation
airs-memspec status --workspace

# Check structure integrity
ls -la .copilot/memory_bank/

# Expected structure:
# ├── current_context.md
# ├── workspace/
# ├── sub_projects/
# └── context_snapshots/
```

## Multi-Project Development Patterns

### Pattern 1: Shared Library + Multiple Consumers

```bash
# Monitor shared library impact across consumers
airs-memspec status --workspace

# Check integration health
airs-memspec context --workspace

# Track dependency-related tasks
airs-memspec tasks list --filter blocked
airs-memspec tasks list --priority high
```

### Pattern 2: Microservices Architecture

```bash
# Service-by-service health check
for service in auth user payment notification; do
  echo "=== $service Service ==="
  airs-memspec context --project $service
  airs-memspec tasks list --project $service --status active
  echo ""
done

# Cross-service dependency tracking
airs-memspec tasks list --filter active | grep -E "(integration|api|service)"
```

### Pattern 3: Frontend + Backend Development

```bash
# Full-stack development workflow

# 1. Check backend status
airs-memspec context --project backend-api
airs-memspec tasks list --project backend-api --status in_progress

# 2. Check frontend status  
airs-memspec context --project frontend-app
airs-memspec tasks list --project frontend-app --status pending

# 3. Monitor integration points
airs-memspec tasks list --filter active | grep -i "integration"
```

## Task Management Workflows

### Sprint Planning Workflow

```bash
# 1. Review current task status across all projects
airs-memspec tasks list --status pending
airs-memspec tasks list --status in_progress

# 2. Check task priorities
airs-memspec tasks list --priority high
airs-memspec tasks list --priority medium

# 3. Identify blockers
airs-memspec tasks list --filter blocked

# 4. Plan next sprint tasks
airs-memspec tasks list --project target-project --status pending
```

### Progress Tracking Workflow

```bash
# Daily progress check
airs-memspec tasks list --filter recent

# Weekly progress review
airs-memspec tasks list --status completed --filter recent

# Monthly milestone review
airs-memspec status --workspace --verbose
```

### Task Prioritization Workflow

```bash
# Check high-priority tasks needing attention
airs-memspec tasks list --priority high --status pending

# Review blocked tasks that might be unblocked
airs-memspec tasks list --filter blocked

# Check overdue or stale tasks
airs-memspec tasks list --filter stale
```

## Context Analysis Workflows

### Architecture Review Workflow

```bash
# 1. Workspace-level architecture overview
airs-memspec context --workspace

# 2. Project-specific architectural focus
airs-memspec context --project core-library

# 3. Integration point analysis
airs-memspec context --workspace | grep -i "integration"
```

### Debugging and Investigation Workflow

```bash
# 1. Understand current system state
airs-memspec status --workspace --verbose

# 2. Check recent changes that might be related
airs-memspec tasks list --filter recent

# 3. Review current focus areas
airs-memspec context --project affected-project

# 4. Check for related blocked tasks
airs-memspec tasks list --project affected-project --filter blocked
```

### Code Review Preparation Workflow

```bash
# 1. Check what's currently in progress
airs-memspec context --project review-target

# 2. Review related tasks and context
airs-memspec tasks list --project review-target --status in_progress

# 3. Understand architectural constraints
airs-memspec context --project review-target | grep -i "constraint\|pattern\|decision"
```

## Automation and Scripting Workflows

### Health Check Script

```bash
#!/bin/bash
# daily-health-check.sh

echo "=== Daily Workspace Health Check ==="
echo "Date: $(date)"
echo ""

# Workspace overview
airs-memspec status --workspace

echo ""
echo "=== Active Tasks Requiring Attention ==="
airs-memspec tasks list --priority high --status pending
airs-memspec tasks list --filter blocked

echo ""
echo "=== Recent Progress ==="
airs-memspec tasks list --filter recent --status completed
```

### Context Switching Script

```bash
#!/bin/bash
# switch-project.sh <project-name>

PROJECT_NAME="$1"

if [ -z "$PROJECT_NAME" ]; then
    echo "Usage: switch-project.sh <project-name>"
    echo "Available projects:"
    airs-memspec status | grep "├─\|└─" | cut -d' ' -f2
    exit 1
fi

echo "=== Switching to $PROJECT_NAME ==="
airs-memspec context --project "$PROJECT_NAME"

echo ""
echo "=== Active Tasks ==="
airs-memspec tasks list --project "$PROJECT_NAME" --filter active
```

### Weekly Report Script

```bash
#!/bin/bash
# weekly-report.sh

echo "=== Weekly Progress Report ==="
echo "Week ending: $(date)"
echo ""

# Get all project names
PROJECTS=$(airs-memspec status --workspace --quiet | grep "📁" | cut -d' ' -f2)

for project in $PROJECTS; do
    echo "=== $project ==="
    echo "Completed this week:"
    airs-memspec tasks list --project "$project" --status completed --filter recent
    
    echo "In progress:"
    airs-memspec tasks list --project "$project" --status in_progress
    
    echo "Blocked items:"
    airs-memspec tasks list --project "$project" --filter blocked
    echo ""
done
```

## Integration with Development Tools

### Git Integration

```bash
# Pre-commit hook - check workspace health
#!/bin/bash
# .git/hooks/pre-commit

echo "Checking workspace health before commit..."
airs-memspec status --workspace --quiet

if [ $? -ne 0 ]; then
    echo "❌ Workspace health check failed"
    exit 1
fi

echo "✅ Workspace health check passed"
```

### IDE Integration

**VS Code Tasks Configuration:**
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
    },
    {
      "label": "Current Context",
      "type": "shell",
      "command": "airs-memspec",
      "args": ["context"],
      "group": "test",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "Active Tasks",
      "type": "shell",
      "command": "airs-memspec",
      "args": ["tasks", "list", "--filter", "active"],
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

### Shell Aliases

Add to your `.bashrc`, `.zshrc`, or similar:

```bash
# airs-memspec aliases
alias mbs="airs-memspec status"
alias mbsw="airs-memspec status --workspace"
alias mbc="airs-memspec context"
alias mbcw="airs-memspec context --workspace"
alias mbt="airs-memspec tasks list --filter active"
alias mbta="airs-memspec tasks list"

# Combined aliases
alias mb-start="mbs && mbc"  # Morning routine
alias mb-status="mbsw && mbt"  # Quick overview
```

## Next Steps

After mastering these essential workflows:

1. **Explore Commands**: Deep dive into [Command Reference](./commands.md)
2. **Advanced Usage**: Learn [Advanced Scenarios](./advanced.md)
3. **Integration**: Set up [Integration Patterns](./integration.md)
4. **Best Practices**: Adopt [Best Practices](./best-practices.md)
