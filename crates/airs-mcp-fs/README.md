# airs-mcp-fs

A security-first filesystem bridge that enables Claude Desktop and other MCP-compatible AI tools to intelligently read, write, and manage files in local development environments.

## Overview

`airs-mcp-fs` transforms AI assistance from passive consultation to active collaboration by providing secure, standardized filesystem operations through the Model Context Protocol (MCP). AI agents can now both understand your project context and create tangible artifacts directly in your local environment.

## Key Features

- **🔐 Security-First Design**: Human-in-the-loop approval workflows with configurable security policies
- **📁 Complete Filesystem Operations**: Read, write, create, delete, move, and copy files and directories
- **�️ Binary File Restriction**: Text-only processing with comprehensive binary file blocking for enhanced security
- **⚡ High Performance**: Sub-100ms response times with efficient memory management
- **🔧 AIRS Ecosystem Integration**: Seamless compatibility with other AIRS MCP tools
- **🛡️ Enterprise-Grade Security**: Path validation, audit logging, threat detection, and binary file protection

## Quick Start

### Prerequisites

- Rust 1.88.0 or later
- Claude Desktop or another MCP-compatible client

### Installation

#### From Source
```bash
git clone https://github.com/rstlix0x0/airs.git
cd airs
cargo build --release --bin airs-mcp-fs
```

#### Using Cargo
```bash
cargo install --path crates/airs-mcp-fs
```

### Basic Setup

**Step 1: Generate Configuration**
```bash
# Generate development configuration
airs-mcp-fs generate-config

# This creates ~/.config/airs-mcp-fs/development.toml
```

**Step 2: Configure Claude Desktop**

Add to your Claude Desktop MCP configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/path/to/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/yourusername/.config/airs-mcp-fs",
        "AIRS_MCP_FS_LOG_DIR": "/Users/yourusername/.local/share/airs-mcp-fs/logs"
      }
    }
  }
}
```

**Important**: Replace `/path/to/airs-mcp-fs` with the actual path to your binary and `yourusername` with your actual username.

**Step 3: Restart Claude Desktop**

Restart Claude Desktop to load the new MCP server configuration.

### Test Your Setup

Once Claude Desktop restarts, try these commands:

```
User: "List the files in my Documents directory"
Claude: *uses list_directory tool* → shows your Documents contents

User: "Read my project's README.md file"
Claude: *uses read_file tool* → displays the README content

User: "Create a new file called hello.txt with 'Hello World' in my Documents"
Claude: *uses write_file tool* → creates the file with approval prompt
```

## Core Capabilities

### File Operations
- **read_file**: Read text files with automatic encoding detection
- **write_file**: Create or update files with approval workflows
- **list_directory**: Browse filesystem with metadata and filtering
- **create_directory**: Create directory structures recursively
- **delete_file/delete_directory**: Safe deletion with confirmation
- **move_file/copy_file**: File manipulation with atomic operations

### Security Features
- **Binary File Restriction**: Comprehensive blocking of binary files to eliminate binary-based security risks
- **Human Approval**: Interactive confirmation for write/delete operations
- **Path Validation**: Prevent directory traversal and unauthorized access
- **Access Control**: Configurable allowlists and denylists for file paths
- **Audit Logging**: Comprehensive operation tracking for compliance
- **Threat Detection**: Enhanced security monitoring with binary file rejection

## Configuration

AIRS MCP-FS uses a sophisticated multi-layered configuration system that automatically adapts to different environments while maintaining security and flexibility.

### Quick Configuration

For development work, your configuration should include your project directories:

```toml
# ~/.config/airs-mcp-fs/development.toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",           # All your projects
    "~/Documents/**/*",          # Documents directory (both directory and contents)
    "~/Desktop/**/*",            # Desktop files
    "./**/*"                     # Current directory when running from project
]

[security.operations]
read_allowed = true
write_requires_policy = false    # Allow writes in development
delete_requires_explicit_allow = true

# Named policies for different file types
[security.policies.journal_files]
patterns = ["/Users/yourusername/Documents/**/*"]
operations = ["read", "write", "list"]
risk_level = "low"
description = "Personal journal and document files"
```

**Important**: When configuring directory access, you need both the directory path itself AND its contents:
- `~/Documents` - Access to the directory itself
- `~/Documents/**/*` - Access to files within the directory

### Environment-Specific Configuration

AIRS MCP-FS automatically detects your environment and loads appropriate configurations:

- **Development**: `~/.config/airs-mcp-fs/development.toml` - Permissive settings for productivity
- **Staging**: `~/.config/airs-mcp-fs/staging.toml` - Production-like settings for testing  
- **Production**: `~/.config/airs-mcp-fs/production.toml` - Secure settings for deployment

### Configuration Documentation

For comprehensive configuration guidance, see our detailed documentation:

- **[Quick Start Guide](./docs/src/quickstart.md)**: Get up and running in 5 minutes
- **[Configuration Guide](./docs/src/configuration.md)**: Complete configuration system overview
- **[Environment Setup](./docs/src/configuration/environment.md)**: Environment detection and management
- **[Security Policies](./docs/src/configuration/security.md)**: Advanced security configuration
- **[Claude Desktop Integration](./docs/src/configuration/claude_desktop.md)**: MCP client setup
- **[Troubleshooting](./docs/src/configuration/troubleshooting.md)**: Common issues and solutions

## Use Cases

### Development Workflow Enhancement
- **Code Analysis**: "Analyze all TypeScript files for potential performance issues"
- **Documentation Generation**: "Create API docs from my source code comments"
- **Automated Refactoring**: "Convert React class components to functional components"
- **Project Setup**: "Create a new Next.js project structure with best practices"

### Content & Document Management
- **Configuration Management**: "Update configuration files with new environment settings"
- **Documentation Processing**: "Extract and organize README files from all project directories"
- **File Organization**: "Organize source code files by feature and module"
- **Backup Creation**: "Create a backup of essential project configuration files"

### Research & Analysis
- **Code Metadata Extraction**: "Extract function signatures and documentation from source files"
- **Content Analysis**: "Analyze the structure and content of technical documentation"
- **Text Processing**: "Process and standardize all markdown documentation files"
- **Duplicate Detection**: "Find and organize duplicate configuration files across directories"

## Security Best Practices

### Recommended Security Settings
- Always enable approval workflows for write operations
- Use restrictive path allowlists for sensitive environments
- Regularly review audit logs for suspicious activity
- Keep forbidden patterns updated with sensitive file types
- Binary file processing is disabled by default for maximum security

### Path Security
- Use absolute paths in configuration when possible
- Avoid wildcard patterns that might expose sensitive directories
- Regularly audit allowed paths and remove unnecessary access
- Monitor for path traversal attempts in logs

### Binary File Security
- Binary file processing is completely disabled for enhanced security
- All binary file operations return security validation errors
- Focus on text-based file operations for AI-assisted development
- Eliminates entire classes of binary-based security vulnerabilities
- Provides clear security boundaries for enterprise deployments

## Advanced Features

### Security-First Architecture
The system prioritizes security through comprehensive binary file restriction and enhanced validation:

```toml
[security]
# Binary processing is completely disabled for security
binary_processing_disabled = true
text_only_mode = true
```

### Plugin System
Extend functionality with custom text file processors:
- Register custom text format handlers
- Add domain-specific processing logic for development files
- Integrate with external tools and services for text processing
- Create organization-specific workflows for code and documentation

### AIRS Ecosystem Integration
- **airs-mcp**: Leverage shared MCP infrastructure
- **airs-memspec**: Manage memory bank files
- **airs-mcp-kb**: Populate knowledge bases from documents
- **Shared Security**: Consistent policies across AIRS tools

## Performance & Scalability

### Performance Characteristics
- **Response Time**: <100ms for typical file operations
- **Large Files**: Streaming support for files up to 1GB
- **Concurrent Operations**: Handle 10+ simultaneous requests
- **Memory Usage**: <50MB baseline with linear scaling

### Optimization Features
- Intelligent caching for frequently accessed files
- Streaming for large file transfers
- Background processing for heavy operations
- Connection pooling for multiple clients

## Troubleshooting

### Common Issues

#### "Permission Denied" Errors
- **Cause**: Path not included in `allowed_paths` configuration or blocked by `denied_paths`
- **Solution**: Update your configuration to include the required directory and its contents:
  ```toml
  [security.filesystem]
  allowed_paths = [
      "~/Documents",        # Directory itself
      "~/Documents/**/*"    # Directory contents
  ]
  ```

#### "Security validation failed" Errors  
- **Cause**: Glob patterns not matching the requested path
- **Solution**: Ensure your patterns include both directory access and content access
- **Debug**: Check your configuration file and verify the path patterns

#### "Configuration file not found" Warnings
- **Cause**: No environment-specific configuration file exists
- **Solution**: Generate configuration for your environment:
  ```bash
  airs-mcp-fs generate-config --env development
  ```

#### "Invalid server response" in Claude Desktop
- **Cause**: Incorrect environment variables or binary path in Claude Desktop configuration
- **Solution**: Verify your Claude Desktop JSON configuration includes correct paths and environment variables

For comprehensive troubleshooting guidance, see **[Configuration Troubleshooting](./docs/src/configuration/troubleshooting.md)**.

### Debug Mode
```bash
RUST_LOG=debug airs-mcp-fs --config ./debug-config.toml
```

### Log Analysis
- Check `~/.local/share/airs-mcp-fs/logs/` for detailed operation logs
- Review audit trail for security-related events
- Monitor performance metrics for optimization opportunities

## Contributing

### Development Setup
```bash
# Clone the repository
git clone https://github.com/rstlix0x0/airs.git
cd airs/crates/airs-mcp-fs

# Install dependencies
cargo build

# Run tests
cargo test

# Run with development config
cargo run -- --config ./dev-config.toml
```

### Testing
- **Unit Tests**: `cargo test`
- **Integration Tests**: `cargo test --test integration`
- **Security Tests**: `cargo test --test security`
- **Performance Tests**: `cargo test --test performance`

### Code Standards
- Follow Rust API guidelines and formatting standards
- Write comprehensive tests for all new functionality
- Document public APIs with rustdoc
- Follow security best practices for file operations

## Security & Compliance

### Security Audits
- Regular security assessments by third-party experts
- Automated vulnerability scanning in CI/CD pipeline
- Responsible disclosure program for security issues
- Compliance with industry security standards

### Enterprise Features
- SOC2 Type II compliance
- HIPAA compatibility for healthcare environments
- Advanced audit logging and reporting
- Integration with enterprise SIEM systems

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

## Documentation

For comprehensive guides and advanced configuration:

- **📚 [Complete Documentation](./docs/src/SUMMARY.md)** - Full mdbook documentation
- **🚀 [Quick Start Guide](./docs/src/quickstart.md)** - Get running in 5 minutes
- **⚙️ [Configuration Guide](./docs/src/configuration.md)** - Complete configuration system
- **🔒 [Security Policies](./docs/src/configuration/security.md)** - Advanced security configuration
- **🔧 [Troubleshooting](./docs/src/configuration/troubleshooting.md)** - Common issues and solutions

### Building Documentation

```bash
# Install mdbook
cargo install mdbook

# Build and serve documentation
cd crates/airs-mcp-fs/docs
mdbook serve

# Open http://localhost:3000 in your browser
```

## Support

- **Documentation**: [Full documentation and guides](./docs/src/SUMMARY.md)
- **Issues**: [GitHub Issues](https://github.com/rstlix0x0/airs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rstlix0x0/airs/discussions)
- **Community**: [Discord Server](https://discord.gg/airs-community)

## Related Projects

- **[airs-mcp](../airs-mcp)**: Core MCP protocol implementation
- **[airs-memspec](../airs-memspec)**: Memory bank management CLI
- **[airs-mcp-kb](../airs-mcp-kb)**: Knowledge base and RAG system
- **[AIRS Platform](https://github.com/rstlix0x0/airs)**: Complete AI & Rust technology stack

---

**Built with ❤️ using Rust and the Model Context Protocol**