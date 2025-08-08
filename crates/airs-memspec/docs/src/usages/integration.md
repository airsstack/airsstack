# Integration Patterns

Comprehensive guide to integrating airs-memspec with GitHub Copilot, IDEs, and development workflows for enhanced AI-assisted development.

## GitHub Copilot Integration

### Custom Instructions Setup

The core integration relies on GitHub Copilot custom instructions that enable AI memory persistence and workspace awareness.

#### Automatic Deployment

```bash
# Deploy instructions to default location (.copilot)
airs-memspec install

# Expected result:
# ✅ Instructions installed to .copilot/instructions/
# 📁 Created: multi_project_memory_bank.instructions.md
```

#### VS Code Configuration

Configure VS Code to use custom instructions:

1. **Open VS Code Settings** (`Cmd+,` on macOS, `Ctrl+,` on Windows/Linux)
2. **Search for**: `github.copilot.customInstructions`
3. **Set the path**: `.copilot/instructions/`

Alternatively, edit `settings.json` directly:

```json
{
  "github.copilot.customInstructions": ".copilot/instructions/"
}
```

#### Global Configuration (Optional)

For workspace-independent instructions:

```bash
# macOS
airs-memspec install --path ~/Library/Application\ Support/Code/User/copilot

# Linux  
airs-memspec install --path ~/.config/Code/User/copilot

# Windows
airs-memspec install --path %APPDATA%\Code\User\copilot
```

### AI Memory Bank Commands

Once instructions are installed, GitHub Copilot responds to specific memory bank commands:

#### Core Commands

```markdown
**update memory bank** - Triggers comprehensive memory bank review and updates
**add task [project_name] [task_description]** - Creates new tracked task
**show tasks [filter]** - Displays filtered task list
**switch context [project_name]** - Changes active project context
**save context [description]** - Creates context snapshot
```

#### Example Workflow

```bash
# In GitHub Copilot chat:
User: "update memory bank"
Copilot: [Reviews all memory bank files, updates current state]

User: "add task backend-api implement user authentication"  
Copilot: [Creates task file, updates index, documents approach]

User: "show tasks active"
Copilot: [Displays currently active tasks with progress]

User: "switch context frontend-app"
Copilot: [Updates current_context.md, loads frontend context]
```

## IDE Integration Patterns

### Visual Studio Code

#### Recommended Extensions

Essential extensions for optimal airs-memspec workflow:

```bash
# Install via VS Code marketplace or command line:
code --install-extension GitHub.copilot
code --install-extension GitHub.copilot-chat  
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
```

#### Workspace Settings

Create `.vscode/settings.json` for project-specific configuration:

```json
{
  "github.copilot.customInstructions": ".copilot/instructions/",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "files.associations": {
    "*.instructions.md": "markdown"
  },
  "markdownlint.config": {
    "MD013": { "line_length": 120 },
    "MD033": false
  }
}
```

#### Task Integration

VS Code tasks for common workflows:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "memspec-status",
      "type": "shell", 
      "command": "airs-memspec",
      "args": ["status", "--workspace"],
      "group": "build",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "memspec-tasks",
      "type": "shell",
      "command": "airs-memspec", 
      "args": ["tasks", "list", "--filter", "active"],
      "group": "build"
    },
    {
      "label": "memspec-context",
      "type": "shell",
      "command": "airs-memspec",
      "args": ["context"],
      "group": "build"
    }
  ]
}
```

#### Keybinding Integration

Add custom keybindings in `.vscode/keybindings.json`:

```json
[
  {
    "key": "ctrl+shift+m s",
    "command": "workbench.action.tasks.runTask",
    "args": "memspec-status"
  },
  {
    "key": "ctrl+shift+m t", 
    "command": "workbench.action.tasks.runTask",
    "args": "memspec-tasks"
  },
  {
    "key": "ctrl+shift+m c",
    "command": "workbench.action.tasks.runTask", 
    "args": "memspec-context"
  }
]
```

### JetBrains IDEs

#### IntelliJ IDEA / CLion / RustRover

Create external tools for airs-memspec commands:

1. **Settings** → **Tools** → **External Tools**
2. **Add new tool** with these configurations:

**Status Tool:**
- Name: `MemSpec Status`
- Program: `airs-memspec`
- Arguments: `status --workspace`
- Working directory: `$ProjectFileDir$`

**Tasks Tool:**
- Name: `MemSpec Tasks`
- Program: `airs-memspec`
- Arguments: `tasks list --filter active`
- Working directory: `$ProjectFileDir$`

**Context Tool:**
- Name: `MemSpec Context`
- Program: `airs-memspec`
- Arguments: `context`
- Working directory: `$ProjectFileDir$`

#### Custom Macros

Create IDE macros for rapid access:

```kotlin
// IntelliJ IDEA macro example
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent

class MemSpecStatusAction : AnAction("MemSpec Status") {
    override fun actionPerformed(e: AnActionEvent) {
        // Execute airs-memspec status and show in tool window
    }
}
```

### Neovim Integration

#### Lua Configuration

```lua
-- ~/.config/nvim/lua/memspec.lua
local M = {}

function M.status()
  vim.cmd('split | terminal airs-memspec status --workspace')
end

function M.tasks()
  vim.cmd('split | terminal airs-memspec tasks list --filter active')
end

function M.context()
  vim.cmd('split | terminal airs-memspec context')
end

-- Keybindings
vim.keymap.set('n', '<leader>ms', M.status, { desc = 'MemSpec Status' })
vim.keymap.set('n', '<leader>mt', M.tasks, { desc = 'MemSpec Tasks' })
vim.keymap.set('n', '<leader>mc', M.context, { desc = 'MemSpec Context' })

return M
```

#### Plugin Integration

```lua
-- Add to ~/.config/nvim/init.lua
require('memspec')

-- Optional: Auto-run status on startup
vim.api.nvim_create_autocmd("VimEnter", {
  callback = function()
    vim.cmd('echo "airs-memspec workspace loaded"')
  end
})
```

## Continuous Integration Patterns

### GitHub Actions

#### Workspace Health Check

`.github/workflows/memspec-health.yml`:

```yaml
name: MemSpec Health Check

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install airs-memspec
      run: cargo install --path crates/airs-memspec
      
    - name: Workspace Status Check
      run: |
        airs-memspec status --workspace --quiet
        echo "STATUS_CODE=$?" >> $GITHUB_ENV
        
    - name: Task Overview
      run: airs-memspec tasks list --filter blocked --quiet
      
    - name: Generate Report
      run: |
        echo "## MemSpec Health Report" >> $GITHUB_STEP_SUMMARY
        echo "" >> $GITHUB_STEP_SUMMARY
        echo "### Workspace Status" >> $GITHUB_STEP_SUMMARY
        airs-memspec status --workspace >> $GITHUB_STEP_SUMMARY
        echo "" >> $GITHUB_STEP_SUMMARY
        echo "### Active Tasks" >> $GITHUB_STEP_SUMMARY
        airs-memspec tasks list --filter active >> $GITHUB_STEP_SUMMARY
        
    - name: Check for Blocked Tasks
      run: |
        BLOCKED_COUNT=$(airs-memspec tasks list --filter blocked --quiet | wc -l)
        if [ "$BLOCKED_COUNT" -gt 0 ]; then
          echo "⚠️ Found $BLOCKED_COUNT blocked tasks"
          airs-memspec tasks list --filter blocked
          exit 1
        fi
```

#### Documentation Validation

`.github/workflows/memspec-docs.yml`:

```yaml
name: MemSpec Documentation

on:
  push:
    paths:
    - '.copilot/memory_bank/**'
    - 'docs/**'

jobs:
  validate-memory-bank:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install airs-memspec
      run: cargo install --path crates/airs-memspec
      
    - name: Validate Memory Bank Structure
      run: |
        # Check for required files
        if [ ! -f ".copilot/memory_bank/current_context.md" ]; then
          echo "❌ Missing current_context.md"
          exit 1
        fi
        
        # Validate workspace files
        for file in workspace/project_brief.md workspace/shared_patterns.md; do
          if [ ! -f ".copilot/memory_bank/$file" ]; then
            echo "❌ Missing workspace file: $file"
            exit 1
          fi
        done
        
        echo "✅ Memory bank structure validated"
        
    - name: Check Memory Bank Health
      run: |
        airs-memspec context --workspace
        airs-memspec status --workspace
```

### GitLab CI

#### Pipeline Configuration

`.gitlab-ci.yml`:

```yaml
stages:
  - validate
  - health-check
  - report

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  
before_script:
  - cargo install --path crates/airs-memspec

memspec-validate:
  stage: validate
  script:
    - airs-memspec status --workspace --quiet
    - airs-memspec tasks list --filter blocked --quiet
  artifacts:
    reports:
      junit: memspec-report.xml
  only:
    - main
    - merge_requests

memspec-health:
  stage: health-check  
  script:
    - |
      echo "Memory Bank Health Check" > health-report.txt
      airs-memspec status --workspace >> health-report.txt
      airs-memspec tasks list --filter active >> health-report.txt
  artifacts:
    paths:
      - health-report.txt
    expire_in: 1 week

memspec-report:
  stage: report
  script:
    - |
      echo "## MemSpec Report for $CI_COMMIT_SHORT_SHA" > memspec-report.md
      echo "" >> memspec-report.md
      airs-memspec status --workspace >> memspec-report.md
      airs-memspec context --workspace >> memspec-report.md
  artifacts:
    paths:
      - memspec-report.md
  only:
    - main
```

## Shell Integration

### Bash Integration

Add to `~/.bashrc` or `~/.bash_profile`:

```bash
# airs-memspec shell integration
export MEMSPEC_DEFAULT_PATH="$PWD"

# Aliases for quick access
alias ms='airs-memspec'
alias mstat='airs-memspec status'
alias mtasks='airs-memspec tasks list'
alias mcontext='airs-memspec context'

# Function for quick workspace switching
memspec_switch() {
  if [ -z "$1" ]; then
    echo "Usage: memspec_switch <workspace_path>"
    return 1
  fi
  
  cd "$1"
  airs-memspec status --workspace
}

# Auto-completion (if available)
if command -v airs-memspec >/dev/null 2>&1; then
  # Enable bash completion
  complete -C airs-memspec airs-memspec
fi

# Workspace health check on directory change
cd() {
  builtin cd "$@"
  if [ -f ".copilot/memory_bank/current_context.md" ]; then
    echo "📋 MemSpec workspace detected"
    airs-memspec status --quiet
  fi
}
```

### Zsh Integration

Add to `~/.zshrc`:

```bash
# airs-memspec zsh integration  
export MEMSPEC_DEFAULT_PATH="$PWD"

# Aliases
alias ms='airs-memspec'
alias mstat='airs-memspec status'
alias mtasks='airs-memspec tasks list'
alias mcontext='airs-memspec context'

# Enhanced prompt with memspec info
precmd() {
  if [ -f ".copilot/memory_bank/current_context.md" ]; then
    # Add memory bank indicator to prompt
    MEMSPEC_INDICATOR="%F{green}📋%f"
  else
    MEMSPEC_INDICATOR=""
  fi
}

# Update prompt to include memspec indicator
PROMPT="${MEMSPEC_INDICATOR} ${PROMPT}"

# Directory change hook
chpwd() {
  if [ -f ".copilot/memory_bank/current_context.md" ]; then
    echo "📋 MemSpec workspace: $(basename $PWD)"
    airs-memspec status --quiet
  fi
}
```

### Fish Shell Integration

Add to `~/.config/fish/config.fish`:

```fish
# airs-memspec fish integration
set -x MEMSPEC_DEFAULT_PATH $PWD

# Aliases
alias ms 'airs-memspec'
alias mstat 'airs-memspec status'
alias mtasks 'airs-memspec tasks list'
alias mcontext 'airs-memspec context'

# Function for workspace switching
function memspec_switch
    if test (count $argv) -eq 0
        echo "Usage: memspec_switch <workspace_path>"
        return 1
    end
    
    cd $argv[1]
    airs-memspec status --workspace
end

# Auto-detection on directory change
function __memspec_auto_detect --on-variable PWD
    if test -f ".copilot/memory_bank/current_context.md"
        echo "📋 MemSpec workspace: "(basename $PWD)
        airs-memspec status --quiet
    end
end
```

## Development Workflow Integration

### Pre-commit Hooks

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: memspec-health
        name: MemSpec Health Check
        entry: airs-memspec
        args: [status, --workspace, --quiet]
        language: system
        pass_filenames: false
        
      - id: memspec-blocked-tasks
        name: Check for Blocked Tasks
        entry: bash
        args: [-c, 'blocked=$(airs-memspec tasks list --filter blocked --quiet | wc -l); if [ $blocked -gt 0 ]; then echo "❌ $blocked blocked tasks found"; exit 1; fi']
        language: system
        pass_filenames: false
```

### Make Integration

`Makefile`:

```makefile
# airs-memspec integration
.PHONY: memspec-status memspec-tasks memspec-context memspec-health

memspec-status:
	@echo "📊 Workspace Status"
	@airs-memspec status --workspace

memspec-tasks:
	@echo "📋 Active Tasks"  
	@airs-memspec tasks list --filter active

memspec-context:
	@echo "🎯 Current Context"
	@airs-memspec context

memspec-health: memspec-status memspec-tasks
	@echo ""
	@echo "🔍 Health Check Complete"
	@blocked=$$(airs-memspec tasks list --filter blocked --quiet | wc -l); \
	if [ $$blocked -gt 0 ]; then \
		echo "⚠️  Warning: $$blocked blocked tasks found"; \
		airs-memspec tasks list --filter blocked; \
	else \
		echo "✅ No blocked tasks"; \
	fi

# Integration with existing targets
build: memspec-health
	@echo "Building after health check..."
	cargo build

test: memspec-health  
	@echo "Testing after health check..."
	cargo test

deploy: memspec-health
	@echo "Deploying after health check..."
	# deployment commands here
```

### Just Integration

`justfile`:

```just
# airs-memspec integration for just command runner

# Show workspace status
status:
    @echo "📊 Workspace Status"
    airs-memspec status --workspace

# Show active tasks
tasks:
    @echo "📋 Active Tasks"
    airs-memspec tasks list --filter active

# Show current context  
context:
    @echo "🎯 Current Context"
    airs-memspec context

# Full health check
health: status tasks
    @echo ""
    @echo "🔍 Health Check Complete"
    @blocked=$(airs-memspec tasks list --filter blocked --quiet | wc -l); \
    if [ $blocked -gt 0 ]; then \
        echo "⚠️  Warning: $blocked blocked tasks found"; \
        airs-memspec tasks list --filter blocked; \
    else \
        echo "✅ No blocked tasks"; \
    fi

# Run health check before build
build: health
    @echo "Building after health check..."
    cargo build --workspace

# Run health check before tests
test: health
    @echo "Testing after health check..."  
    cargo test --workspace
```

## API Integration Examples

### REST API Monitoring

For web applications, integrate memspec status into health endpoints:

```rust
// Example Axum integration
use axum::{Json, response::Json as ResponseJson};
use serde_json::{json, Value};
use std::process::Command;

async fn health_check() -> ResponseJson<Value> {
    let memspec_status = Command::new("airs-memspec")
        .args(&["status", "--workspace", "--quiet"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    ResponseJson(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "memspec": {
            "workspace_status": memspec_status,
            "health": "green"
        }
    }))
}
```

### Slack Integration

```bash
#!/bin/bash
# slack-memspec-notify.sh - Send daily status to Slack

WEBHOOK_URL="your_slack_webhook_url"

STATUS=$(airs-memspec status --workspace)
ACTIVE_TASKS=$(airs-memspec tasks list --filter active)
BLOCKED_TASKS=$(airs-memspec tasks list --filter blocked)

BLOCKED_COUNT=$(echo "$BLOCKED_TASKS" | wc -l)

if [ "$BLOCKED_COUNT" -gt 0 ]; then
    COLOR="warning"
    PRETEXT="⚠️ Workspace has $BLOCKED_COUNT blocked tasks"
else
    COLOR="good"  
    PRETEXT="✅ Workspace health: All systems operational"
fi

curl -X POST -H 'Content-type: application/json' \
    --data "{
        \"attachments\": [{
            \"color\": \"$COLOR\",
            \"pretext\": \"$PRETEXT\",
            \"fields\": [
                {
                    \"title\": \"Workspace Status\",
                    \"value\": \"$STATUS\",
                    \"short\": false
                },
                {
                    \"title\": \"Active Tasks\", 
                    \"value\": \"$ACTIVE_TASKS\",
                    \"short\": false
                }
            ]
        }]
    }" \
    $WEBHOOK_URL
```

## Best Practices for Integration

### 1. **Gradual Adoption**
- Start with manual commands
- Add shell aliases
- Integrate with existing workflows
- Automate common patterns

### 2. **Error Handling**
- Always check command exit codes
- Provide fallback behavior
- Log integration failures
- Graceful degradation

### 3. **Performance Considerations**
- Cache status results when appropriate
- Use `--quiet` mode for scripts
- Avoid excessive automation
- Monitor integration overhead

### 4. **Security**
- Protect memory bank data
- Secure CI/CD integration
- Validate input parameters
- Limit automation scope

### 5. **Team Coordination**
- Document integration patterns
- Share successful configurations
- Standardize across projects
- Regular integration reviews

## Next Steps

Continue exploring:

- **[Advanced Scenarios](./advanced.md)** - Complex automation and multi-environment workflows
- **[Best Practices](./best-practices.md)** - Professional development recommendations  
- **[Troubleshooting](./troubleshooting.md)** - Problem resolution and debugging

---

*Integration patterns enable seamless airs-memspec workflows across tools and environments, maximizing development efficiency and maintaining workspace awareness.*
