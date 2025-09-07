# Migration Guide

Comprehensive guide for migrating between AIRS MCP versions, handling breaking changes, and upgrading existing integrations safely.

## Version Management Strategy

AIRS MCP follows semantic versioning with careful API design for future compatibility:

### Current Version: 0.1.1

The library is currently in initial development phase. While we maintain high code quality and production readiness, API stability guarantees will begin with version 1.0.0.

### Protocol Version Support

```rust
use airs_mcp::shared::protocol::ProtocolVersion;

// Current MCP protocol support
let current_protocol = ProtocolVersion::new("2024-11-05").unwrap();

// Version validation with date format
assert!(ProtocolVersion::new("2024-11-05").is_ok());
assert!(ProtocolVersion::new("invalid").is_err());
```

**Protocol Version Strategy:**
- **Schema Compliance**: 100% adherence to official MCP schema specifications
- **Date-Based Versioning**: Protocol versions use YYYY-MM-DD format for clear temporal ordering
- **Backward Compatibility**: Support for multiple protocol versions when specified by MCP consortium

## Pre-1.0 Migration Path

### Breaking Changes to Expect

Until 1.0.0 release, expect these potential breaking changes:

#### 1. Configuration Structure Evolution

```rust
// Current (0.1.x) - Basic configuration
use airs_mcp::base::JsonRpcMessage;

let message = JsonRpcMessage::from_json_bytes(data)?;

// Future (0.2.x+) - Enhanced configuration management
use airs_mcp::config::{McpConfig, ConfigBuilder};

let config = McpConfig::load()
    .with_timeout(Duration::from_secs(30))
    .with_transport_type(TransportType::Stdio)
    .build()?;
```

#### 2. Transport Layer Enhancements

```rust
// Current - STDIO only
use airs_mcp::transport::stdio::StdioTransport;

let transport = StdioTransport::new().await?;

// Future - Multi-transport support
use airs_mcp::transport::{Transport, TransportBuilder};

let transport = TransportBuilder::new()
    .stdio()  // or .http(), .websocket(), .tcp()
    .with_buffer_size(8192)
    .build().await?;
```

#### 3. MCP Schema Evolution

Current implementation supports MCP 2024-11-05 specification. Future updates will:

```rust
// Version negotiation (planned)
use airs_mcp::integration::mcp::McpClient;

let client = McpClient::builder()
    .protocol_versions(vec!["2024-11-05", "2025-01-15"])
    .connect().await?;

// Automatic capability detection
let server_caps = client.initialize().await?;
println!("Server supports: {:?}", server_caps);
```

## Migration Strategies

### 1. Dependency Version Pinning

**Recommended approach for production systems:**

```toml
[dependencies]
# Pin to specific version for stability
airs-mcp = "=0.1.1"

# Or use compatible range for patch updates
airs-mcp = "~0.1.1"
```

### 2. Version Pinning for Stability

```toml
[dependencies]
# Pin to exact version for production stability
airs-mcp = "=0.1.1"

# Or use tilde for patch-level updates only
airs-mcp = "~0.1.1"
```

### 3. Gradual Migration Pattern

```rust
use airs_mcp::shared::protocol::{JsonRpcRequest, RequestId};

// Step 1: Update imports (safe)
// Step 2: Update data structures (validate)  
// Step 3: Update business logic (test thoroughly)
// Step 4: Update configuration (backup first)

fn migrate_request_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Legacy pattern (still supported)
    let request = JsonRpcRequest::new(
        "initialize", 
        None, 
        RequestId::new_number(1)
    );

    // Modern pattern (recommended)
    let request = JsonRpcRequest::new("initialize", None, RequestId::new_number(1));
    
    // Both produce identical JSON-RPC output
    assert_eq!(
        request.to_json()?,
        r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#
    );
    
    Ok(())
}
```

## Configuration Migration

### Claude Desktop Configuration

**Current configuration format:**

```json
{
  "mcpServers": {
    "airs-mcp-server": {
      "command": "/path/to/your/server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Future enhanced configuration (planned):**

```json
{
  "mcpServers": {
    "airs-mcp-server": {
      "command": "/path/to/your/server",
      "args": ["--config", "server.toml"],
      "env": {
        "RUST_LOG": "info",
        "MCP_PROTOCOL_VERSION": "2024-11-05",
        "MCP_REQUEST_TIMEOUT": "30",
        "MCP_MAX_CONNECTIONS": "10"
      },
      "capabilities": {
        "tools": true,
        "resources": true,
        "prompts": true
      }
    }
  }
}
```

### Server Configuration Files

**Migration from embedded config to external files:**

```rust
// Current: Embedded configuration
use airs_mcp::integration::mcp::McpServerBuilder;

let server = McpServerBuilder::new()
    .add_tool_provider(tool_provider)
    .add_resource_provider(resource_provider)
    .stdio_transport()
    .build().await?;

// Future: External configuration
use airs_mcp::config::McpConfig;

let config = McpConfig::load_from("server.toml")?;
let server = McpServerBuilder::from_config(config)
    .add_providers_from_config()
    .build().await?;
```

**Example server.toml:**

```toml
[server]
max_connections = 10
request_timeout = "30s"
enable_subscriptions = true

[transport]
type = "stdio"
buffer_size = 8192

[security]
strict_permissions = true
audit_logging = false

[logging]
level = "info"
format = "json"

[[providers.tools]]
name = "calculator"
class = "CalculatorProvider"

[[providers.resources]]
name = "filesystem"
class = "FileSystemProvider"
base_path = "/data"
```

## Testing Migration Changes

### 1. Schema Validation Testing

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    use airs_mcp::shared::protocol::*;

    #[test]
    fn test_backward_compatibility() {
        // Test that old message formats still work
        let old_json = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let request = JsonRpcRequest::from_json(old_json).unwrap();
        
        assert_eq!(request.method, "ping");
        assert_eq!(request.id, RequestId::Number(1));
    }

    #[test]
    fn test_content_serialization_compatibility() {
        // Ensure Content enum changes don't break serialization
        let content = Content::text("Hello");
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: Content = serde_json::from_str(&json).unwrap();
        
        assert_eq!(content, deserialized);
    }
}
```

### 2. Integration Testing with Claude Desktop

```bash
#!/bin/bash
# Migration testing script

echo "Testing migration compatibility..."

# 1. Test with existing Claude Desktop config
./scripts/test_inspector.sh

# 2. Backup current config
cp ~/.config/Claude/claude_desktop_config.json config.backup

# 3. Test new configuration format
./scripts/configure_claude.sh --migration-mode

# 4. Verify integration still works
./scripts/debug_integration.sh

# 5. Restore backup if needed
if [ $? -ne 0 ]; then
    echo "Migration failed, restoring backup..."
    cp config.backup ~/.config/Claude/claude_desktop_config.json
fi
```

## Breaking Change Communication

### How We'll Communicate Changes

1. **Changelog**: Detailed breaking change descriptions in CHANGELOG.md
2. **Migration Scripts**: Automated migration tools when possible
3. **Examples**: Updated examples showing old vs new patterns
4. **Documentation**: Clear migration guides for each major change

### Example Breaking Change Notice

```markdown
## BREAKING CHANGE: Content Enum Restructure (v0.2.0)

### What Changed
- `Content::Text` now includes optional `uri` and `mime_type` fields
- `Content::Image` requires `mime_type` parameter

### Migration Required
```rust
// Before (v0.1.x)
let content = Content::Text { text: "Hello".to_string() };

// After (v0.2.x)  
let content = Content::text("Hello");  // Use convenience method
// or
let content = Content::Text { 
    text: "Hello".to_string(), 
    uri: None, 
    mime_type: None 
};
```

### Migration Script
```bash
# Run automated migration
cargo install airs-mcp-migrate
airs-mcp-migrate --from 0.1 --to 0.2 src/
```
```

## Best Practices for Smooth Migrations

### 1. Version Pinning Strategy

```toml
# Production systems - pin exact version
airs-mcp = "=0.1.1"

# Development - allow patch updates
airs-mcp = "~0.1.1"

# Bleeding edge - latest features (higher risk)
airs-mcp = "0.1"
```

### 2. Progressive Migration

1. **Update Tests First**: Ensure your test suite covers current behavior
2. **Update Dependencies**: Upgrade to new version in isolated branch
3. **Fix Compilation**: Address breaking changes systematically
4. **Validate Behavior**: Ensure functional equivalence
5. **Update Configuration**: Migrate configuration files last
6. **Deploy Gradually**: Canary deployments for production systems

### 3. Rollback Strategy

```bash
# Prepare rollback artifacts
git tag v0.1-stable
cargo package --list > package-manifest.txt

# Quick rollback process
git checkout v0.1-stable
cargo clean && cargo build --release
./scripts/configure_claude.sh --restore-backup
```

## Future Migration Planning

### Roadmap to 1.0.0

**Version 0.2.0 (Planned)**:
- Enhanced configuration management
- Multi-transport support
- Breaking changes in transport layer

**Version 0.3.0 (Planned)**:
- Advanced MCP capabilities
- Resource subscription support
- Potential breaking changes in provider interfaces

**Version 1.0.0 (Target)**:
- Stable API guarantees
- Long-term support commitment
- Semantic versioning compliance

### API Stability Commitment

Starting with 1.0.0:
- **Major versions** (1.x.x → 2.x.x): Breaking changes allowed with migration guide
- **Minor versions** (1.1.x → 1.2.x): New features, backward compatible
- **Patch versions** (1.1.1 → 1.1.2): Bug fixes only

## Getting Help with Migrations

### Resources

1. **Documentation**: Check updated usage examples for new patterns
2. **Issues**: Report migration problems on GitHub
3. **Examples**: Reference updated example servers
4. **Community**: Discussion forums for migration strategies

### Migration Support Tools

```bash
# Version checking
cargo tree | grep airs-mcp

# Compatibility testing
cargo test --features migration-tests

# Schema validation
./scripts/validate_schema.sh
```

Remember: While we're in pre-1.0 development, we prioritize code quality and production readiness while allowing for API evolution based on real-world usage feedback.

## Content Coming Soon

This section will cover:

- Version upgrade procedures
- Breaking change migration
- API compatibility guides
- Configuration migration
- Performance impact analysis
- Best practices for updates

Check back soon for comprehensive migration guidance.
