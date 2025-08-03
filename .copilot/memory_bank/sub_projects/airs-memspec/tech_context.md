# Tech Context: airs-memspec

**Technologies Used:**
- Rust (CLI, file system, serialization, markdown parsing)
- pulldown-cmark for robust markdown processing
- serde_yml for YAML frontmatter parsing
- clap for CLI framework
- colored for terminal color support
- terminal_size for responsive formatting
- walkdir for file system traversal
- mdBook for documentation
- Standard Rust crates for CLI parsing and file operations

**Development Setup:**
- Built and run with Cargo
- Documentation generated with mdBook
- Designed for cross-platform use (macOS, Linux, Windows)
- Comprehensive test coverage with unit and integration tests

**Technical Constraints:**
- Must follow Multi-Project Memory Bank file and directory conventions
- Output must be scriptable and human-readable
- Integration with Copilot custom instructions is required
- Markdown parsing must handle various task formats and YAML frontmatter
- Error handling must be user-friendly and actionable

**Key Dependencies:**
- `pulldown-cmark = "0.13"` - Markdown parsing
- `serde_yml = "0.0.12"` - YAML frontmatter handling
- `clap = { version = "4.5", features = ["derive"] }` - CLI framework
- `colored = "3.0"` - Terminal color support
- `terminal_size = "0.4"` - Terminal detection
- `walkdir = "2"` - File system traversal
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `anyhow = "1.0"` - Error handling

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/Cargo.toml
- crates/airs-memspec/src/parser/markdown.rs implementation
- crates/airs-memspec/docs/book/architecture/stack.html
- crates/airs-memspec/docs/book/development/technical.html
