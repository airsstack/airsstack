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

# System Patterns: airs-memspec

**Architecture:**
- Modular CLI tool written in Rust with domain-driven design
- Command-based structure (install, status, context, tasks)
- Reads and parses memory bank directory structure and files
- Outputs summaries and actionable state for workspace and sub-projects
- Clean domain separation with 10 focused modules for maintainability
- Comprehensive context correlation system for workspace state management

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

**Parser Module Architecture:**
- **markdown** - Comprehensive markdown parsing with YAML frontmatter support
- **navigation** - Memory bank file system discovery and validation
- **context** - Context correlation system for workspace state management

**Key Technical Decisions:**
- Use of Rust for performance and reliability
- Domain-driven architecture following Single Responsibility Principle
- Direct module access without backward compatibility overhead (new project approach)
- Comprehensive Serde serialization support across all domains
- Follows Multi-Project Memory Bank conventions for file structure and naming
- CLI designed for extensibility and integration with Copilot
- Professional documentation strategies with functional and conceptual examples
- **Import Organization**: Centralized top-level imports for better maintainability
- **Error Handling**: Direct error type usage with proper imports for cleaner code

**Design Patterns in Use:**
- Command pattern for CLI operations
- Domain-driven design for module organization
- Single Responsibility Principle for focused module design
- File system abstraction for cross-platform compatibility
- Structured output for easy parsing and scripting
- Clean separation of concerns across logical domains
- **Context Correlation Pattern**: Workspace-level aggregation with sub-project granularity
- **Import Consolidation Pattern**: Module-level imports to eliminate duplication

**Code Quality Standards:**
- **Import Organization**: All imports centralized at module top-level
- **Error Handling**: Direct type usage (e.g., `FsError` vs `crate::utils::fs::FsError`)
- **Function Clarity**: Functions focus on logic, not import declarations
- **Rust Best Practices**: Consistent application throughout codebase
- **Test Coverage**: Comprehensive unit tests with validation
- **Documentation**: Clear API documentation with usage examples

**Context Correlation Architecture:**
- **ContextCorrelator**: Main engine for workspace context discovery and correlation
- **WorkspaceContext**: Complete workspace state with sub-project aggregation
- **SubProjectContext**: Individual project context with files and task tracking
- **TaskSummary**: Aggregated task status across all projects with progress indicators
- **ProjectHealth**: Health assessment with ordering: Critical < Warning < Healthy
- **Context Switching**: Updates current_context.md with switch tracking metadata

**Module Dependencies:**
- Cross-domain imports carefully managed for clean architecture
- Shared types module provides common enumerations
- Full type safety maintained across module boundaries
- Zero compilation warnings with professional code organization
- Parser modules integrate seamlessly with domain models

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/docs/book/architecture/system_components.html
- crates/airs-memspec/docs/book/architecture/feature.html
- Implementation experience: task_007 (markdown parser) and task_008 (context correlation)
- Code quality improvements: import consolidation and error handling patterns

---

*Knowledge synthesized from:*
- crates/airs-memspec/README.md
- crates/airs-memspec/docs/book/architecture/system_components.html
- crates/airs-memspec/docs/book/architecture/feature.html
