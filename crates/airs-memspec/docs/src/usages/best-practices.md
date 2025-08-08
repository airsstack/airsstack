# Best Practices

Professional development standards, workflow optimization, and recommended patterns for maximizing airs-memspec effectiveness in software development teams.

## Memory Bank Architecture

### File Organization Standards

#### Workspace Structure

```
.copilot/
├── memory_bank/
│   ├── current_context.md           # Active project tracking
│   ├── workspace/
│   │   ├── project_brief.md         # Workspace vision and goals
│   │   ├── shared_patterns.md       # Common implementation patterns
│   │   ├── workspace_architecture.md # High-level system design
│   │   └── workspace_progress.md    # Cross-project milestones
│   ├── context_snapshots/           # Historical state preservation
│   │   ├── 2025_01_15_sprint_end.md
│   │   └── 2025_01_30_release_prep.md
│   └── sub_projects/
│       ├── backend_api/
│       │   ├── project_brief.md     # Project-specific foundation
│       │   ├── product_context.md   # Business context and goals
│       │   ├── active_context.md    # Current development focus
│       │   ├── system_patterns.md   # Technical architecture decisions
│       │   ├── tech_context.md      # Technology stack and constraints
│       │   ├── progress.md          # Implementation status
│       │   └── tasks/
│       │       ├── _index.md        # Task summary and navigation
│       │       ├── task_001_auth_system.md
│       │       └── task_002_user_management.md
│       └── frontend_app/
│           └── [same structure]
└── instructions/
    └── multi_project_memory_bank.instructions.md
```

#### Naming Conventions

**Files and Directories:**
- Use `snake_case` for all files and directories
- Descriptive names that indicate content purpose
- Consistent task numbering: `task_NNN_descriptive_name.md`

**Task IDs:**
- Sequential numbering: `TASK001`, `TASK002`, etc.
- Project-specific prefixes: `AUTH001`, `UI001` for large projects
- Never reuse task IDs, even for abandoned tasks

**Projects:**
- Clear, consistent naming: `backend_api`, `frontend_app`, `mobile_client`
- Avoid abbreviations unless universally understood
- Match repository or service names when possible

### Content Quality Standards

#### Documentation Depth

**Workspace Files (High Level):**
```markdown
# project_brief.md - Strategic Foundation
- Vision and long-term objectives
- Success criteria and metrics  
- Stakeholder requirements
- Resource constraints and timelines

# shared_patterns.md - Implementation Standards
- Coding standards and conventions
- Architecture patterns in use
- Common libraries and frameworks
- Integration approaches

# workspace_architecture.md - System Design
- Component relationships and boundaries
- Data flow and communication patterns
- Infrastructure and deployment strategy
- Security and compliance requirements
```

**Project Files (Detailed):**
```markdown
# product_context.md - Business Context
- Problem statement and user needs
- Market requirements and competitive analysis
- Success metrics and key performance indicators
- User experience goals and constraints

# active_context.md - Current State
- Immediate development priorities
- Active decisions and trade-offs
- Recent changes and their rationale
- Next planned actions and dependencies

# system_patterns.md - Technical Implementation
- Detailed architecture and design patterns
- Technology choices and justifications
- Integration points and interfaces
- Error handling and resilience strategies
```

#### Task Documentation Standards

**Task File Template:**
```markdown
# [TASK_ID] - [Descriptive Title]

**Status:** [pending|in_progress|completed|blocked|abandoned]
**Priority:** [critical|high|medium|low]
**Added:** YYYY-MM-DD
**Updated:** YYYY-MM-DD
**Estimated Effort:** [hours or story points]
**Dependencies:** [List of blocking tasks or external dependencies]

## Original Request
[Complete, unedited description as initially provided]

## Business Context
[Why this task matters, impact on users/business]

## Thought Process
[Design considerations, alternative approaches evaluated, decision rationale]

## Technical Approach
[Specific implementation strategy, architecture considerations]

## Implementation Plan
### Phase 1: [Phase Name]
- [ ] Step 1.1: [Specific action with clear completion criteria]
- [ ] Step 1.2: [Another specific action]

### Phase 2: [Phase Name]  
- [ ] Step 2.1: [Continue with clear, measurable steps]

## Acceptance Criteria
- [ ] Functional requirement 1
- [ ] Functional requirement 2
- [ ] Non-functional requirement 1
- [ ] Testing requirement

## Progress Tracking

**Overall Status:** [Not Started|In Progress|Blocked|Completed] - [XX%]

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | [Subtask] | [Status] | YYYY-MM-DD | [Any relevant notes] |

## Progress Log
### YYYY-MM-DD
- [Specific accomplishments]
- [Decisions made]
- [Issues encountered]
- [Next actions planned]

## Links and References
- [Related documentation]
- [Code repositories]
- [External resources]
- [Design documents]
```

## Workflow Best Practices

### Daily Development Routine

#### Morning Routine (5-10 minutes)

```bash
# 1. Check workspace health
airs-memspec status --workspace

# 2. Review active context
airs-memspec context

# 3. Check active tasks
airs-memspec tasks list --filter active

# 4. Identify any blockers
airs-memspec tasks list --filter blocked
```

**Expected Output Review:**
- **Green indicators**: Continue with planned work
- **Yellow indicators**: Address attention items first
- **Red indicators**: Immediate action required, escalate if needed
- **No active tasks**: Review and prioritize pending tasks

#### During Development

**GitHub Copilot Commands:**
```markdown
# Context management
"update memory bank"           # Comprehensive state update
"show tasks active"           # Current work overview
"add task [description]"      # Capture new work items

# Focus switching
"switch context [project]"    # Change active project
"save context [description]"  # Preserve current state

# Progress tracking
"update task [ID]"           # Document progress and decisions
"show memory bank summary"   # Quick workspace overview
```

**Decision Documentation:**
- **Immediate**: Document architectural decisions in active_context.md
- **Session End**: Update relevant task progress logs
- **Daily**: Ensure all significant decisions are captured

#### End of Day Routine (5 minutes)

```bash
# 1. Update active tasks with progress
# (Use GitHub Copilot: "update task [ID]" for each active task)

# 2. Save current context for tomorrow
# (Use GitHub Copilot: "save context end-of-day-[date]")

# 3. Quick health check
airs-memspec status --workspace --quiet

# 4. Plan tomorrow's priorities
airs-memspec tasks list --priority high
```

### Sprint and Project Management

#### Sprint Planning Integration

**Pre-Sprint Activities:**
1. **Memory bank health check**
   ```bash
   airs-memspec status --workspace
   airs-memspec tasks list --filter blocked
   ```

2. **Context snapshot creation**
   ```markdown
   # In GitHub Copilot chat:
   save context sprint-N-planning
   ```

3. **Task prioritization review**
   ```bash
   airs-memspec tasks list --priority high
   airs-memspec tasks list --filter stale
   ```

**During Sprint:**
- **Daily standups**: Use `airs-memspec tasks list --filter active` for status updates
- **Sprint reviews**: Reference task progress logs for detailed reporting
- **Mid-sprint adjustments**: Update active_context.md with scope changes

**Sprint Retrospectives:**
- Review memory bank evolution during sprint
- Identify patterns in blocked tasks
- Assess accuracy of effort estimates
- Update shared_patterns.md with learnings

#### Release Management

**Pre-Release Checklist:**
```bash
# 1. Comprehensive health check
airs-memspec status --workspace

# 2. Verify no critical blocked tasks
airs-memspec tasks list --filter blocked --priority critical

# 3. Create release context snapshot
# In GitHub Copilot: "save context release-v1.2.0-candidate"

# 4. Document release decisions
# Update workspace_progress.md with release notes
```

**Release Documentation:**
- Update workspace_progress.md with release highlights
- Create context snapshot for historical reference
- Document any technical debt incurred for post-release planning

### Team Collaboration Standards

#### Onboarding New Team Members

**Week 1: Foundation**
1. **Install and configure airs-memspec**
   ```bash
   cargo install airs-memspec
   airs-memspec install
   ```

2. **Memory bank orientation**
   ```bash
   airs-memspec status --workspace
   airs-memspec context --workspace
   ```

3. **Review documentation structure**
   - Read workspace/project_brief.md
   - Understand shared_patterns.md
   - Review active projects in current_context.md

**Week 2: Integration**
1. **GitHub Copilot setup with custom instructions**
2. **Practice with memory bank commands**
3. **First task assignment with mentoring**

**Week 3: Independence**
1. **Independent task management**
2. **Contribution to shared patterns**
3. **Team workflow integration**

#### Code Review Integration

**Pre-Review (Author):**
```markdown
# In GitHub Copilot chat before creating PR:
"update memory bank"
"show memory bank summary"
```

**Review Context (Reviewer):**
- Check related task documentation
- Verify alignment with system_patterns.md
- Review decision rationale in progress logs

**Post-Review Updates:**
- Update task progress with review feedback
- Document any pattern changes in shared_patterns.md
- Update active_context.md if scope changed

#### Knowledge Sharing

**Documentation Standards:**
- **Real-time**: Capture decisions in active_context.md
- **Weekly**: Update shared_patterns.md with reusable insights
- **Monthly**: Review and clean up memory bank structure

**Team Sync Integration:**
```bash
# Weekly team sync preparation
airs-memspec tasks list --filter blocked  # Identify blockers
airs-memspec context --workspace          # Share current focus
```

## Quality Assurance

### Automated Quality Checks

#### CI/CD Integration

**GitHub Actions Workflow:**
```yaml
name: MemSpec Quality Check

on: [push, pull_request]

jobs:
  memspec-quality:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install airs-memspec
      run: cargo install airs-memspec
      
    - name: Memory Bank Structure Check
      run: |
        # Verify required files exist
        test -f .copilot/memory_bank/current_context.md
        test -f .copilot/memory_bank/workspace/project_brief.md
        test -f .copilot/memory_bank/workspace/shared_patterns.md
        
    - name: Task Consistency Check  
      run: |
        # Verify all tasks have required fields
        for task_file in .copilot/memory_bank/sub_projects/*/tasks/task_*.md; do
          if [ -f "$task_file" ]; then
            grep -q "^**Status:**" "$task_file" || {
              echo "Missing status in $task_file"
              exit 1
            }
          fi
        done
        
    - name: No Critical Blocked Tasks
      run: |
        blocked_count=$(airs-memspec tasks list --filter blocked --priority critical --quiet | wc -l)
        if [ "$blocked_count" -gt 0 ]; then
          echo "❌ Found $blocked_count critical blocked tasks"
          airs-memspec tasks list --filter blocked --priority critical
          exit 1
        fi
```

#### Pre-commit Hooks

**.pre-commit-config.yaml:**
```yaml
repos:
  - repo: local
    hooks:
      - id: memspec-format-check
        name: MemSpec Format Check
        entry: bash
        args: [-c, '
          # Check snake_case naming
          find .copilot/memory_bank -name "*.md" | grep -E "[A-Z]" && {
            echo "Files must use snake_case naming"
            exit 1
          }
          
          # Check task ID format
          find .copilot/memory_bank -name "task_*.md" | while read f; do
            basename "$f" | grep -qE "^task_[0-9]{3}_[a-z_]+\.md$" || {
              echo "Invalid task filename format: $f"
              exit 1
            }
          done
        ']
        language: system
        pass_filenames: false
        
      - id: memspec-content-check
        name: MemSpec Content Check
        entry: bash
        args: [-c, '
          # Check for required sections in task files
          for task_file in .copilot/memory_bank/sub_projects/*/tasks/task_*.md; do
            if [ -f "$task_file" ]; then
              for section in "## Original Request" "## Implementation Plan" "## Progress Log"; do
                grep -q "$section" "$task_file" || {
                  echo "Missing required section \"$section\" in $task_file"
                  exit 1
                }
              done
            fi
          done
        ']
        language: system
        pass_filenames: false
```

### Documentation Quality Standards

#### Content Guidelines

**Writing Style:**
- **Clear and concise**: Avoid unnecessary complexity
- **Action-oriented**: Use active voice and specific verbs
- **Consistent terminology**: Maintain project glossary
- **Future-friendly**: Write for readers who join later

**Technical Accuracy:**
- **Verifiable claims**: Include references and examples
- **Current information**: Regular review and updates
- **Realistic estimates**: Track and improve estimation accuracy
- **Complete context**: Sufficient information for implementation

#### Review Process

**Memory Bank Reviews (Monthly):**
1. **Accuracy audit**: Verify current state matches documentation
2. **Relevance check**: Remove or archive outdated information
3. **Completeness review**: Identify gaps in documentation
4. **Structure optimization**: Improve organization and navigation

**Quality Metrics:**
- **Task completion accuracy**: Actual vs. estimated effort
- **Decision tracking**: Percentage of architectural decisions documented
- **Context freshness**: Time since last update of active files
- **Blocker resolution time**: Average time to resolve blocked tasks

## Security and Privacy

### Sensitive Information Handling

#### Data Classification

**Safe for Memory Bank:**
- Architecture decisions and patterns
- Implementation approaches and trade-offs
- Task status and progress tracking
- Team workflow and process information

**Exclude from Memory Bank:**
- API keys, passwords, or credentials
- Personal identifying information (PII)
- Customer data or business secrets
- Infrastructure details that could enable attacks

#### Access Control

**Repository Permissions:**
- Memory bank files should match repository access levels
- Consider separate private repositories for sensitive projects
- Use `.gitignore` to exclude local-only memory bank data

**Team Access Patterns:**
```gitignore
# Example .gitignore entries
.copilot/memory_bank/local_notes/
.copilot/memory_bank/personal_context.md
*.personal.md
**/local_overrides/
```

### Compliance Considerations

#### Data Retention

**Automatic Cleanup:**
```bash
#!/bin/bash
# cleanup-old-data.sh - Automated memory bank maintenance

# Archive context snapshots older than 1 year
find .copilot/memory_bank/context_snapshots -name "*.md" -mtime +365 -exec gzip {} \;

# Remove abandoned tasks older than 6 months  
find .copilot/memory_bank -name "task_*.md" -mtime +180 -exec grep -l "abandoned" {} \; | xargs rm

# Compress large progress logs
find .copilot/memory_bank -name "*.md" -size +100k -exec gzip {} \;
```

**Retention Policies:**
- **Active tasks**: Retain indefinitely while active
- **Completed tasks**: Archive after 1 year
- **Context snapshots**: Compress after 6 months, delete after 2 years
- **Decision records**: Retain permanently for historical reference

## Performance Optimization

### Scalability Patterns

#### Large Workspace Management

**Project Segmentation:**
```bash
# Separate memory banks for major components
/frontend/.copilot/memory_bank/
/backend/.copilot/memory_bank/  
/mobile/.copilot/memory_bank/
/shared/.copilot/memory_bank/   # Common patterns and decisions
```

**Cross-Project Coordination:**
```bash
# Synchronize shared patterns
rsync -av shared/.copilot/memory_bank/workspace/ \
    frontend/.copilot/memory_bank/workspace/
```

#### Task Management at Scale

**Task Hierarchy:**
```markdown
# Epic-level tracking in workspace_progress.md
## Q1 2025 Objectives
- Epic 1: User Authentication System
  - AUTH001: OAuth 2.1 implementation
  - AUTH002: JWT token management  
  - AUTH003: Session handling

# Detailed tracking in project-specific tasks/
```

**Batch Operations:**
```bash
# Weekly task health check across all projects
find . -path "*/.copilot/memory_bank/sub_projects/*/tasks" -name "_index.md" \
  -exec dirname {} \; | while read task_dir; do
    cd "$task_dir"
    project_name=$(basename "$(dirname "$(dirname "$task_dir")")")
    echo "=== $project_name ==="
    airs-memspec tasks list --filter blocked
    cd - >/dev/null
done
```

### Monitoring and Metrics

#### Workspace Health Metrics

**Key Performance Indicators:**
```bash
#!/bin/bash
# generate-kpis.sh - Workspace health metrics

echo "Memory Bank Health Report - $(date)"
echo "=================================="

# Task flow metrics
total_tasks=$(find .copilot/memory_bank -name "task_*.md" | wc -l)
active_tasks=$(airs-memspec tasks list --filter active --quiet | wc -l)
blocked_tasks=$(airs-memspec tasks list --filter blocked --quiet | wc -l)
completion_rate=$((($total_tasks - $active_tasks) * 100 / $total_tasks))

echo "Task Metrics:"
echo "  Total tasks: $total_tasks"
echo "  Active tasks: $active_tasks"
echo "  Blocked tasks: $blocked_tasks"
echo "  Completion rate: $completion_rate%"

# Documentation freshness
old_context_files=$(find .copilot/memory_bank -name "active_context.md" -mtime +7 | wc -l)
echo ""
echo "Documentation Freshness:"
echo "  Stale active contexts: $old_context_files"

# Memory bank size
mb_size=$(du -sh .copilot/memory_bank | cut -f1)
echo ""
echo "Storage Metrics:"
echo "  Memory bank size: $mb_size"
```

**Automated Alerting:**
```bash
# Add to crontab for daily monitoring
0 9 * * * /path/to/workspace-health-check.sh | mail -s "Workspace Health" team@company.com
```

## Troubleshooting Common Issues

### Memory Bank Corruption

**Symptoms:**
- Missing required files
- Malformed task files
- Inconsistent task indices

**Recovery Process:**
```bash
# 1. Create backup
cp -r .copilot/memory_bank .copilot/memory_bank.backup.$(date +%Y%m%d)

# 2. Reinstall instructions
airs-memspec install --force

# 3. Rebuild task indices
find .copilot/memory_bank -name "_index.md" -delete
# Then use GitHub Copilot: "update memory bank" to rebuild
```

### Performance Issues

**Large Memory Bank:**
```bash
# Compress old data
find .copilot/memory_bank -name "*.md" -mtime +90 -exec gzip {} \;

# Split large projects
# Move sub-projects to separate repositories
```

**Slow Commands:**
```bash
# Use caching for repeated operations
export MEMSPEC_CACHE_TTL=300  # 5 minutes
```

### Team Synchronization Problems

**Merge Conflicts in Memory Bank:**
```bash
# Use project-specific branches for memory bank updates
git checkout -b membank-update-$(date +%Y%m%d)
# Make memory bank changes
git add .copilot/memory_bank/
git commit -m "Update memory bank: [description]"
```

**Inconsistent Documentation:**
```bash
# Regular sync meetings to align memory bank content
# Use shared patterns document as single source of truth
# Implement memory bank review process
```

## Next Steps

Continue exploring:

- **[Troubleshooting](./troubleshooting.md)** - Problem resolution and debugging guides

---

*Best practices ensure sustainable, scalable, and effective use of airs-memspec across development teams and complex projects.*
