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

## Quick Start

### 1. Install Custom Instructions
```bash
# Install custom instructions to .copilot directory
airs-memspec install --path .copilot

# Install to custom path
airs-memspec install --path /path/to/your/copilot/config
```

### 2. Check Memory Bank Status
```bash
# Show workspace overview
airs-memspec status --workspace

# Show specific sub-project status
airs-memspec status --project airs-mcp

# Show all sub-projects
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