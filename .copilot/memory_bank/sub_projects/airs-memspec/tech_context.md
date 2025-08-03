# Tech Context: airs-memspec

# Tech Context: airs-memspec

**Technologies Used:**
- Rust (CLI, file system, serialization, markdown parsing, context correlation)
- pulldown-cmark for robust markdown processing
- serde_yml for YAML frontmatter parsing
- clap for CLI framework
- colored for terminal color support
- terminal_size for responsive formatting
- walkdir for file system traversal
- chrono for timestamp management
- mdBook for documentation
- Standard Rust crates for CLI parsing and file operations

**Implementation Status:**
- ✅ **Core Parsing Pipeline**: Complete markdown parsing with YAML frontmatter support
- ✅ **File System Navigation**: Comprehensive memory bank discovery and validation
- ✅ **Context Correlation**: Full workspace context management with sub-project aggregation
- ✅ **Output Framework**: Terminal-adaptive formatting with color support
- ✅ **CLI Framework**: Complete command structure with proper argument handling
- ✅ **Domain Models**: Comprehensive data structures with Serde support

**Development Setup:**
- Built and run with Cargo
- Documentation generated with mdBook
- Designed for cross-platform use (macOS, Linux, Windows)
- Comprehensive test coverage with unit and integration tests
- All compilation clean with zero warnings
- Professional code organization following Rust best practices

**Technical Constraints:**
- Must follow Multi-Project Memory Bank file and directory conventions
- Output must be scriptable and human-readable
- Integration with Copilot custom instructions is required
- Markdown parsing must handle various task formats and YAML frontmatter
- Error handling must be user-friendly and actionable
- Code quality must follow Rust best practices with proper import organization

**Key Dependencies:**
- `pulldown-cmark = "0.13"` - Markdown parsing engine
- `serde_yml = "0.0.12"` - YAML frontmatter handling
- `clap = { version = "4.5", features = ["derive"] }` - CLI framework
- `colored = "3.0"` - Terminal color support
- `terminal_size = "0.4"` - Terminal detection and adaptation
- `walkdir = "2"` - File system traversal and discovery
- `chrono = { version = "0.4", features = ["serde"] }` - Timestamp management
- `serde = { version = "1.0", features = ["derive"] }` - Serialization framework
- `anyhow = "1.0"` - Error handling and reporting

**Code Quality Standards:**
- **Import Organization**: All imports centralized at module level
- **Error Handling**: Direct type usage with proper top-level imports
- **Documentation**: Comprehensive API documentation with examples
- **Testing**: Unit tests with comprehensive coverage validation
- **Architecture**: Domain-driven design with clean module separation
- **Performance**: Efficient parsing and correlation algorithms

**Technical Architecture:**
- **Parser Module**: Markdown processing, file navigation, context correlation
- **Models Module**: Domain-driven data structures with full Serde support
- **CLI Module**: Command handling with clap integration
- **Utils Module**: File operations, output formatting, error handling
- **Embedded Module**: Multi-Project Memory Bank instructions integration

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/Cargo.toml
- Implementation experience from tasks 001-008
- Code quality improvements and refactoring efforts
- crates/airs-memspec/docs/book/architecture/stack.html
- crates/airs-memspec/docs/book/development/technical.html

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/Cargo.toml
- crates/airs-memspec/src/parser/markdown.rs implementation
- crates/airs-memspec/docs/book/architecture/stack.html
- crates/airs-memspec/docs/book/development/technical.html
