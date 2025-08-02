# Technical Implementation Details

## Technology Stack

```toml
[dependencies]
# CLI Framework
clap = { version = "4.0", features = ["derive", "color"] }

# Async Operations
tokio = { version = "1.0", features = ["fs", "macros", "rt"] }

# Serialization & Configuration
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Output Formatting
colored = "2.0"
unicode-width = "0.1"
terminal_size = "0.3"

# Text Processing
regex = "1.0"
pulldown-cmark = "0.9"
```

## Project Structure

```
crates/airs-memspec/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   ├── cli/                    # Command-line interface
│   │   ├── mod.rs
│   │   ├── commands.rs         # Command definitions
│   │   └── args.rs             # Argument parsing
│   ├── parser/                 # Memory bank parser
│   │   ├── mod.rs
│   │   ├── workspace.rs        # Workspace parsing
│   │   ├── project.rs          # Project parsing
│   │   ├── tasks.rs            # Task parsing
│   │   └── context.rs          # Context correlation
│   ├── output/                 # Output formatting
│   │   ├── mod.rs
│   │   ├── formatter.rs        # Output formatter
│   │   ├── status.rs           # Status display
│   │   ├── context.rs          # Context display
│   │   └── tasks.rs            # Tasks display
│   ├── instructions/           # Embedded custom instructions
│   │   ├── mod.rs
│   │   └── memory_bank.md      # Multi-Project Memory Bank instructions
│   └── utils/                  # Utility functions
│       ├── mod.rs
│       ├── fs.rs               # File system utilities
│       └── terminal.rs         # Terminal detection
├── tests/                      # Integration tests
└── README.md
```

## Data Model Architecture

```rust
// Core data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub overview: WorkspaceOverview,
    pub architecture: Vec<ArchitectureDecision>,
    pub patterns: Vec<Pattern>,
    pub projects: Vec<ProjectSummary>,
    pub current_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub overview: ProjectOverview,
    pub status: DevelopmentStatus,
    pub active_context: ActiveContext,
    pub tasks: TaskCollection,
    pub technical_context: TechnicalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCollection {
    pub index: TaskIndex,
    pub tasks: Vec<Task>,
    pub status_summary: TaskStatusSummary,
}
```