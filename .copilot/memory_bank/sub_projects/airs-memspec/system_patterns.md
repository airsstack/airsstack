# System Patterns: airs-memspec

**Architecture:**
- Modular CLI tool written in Rust
- Command-based structure (install, status, context, tasks)
- Reads and parses memory bank directory structure and files
- Outputs summaries and actionable state for workspace and sub-projects

**Key Technical Decisions:**
- Use of Rust for performance and reliability
- Follows Multi-Project Memory Bank conventions for file structure and naming
- CLI designed for extensibility and integration with Copilot

**Design Patterns in Use:**
- Command pattern for CLI operations
- File system abstraction for cross-platform compatibility
- Structured output for easy parsing and scripting

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/docs/book/architecture/system_components.html
- crates/airs-memspec/docs/book/architecture/feature.html
