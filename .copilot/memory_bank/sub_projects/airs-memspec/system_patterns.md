# System Patterns: airs-memspec

**Architecture:**
- Modular CLI tool written in Rust with domain-driven design
- Command-based structure (install, status, context, tasks)
- Reads and parses memory bank directory structure and files
- Outputs summaries and actionable state for workspace and sub-projects
- Clean domain separation with 10 focused modules for maintainability

**Memory Bank Module Architecture:**
- **Domain-Driven Design**: Refactored from 2,116-line monolith to focused modules
- **workspace** - Workspace-level configuration and context management
- **sub_project** - Individual project management and metadata
- **system** - System architecture and technical decisions
- **tech** - Technology context and infrastructure requirements
- **monitoring** - Observability and monitoring setup
- **progress** - Progress tracking and metrics
- **testing** - Testing and quality assurance framework
- **review** - Code review management
- **task_management** - Comprehensive task tracking system
- **types** - Shared enumerations and common types

**Key Technical Decisions:**
- Use of Rust for performance and reliability
- Domain-driven architecture following Single Responsibility Principle
- Direct module access without backward compatibility overhead (new project approach)
- Comprehensive Serde serialization support across all domains
- Follows Multi-Project Memory Bank conventions for file structure and naming
- CLI designed for extensibility and integration with Copilot
- Professional documentation strategies with functional and conceptual examples

**Design Patterns in Use:**
- Command pattern for CLI operations
- Domain-driven design for module organization
- Single Responsibility Principle for focused module design
- File system abstraction for cross-platform compatibility
- Structured output for easy parsing and scripting
- Clean separation of concerns across logical domains

**Module Dependencies:**
- Cross-domain imports carefully managed for clean architecture
- Shared types module provides common enumerations
- Full type safety maintained across module boundaries
- Zero compilation warnings with professional code organization

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/docs/book/architecture/system_components.html
- crates/airs-memspec/docs/book/architecture/feature.html
