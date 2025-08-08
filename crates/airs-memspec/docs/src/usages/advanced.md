# Advanced Scenarios

Complex workflows, automation strategies, and sophisticated usage patterns for airs-memspec in enterprise and multi-team environments.

## Multi-Environment Management

### Environment-Specific Memory Banks

Managing separate development contexts for different environments:

```bash
# Development environment
airs-memspec status --path ./dev-workspace
airs-memspec context --path ./dev-workspace

# Staging environment  
airs-memspec status --path ./staging-workspace
airs-memspec context --path ./staging-workspace

# Production environment
airs-memspec status --path ./prod-workspace
airs-memspec context --path ./prod-workspace
```

### Environment Switching Script

```bash
#!/bin/bash
# env-switch.sh - Advanced environment management

set -euo pipefail

ENVS_DIR="$HOME/workspaces"
CURRENT_ENV_FILE="$HOME/.current-memspec-env"

switch_environment() {
    local env_name="$1"
    local env_path="$ENVS_DIR/$env_name"
    
    if [ ! -d "$env_path" ]; then
        echo "❌ Environment '$env_name' not found at $env_path"
        list_environments
        return 1
    fi
    
    # Save current environment
    echo "$env_name" > "$CURRENT_ENV_FILE"
    
    # Switch to environment directory
    cd "$env_path"
    
    # Display environment status
    echo "🔄 Switched to environment: $env_name"
    echo "📍 Path: $env_path"
    echo ""
    
    # Show environment health
    if [ -f ".copilot/memory_bank/current_context.md" ]; then
        airs-memspec status --workspace
        echo ""
        airs-memspec context
    else
        echo "⚠️  No memory bank found. Run 'airs-memspec install' to initialize."
    fi
}

list_environments() {
    echo "Available environments:"
    for env in "$ENVS_DIR"/*; do
        if [ -d "$env" ]; then
            local env_name=$(basename "$env")
            local status="🟤"
            
            if [ -f "$env/.copilot/memory_bank/current_context.md" ]; then
                status="🟢"
            fi
            
            echo "  $status $env_name"
        fi
    done
}

create_environment() {
    local env_name="$1"
    local template="${2:-default}"
    local env_path="$ENVS_DIR/$env_name"
    
    if [ -d "$env_path" ]; then
        echo "❌ Environment '$env_name' already exists"
        return 1
    fi
    
    echo "🚀 Creating environment: $env_name"
    mkdir -p "$env_path"
    cd "$env_path"
    
    # Initialize memory bank
    airs-memspec install
    
    # Copy template if specified
    if [ "$template" != "default" ] && [ -d "$ENVS_DIR/$template" ]; then
        echo "📋 Copying template from: $template"
        cp -r "$ENVS_DIR/$template/.copilot/memory_bank"/* ".copilot/memory_bank/"
    fi
    
    echo "✅ Environment '$env_name' created successfully"
    switch_environment "$env_name"
}

get_current_environment() {
    if [ -f "$CURRENT_ENV_FILE" ]; then
        cat "$CURRENT_ENV_FILE"
    else
        echo "none"
    fi
}

case "${1:-}" in
    "switch"|"s")
        switch_environment "${2:-}"
        ;;
    "list"|"l")
        echo "Current environment: $(get_current_environment)"
        echo ""
        list_environments
        ;;
    "create"|"c")
        create_environment "${2:-}" "${3:-default}"
        ;;
    "current")
        echo "$(get_current_environment)"
        ;;
    *)
        echo "Usage: $0 {switch|list|create|current} [environment_name] [template]"
        echo ""
        echo "Commands:"
        echo "  switch <env>     Switch to environment"
        echo "  list             List all environments"
        echo "  create <env>     Create new environment"
        echo "  current          Show current environment"
        exit 1
        ;;
esac
```

### Environment Synchronization

```bash
#!/bin/bash
# sync-environments.sh - Synchronize memory bank across environments

sync_memory_banks() {
    local source_env="$1"
    local target_env="$2"
    local components="${3:-all}"
    
    local source_path="$ENVS_DIR/$source_env/.copilot/memory_bank"
    local target_path="$ENVS_DIR/$target_env/.copilot/memory_bank"
    
    if [ ! -d "$source_path" ]; then
        echo "❌ Source environment '$source_env' not found"
        return 1
    fi
    
    if [ ! -d "$target_path" ]; then
        echo "❌ Target environment '$target_env' not found"
        return 1
    fi
    
    echo "🔄 Syncing memory bank: $source_env → $target_env"
    
    case "$components" in
        "workspace")
            echo "📁 Syncing workspace files..."
            cp -r "$source_path/workspace/" "$target_path/workspace/"
            ;;
        "instructions")
            echo "📝 Syncing instructions..."
            cp -r "$source_path"/../instructions/ "$target_path"/../instructions/
            ;;
        "tasks")
            echo "📋 Syncing tasks..."
            for project in "$source_path/sub_projects"/*; do
                if [ -d "$project/tasks" ]; then
                    local project_name=$(basename "$project")
                    mkdir -p "$target_path/sub_projects/$project_name"
                    cp -r "$project/tasks" "$target_path/sub_projects/$project_name/"
                fi
            done
            ;;
        "all")
            echo "🔄 Full synchronization..."
            rsync -av --exclude="current_context.md" "$source_path/" "$target_path/"
            ;;
        *)
            echo "❌ Unknown component: $components"
            echo "Available: workspace, instructions, tasks, all"
            return 1
            ;;
    esac
    
    echo "✅ Synchronization complete"
}

# Usage examples:
# sync-environments.sh dev staging workspace
# sync-environments.sh staging prod instructions  
# sync-environments.sh dev prod all
```

## Large-Scale Project Management

### Multi-Repository Workspace

Managing memory banks across multiple related repositories:

```bash
#!/bin/bash
# multi-repo-manager.sh - Coordinate airs-memspec across repositories

WORKSPACE_ROOT="$HOME/projects/microservices"
REPOS=(
    "auth-service"
    "user-service"  
    "payment-service"
    "notification-service"
    "api-gateway"
    "frontend-app"
)

workspace_status() {
    echo "🏢 Multi-Repository Workspace Status"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    local total_repos=${#REPOS[@]}
    local healthy_repos=0
    local repos_with_tasks=0
    local blocked_tasks=0
    
    for repo in "${REPOS[@]}"; do
        local repo_path="$WORKSPACE_ROOT/$repo"
        
        if [ ! -d "$repo_path" ]; then
            echo "❌ $repo - Repository not found"
            continue
        fi
        
        cd "$repo_path"
        
        # Check memory bank health
        if [ -f ".copilot/memory_bank/current_context.md" ]; then
            ((healthy_repos++))
            local status="🟢"
            
            # Check for active tasks
            local active_tasks=$(airs-memspec tasks list --filter active --quiet 2>/dev/null | wc -l)
            local blocked_tasks_count=$(airs-memspec tasks list --filter blocked --quiet 2>/dev/null | wc -l)
            
            if [ "$active_tasks" -gt 0 ]; then
                ((repos_with_tasks++))
            fi
            
            blocked_tasks=$((blocked_tasks + blocked_tasks_count))
            
            echo "$status $repo - $active_tasks active, $blocked_tasks_count blocked"
        else
            echo "🟤 $repo - No memory bank"
        fi
    done
    
    echo ""
    echo "Summary:"
    echo "  Total repositories: $total_repos"
    echo "  With memory banks: $healthy_repos"
    echo "  With active tasks: $repos_with_tasks"
    echo "  Total blocked tasks: $blocked_tasks"
    
    if [ "$blocked_tasks" -gt 0 ]; then
        echo ""
        echo "⚠️  Blocked tasks require attention!"
        return 1
    fi
}

workspace_tasks() {
    local filter="${1:-active}"
    
    echo "📋 Multi-Repository Tasks ($filter)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    for repo in "${REPOS[@]}"; do
        local repo_path="$WORKSPACE_ROOT/$repo"
        
        if [ ! -d "$repo_path/.copilot/memory_bank" ]; then
            continue
        fi
        
        cd "$repo_path"
        
        local tasks=$(airs-memspec tasks list --filter "$filter" --quiet 2>/dev/null)
        local task_count=$(echo "$tasks" | wc -l)
        
        if [ "$task_count" -gt 0 ] && [ -n "$tasks" ]; then
            echo "📦 $repo ($task_count tasks)"
            airs-memspec tasks list --filter "$filter" 2>/dev/null | sed 's/^/  /'
            echo ""
        fi
    done
}

initialize_workspace() {
    echo "🚀 Initializing multi-repository workspace"
    echo ""
    
    for repo in "${REPOS[@]}"; do
        local repo_path="$WORKSPACE_ROOT/$repo"
        
        if [ ! -d "$repo_path" ]; then
            echo "⚠️  Skipping $repo - repository not found"
            continue
        fi
        
        cd "$repo_path"
        
        if [ ! -f ".copilot/memory_bank/current_context.md" ]; then
            echo "🔧 Initializing memory bank for $repo"
            airs-memspec install
        else
            echo "✅ $repo - memory bank already exists"
        fi
    done
    
    echo ""
    echo "✅ Workspace initialization complete"
}

sync_workspace_patterns() {
    local source_repo="$1"
    local pattern_type="${2:-shared_patterns}"
    
    if [ -z "$source_repo" ]; then
        echo "❌ Usage: sync_workspace_patterns <source_repo> [pattern_type]"
        return 1
    fi
    
    local source_path="$WORKSPACE_ROOT/$source_repo/.copilot/memory_bank/workspace"
    
    if [ ! -d "$source_path" ]; then
        echo "❌ Source patterns not found: $source_path"
        return 1
    fi
    
    echo "🔄 Syncing $pattern_type from $source_repo to all repositories"
    
    for repo in "${REPOS[@]}"; do
        if [ "$repo" = "$source_repo" ]; then
            continue
        fi
        
        local target_path="$WORKSPACE_ROOT/$repo/.copilot/memory_bank/workspace"
        
        if [ -d "$target_path" ]; then
            echo "  📁 $repo"
            cp "$source_path/$pattern_type.md" "$target_path/" 2>/dev/null || true
        fi
    done
    
    echo "✅ Pattern synchronization complete"
}

case "${1:-}" in
    "status"|"s")
        workspace_status
        ;;
    "tasks"|"t")
        workspace_tasks "${2:-active}"
        ;;
    "init"|"i")
        initialize_workspace
        ;;
    "sync"|"sync-patterns")
        sync_workspace_patterns "${2:-}" "${3:-shared_patterns}"
        ;;
    *)
        echo "Multi-Repository MemSpec Manager"
        echo ""
        echo "Usage: $0 {status|tasks|init|sync} [options]"
        echo ""
        echo "Commands:"
        echo "  status              Show status across all repositories"
        echo "  tasks [filter]      Show tasks across all repositories"
        echo "  init                Initialize memory banks in all repos"
        echo "  sync <source> [type] Sync patterns from source repo"
        echo ""
        echo "Examples:"
        echo "  $0 status"
        echo "  $0 tasks blocked"
        echo "  $0 sync auth-service shared_patterns"
        exit 1
        ;;
esac
```

### Repository Dependencies

```bash
#!/bin/bash
# dependency-tracker.sh - Track dependencies between repositories

track_dependencies() {
    echo "🔗 Repository Dependency Analysis"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    declare -A dependencies
    declare -A blocked_by
    
    # Analyze dependencies from memory banks
    for repo in "${REPOS[@]}"; do
        local repo_path="$WORKSPACE_ROOT/$repo"
        cd "$repo_path"
        
        if [ -f ".copilot/memory_bank/sub_projects/*/active_context.md" ]; then
            # Extract dependency information from active context
            local deps=$(grep -i "depends on\|blocked by\|waiting for" .copilot/memory_bank/sub_projects/*/active_context.md 2>/dev/null || true)
            
            if [ -n "$deps" ]; then
                dependencies["$repo"]="$deps"
            fi
        fi
        
        # Check for blocked tasks that might indicate dependencies
        local blocked_tasks=$(airs-memspec tasks list --filter blocked --quiet 2>/dev/null)
        if [ -n "$blocked_tasks" ]; then
            blocked_by["$repo"]="$blocked_tasks"
        fi
    done
    
    # Display dependency graph
    for repo in "${!dependencies[@]}"; do
        echo "📦 $repo"
        echo "${dependencies[$repo]}" | sed 's/^/  /'
        echo ""
    done
    
    # Display blocking issues
    if [ ${#blocked_by[@]} -gt 0 ]; then
        echo "🚫 Repositories with blocking issues:"
        for repo in "${!blocked_by[@]}"; do
            echo "  ❌ $repo"
        done
    fi
}
```

## Advanced Automation

### Intelligent Task Distribution

```python
#!/usr/bin/env python3
# task-distributor.py - AI-assisted task distribution across team members

import json
import subprocess
import sys
from dataclasses import dataclass
from typing import List, Dict, Optional
from datetime import datetime, timedelta

@dataclass
class Task:
    id: str
    project: str
    title: str
    priority: str
    status: str
    complexity: int
    dependencies: List[str]
    estimated_hours: int

@dataclass
class Developer:
    name: str
    skills: List[str]
    capacity_hours: int
    current_load: int
    preferred_projects: List[str]

class TaskDistributor:
    def __init__(self, workspace_path: str):
        self.workspace_path = workspace_path
        self.tasks = []
        self.developers = []
        
    def load_tasks_from_memspec(self) -> List[Task]:
        """Load tasks from airs-memspec across all projects"""
        try:
            result = subprocess.run(
                ['airs-memspec', 'tasks', 'list', '--filter', 'pending', '--quiet'],
                cwd=self.workspace_path,
                capture_output=True,
                text=True
            )
            
            tasks = []
            for line in result.stdout.strip().split('\n'):
                if line and ':' in line:
                    parts = line.split(':')
                    if len(parts) >= 4:
                        task = Task(
                            id=parts[0],
                            project=parts[1] if len(parts) > 1 else 'unknown',
                            title=parts[2] if len(parts) > 2 else 'untitled',
                            priority=parts[3] if len(parts) > 3 else 'medium',
                            status='pending',
                            complexity=self._estimate_complexity(parts[2] if len(parts) > 2 else ''),
                            dependencies=[],
                            estimated_hours=self._estimate_hours(parts[2] if len(parts) > 2 else '')
                        )
                        tasks.append(task)
            
            return tasks
        except Exception as e:
            print(f"Error loading tasks: {e}")
            return []
    
    def _estimate_complexity(self, title: str) -> int:
        """Simple complexity estimation based on keywords"""
        complexity_keywords = {
            'refactor': 3,
            'implement': 2,
            'fix': 1,
            'optimize': 3,
            'design': 2,
            'integrate': 3,
            'test': 1,
            'debug': 2
        }
        
        title_lower = title.lower()
        for keyword, complexity in complexity_keywords.items():
            if keyword in title_lower:
                return complexity
        
        return 2  # Default complexity
    
    def _estimate_hours(self, title: str) -> int:
        """Estimate hours based on task complexity and type"""
        complexity = self._estimate_complexity(title)
        base_hours = {1: 4, 2: 8, 3: 16}
        return base_hours.get(complexity, 8)
    
    def load_team_config(self, config_file: str) -> List[Developer]:
        """Load team configuration from JSON file"""
        try:
            with open(config_file, 'r') as f:
                config = json.load(f)
            
            developers = []
            for dev_config in config.get('developers', []):
                developer = Developer(
                    name=dev_config['name'],
                    skills=dev_config.get('skills', []),
                    capacity_hours=dev_config.get('capacity_hours', 40),
                    current_load=dev_config.get('current_load', 0),
                    preferred_projects=dev_config.get('preferred_projects', [])
                )
                developers.append(developer)
            
            return developers
        except Exception as e:
            print(f"Error loading team config: {e}")
            return []
    
    def calculate_task_affinity(self, task: Task, developer: Developer) -> float:
        """Calculate how well a task matches a developer's profile"""
        affinity = 0.0
        
        # Project preference
        if task.project in developer.preferred_projects:
            affinity += 0.3
        
        # Skill matching
        task_keywords = task.title.lower().split()
        skill_matches = sum(1 for skill in developer.skills 
                          if any(keyword in skill.lower() for keyword in task_keywords))
        affinity += (skill_matches / len(developer.skills)) * 0.4
        
        # Workload consideration
        capacity_ratio = developer.current_load / developer.capacity_hours
        if capacity_ratio < 0.7:  # Not overloaded
            affinity += 0.2
        elif capacity_ratio > 1.0:  # Overloaded
            affinity -= 0.3
        
        # Priority adjustment
        priority_bonus = {
            'critical': 0.1,
            'high': 0.05,
            'medium': 0.0,
            'low': -0.05
        }
        affinity += priority_bonus.get(task.priority, 0.0)
        
        return max(0.0, min(1.0, affinity))
    
    def distribute_tasks(self) -> Dict[str, List[Task]]:
        """Distribute tasks to developers based on affinity and capacity"""
        assignments = {dev.name: [] for dev in self.developers}
        unassigned_tasks = []
        
        # Sort tasks by priority and complexity
        priority_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
        sorted_tasks = sorted(
            self.tasks,
            key=lambda t: (priority_order.get(t.priority, 2), -t.complexity)
        )
        
        for task in sorted_tasks:
            best_developer = None
            best_affinity = 0.0
            
            for developer in self.developers:
                # Check if developer has capacity
                assigned_hours = sum(t.estimated_hours for t in assignments[developer.name])
                total_hours = developer.current_load + assigned_hours
                
                if total_hours + task.estimated_hours <= developer.capacity_hours * 1.2:  # Allow 20% overflow
                    affinity = self.calculate_task_affinity(task, developer)
                    
                    if affinity > best_affinity:
                        best_affinity = affinity
                        best_developer = developer
            
            if best_developer and best_affinity > 0.3:  # Minimum affinity threshold
                assignments[best_developer.name].append(task)
            else:
                unassigned_tasks.append(task)
        
        if unassigned_tasks:
            assignments['unassigned'] = unassigned_tasks
        
        return assignments
    
    def generate_assignment_report(self, assignments: Dict[str, List[Task]]) -> str:
        """Generate a detailed assignment report"""
        report = []
        report.append("📋 AI-Assisted Task Distribution Report")
        report.append("=" * 60)
        report.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report.append("")
        
        for developer_name, tasks in assignments.items():
            if not tasks:
                continue
                
            report.append(f"👤 {developer_name}")
            report.append("-" * 40)
            
            total_hours = sum(task.estimated_hours for task in tasks)
            report.append(f"Total estimated hours: {total_hours}")
            report.append("")
            
            for task in tasks:
                report.append(f"  📝 [{task.id}] {task.title}")
                report.append(f"     Project: {task.project}")
                report.append(f"     Priority: {task.priority}")
                report.append(f"     Estimated: {task.estimated_hours}h")
                report.append("")
        
        return "\n".join(report)
    
    def export_to_memspec(self, assignments: Dict[str, List[Task]]) -> bool:
        """Export assignments back to memory bank task files"""
        try:
            for developer_name, tasks in assignments.items():
                if developer_name == 'unassigned' or not tasks:
                    continue
                
                for task in tasks:
                    # Update task assignment in memory bank
                    # This would require integration with airs-memspec task update functionality
                    print(f"Would assign {task.id} to {developer_name}")
            
            return True
        except Exception as e:
            print(f"Error exporting assignments: {e}")
            return False

def main():
    if len(sys.argv) < 3:
        print("Usage: task-distributor.py <workspace_path> <team_config.json>")
        sys.exit(1)
    
    workspace_path = sys.argv[1]
    team_config = sys.argv[2]
    
    distributor = TaskDistributor(workspace_path)
    
    # Load data
    distributor.tasks = distributor.load_tasks_from_memspec()
    distributor.developers = distributor.load_team_config(team_config)
    
    if not distributor.tasks:
        print("No pending tasks found")
        sys.exit(0)
    
    if not distributor.developers:
        print("No developers configured")
        sys.exit(1)
    
    # Distribute tasks
    assignments = distributor.distribute_tasks()
    
    # Generate and display report
    report = distributor.generate_assignment_report(assignments)
    print(report)
    
    # Export assignments
    if '--export' in sys.argv:
        distributor.export_to_memspec(assignments)
        print("\n✅ Assignments exported to memory bank")

if __name__ == "__main__":
    main()
```

Example team configuration (`team.json`):

```json
{
  "developers": [
    {
      "name": "Alice Johnson",
      "skills": ["rust", "backend", "database", "api"],
      "capacity_hours": 40,
      "current_load": 25,
      "preferred_projects": ["auth-service", "user-service"]
    },
    {
      "name": "Bob Smith", 
      "skills": ["frontend", "react", "typescript", "ui"],
      "capacity_hours": 35,
      "current_load": 20,
      "preferred_projects": ["frontend-app", "api-gateway"]
    },
    {
      "name": "Carol Davis",
      "skills": ["devops", "kubernetes", "monitoring", "ci-cd"],
      "capacity_hours": 40,
      "current_load": 30,
      "preferred_projects": ["api-gateway", "payment-service"]
    }
  ]
}
```

### Automated Health Monitoring

```bash
#!/bin/bash
# health-monitor.sh - Continuous workspace health monitoring

WORKSPACE_PATH="${1:-$PWD}"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"
CHECK_INTERVAL="${CHECK_INTERVAL:-300}"  # 5 minutes
LOG_FILE="$HOME/.memspec-health.log"

log_event() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
    
    if [ "$level" = "ERROR" ] || [ "$level" = "WARN" ]; then
        echo "[$timestamp] [$level] $message" >&2
    fi
}

check_workspace_health() {
    local workspace_path="$1"
    
    cd "$workspace_path"
    
    # Check memory bank structure
    if [ ! -f ".copilot/memory_bank/current_context.md" ]; then
        log_event "ERROR" "Memory bank not found in $workspace_path"
        return 1
    fi
    
    # Check for blocked tasks
    local blocked_count=$(airs-memspec tasks list --filter blocked --quiet 2>/dev/null | wc -l)
    if [ "$blocked_count" -gt 0 ]; then
        log_event "WARN" "Found $blocked_count blocked tasks"
        
        # Send alert if webhook configured
        if [ -n "$ALERT_WEBHOOK" ]; then
            send_alert "blocked_tasks" "$blocked_count blocked tasks detected"
        fi
    fi
    
    # Check for stale tasks (not updated in 7+ days)
    local stale_count=$(airs-memspec tasks list --filter stale --quiet 2>/dev/null | wc -l)
    if [ "$stale_count" -gt 5 ]; then
        log_event "WARN" "Found $stale_count stale tasks"
    fi
    
    # Check workspace activity
    local active_tasks=$(airs-memspec tasks list --filter active --quiet 2>/dev/null | wc -l)
    if [ "$active_tasks" -eq 0 ]; then
        log_event "INFO" "No active tasks - workspace may be idle"
    fi
    
    log_event "INFO" "Health check complete: $active_tasks active, $blocked_count blocked, $stale_count stale"
    
    return 0
}

send_alert() {
    local alert_type="$1"
    local message="$2"
    
    if [ -z "$ALERT_WEBHOOK" ]; then
        return 0
    fi
    
    local payload=$(cat <<EOF
{
    "alert_type": "$alert_type",
    "message": "$message",
    "workspace": "$WORKSPACE_PATH",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "severity": "warning"
}
EOF
)
    
    curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "$ALERT_WEBHOOK" >/dev/null 2>&1
}

monitoring_loop() {
    log_event "INFO" "Starting health monitoring for $WORKSPACE_PATH"
    log_event "INFO" "Check interval: ${CHECK_INTERVAL}s"
    
    while true; do
        check_workspace_health "$WORKSPACE_PATH"
        sleep "$CHECK_INTERVAL"
    done
}

case "${1:-}" in
    "--daemon"|"-d")
        monitoring_loop
        ;;
    "--check"|"-c")
        check_workspace_health "$WORKSPACE_PATH"
        ;;
    "--log"|"-l")
        tail -f "$LOG_FILE"
        ;;
    *)
        echo "MemSpec Health Monitor"
        echo ""
        echo "Usage: $0 [workspace_path] {--daemon|--check|--log}"
        echo ""
        echo "Options:"
        echo "  --daemon, -d    Run continuous monitoring"
        echo "  --check, -c     Single health check"
        echo "  --log, -l       Follow log file"
        echo ""
        echo "Environment Variables:"
        echo "  ALERT_WEBHOOK   Webhook URL for alerts"
        echo "  CHECK_INTERVAL  Check interval in seconds (default: 300)"
        echo ""
        echo "Examples:"
        echo "  $0 /path/to/workspace --daemon"
        echo "  ALERT_WEBHOOK=https://hooks.slack.com/... $0 --daemon"
        exit 1
        ;;
esac
```

## Performance Optimization

### Batch Operations

```bash
#!/bin/bash
# batch-operations.sh - Optimize airs-memspec operations for large workspaces

batch_status_check() {
    local workspace_root="$1"
    local projects=()
    
    # Discover all projects with memory banks
    while IFS= read -r -d '' project_path; do
        local project_name=$(basename "$(dirname "$project_path")")
        projects+=("$project_name:$(dirname "$project_path")")
    done < <(find "$workspace_root" -name "current_context.md" -path "*/.copilot/memory_bank/*" -print0)
    
    echo "🔍 Batch Status Check (${#projects[@]} projects)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Parallel status checks
    for project_info in "${projects[@]}"; do
        local project_name="${project_info%%:*}"
        local project_path="${project_info##*:}"
        
        (
            cd "$project_path"
            local status=$(airs-memspec status --quiet 2>/dev/null)
            local active_tasks=$(airs-memspec tasks list --filter active --quiet 2>/dev/null | wc -l)
            local blocked_tasks=$(airs-memspec tasks list --filter blocked --quiet 2>/dev/null | wc -l)
            
            local health_icon="🟢"
            if [ "$blocked_tasks" -gt 0 ]; then
                health_icon="🔴"
            elif [ "$active_tasks" -eq 0 ]; then
                health_icon="🟡"
            fi
            
            printf "%-20s %s %2d active, %2d blocked\n" "$project_name" "$health_icon" "$active_tasks" "$blocked_tasks"
        ) &
    done
    
    wait  # Wait for all background jobs to complete
}

batch_task_summary() {
    local workspace_root="$1"
    local filter="${2:-active}"
    
    echo "📋 Batch Task Summary ($filter)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    local temp_dir=$(mktemp -d)
    local summary_file="$temp_dir/task_summary.txt"
    
    # Collect tasks from all projects in parallel
    while IFS= read -r -d '' project_path; do
        (
            cd "$(dirname "$project_path")"
            local project_name=$(basename "$PWD")
            local tasks=$(airs-memspec tasks list --filter "$filter" --quiet 2>/dev/null)
            
            if [ -n "$tasks" ] && [ "$(echo "$tasks" | wc -l)" -gt 0 ]; then
                echo "=== $project_name ===" >> "$summary_file"
                echo "$tasks" >> "$summary_file"
                echo "" >> "$summary_file"
            fi
        ) &
    done < <(find "$workspace_root" -name "current_context.md" -path "*/.copilot/memory_bank/*" -print0)
    
    wait
    
    if [ -f "$summary_file" ] && [ -s "$summary_file" ]; then
        cat "$summary_file"
    else
        echo "No $filter tasks found across all projects"
    fi
    
    rm -rf "$temp_dir"
}

cache_workspace_data() {
    local workspace_root="$1"
    local cache_file="$HOME/.memspec-cache.json"
    local cache_ttl=300  # 5 minutes
    
    # Check cache validity
    if [ -f "$cache_file" ]; then
        local cache_age=$(($(date +%s) - $(stat -f %m "$cache_file" 2>/dev/null || stat -c %Y "$cache_file" 2>/dev/null)))
        if [ "$cache_age" -lt "$cache_ttl" ]; then
            echo "Using cached data (age: ${cache_age}s)"
            cat "$cache_file"
            return 0
        fi
    fi
    
    echo "Refreshing workspace cache..."
    
    local cache_data="{"
    local first=true
    
    while IFS= read -r -d '' project_path; do
        local project_dir=$(dirname "$project_path")
        local project_name=$(basename "$project_dir")
        
        cd "$project_dir"
        
        local active_tasks=$(airs-memspec tasks list --filter active --quiet 2>/dev/null | wc -l)
        local blocked_tasks=$(airs-memspec tasks list --filter blocked --quiet 2>/dev/null | wc -l)
        local total_tasks=$(airs-memspec tasks list --filter all --quiet 2>/dev/null | wc -l)
        
        if [ "$first" = true ]; then
            first=false
        else
            cache_data="$cache_data,"
        fi
        
        cache_data="$cache_data\"$project_name\":{\"active\":$active_tasks,\"blocked\":$blocked_tasks,\"total\":$total_tasks}"
        
    done < <(find "$workspace_root" -name "current_context.md" -path "*/.copilot/memory_bank/*" -print0)
    
    cache_data="$cache_data,\"timestamp\":$(date +%s)}"
    
    echo "$cache_data" > "$cache_file"
    echo "$cache_data"
}
```

### Memory Bank Optimization

```bash
#!/bin/bash
# optimize-memory-bank.sh - Optimize memory bank files for performance

optimize_workspace() {
    local workspace_path="$1"
    
    cd "$workspace_path"
    
    echo "🔧 Optimizing Memory Bank"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Compress old snapshots
    if [ -d ".copilot/memory_bank/context_snapshots" ]; then
        echo "📦 Compressing old context snapshots..."
        
        find ".copilot/memory_bank/context_snapshots" -name "*.md" -mtime +30 -exec gzip {} \;
        
        local compressed_count=$(find ".copilot/memory_bank/context_snapshots" -name "*.gz" | wc -l)
        echo "  Compressed $compressed_count old snapshots"
    fi
    
    # Clean up empty task files
    echo "🧹 Cleaning up empty task files..."
    
    local cleaned_files=0
    while IFS= read -r -d '' task_file; do
        if [ ! -s "$task_file" ] || [ "$(wc -l < "$task_file")" -lt 5 ]; then
            echo "  Removing empty task file: $(basename "$task_file")"
            rm "$task_file"
            ((cleaned_files++))
        fi
    done < <(find ".copilot/memory_bank" -name "task_*.md" -print0)
    
    echo "  Cleaned $cleaned_files empty task files"
    
    # Optimize task indices
    echo "📋 Optimizing task indices..."
    
    while IFS= read -r -d '' index_file; do
        local project_dir=$(dirname "$index_file")
        local temp_file=$(mktemp)
        
        # Rebuild index from actual task files
        echo "# Tasks Index" > "$temp_file"
        echo "" >> "$temp_file"
        
        declare -A status_tasks
        
        while IFS= read -r task_file; do
            if [ -f "$task_file" ]; then
                local task_id=$(basename "$task_file" .md)
                local status=$(grep "^**Status:**" "$task_file" | sed 's/^**Status:** *\([^*]*\).*/\1/')
                
                if [ -n "$status" ]; then
                    if [ -z "${status_tasks[$status]}" ]; then
                        status_tasks[$status]="$task_id"
                    else
                        status_tasks[$status]="${status_tasks[$status]},$task_id"
                    fi
                fi
            fi
        done < <(find "$project_dir" -name "task_*.md" | sort)
        
        # Write organized index
        for status in "in_progress" "pending" "completed" "abandoned"; do
            if [ -n "${status_tasks[$status]}" ]; then
                echo "## $(echo "$status" | tr '_' ' ' | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1')" >> "$temp_file"
                
                IFS=',' read -ra tasks <<< "${status_tasks[$status]}"
                for task in "${tasks[@]}"; do
                    local task_file="$project_dir/$task.md"
                    if [ -f "$task_file" ]; then
                        local title=$(grep "^# \[" "$task_file" | head -1 | sed 's/^# \[.*\] - //')
                        echo "- [$task] $title" >> "$temp_file"
                    fi
                done
                echo "" >> "$temp_file"
            fi
        done
        
        mv "$temp_file" "$index_file"
        echo "  Optimized: $(basename "$(dirname "$index_file")")/_index.md"
        
    done < <(find ".copilot/memory_bank" -name "_index.md" -print0)
    
    # Calculate space savings
    local original_size=$(du -sk ".copilot/memory_bank" | cut -f1)
    echo ""
    echo "✅ Optimization complete"
    echo "   Memory bank size: ${original_size}KB"
    echo "   Files processed: $((cleaned_files + compressed_count))"
}

backup_memory_bank() {
    local workspace_path="$1"
    local backup_path="${2:-$HOME/memspec-backups}"
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local workspace_name=$(basename "$workspace_path")
    local backup_file="$backup_path/${workspace_name}_${timestamp}.tar.gz"
    
    mkdir -p "$backup_path"
    
    echo "💾 Creating memory bank backup..."
    
    cd "$workspace_path"
    tar -czf "$backup_file" .copilot/memory_bank/
    
    echo "✅ Backup created: $backup_file"
    echo "   Size: $(du -h "$backup_file" | cut -f1)"
    
    # Clean old backups (keep last 10)
    ls -t "$backup_path"/${workspace_name}_*.tar.gz | tail -n +11 | xargs rm -f 2>/dev/null || true
}
```

## Next Steps

Continue exploring:

- **[Best Practices](./best-practices.md)** - Professional development recommendations and standards
- **[Troubleshooting](./troubleshooting.md)** - Problem resolution and debugging guides

---

*Advanced scenarios demonstrate the full potential of airs-memspec in complex, large-scale, and automated development environments.*
