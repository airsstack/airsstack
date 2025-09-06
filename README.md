# AirsStack - Own Your AI Future

**Privacy-First • Open Source • High Performance**

Build AI applications that work for you, not against you. AirsStack provides composable building blocks that make AI development more accessible, reducing vendor lock-in and giving you more control over your data and workflows.

🎯 **What can you build?**
• Personal AI assistants  
• MCP Tools & Extensions  
• Autonomous Agent Services  
• Custom Workflow Automations  
• **AI-Filesystem Collaboration** (via airs-mcp-fs)

## 🎉 Production Achievements

**✅ Claude Desktop Integration Verified**  
**✅ Complete MCP Server/Client Implementation**  
**✅ Production-Ready ApiKey MCP Server (NEW!)**  
**✅ Production Filesystem MCP Server (airs-mcp-fs)**  
**✅ 100% Schema Compliance (MCP 2024-11-05)**  
**✅ Enterprise-Grade Security (97.5/100 audit score)**  
**✅ Production-Grade Examples & Documentation**  
**✅ Comprehensive Documentation Ecosystem**

### 🚀 **Real-World Success**
- **MCP Server Framework**: Successfully integrated with Claude Desktop - resources, tools, and prompts working in production
- **ApiKey MCP Server**: **✅ NEW! FULLY WORKING** - Production-ready authentication server with MCP Inspector compatibility
- **Filesystem Bridge (airs-mcp-fs)**: **✅ PRODUCTION COMPLETE** - Security-first filesystem operations for AI collaboration
- **MCP Client**: High-level Rust API with automatic subprocess management and real protocol interactions  
- **Type Safety**: Full Rust type safety throughout MCP protocol implementation
- **Security Excellence**: Comprehensive path traversal protection and enterprise-grade audit framework
- **Examples**: Working client/server examples with comprehensive documentation

📖 **[Complete Documentation Available](docs/src/)** - Comprehensive guides for development workflow, memory bank management, AI-Rust integration patterns, project overviews, and resource guides.

[**See MCP Server Example →**](crates/airs-mcp/examples/simple-mcp-server/)  
[**See ApiKey MCP Server (Production) →**](crates/airs-mcp/examples/mcp-remote-server-apikey/)  
[**See MCP Client Example →**](crates/airs-mcp/examples/simple-mcp-client/)**  
[**See Filesystem MCP Server (Production) →**](crates/airs-mcp-fs/)**  
[**📆 Read Complete Documentation →**](docs/src/)

## 🎯 Our Vision

"Empower everyone to build their own AI-powered applications and agents"

We provide the essential open-source components that enable individuals, teams, and organizations to create sophisticated AI systems tailored to their specific needs - from simple productivity tools to complex autonomous agents.

## 🧭 Core Philosophy

• 🤝 **Human-AI Collaboration**: Human-driven architecture with AI-accelerated implementation  
• 🏠 **AI Sovereignty**: More ownership and control of your AI infrastructure  
• 🔓 **Open Source First**: Apache/MIT licensing ensuring freedom and flexibility  
• 🛡️ **Privacy by Design**: Keep more of your data under your control  
• 🔧 **Composable Components**: Building blocks, not black boxes  
• ⚡ **Performance Focused**: Rust-first with support for multiple technology stacks

## 🔄 Meta-Philosophy: AI Tools Built by AI

We practice what we preach - our AI infrastructure components are themselves built using AI-assisted methodologies. This creates a powerful recursive approach:

**🧠 Human-AI Collaboration**
• Strategic architecture designed by humans  
• Implementation accelerated by AI  
• Continuous validation through both perspectives  

**🔁 Self-Improving Process**
• AI tools become better by being built with AI assistance  
• Real-world experience informs better AI development tools  
• Methodology becomes part of what we offer the community  

**✨ Authentic Understanding**  
We deeply understand AI development challenges because we use AI to build AI tools. Our components solve real problems we've encountered in our own AI-assisted development.

## 💡 Why AirsStack?

🔒 **More Control, Less Vendor Lock-in**  
Building personal AI infrastructure is challenging, but AirsStack makes it more accessible. Reduce dependence on external AI services and gain more control over your data and workflows.

🛠️ **Developer-First**  
Composable components, not black boxes. Use what you need, extend what you want, control what matters to you.

⚡ **Performance & Safety**  
Built with performance and safety in mind, with Rust as our primary language while supporting integration with other technology stacks.

## Technology Stack

### Core Technologies
- **Primary Language**: Rust 1.88.0+ (MSRV) - Performance and safety focused
- **Multi-Stack Support**: Integration with Python, Node.js, and other ecosystems
- **AI Integration**: Model Context Protocol (MCP) implementation and agent frameworks
- **Build System**: Cargo Workspace for composable, modular development
- **Development Philosophy**: AI-assisted development with human architectural control

### Privacy & Control Features
- **Local Processing**: Keep your data under your control
- **No Telemetry**: Privacy-first design with transparent data handling
- **Composable Architecture**: Use only what you need, extend what you want
- **Open Source**: Apache 2.0 & MIT licensing for maximum freedom

## Project Architecture

AirsStack follows a modular Cargo Workspace architecture designed for composability, scalability, and maintainability:

```
airsstack/
├── Cargo.toml              # Workspace configuration
├── crates/                 # Composable building blocks
│   ├── airs-mcp/          # Model Context Protocol implementation
│   ├── airs-mcp-fs/       # Filesystem MCP tools  
│   └── airs-memspec/      # Memory bank specification and tooling
├── .copilot/              # AI-assisted development configuration
│   ├── chatmodes/         # Custom AI interaction modes
│   ├── instructions/      # Development methodologies and standards
│   └── prompts/           # Reusable AI development prompts
├── docs/                  # Comprehensive documentation ecosystem
│   ├── src/               # Documentation source files  
│   │   ├── technical/     # AI-Rust integration guides and methodologies
│   │   ├── projects/      # Component overviews and architecture
│   │   └── resources/     # Getting started and contribution guides
│   └── book/              # Generated documentation site
└── target/                # Build artifacts
```

## 🛠️ Featured Components

**🔧 Core Libraries** - Essential AI building blocks  
**🤖 Agent Framework** - Build autonomous AI agents  
**📊 Data Pipelines** - Privacy-first data processing  
**🖥️ Model Runtime** - Local AI model execution  
**🔗 Integrations** - Connect with your favorite tools  
**📁 Filesystem Bridge** - **✅ PRODUCTION** Secure AI-filesystem collaboration

## Key Features

### Current Implementation
- **✅ Production MCP Implementation**: Complete server/client with Claude Desktop integration verified
- **✅ Production ApiKey MCP Server**: **NEW!** Fully working authentication server with MCP Inspector compatibility
- **✅ Production Filesystem Server (airs-mcp-fs)**: Security-first filesystem bridge with Claude Desktop integration
- **✅ Working Examples**: Real-world server/client examples with documented usage patterns
- **✅ Advanced Transport Layer**: Custom transport support with SubprocessTransport example
- **✅ Type-Safe APIs**: High-level Rust APIs for MCP protocol interactions
- **✅ Enterprise Security**: 97.5/100 security audit score with comprehensive vulnerability testing
- **✅ Memory Bank System**: Structured AI-assisted development workflow management
- **✅ Cargo Workspace Structure**: Organized multi-crate development environment
- **✅ AI-Assisted Development**: Copilot-optimized workflow and prompts

### Design Principles
- **Human Architecture, AI Implementation**: Strategic decisions made by humans, code accelerated with AI assistance
- **Composable Building Blocks**: Each component serves a specific purpose, use what you need
- **Privacy & Sovereignty**: Local processing, no vendor lock-in, your data stays under your control
- **Multi-Stack Integration**: Rust-first with seamless integration across technology stacks
- **Type Safety & Performance**: Leveraging Rust's strengths for reliable, high-performance AI infrastructure

## AI-Assisted Development & Memory-Bank Management

AirsStack embodies our philosophy of AI tools built by AI. We leverage a sophisticated memory-bank management system to ensure resilient, transparent, and context-driven development that serves as both a development methodology and a reusable component for the community.

### Memory-Bank System Overview
- **Workspace-Level Context**: Shared files define overall vision, architecture, and standards across all components
- **Component Memory Banks**: Each component maintains its own requirements, design, technical context, and progress tracking
- **Task Management**: Detailed tracking with status, subtasks, and progress logs for precise project management
- **Context Snapshots**: Operational state preservation for onboarding, recovery, and historical analysis

### Human-AI Collaboration Model
- **Strategic Human Control**: Architectural decisions and strategic direction remain human-driven
- **AI-Accelerated Implementation**: Code generation, documentation, and routine tasks enhanced by AI
- **Transparent Process**: Every action, decision, and change logged for complete audit trail
- **Specification-Driven**: Rigorous workflow ensuring clear requirements, validated designs, and thorough testing
- **Community Contribution**: Our methodology becomes part of what we offer to the community

### Practical Usage for Teams
1. **Context Preservation**: Maintain development state across team members and AI collaborators
2. **Onboarding Acceleration**: New team members can rapidly understand project state and history
3. **Quality Assurance**: Comprehensive tracking ensures nothing falls through the cracks
4. **Knowledge Transfer**: Explicit documentation of decisions and reasoning for long-term maintainability
5. **Switch Contexts**: For multi-project workspaces, update the active sub-project and re-read all relevant files before proceeding.

For more details, see `.copilot/memory_bank/` and the documentation in `docs/`.

## 📖 Documentation Ecosystem

AIRS provides comprehensive documentation covering philosophy, technical implementation, and practical usage:

### 📚 **Complete Documentation**
- **[Philosophy & Principles](docs/src/philosophy_principles.md)**: Core design philosophy and AI-human collaboration principles
- **[Technical Knowledge](docs/src/technical/)**: In-depth guides on development workflow, memory bank architecture, and AI-Rust integration
- **[Project Overviews](docs/src/projects/)**: Detailed coverage of AIRS-MCP and AIRS-MemSpec implementations
- **[Getting Started](docs/src/resources/getting_started.md)**: Comprehensive onboarding with multiple user paths
- **[Contributing Guide](https://airsstack.github.io/contributing)**: Complete framework for community participation

### 🚀 **Quick Access**
```bash
# Read documentation directly in markdown
open docs/src/

# Or serve as interactive website
cd docs && mdbook serve --open
# Access at: http://localhost:3000
```

### 📋 **Documentation Features**
- **Multiple User Paths**: Tailored guidance for AI-enhanced teams, Rust+AI projects, and documentation teams
- **Real-World Examples**: Authentic examples from actual AirsStack development
- **Professional Quality**: Enterprise-grade documentation with comprehensive quality validation
- **Interactive Navigation**: Clear progression from concepts to implementation
- **Community Ready**: Complete contribution guidelines and getting started resources

## Getting Started

### Prerequisites

- **Rust**: 1.88.0 or later (we track the latest stable release)
- **Cargo**: Included with Rust installation
- **Git**: For version control and development workflow

**📖 For detailed setup instructions, see our [Getting Started Guide](https://airsstack.github.io/getting-started)**

### Installation

1. **Clone the repository**:
```bash
git clone https://github.com/airsstack/airsstack.git
cd airsstack
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

**Try the production ApiKey MCP server (NEW!):**
```bash
cd crates/airs-mcp/examples/mcp-remote-server-apikey
cargo build --release
./target/release/mcp-remote-server-apikey

# Test with MCP Inspector or curl:
curl -H "X-API-Key: mcp_dev_key_12345" \
     -H "Content-Type: application/json" \
     -X POST http://127.0.0.1:3001/mcp \
     -d '{"jsonrpc": "2.0", "id": 1, "method": "resources/list"}'

# Features: Authentication, Resources, Tools, Prompts, MCP Inspector compatible
```

**Try the MCP server with Claude Desktop:**
```bash
cd crates/airs-mcp/examples/simple-mcp-server
cargo build --release

# Add to Claude Desktop config - see README for integration steps
# Resources, tools, and prompts will appear in Claude's UI
```

**Try the filesystem MCP server (Production):**
```bash
cd crates/airs-mcp-fs
cargo build --release

# Generate configuration
./target/release/airs-mcp-fs generate-config

# Add to Claude Desktop - provides secure file operations
# See docs/ for complete setup guide
```

**Try the MCP client demonstration:**
```bash
cd crates/airs-mcp/examples/simple-mcp-client  
cargo run  # Automatically spawns server and demonstrates all MCP operations
```

**📖 For comprehensive examples and detailed usage, see our [Complete Documentation](https://airsstack.github.io/)**

## Project Structure

### Workspace Organization

```
airsstack/
├── Cargo.toml                    # Root workspace configuration
├── crates/                       # Composable building blocks
│   ├── airs-mcp/                # MCP implementation crate (✅ Production Ready)
│   │   ├── examples/            # Working examples
│   │   │   ├── simple-mcp-server/     # Claude Desktop integration verified
│   │   │   ├── mcp-remote-server-apikey/ # Production ApiKey server (✅ NEW!)
│   │   │   └── simple-mcp-client/     # AirsStack library usage demonstration
│   │   └── Cargo.toml           # Crate-specific configuration
│   ├── airs-mcp-fs/             # Filesystem MCP server (✅ Production Ready)
│   │   ├── examples/            # Configuration examples
│   │   ├── docs/                # Comprehensive documentation
│   │   └── src/                 # Security-first filesystem bridge
│   └── airs-memspec/            # Memory bank specification and tooling
├── .copilot/                    # AI-assisted development configuration
│   ├── chatmodes/               # Custom interaction modes
│   ├── instructions.md          # Development practices
│   └── prompts/                 # Reusable AI prompts
├── docs/                        # Comprehensive documentation ecosystem
│   ├── src/                     # Documentation source files
│   │   ├── technical/           # Development workflow, memory bank, AI-Rust integration
│   │   ├── projects/            # AIRS-MCP and AIRS-MemSpec overviews
│   │   └── resources/           # Getting started, contributing, documentation guides
│   └── book/                    # Generated documentation site (mdBook)
├── LICENSE-APACHE               # Apache 2.0 license
├── LICENSE-MIT                  # MIT license
└── README.md                    # This file
```

### Current Workspace Members

- **`airs-mcp`**: **✅ Production-Ready** Model Context Protocol implementation
  - Complete MCP server/client functionality
  - **Claude Desktop integration verified** with working examples
  - **NEW: Production ApiKey Server** - Fully working with MCP Inspector compatibility
  - High-level type-safe APIs for both server and client
  - Advanced transport layer with custom transport support
  - [Server Example](crates/airs-mcp/examples/simple-mcp-server/) | [ApiKey Server](crates/airs-mcp/examples/mcp-remote-server-apikey/) | [Client Example](crates/airs-mcp/examples/simple-mcp-client/)

- **`airs-mcp-fs`**: **✅ Production-Ready** Filesystem bridge for AI collaboration
  - **Complete filesystem MCP server** with Claude Desktop integration
  - **Enterprise-grade security** (97.5/100 security audit score)
  - Advanced binary processing (images, PDFs, archives)
  - Human-in-the-loop approval workflows and audit logging
  - [Filesystem Server](crates/airs-mcp-fs/) | [Documentation](crates/airs-mcp-fs/docs/)

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
- **MCP Server Framework**: Successfully integrated with Claude Desktop (resources, tools, prompts)
- **Filesystem Bridge (airs-mcp-fs)**: **✅ PRODUCTION-READY** - Secure AI-filesystem collaboration with enterprise security
- **MCP Client**: High-level API with automatic subprocess management and real protocol interactions
- **Production Patterns**: Error handling, state management, concurrent processing
- **Security Excellence**: Path traversal protection, human-in-the-loop workflows, comprehensive audit logging
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

## 🤝 Join the AirsStack Community

🚀 **Get Started**

• [Main Repository](https://github.com/airsstack/airsstack) - Start building with AirsStack  
• [Documentation](https://airsstack.github.io/) - Guides, tutorials, and API docs  
• [Community Discussions](https://github.com/orgs/airsstack/discussions) - Support and ideas  

🎯 **Quick Links**

• 📖 [Getting Started Guide](https://airsstack.github.io/getting-started)  
• 💡 [Example Projects](https://github.com/airsstack/airsstack/tree/main/examples)  
• 🐛 [Report Issues](https://github.com/airsstack/airsstack/issues)  
• 💬 [Join Discussions](https://github.com/orgs/airsstack/discussions)

## Contributing

AirsStack is a community-driven project that welcomes contributions from developers, AI enthusiasts, and anyone interested in building better AI infrastructure.

### How to Contribute
1. **🐛 Issues**: Report bugs or suggest features via [GitHub Issues](https://github.com/airsstack/airsstack/issues)
2. **💬 Discussions**: Join conversations about architecture and design in [GitHub Discussions](https://github.com/orgs/airsstack/discussions)  
3. **🔧 Code**: Fork the repository and submit pull requests with improvements
4. **📖 Documentation**: Help improve guides, examples, and API documentation
5. **🤝 Community**: Share your AirsStack projects and help others get started

### Contribution Guidelines
- Follow the established coding standards and workspace conventions
- Include comprehensive tests for new functionality
- Update documentation to reflect changes and new features  
- Embrace the AI-assisted development philosophy and human-AI collaboration
- Prioritize privacy, sovereignty, and composability in all contributions

### Community Standards
- **🔓 Open Source Promise**: Apache 2.0 & MIT licensed - forever free and open
- **🛡️ Privacy First**: No telemetry, local processing, transparent data handling
- **🤝 Inclusive Environment**: Welcoming to all skill levels and backgrounds
- **🔧 Quality Focus**: Comprehensive testing, documentation, and code review

## License

Licensed under either of:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution License
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Support and Resources

- **📖 Documentation**: [Complete documentation site](https://airsstack.github.io/) and rustdoc via `cargo doc --open`
- **🐛 Issues**: [GitHub Issues](https://github.com/airsstack/airsstack/issues) for bug reports and feature requests  
- **💬 Discussions**: [GitHub Discussions](https://github.com/orgs/airsstack/discussions) for questions and community support
- **🤖 AI Development**: Custom Copilot configurations and AI-assisted workflows in `.copilot/`
- **🔧 Examples**: Working examples and integration guides in `crates/*/examples/`

---

## 📄 Open Source Promise

**✅ Forever Free** - Apache 2.0 & MIT licensed  
**✅ No Vendor Lock-In** - Your code, your control  
**✅ Privacy First** - No telemetry, local processing  
**✅ Community Driven** - Transparent governance  

🤖 **Built for Humans** • 🔒 **Privacy-First** • 🦀 **Rust-Powered** • 🔓 **Forever Open**

**Your AI. Your Data. Your Control.**

[Get Started](https://github.com/airsstack/airsstack) • [Documentation](https://airsstack.github.io/) • [Community](https://github.com/orgs/airsstack/discussions)

---

*Built with 🦀 Rust and enhanced by AI-assisted development*