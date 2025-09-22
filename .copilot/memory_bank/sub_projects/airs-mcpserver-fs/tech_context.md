# Technical Context: AIRS MCP Server - Filesystem

**Updated:** 2025-09-22  
**Sub-Project:** airs-mcpserver-fs  
**Context:** Technical requirements and implementation details for migration

## Technology Stack

### Core Technologies
- **Language**: Rust 2021 Edition (minimum version 1.88)
- **Async Runtime**: Tokio 1.47 with full feature set
- **MCP Integration**: airs-mcp (workspace dependency)
- **Serialization**: serde 1.0 with derive features
- **Error Handling**: thiserror 1.0 for structured errors

### Key Dependencies
```toml
[dependencies]
# AIRS Foundation (Priority Layer 1)
airs-mcp = { workspace = true }

# Core Runtime (Layer 2)
tokio = { workspace = true }
futures = { workspace = true }

# Serialization (Layer 3)
serde = { workspace = true }
serde_json = { workspace = true }

# Security and File Operations (Layer 4)
globset = { workspace = true }      # Pattern matching for security policies
walkdir = { workspace = true }      # Directory traversal
path-clean = { workspace = true }   # Path normalization
glob = { workspace = true }         # File pattern matching

# Configuration Management
config = { workspace = true }       # Hierarchical configuration
toml = { workspace = true }         # TOML configuration files
dirs = { workspace = true }         # Standard directories

# Utilities
uuid = { workspace = true }         # Operation tracking
chrono = { workspace = true }       # Time handling (workspace standard)
tracing = { workspace = true }      # Logging framework
```

## Development Setup

### Prerequisites
- **Rust**: Version 1.88 or later with Cargo
- **Development Tools**: cargo-watch, cargo-audit, cargo-clippy
- **Testing Tools**: cargo-test, cargo-tarpaulin (coverage)
- **Documentation**: mdbook for documentation generation

### Local Development Environment
```bash
# Clone repository and navigate to project
cd airsstack/mcp-servers/airs-mcpserver-fs

# Install development dependencies
cargo install cargo-watch cargo-audit mdbook

# Build project
cargo build

# Run tests
cargo test

# Start development server with auto-reload
cargo watch -x 'run --example stdio_integration'
```

### Build Configuration
```toml
# Cargo.toml build optimizations
[profile.dev]
debug = true
opt-level = 0

[profile.release]
debug = false
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[profile.test]
debug = true
opt-level = 0
```

## Technical Constraints

### Performance Requirements
- **Response Time**: < 100ms for standard file operations
- **Memory Usage**: < 50MB baseline memory footprint
- **Concurrency**: Support for 100+ concurrent operations
- **File Size Limits**: Efficient handling up to 1GB file sizes
- **Throughput**: > 1,000 operations per second under load

### Security Requirements
- **Path Validation**: Prevent directory traversal attacks
- **Binary Restrictions**: Block potentially dangerous file types
- **Approval Workflows**: Human authorization for write operations
- **Audit Logging**: Comprehensive operation tracking
- **Threat Detection**: Basic malware and anomaly scanning
- **Access Controls**: Configurable allowlist/denylist policies

### Compatibility Requirements
- **MCP Protocol**: Full JSON-RPC 2.0 compliance
- **Claude Desktop**: Seamless STDIO transport integration
- **Cross-Platform**: Windows, macOS, and Linux support
- **File Systems**: Support for common file systems (NTFS, ext4, APFS)
- **Path Handling**: Unicode path support with normalization

## Architecture Constraints

### Workspace Standards Compliance
**Reference**: `workspace/shared_patterns.md`

#### Import Organization (§2.1)
```rust
// Standard library imports
use std::path::{Path, PathBuf};
use std::fs;

// Third-party crate imports
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;

// Internal module imports
use airs_mcp::protocol::types::{Tool, Content};
use crate::security::SecurityFramework;
```

#### Time Management (§3.2)
```rust
// MANDATORY: Use chrono DateTime<Utc> for all time operations
use chrono::{DateTime, Utc};

impl AuditLogger {
    pub fn log_operation(&self, operation: &FilesystemOperation) {
        let timestamp = Utc::now(); // ✅ Workspace standard compliance
        // Never use std::time::SystemTime for business logic
    }
}
```

#### Module Architecture (§4.3)
```rust
// src/lib.rs - Library root
pub mod config;
pub mod security;
pub mod operations;
pub mod audit;

pub use config::ServerConfig;
pub use security::SecurityFramework;

// src/operations/mod.rs - Module organization
pub mod file_ops;
pub mod directory_ops;

pub use file_ops::FileOperations;
pub use directory_ops::DirectoryOperations;
```

#### Zero Warning Policy
```bash
# MANDATORY: All commands must pass with zero warnings
cargo check --workspace          # Must return clean
cargo clippy --workspace         # Must pass all lints
cargo test --workspace           # All tests must pass
```

### Migration-Specific Constraints

#### Project Structure Migration
```
# Source Structure (Legacy)
crates/airs-mcp-fs/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config/
│   ├── security/
│   └── operations/
├── tests/
├── examples/
└── docs/

# Target Structure (New)
mcp-servers/airs-mcpserver-fs/
├── Cargo.toml              # Updated project name
├── src/                    # Identical structure, updated imports
│   ├── lib.rs
│   ├── config/
│   ├── security/
│   └── operations/
├── tests/                  # Updated imports only
├── examples/               # Updated imports and paths
└── docs/                   # Updated documentation
```

#### Dependency Constraints
- **No New Dependencies**: Migration should not introduce new dependencies
- **Version Consistency**: All dependencies must use workspace versions
- **Import Updates**: Only import paths change, not functionality
- **Backward Compatibility**: Temporary alias support during transition

#### Configuration Constraints
```toml
# New Cargo.toml structure
[package]
name = "airs-mcpserver-fs"           # New name
version = "0.1.0"                    # Preserve version
edition = "2021"
authors = { workspace = true }
license = { workspace = true }
repository = { workspace = true }

[dependencies]
airs-mcp = { workspace = true }      # Unchanged dependency
# ... all other dependencies unchanged
```

## Integration Requirements

### AIRS-MCP Integration
```rust
// Required integration pattern for latest airs-mcp architecture
use airs_mcp::integration::MessageHandler;
use airs_mcp::transport::stdio::StdioTransportBuilder;

pub struct FilesystemMessageHandler {
    provider: Arc<FilesystemToolProvider>,
}

impl MessageHandler<()> for FilesystemMessageHandler {
    async fn handle_message(&self, message: JsonRpcMessage) -> JsonRpcResponse {
        self.provider.handle_message(message).await
    }
}
```

### Transport Layer Integration
```rust
// STDIO transport configuration for Claude Desktop
pub async fn start_server() -> Result<()> {
    let handler = FilesystemMessageHandler::new().await?;
    
    StdioTransportBuilder::new()
        .with_message_handler(handler)
        .build()
        .await?
        .run()
        .await
}
```

### Tool Registration Pattern
```rust
// MCP tool registration following airs-mcp patterns
impl FilesystemToolProvider {
    pub fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new("read_file")
                .with_description("Read contents of a file")
                .with_parameter("path", "File path to read"),
            Tool::new("write_file")
                .with_description("Write content to a file")
                .with_parameter("path", "File path to write")
                .with_parameter("content", "Content to write"),
            // ... additional tools
        ]
    }
}
```

## Testing Strategy

### Test Categories
1. **Unit Tests**: Individual component functionality
2. **Integration Tests**: End-to-end workflow testing
3. **Security Tests**: Security framework validation
4. **Performance Tests**: Response time and resource usage
5. **Compatibility Tests**: MCP protocol and Claude Desktop integration

### Test Environment Setup
```rust
// Test utilities for filesystem operations
#[cfg(test)]
pub mod test_utils {
    use tempfile::TempDir;
    
    pub fn create_test_environment() -> (TempDir, FilesystemServer) {
        let temp_dir = TempDir::new().unwrap();
        let config = ServerConfig::test_config(temp_dir.path());
        let server = FilesystemServer::new(config).unwrap();
        (temp_dir, server)
    }
}
```

### Security Test Requirements
```rust
// Security test patterns
#[tokio::test]
async fn test_path_traversal_prevention() {
    let server = create_test_server().await;
    
    // Test various path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\windows\\system32\\config\\sam",
        "/proc/self/mem",
    ];
    
    for path in malicious_paths {
        let result = server.read_file(path).await;
        assert!(matches!(result, Err(FilesystemError::SecurityViolation { .. })));
    }
}
```

## Documentation Requirements

### Code Documentation
- **Public APIs**: Comprehensive rustdoc documentation
- **Internal Modules**: Clear module-level documentation
- **Examples**: Working code examples for all public functions
- **Error Handling**: Documented error conditions and recovery

### User Documentation
- **Installation Guide**: Step-by-step setup instructions
- **Configuration Reference**: Complete configuration options
- **Security Guide**: Security best practices and policies
- **Troubleshooting**: Common issues and solutions
- **Migration Guide**: Transition from legacy version

### Development Documentation
- **Architecture Guide**: System design and component interaction
- **Contributing Guide**: Development workflow and standards
- **Testing Guide**: Test strategy and execution
- **Release Process**: Version management and deployment

## Migration Technical Requirements

### Phase-Based Migration Strategy
1. **Phase 1**: Create new project structure with updated metadata
2. **Phase 2**: Update import paths and workspace configuration
3. **Phase 3**: Migrate documentation and examples
4. **Phase 4**: Validation and compatibility testing
5. **Phase 5**: Legacy deprecation and cleanup

### Validation Requirements
```bash
# Required validation commands for each phase
cargo build --package airs-mcpserver-fs    # Must succeed
cargo test --package airs-mcpserver-fs     # All tests pass
cargo clippy --package airs-mcpserver-fs   # Zero warnings
cargo doc --package airs-mcpserver-fs      # Documentation builds
```

### Compatibility Validation
- **Functional Testing**: All existing functionality works identically
- **Performance Testing**: Response times within established baselines
- **Security Testing**: All security features operational
- **Integration Testing**: Claude Desktop integration verified
- **Documentation Testing**: All examples and guides work correctly

This technical context provides the complete technical foundation for successfully migrating `airs-mcp-fs` to `airs-mcpserver-fs` while maintaining full functionality and compliance with AIRS workspace standards.