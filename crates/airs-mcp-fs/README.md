# airs-mcp-fs

A security-first filesystem bridge that enables Claude Desktop and other MCP-compatible AI tools to intelligently read, write, and manage files in local development environments.

## Overview

`airs-mcp-fs` transforms AI assistance from passive consultation to active collaboration by providing secure, standardized filesystem operations through the Model Context Protocol (MCP). AI agents can now both understand your project context and create tangible artifacts directly in your local environment.

## Key Features

- **🔐 Security-First Design**: Human-in-the-loop approval workflows with configurable security policies
- **📁 Complete Filesystem Operations**: Read, write, create, delete, move, and copy files and directories
- **🖼️ Advanced Binary Processing**: Intelligent handling of images, PDFs, and other binary formats
- **⚡ High Performance**: Sub-100ms response times with efficient memory management
- **🔧 AIRS Ecosystem Integration**: Seamless compatibility with other AIRS MCP tools
- **🛡️ Enterprise-Grade Security**: Path validation, audit logging, and threat detection

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

### Configuration Setup

**Step 1: Generate Configuration Files**
```bash
# Generate development configuration
airs-mcp-fs generate-config

# Generate for specific environment
airs-mcp-fs generate-config --env production --output ~/.config/airs-mcp-fs

# Generate with custom output directory
airs-mcp-fs generate-config --output ./config --env staging
```

**Step 2: Customize Your Configuration**
Edit the generated configuration file to match your needs:
```toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",          # Your development projects
    "~/Documents/**/*.md",      # Documentation files
]

[security.operations]
read_allowed = true
write_requires_policy = false   # Set to true for production
delete_requires_explicit_allow = true
```

**Step 3: Test Configuration**
```bash
cargo run --example configuration_demo
```

### Claude Desktop Integration

Add to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development"
      }
    }
  }
}
```

### Basic Usage

Once connected, you can interact with your filesystem through Claude Desktop:

```
User: "Read my package.json file and analyze the dependencies"
Claude: *uses read_file tool* → analyzes dependencies → provides insights

User: "Create a new React component for user authentication"
Claude: *uses write_file tool* → generates component → saves to src/components/Auth.tsx

User: "Optimize all images in my assets folder for web"
Claude: *processes images* → resizes and compresses → saves optimized versions
```

## Core Capabilities

### File Operations
- **read_file**: Read text and binary files with automatic encoding detection
- **write_file**: Create or update files with approval workflows
- **list_directory**: Browse filesystem with metadata and filtering
- **create_directory**: Create directory structures recursively
- **delete_file/delete_directory**: Safe deletion with confirmation
- **move_file/copy_file**: File manipulation with atomic operations

### Binary File Processing
- **Image Support**: JPEG, PNG, GIF, WebP, TIFF, BMP with resizing and thumbnails
- **PDF Processing**: Text extraction, image extraction, metadata parsing
- **Format Detection**: Magic number-based file type identification
- **EXIF Metadata**: Camera information, GPS coordinates, timestamps
- **Compression**: Automatic optimization for large file transfers

### Security Features
- **Human Approval**: Interactive confirmation for write/delete operations
- **Path Validation**: Prevent directory traversal and unauthorized access
- **Access Control**: Configurable allowlists and denylists for file paths
- **Audit Logging**: Comprehensive operation tracking for compliance
- **Threat Detection**: Basic malware scanning and suspicious file identification

## Configuration

### Basic Configuration

Create `.airs-mcp-fs.toml` in your project root:

```toml
[security]
# Paths where read operations are allowed
allowed_read_paths = [
    "~/Documents/**",
    "~/Desktop/**",
    "./**"
]

# Paths where write operations are allowed
allowed_write_paths = [
    "~/Documents/**",
    "./src/**",
    "./docs/**"
]

# File patterns to never access
forbidden_patterns = [
    "\\.env$",
    "\\.ssh/.*",
    ".*\\.key$",
    ".*password.*"
]

# File size limits (in MB)
max_file_size_mb = 100
max_binary_size_mb = 50

# Approval requirements
require_approval_for_writes = true
require_approval_for_deletes = true

[performance]
max_concurrent_operations = 10
cache_size_mb = 50
```

### Global Configuration

System-wide settings at `~/.config/airs-mcp-fs/config.toml`:

```toml
[server]
name = "airs-mcp-fs"
version = "1.0.0"
transport = "stdio"

[logging]
level = "info"
file = "~/.config/airs-mcp-fs/logs/airs-mcp-fs.log"
max_size_mb = 100

[security]
enable_threat_detection = true
scan_binary_files = true
```

## Use Cases

### Development Workflow Enhancement
- **Code Analysis**: "Analyze all TypeScript files for potential performance issues"
- **Documentation Generation**: "Create API docs from my OpenAPI specification"
- **Automated Refactoring**: "Convert React class components to functional components"
- **Project Setup**: "Create a new Next.js project structure with best practices"

### Content & Asset Management
- **Image Optimization**: "Resize all product images to 800px width and generate thumbnails"
- **Document Processing**: "Extract text from all PDF reports in the reports/ folder"
- **File Organization**: "Organize my Downloads folder by file type and date"
- **Backup Creation**: "Create a backup of essential project files"

### Research & Analysis
- **Data Extraction**: "Extract metadata from all images for a photo catalog"
- **Content Analysis**: "Analyze the structure and content of technical documentation"
- **Format Conversion**: "Convert all PNG images to optimized WebP format"
- **Duplicate Detection**: "Find and organize duplicate files across directories"

## Security Best Practices

### Recommended Security Settings
- Always enable approval workflows for write operations
- Use restrictive path allowlists for sensitive environments
- Regularly review audit logs for suspicious activity
- Keep forbidden patterns updated with sensitive file types
- Enable threat detection for binary file processing

### Path Security
- Use absolute paths in configuration when possible
- Avoid wildcard patterns that might expose sensitive directories
- Regularly audit allowed paths and remove unnecessary access
- Monitor for path traversal attempts in logs

### Binary File Safety
- Enable binary file scanning for malware detection
- Set reasonable file size limits to prevent resource exhaustion
- Use sandboxed processing for untrusted binary files
- Validate file formats before processing

## Advanced Features

### Binary Processing Options
```toml
[binary_processing.image]
generate_thumbnails = true
max_dimension = 1920
compression_quality = 85
extract_metadata = true

[binary_processing.pdf]
extract_text = true
extract_images = true
process_tables = true
```

### Plugin System
Extend functionality with custom file processors:
- Register custom format handlers
- Add domain-specific processing logic
- Integrate with external tools and services
- Create organization-specific workflows

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
- Check file system permissions for the target path
- Verify path is included in allowed_read_paths or allowed_write_paths
- Ensure no forbidden patterns match the file path

#### "File Too Large" Errors
- Adjust max_file_size_mb in configuration
- Use streaming operations for very large files
- Consider processing files in smaller chunks

#### "Approval Required" Prompts
- Respond to interactive approval prompts in terminal
- Configure approval settings in security section
- Review operation details before approving

#### Binary Processing Failures
- Verify file format is supported
- Check available memory for large files
- Enable debug logging for detailed error information

### Debug Mode
```bash
RUST_LOG=debug airs-mcp-fs --config ./debug-config.toml
```

### Log Analysis
- Check `~/.config/airs-mcp-fs/logs/` for detailed operation logs
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

## Roadmap

### Short-term (v1.1-1.2)
- Enhanced video file metadata extraction
- Advanced archive file processing (ZIP, TAR, RAR)
- OCR text extraction from images
- Version control integration (Git-aware operations)

### Medium-term (v1.3-1.5)
- Cloud storage integration (Dropbox, Google Drive)
- Collaborative multi-user features
- Advanced content analysis and similarity detection
- Mobile device synchronization

### Long-term (v2.0+)
- AI-powered file organization recommendations
- Cross-platform filesystem synchronization
- Enterprise SSO and advanced compliance features
- Plugin marketplace and community extensions

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

## Support

- **Documentation**: [Full documentation and guides](https://docs.airs.dev/mcp-fs)
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