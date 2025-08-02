# Feature Specifications

## Core Commands

### Installation Commands

```bash
airs-memspec install --path <PATH>
```

Functionality:

- Deploy embedded Multi-Project Memory Bank custom instructions
- Create necessary directory structure if missing
- Provide clear success/failure feedback
- Support custom installation paths

### Status Commands

```bash
airs-memspec status [--workspace] [--project <name>]
```

Functionality:

- Display development phase and progress
- Show active work and current focus
- List sprint objectives and blockers
- Provide milestone information

### Context Commands

```bash
airs-memspec context [--workspace] [--project <name>]
```

Functionality:

- Show active development context
- Display integration points and constraints
- Present architectural decisions
- Highlight current technical focus

### Task Commands

```bash
airs-memspec tasks --project <name> [--filter <type>]
```

Functionality:

- List tasks with priority and status
- Show progress tracking and estimates
- Support filtering (active, pending, completed, blocked)
- Display task hierarchy and subtasks

## Global Options

- `--path <PATH>`: Memory bank root directory [default: .copilot]
- `--verbose, -v:` Enable detailed output
- `--quiet, -q`: Minimal output for scripting
- `--no-color`: Disable colored output
- `--help, -h`: Command help
- `--version, -V`: Version information
