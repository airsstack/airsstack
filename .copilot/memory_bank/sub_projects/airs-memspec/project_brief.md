# Project Brief: airs-memspec

**Purpose:**
airs-memspec is a CLI tool for managing Multi-Project Memory Bank structures and integrating GitHub Copilot custom instructions. It enables context preservation and intelligent project state management across multiple sub-projects in a workspace.

**Scope:**
- Custom instructions management for Copilot
- Memory bank structure parsing and visualization
- Project and sub-project state reporting
- Context-aware reading of workspace hierarchy
- Lightweight, fast, and focused on essential memory bank operations

**Core Problem Statement:**
Developers need a reliable, automated way to manage project context and Copilot instructions across complex, multi-project workspaces. airs-memspec addresses this by providing a unified CLI for memory bank and instruction management.

**Vision:**
To streamline AI-assisted development workflows and make context management seamless, actionable, and robust for all contributors.


*Knowledge synthesized from:*

---

## Comprehensive Knowledge Summary (from docs/src/ - Markdown Only)

### Overview & Vision
- Project purpose, vision, and the core problem of AI context loss are described in detail. airs-memspec bridges Multi-Project Memory Bank structures and Copilot, solving context amnesia, setup friction, and knowledge fragmentation. See: `crates/airs-memspec/docs/src/overview.md`

### Data Model & Workspace Structure
- The memory bank is organized as a hierarchical directory of markdown files, with strict snake_case naming and clear separation between workspace-level and sub-project context. See: `crates/airs-memspec/docs/src/architecture/data_model.md`

### System Components & Parser Requirements
- Core components include embedded instructions, a flexible installation system, a robust memory bank parser, and a context-aware output system. The parser extracts structured data from markdown, maps relationships, and is resilient to missing/incomplete files. See: `crates/airs-memspec/docs/src/architecture/system_components.md`

### CLI Design & Features
- The CLI exposes commands for install, status, context, and tasks, supporting context switching, filtering, and batch operations. Each command is designed for clarity, scriptability, and actionable output. See: `crates/airs-memspec/docs/src/architecture/feature.md`

### Technology Stack & Implementation
- Built in Rust 2021 with `clap` for CLI, `tokio` for async, `serde` for serialization, and `anyhow` for error handling. Output formatting uses `colored`, `unicode-width`, and `terminal_size`. Project structure is modular, with clear separation of CLI, parser, output, and utility logic. See: `crates/airs-memspec/docs/src/architecture/stack.md`, `crates/airs-memspec/docs/src/development/technical.md`

### Integration Strategy & Copilot Workflow
- Designed for seamless integration with GitHub Copilot and the Multi-Project Memory Bank convention. Supports context switching, state monitoring, and historical tracking via context snapshots. See: `crates/airs-memspec/docs/src/architecture/integration.md`

### Development Plans & Technical Details
- The development process, objectives, and technical implementation details are documented in day-by-day plans and technical references. See: `crates/airs-memspec/docs/src/development_plans.md`, `crates/airs-memspec/docs/src/development/technical.md`, and all `crates/airs-memspec/docs/src/development/day_*.md` files.

---

*For further detail, consult the referenced Markdown docs/src/ files directly. This summary ensures the memory bank is a complete, actionable context source for all future work.*
  - crates/airs-memspec/docs/src/SUMMARY.md
  - crates/airs-memspec/docs/src/overview.md
  - crates/airs-memspec/docs/src/architecture/architecture.md
  - crates/airs-memspec/docs/src/architecture/data_model.md
  - crates/airs-memspec/docs/src/architecture/feature.md
  - crates/airs-memspec/docs/src/architecture/integration.md
  - crates/airs-memspec/docs/src/architecture/stack.md
  - crates/airs-memspec/docs/src/architecture/system_components.md
  - crates/airs-memspec/docs/src/development_plans.md
  - crates/airs-memspec/docs/src/development/technical.md
  - crates/airs-memspec/docs/src/development/day_1.md
  - crates/airs-memspec/docs/src/development/day_2.md
  - crates/airs-memspec/docs/src/development/day_3.md
  - crates/airs-memspec/docs/src/development/day_4.md

**Context Loss Problem:**
Developers using AI coding assistants (especially Copilot) face persistent context loss between sessions, manual setup overhead, and knowledge fragmentation—especially in multi-project workspaces.

**Solution Architecture:**
airs-memspec bridges structured context (Multi-Project Memory Bank) and AI workflows. It delivers custom instructions, parses and displays current state, and enables workspace intelligence and natural AI collaboration for memory bank maintenance.

---

*Further details synthesized from:*
- crates/airs-memspec/docs/book/overview.html
