# AIRS - AI & Rust Technology Stack

A personal AI technology stack built entirely in Rust, designed as a foundational framework for AI engineering tools and software. AIRS emphasizes type safety, performance, and clean architecture for building AI-powered applications with human-designed architecture and AI-assisted implementation.

## 🎉 Production Achievements

**✅ Claude Desktop Integration Verified**  
**✅ Complete MCP Server/Client Implementation**  
**✅ 100% Schema Compliance (MCP 2024-11-05)**  
**✅ Production-Grade Examples & Documentation**

### 🚀 **Real-World Success**
- **MCP Server**: Successfully integrated with Claude Desktop - resources, tools, and prompts working in production
- **MCP Client**: High-level Rust API with automatic subprocess management and real protocol interactions  
- **Type Safety**: Full Rust type safety throughout MCP protocol implementation
- **Examples**: Working client/server examples with comprehensive documentation

[**See MCP Server Example →**](crates/airs-mcp/examples/simple-mcp-server/)  
[**See MCP Client Example →**](crates/airs-mcp/examples/simple-mcp-client/)

## Technology Stack

### Core Technologies
- **Language**: Rust 1.88.0+ (MSRV)
- **Build System**: Cargo with Workspace structure
- **AI Integration**: Model Context Protocol (MCP) implementation
- **Development**: GitHub Copilot-assisted development workflow

### Dependencies & Tools
- **Workspace Management**: Cargo Workspace for multi-crate organization
- **AI Tooling**: Custom Copilot configurations and prompts
- **Version Control**: Git with structured branching
- **Documentation**: Markdown-based documentation system

## Project Architecture

AIRS follows a modular Cargo Workspace architecture designed for scalability and maintainability:

```
airs/
├── Cargo.toml              # Workspace configuration
├── crates/                 # Individual workspace members
│   ├── airs-mcp/          # Model Context Protocol implementation
│   └── airs-memspec/      # Memory bank specification and tooling
├── .copilot/              # AI-assisted development configuration
│   ├── chatmodes/         # Custom chat interaction modes
│   ├── instructions.md    # Development guidelines
│   └── prompts/           # Reusable AI prompts
└── docs/                  # Project documentation
```


### Design Principles
- **Human Architecture, AI Implementation**: Strategic decisions made by humans, code generated with AI assistance
- **Rust-First Approach**: Leveraging Rust's memory safety and performance for AI infrastructure
- **Modular Design**: Each crate serves a specific purpose in the AI ecosystem
- **Type Safety**: Strong typing for reliable AI tool development

## AI Collaboration & Memory-Bank Management

This project leverages a robust memory-bank management system to ensure resilient, transparent, and context-driven development. The memory bank is a structured set of Markdown files that track requirements, architecture, technical decisions, implementation plans, and progress for every sub-project and the workspace as a whole.

### Memory-Bank System Overview
- **Workspace-Level Context**: Shared files define the overall vision, architecture, and standards for all sub-projects.
- **Sub-Project Memory Bank**: Each sub-project maintains its own set of files for requirements, design, tech context, active decisions, and progress.
- **Task Management**: Every development task is tracked in detail, with status, subtasks, and progress logs, enabling precise project management and onboarding.
- **Context Snapshots**: Operational state can be saved and restored at any time, supporting historical analysis and rapid recovery from context loss.

### Human/AI Collaboration
- **Human Architecture, AI Implementation**: Strategic decisions are made by humans, while code and documentation are generated and maintained with AI assistance.
- **AI Agent Workflow**: The AI agent operates autonomously, executing tasks, updating documentation, and managing context without requiring confirmation or permission.
- **Specification-Driven Execution**: All work follows a rigorous, specification-driven workflow, ensuring requirements are clear, designs are validated, and implementations are thoroughly documented and tested.
- **Transparency & Traceability**: Every action, decision, and change is logged in the memory bank, providing a complete audit trail of human/AI collaboration.

### How to Use the Memory Bank
1. **Review Context**: Start every session by reading all relevant memory bank files for the workspace and active sub-project.
2. **Track Tasks**: Use the tasks index and individual task files to monitor progress, update statuses, and document decisions.
3. **Update Documentation**: After every significant change, update the memory bank to reflect the current state and next steps.
4. **Save Context Snapshots**: Use context snapshots to preserve operational state for onboarding, recovery, or historical analysis.
5. **Switch Contexts**: For multi-project workspaces, update the active sub-project and re-read all relevant files before proceeding.

For more details, see `.copilot/memory_bank/` and the documentation in `docs/`.

## Getting Started

### Prerequisites

- **Rust**: 1.88.0 or later (we track the latest stable release)
- **Cargo**: Included with Rust installation
- **Git**: For version control and development workflow

### Installation

1. **Clone the repository**:
```bash
git clone https://github.com/your-username/airs.git
cd airs
```

2. **Build the workspace**:
```bash
cargo build
```

3. **Try the working examples**:
```bash
# Test the MCP server (Claude Desktop integration)
cd crates/airs-mcp/examples/simple-mcp-server
cargo run

# Test the MCP client (automatic server interaction)  
cd ../simple-mcp-client
cargo run
```


4. **Check all workspace members**:
```bash
cargo check --workspace
```

5. **Run tests to verify everything works**:
```bash
cargo test --workspace
```

### Quick Start Examples

**Try the MCP server with Claude Desktop:**
```bash
cd crates/airs-mcp/examples/simple-mcp-server
cargo build --release

# Add to Claude Desktop config - see README for integration steps
# Resources, tools, and prompts will appear in Claude's UI
```

**Try the MCP client demonstration:**
```bash
cd crates/airs-mcp/examples/simple-mcp-client  
cargo run  # Automatically spawns server and demonstrates all MCP operations
```

## Project Structure

### Workspace Organization

```
airs/
├── Cargo.toml                    # Root workspace configuration
├── crates/                       # All workspace members
│   ├── airs-mcp/                # MCP implementation crate (✅ Production Ready)
│   │   ├── examples/            # Working examples
│   │   │   ├── simple-mcp-server/  # Claude Desktop integration verified
│   │   │   └── simple-mcp-client/  # AIRS library usage demonstration
│   │   └── Cargo.toml           # Crate-specific configuration
│   └── airs-memspec/            # Memory bank specification and tooling
├── .copilot/                    # AI-assisted development configuration
│   ├── chatmodes/               # Custom interaction modes
│   ├── instructions.md          # Development practices
│   └── prompts/                 # Reusable AI prompts
├── docs/                        # Additional documentation
├── LICENSE-APACHE               # Apache 2.0 license
├── LICENSE-MIT                  # MIT license
└── README.md                    # This file
```

### Current Workspace Members

- **`airs-mcp`**: **✅ Production-Ready** Model Context Protocol implementation
  - Complete MCP server/client functionality
  - **Claude Desktop integration verified** with working examples
  - High-level type-safe APIs for both server and client
  - Advanced transport layer with custom transport support
  - [Server Example](crates/airs-mcp/examples/simple-mcp-server/) | [Client Example](crates/airs-mcp/examples/simple-mcp-client/)

- **`airs-memspec`**: Memory bank specification and tooling
  - Structured memory bank management for AI-assisted development
  - Context preservation and snapshot functionality
  - Multi-project workspace support
  - Task tracking and progress management

## Key Features

### Current Implementation
- **✅ Production MCP Implementation**: Complete server/client with Claude Desktop integration verified
- **✅ Working Examples**: Real-world server/client examples with documented usage patterns
- **✅ Advanced Transport Layer**: Custom transport support with SubprocessTransport example
- **✅ Type-Safe APIs**: High-level Rust APIs for MCP protocol interactions
- **✅ Memory Bank System**: Structured AI-assisted development workflow management
- **✅ Cargo Workspace Structure**: Organized multi-crate development environment
- **✅ AI-Assisted Development**: Copilot-optimized workflow and prompts

### Demonstrated Capabilities
- **MCP Server**: Successfully integrated with Claude Desktop (resources, tools, prompts)
- **MCP Client**: High-level API with automatic subprocess management and real protocol interactions
- **Production Patterns**: Error handling, state management, concurrent processing
- **Schema Compliance**: 100% MCP 2024-11-05 specification compliance

### Planned Features
- **Extended MCP Capabilities**: Streaming, notifications, and progress tracking
- **CLI Tools**: Command-line utilities for AI workflows  
- **API Integrations**: Notion, Slack, and other service connectors
- **Python Bindings**: FFI bindings for broader ecosystem compatibility
- **AI Agent Framework**: Tools for building autonomous AI agents
- **Plugin System**: Extensible architecture for custom AI tools

## Development Workflow

### Workspace Development
1. **Feature Development**: Create new crates in `crates/` directory
2. **Dependency Management**: Shared dependencies in workspace root
3. **Testing Strategy**: Each crate maintains its own test suite
4. **Documentation**: Crate-level and workspace-level documentation
5. **AI-Assisted Coding**: Leverage `.copilot/` configurations for consistent development

### Adding New Workspace Members
```bash
# Create new crate
cargo new --lib crates/new-crate-name

# Add to workspace Cargo.toml
[workspace]
members = [
    "crates/airs-mcp",
    "crates/airs-memspec", 
    "crates/new-crate-name",  # Add here
]
```

### Branch Strategy
- **Main Branch**: Stable, production-ready code
- **Feature Branches**: Individual feature development
- **Crate-Specific**: Features can be developed per crate independently

## Coding Standards

### Rust Best Practices
- **Formatting**: Use `cargo fmt` for consistent code formatting
- **Linting**: Address all `cargo clippy` warnings
- **Documentation**: Comprehensive rustdoc comments for public APIs
- **Error Handling**: Proper `Result` and `Option` usage
- **Memory Safety**: Leverage Rust's ownership system effectively

### Project Conventions
- **Naming**: Use clear, descriptive names following Rust conventions
- **Modules**: Organize code into logical modules and sub-modules
- **Visibility**: Minimize public API surface, expose only necessary items
- **Dependencies**: Prefer minimal, well-maintained crates
- **Testing**: Unit tests alongside code, integration tests in `tests/` directory

## Testing

### Testing Framework
- **Unit Tests**: In-module tests using `#[cfg(test)]`
- **Integration Tests**: Separate `tests/` directory for each crate
- **Documentation Tests**: Embedded examples in rustdoc comments
- **Workspace Testing**: `cargo test --workspace` for comprehensive testing

### Test Commands
```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p airs-mcp

# Test with output
cargo test -- --nocapture

# Test specific example
cd crates/airs-mcp/examples/simple-mcp-client
cargo test
```

### Coverage and Quality
- Aim for comprehensive test coverage of public APIs
- Test error conditions and edge cases
- Use property-based testing where appropriate
- Benchmark performance-critical code

## Contributing

This is currently a personal project, but contributions and suggestions are welcome!

### How to Contribute
1. **Issues**: Report bugs or suggest features via GitHub Issues
2. **Discussions**: Join conversations about architecture and design
3. **Code**: Fork the repository and submit pull requests

### Contribution Guidelines
- Follow the established coding standards
- Include tests for new functionality
- Update documentation as needed
- Respect the AI-assisted development philosophy

## License

Licensed under either of:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution License
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Support and Resources

- **Documentation**: Comprehensive rustdoc available via `cargo doc --open`
- **Issues**: GitHub Issues for bug reports and feature requests
- **Discussions**: GitHub Discussions for questions and ideas
- **AI Development**: Custom Copilot configurations in `.copilot/`

---

*Built with 🦀 Rust and enhanced by AI-assisted development*