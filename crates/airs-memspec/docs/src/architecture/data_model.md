# Data Model Understanding

## Workspace Structure

```
.copilot/memory_bank/
├── current_context.md              # Active sub-project tracking
├── workspace/                      # Shared workspace knowledge
│   ├── project_brief.md            # Workspace vision & objectives
│   ├── shared_patterns.md          # Cross-project patterns
│   ├── workspace_architecture.md   # High-level relationships
│   └── workspace_progress.md       # Strategic milestones
├── context_snapshots/              # Historical state preservation
└── sub_projects/                   # Individual project contexts
    └── [project_name]/
        ├── project_brief.md         # Project foundation
        ├── product_context.md       # Purpose & user experience
        ├── active_context.md        # Current work focus
        ├── system_patterns.md       # Technical architecture
        ├── tech_context.md          # Technology stack
        ├── progress.md              # Implementation status
        └── tasks/                   # Task management
            ├── _index.md            # Task registry
            └── task_[id]_[name].md  # Individual tasks
```

## Parser Requirements

- File Format: Markdown with optional YAML frontmatter
- Naming Convention: snake_case throughout all structures
- Content Structure: Extract sections, lists, and metadata
- Relationship Mapping: Understand project dependencies and relationships
