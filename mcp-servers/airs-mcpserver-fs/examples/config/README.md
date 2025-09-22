# airs-mcpserver-fs Configuration Examples

This directory contains example configurations for different deployment environments.

## Configuration Files

- `config.toml` - Base configuration (shared across all environments)
- `development.toml` - Development environment overrides
- `staging.toml` - Staging environment configuration  
- `production.toml` - Production environment configuration
- `local.toml` - Local developer overrides (development only)

## Environment Selection

The configuration system automatically detects the environment using:

1. `AIRS_MCP_FS_ENV` environment variable
2. `NODE_ENV` environment variable (Node.js convention)
3. `ENVIRONMENT` environment variable
4. Compile-time defaults (test mode vs debug/release)

## Configuration Loading Order

The system loads configuration in this order (later sources override earlier ones):

1. **Built-in defaults** - Hard-coded secure defaults
2. **Base configuration** - `config.toml` (if present)
3. **Environment-specific** - `{environment}.toml` (if present)
4. **Local overrides** - `local.toml` (development only)
5. **Environment variables** - `AIRS_MCP_FS_*` prefixed variables

## Environment Variable Overrides

Use double underscores for nested configuration:

```bash
# Override server.name
export AIRS_MCP_FS_SERVER__NAME="custom-mcp-fs"

# Override security.operations.write_requires_policy
export AIRS_MCP_FS_SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY=true

# Override security.filesystem.allowed_paths (array)
export AIRS_MCP_FS_SECURITY__FILESYSTEM__ALLOWED_PATHS="['/app/**/*', '/data/**/*']"
```

## Usage Examples

### Load configuration with auto-detection
```rust
use airs_mcp_fs::config::{Settings, ConfigurationLoader};

// Automatic environment detection
let settings = Settings::load()?;

// Manual environment specification  
let loader = ConfigurationLoader::with_environment(ConfigEnvironment::Production);
let (settings, source_info) = loader.load()?;
```

### Load from specific file
```rust
use airs_mcp_fs::config::ConfigurationLoader;

let settings = ConfigurationLoader::load_from_file("./config/production.toml")?;
```

### Validate configuration file
```rust
use airs_mcp_fs::config::ConfigurationLoader;

let issues = ConfigurationLoader::validate_file("./config/production.toml")?;
if !issues.is_empty() {
    println!("Configuration issues found: {:?}", issues);
}
```
