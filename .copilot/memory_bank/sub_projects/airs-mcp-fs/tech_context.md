# Tech Context: AIRS MCP-FS

**Updated:** 2025-08-16  
**Status:** Foundation Phase - Ready for Implementation  
**Rust Version:** 1.88.0 or later

## Technology Stack

### Core Language & Runtime
- **Language**: Rust 2021 Edition
  - Memory safety without garbage collection
  - Zero-cost abstractions for performance
  - Strong type system for correctness
  - Excellent error handling with `Result<T, E>`

- **Async Runtime**: Tokio
  - High-performance async I/O for file operations
  - Task scheduling and concurrent operation management
  - Network transport support for MCP communication
  - Timer and timeout functionality

### MCP Foundation Dependencies
```toml
[dependencies]
# AIRS MCP Foundation
airs-mcp = { path = "../airs-mcp" }

# Core async runtime
tokio = { version = "1.0", features = ["full"] }

# JSON-RPC and serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

### Binary Processing Dependencies
```toml
[dependencies]
# Image processing
image = { version = "0.24", features = ["jpeg", "png", "gif", "webp", "tiff", "bmp"] }
imageproc = "0.23"  # Advanced image processing operations
exif = "0.2"        # EXIF metadata extraction

# PDF processing
pdf = "0.8"           # PDF parsing and structure analysis
pdf-extract = "0.7"   # Text and image extraction

# Format detection
infer = "0.15"        # Magic number-based file type detection
mime = "0.3"          # MIME type handling

# Compression and encoding
base64 = "0.21"       # Base64 encoding for binary data transfer
lz4_flex = "0.11"     # Fast compression for large files
```

### Security & Configuration Dependencies
```toml
[dependencies]
# Configuration management
config = "0.13"       # Hierarchical configuration loading
toml = "0.8"          # TOML configuration file parsing

# Path and filesystem utilities
path-clean = "1.0"    # Path canonicalization and cleaning
glob = "0.3"          # Pattern matching for path allowlists/denylists
walkdir = "2.0"       # Recursive directory traversal

# Security and validation
regex = "1.0"         # Pattern matching for forbidden files
uuid = { version = "1.0", features = ["v4"] } # Operation tracking

# Audit logging
tracing = "0.1"       # Structured logging
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] } # Timestamp handling
```

### Development & Testing Dependencies
```toml
[dev-dependencies]
# Testing framework
tokio-test = "0.4"    # Async testing utilities
tempfile = "3.0"      # Temporary file creation for tests
assert_fs = "1.0"     # Filesystem assertion utilities

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

# Test data generation
fake = "2.0"          # Fake data generation for testing
rand = "0.8"          # Random data for stress testing
```

## Development Environment Setup

### Required Tools
```bash
# Rust toolchain (1.88.0 or later)
rustup update stable
rustup default stable

# Development tools
cargo install cargo-watch    # File watching for development
cargo install cargo-audit    # Security vulnerability scanning
cargo install cargo-deny     # License and dependency checking
cargo install mdbook         # Documentation building
```

### Development Workflow
```bash
# Watch for changes during development
cargo watch -x check -x test -x "run -- --config ./dev-config.toml"

# Run comprehensive tests
cargo test --workspace
cargo test --test integration
cargo test --test security

# Performance benchmarking
cargo bench

# Documentation generation
mdbook build docs/
```

### Project Structure
```
crates/airs-mcp-fs/
├── src/
│   ├── lib.rs              # Public API and core types
│   ├── main.rs             # Binary entry point
│   ├── mcp/                # MCP server implementation
│   │   ├── mod.rs
│   │   ├── server.rs       # Main MCP server
│   │   ├── tools.rs        # Tool implementations
│   │   └── transport.rs    # STDIO transport handling
│   ├── security/           # Security framework
│   │   ├── mod.rs
│   │   ├── manager.rs      # Security validation
│   │   ├── approval.rs     # Human approval workflows
│   │   └── audit.rs        # Audit logging
│   ├── binary/             # Binary processing engine
│   │   ├── mod.rs
│   │   ├── processor.rs    # Core binary processing
│   │   ├── image.rs        # Image processing
│   │   ├── pdf.rs          # PDF processing
│   │   └── detection.rs    # Format detection
│   ├── filesystem/         # Filesystem abstraction
│   │   ├── mod.rs
│   │   ├── operations.rs   # Core file operations
│   │   ├── streaming.rs    # Large file streaming
│   │   └── paths.rs        # Path validation and utilities
│   ├── config/             # Configuration management
│   │   ├── mod.rs
│   │   ├── loader.rs       # Hierarchical config loading
│   │   └── validation.rs   # Configuration validation
│   └── error.rs            # Error types and handling
├── tests/
│   ├── integration/        # Integration tests
│   ├── security/           # Security-focused tests
│   └── performance/        # Performance tests
├── benches/               # Benchmarking suites
├── examples/              # Usage examples
└── docs/                  # Documentation source
```

## Technical Constraints & Considerations

### Performance Requirements
- **Response Time**: <100ms for basic file operations
- **Large File Support**: Streaming architecture for files up to 1GB
- **Memory Usage**: <50MB baseline with linear scaling for operations
- **Concurrent Operations**: Support 10+ simultaneous requests

### Security Constraints
- **Path Validation**: Prevent directory traversal attacks
- **Access Control**: Configurable allowlists and denylists
- **Human Approval**: Interactive approval for write operations
- **Audit Logging**: Comprehensive operation tracking for compliance

### Compatibility Requirements
- **Rust Version**: 1.88.0 or later for latest language features
- **Platform Support**: macOS, Linux, Windows (cross-platform paths)
- **MCP Clients**: Claude Desktop, VS Code with MCP extensions
- **File Systems**: NTFS, APFS, ext4, with proper permission handling

### Resource Limitations
- **File Size Limits**: Configurable limits to prevent resource exhaustion
- **Concurrent Operations**: Rate limiting to prevent abuse
- **Memory Management**: Streaming and careful buffer management
- **Disk Space**: Temporary file cleanup and garbage collection

## Configuration Architecture

### Configuration File Hierarchy
1. **Environment Variables**: `AIRS_MCP_FS_*` prefixed variables
2. **Project Config**: `./.airs-mcp-fs.toml` in working directory
3. **User Config**: `~/.config/airs-mcp-fs/config.toml`
4. **System Config**: `/etc/airs-mcp-fs/config.toml` (Linux/macOS)
5. **Default Values**: Built-in sensible defaults

### Configuration Schema
```toml
[server]
name = "airs-mcp-fs"
version = "1.0.0"
transport = "stdio"

[security]
allowed_read_paths = ["~/Documents/**", "~/Projects/**", "./**"]
allowed_write_paths = ["~/Documents/**", "~/Projects/**"]
forbidden_patterns = ["\\.env$", "\\.ssh/.*", ".*\\.key$"]
max_file_size_mb = 100
require_approval_for_writes = true
enable_threat_detection = true

[binary_processing]
max_image_dimension = 1920
pdf_extract_text = true
generate_thumbnails = true
compression_quality = 85

[performance]
max_concurrent_operations = 10
buffer_size_kb = 64
cache_size_mb = 50
streaming_threshold_mb = 10

[logging]
level = "info"
file = "~/.config/airs-mcp-fs/logs/airs-mcp-fs.log"
max_size_mb = 100
audit_file = "~/.config/airs-mcp-fs/logs/audit.log"
```

## Integration with AIRS Ecosystem

### Shared Dependencies
- **airs-mcp**: MCP client infrastructure, transport, and tool registration
- **Tokio**: Consistent async runtime across all AIRS components
- **Serde**: Shared serialization patterns and error handling
- **Tracing**: Unified logging and observability

### Common Patterns
- **Error Handling**: Consistent error types and recovery strategies
- **Configuration**: Shared configuration loading and validation patterns
- **Testing**: Common testing utilities and integration test patterns
- **Documentation**: Shared documentation standards and tooling

### Cross-Component Benefits
- **Memory Bank Integration**: Filesystem access for airs-memspec operations
- **Configuration Sharing**: Consistent configuration management across tools
- **Security Standards**: Shared security validation and audit patterns
- **Performance Optimization**: Common caching and optimization strategies

## Development Standards

### Code Quality Standards
- **Zero Warnings**: All code must compile without warnings
- **Test Coverage**: >95% coverage for core functionality
- **Documentation**: All public APIs documented with rustdoc
- **Error Handling**: Comprehensive error types with helpful messages

### Security Standards
- **Input Validation**: All external inputs validated and sanitized
- **Path Security**: Canonical path resolution and traversal prevention
- **Audit Logging**: All security-relevant operations logged
- **Approval Workflows**: Human oversight for dangerous operations

### Performance Standards
- **Async-First**: All I/O operations must be async
- **Memory Management**: Careful resource management for large files
- **Benchmarking**: Performance regression testing in CI
- **Optimization**: Profile-guided optimization for hot paths

## Deployment & Operations

### Build Configuration
```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization
panic = "abort"         # Smaller binary size
strip = true            # Remove debug symbols
```

### Monitoring & Observability
- **Structured Logging**: JSON logs for machine processing
- **Metrics Collection**: Performance and usage metrics
- **Health Checks**: Built-in health check endpoints
- **Error Tracking**: Comprehensive error reporting and aggregation

### Security Operations
- **Audit Trail**: Complete operation logging for compliance
- **Threat Detection**: Basic malware and anomaly detection
- **Access Control**: Fine-grained permission management
- **Incident Response**: Clear procedures for security incidents
