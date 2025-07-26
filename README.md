# AIRS - AI & Rust Technology Stack

A personal AI technology stack built entirely in Rust, designed as a foundational framework for AI engineering tools and software. AIRS emphasizes type safety, performance, and clean architecture for building AI-powered applications with human-designed architecture and AI-assisted implementation.

## Technology Stack

### Core Technologies
- **Language**: Rust 1.84.0+ (MSRV)
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
│   └── airs-mcp/          # Model Context Protocol implementation
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

## Getting Started

### Prerequisites

- **Rust**: 1.84.0 or later (we track the latest stable release)
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

3. **Run tests**:
```bash
cargo test
```

4. **Check all workspace members**:
```bash
cargo check --workspace
```

### Quick Start

```bash
# Build specific crate
cargo build -p airs-mcp

# Run with specific features
cargo run --bin airs-mcp --features "feature-name"

# Development build with optimizations
cargo build --release
```

## Project Structure

### Workspace Organization

```
airs/
├── Cargo.toml                    # Root workspace configuration
├── crates/                       # All workspace members
│   └── airs-mcp/                # MCP implementation crate
│       ├── Cargo.toml           # Crate-specific configuration
│       ├── src/                 # Source code
│       └── README.md            # Crate documentation
├── .copilot/                    # AI development tools
│   ├── chatmodes/               # Custom interaction modes
│   ├── instructions.md          # Development practices
│   └── prompts/                 # Reusable AI prompts
├── docs/                        # Additional documentation
├── LICENSE-APACHE               # Apache 2.0 license
├── LICENSE-MIT                  # MIT license
└── README.md                    # This file
```

### Current Workspace Members

- **`airs-mcp`**: Model Context Protocol implementation *(in development)*
  - Core MCP functionality
  - AI model integration protocols
  - Communication interfaces

## Key Features

### Current Implementation
- **Cargo Workspace Structure**: Organized multi-crate development environment
- **MCP Foundation**: Building blocks for Model Context Protocol integration
- **AI-Assisted Development**: Copilot-optimized workflow and prompts
- **Type-Safe Architecture**: Rust's type system for reliable AI tools

### Planned Features
- **MCP Server/Client**: Complete Model Context Protocol implementation
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
cargo test

# Test specific crate
cargo test -p airs-mcp

# Test with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
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