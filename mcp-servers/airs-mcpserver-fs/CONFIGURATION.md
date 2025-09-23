# AIRS MCP-FS Configuration Guide

This comprehensive guide explains how to configure `airs-mcpserver-fs` using TOML files for a clean, maintainable, and secure setup.

## Overview

AIRS MCP-FS uses a sophisticated 5-layer configuration system that prioritizes security while providing flexibility for different environments:

1. **Built-in defaults** (secure by default)
2. **Base config.toml** (shared settings across environments)  
3. **Environment-specific TOML** (development.toml, production.toml, etc.)
4. **Local overrides** (local.toml for development only)
5. **Environment variables** (final overrides and deployment configuration)

## Quick Start

### 1. Automatic Setup (Recommended)

Use the new setup command to automatically create the directory structure:

```bash
# Automatic setup with default directories
airs-mcpserver-fs setup

# Setup with custom directories
airs-mcpserver-fs setup --config-dir ~/.config/airs-mcpserver-fs --logs-dir ~/.local/share/airs-mcpserver-fs/logs

# Generate configuration for specific environment
airs-mcpserver-fs config --env production --output ~/.config/airs-mcpserver-fs
```

The setup command will:
- Create `~/.airs-mcpserver-fs/config` and `~/.airs-mcpserver-fs/logs` directories
- Generate a sample `development.toml` configuration
- Provide next steps for customization

### 2. Manual Configuration (Alternative)

### 2a. Choose Your Configuration

Start with one of our pre-built configurations from [`examples/config/`](./examples/config/):

- **`claude-desktop.toml`** - Optimized for Claude Desktop integration
- **`secure.toml`** - High-security for sensitive environments
- **`educational.toml`** - Permissive for learning and tutorials  
- **`development.toml`** - Balanced for daily development work

### 2b. Set Up Configuration Directory

```bash
# Create configuration directory
mkdir -p ~/.config/airs-mcp-fs

# Copy your chosen configuration
cp examples/config/claude-desktop.toml ~/.config/airs-mcp-fs/config.toml

# Customize paths for your environment
editor ~/.config/airs-mcp-fs/config.toml
```

### 3. Configure Claude Desktop

Use our example configuration from [`examples/claude-desktop/`](./examples/claude-desktop/):

```bash
# Copy the example
cp examples/claude-desktop/claude_desktop_config.json ~/temp-claude-config.json

# Edit paths and add to your Claude Desktop configuration
editor ~/temp-claude-config.json
```

## Configuration Structure

### Basic TOML Configuration

```toml
[server]
name = "airs-mcp-fs"
version = "0.1.0"

[binary]
max_file_size = 52428800  # 50MB for text files
binary_processing_disabled = true  # Security hardened

[security.filesystem]
allowed_paths = [
    "~/Projects/**/*",
    "~/Documents/**/*.{md,txt,rst}",
]

denied_paths = [
    "**/.git/**",
    "**/.env*",
    "~/.*/**",  # Hidden directories
]

[security.operations]
read_allowed = true
create_dir_allowed = true
write_requires_policy = false  # Set to true for production
delete_requires_explicit_allow = true

[security.policies.source_code]
patterns = ["**/*.{rs,py,js,ts}"]
operations = ["read", "write"]
risk_level = "low"
description = "Source code files - safe for development"
```

### Path Configuration

**Allowed Paths** use glob patterns for flexible matching:
- `~/Projects/**/*` - All files in Projects directory
- `~/Documents/**/*.md` - Only markdown files in Documents
- `./src/**/*.rs` - Rust files in current project's src directory
- `**/*.{md,txt,rst}` - Documentation files anywhere

**Denied Paths** take precedence over allowed paths:
- `**/.git/**` - Git repository internals
- `**/.env*` - Environment files with secrets
- `~/.*/**` - Hidden system directories

### Security Policies

Define granular permissions for different file types:

```toml
[security.policies.documentation]
patterns = [
    "**/*.{md,txt,rst,adoc}",
    "**/README*",
    "**/CHANGELOG*",
]
operations = ["read", "write"]
risk_level = "low"
description = "Documentation files - safe for editing"

[security.policies.config_files]
patterns = [
    "**/*.{json,yaml,yml,toml}",
    "**/Cargo.toml",
    "**/package.json",
]
operations = ["read", "write"]
risk_level = "medium"
description = "Configuration files - review changes carefully"
```

## Environment-Specific Configuration

### Development Environment

**File**: `~/.config/airs-mcp-fs/development.toml`

```toml
[security.filesystem]
allowed_paths = [
    "~/Projects/**/*",                    # All project files
    "~/Documents/**/*.{md,txt,rst}",      # Documentation
    "~/Desktop/**/*.{md,txt,json,toml}",  # Quick access files
]

[security.operations]
write_requires_policy = false   # Allow writes for productivity
delete_requires_explicit_allow = true
```

### Production Environment

**File**: `~/.config/airs-mcp-fs/production.toml`

```toml
[security.filesystem]
allowed_paths = [
    "/app/data/**/*.md",           # Only specific data files
    "/app/config/**/*.toml",       # Configuration files
]

[security.operations]
write_requires_policy = true      # Require explicit policies
delete_requires_explicit_allow = true
create_dir_allowed = false        # No directory creation
```

### Secure Environment

**File**: `~/.config/airs-mcp-fs/secure.toml`

```toml
[security.filesystem]
allowed_paths = [
    "~/work/safe-project/**/*.{md,txt}",  # Very limited access
]

[security.operations]
read_allowed = true
write_requires_policy = true
delete_requires_explicit_allow = true
create_dir_allowed = false

# All operations require explicit policies
[security.policies.readonly_docs]
patterns = ["**/*.{md,txt,rst}"]
operations = ["read"]  # Read-only
risk_level = "medium"
```

## Claude Desktop Integration

### Complete Setup Example

1. **Configuration Setup**:
   ```bash
   mkdir -p ~/.config/airs-mcp-fs
   cp examples/config/claude-desktop.toml ~/.config/airs-mcp-fs/config.toml
   ```

2. **Claude Desktop Configuration**:
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "command": "/path/to/airs-mcpserver-fs",
         "env": {
           "AIRS_MCPSERVER_FS_CONFIG_DIR": "/Users/yourname/.config/airs-mcpserver-fs",
           "AIRS_MCPSERVER_FS_ENV": "development"
         }
       }
     }
   }
   ```

3. **Test the Setup**:
   - Restart Claude Desktop
   - Try: "List files in my Projects directory"
   - Try: "Read my project's README.md"
   - Try: "Create a test file in my Documents"

### Alternative: Project-Specific Configuration

For different projects with different security requirements:

```json
{
  "mcpServers": {
    "airs-mcp-fs-project": {
      "command": "/path/to/airs-mcpserver-fs",
      "env": {
        "AIRS_MCPSERVER_FS_CONFIG_DIR": "/Users/yourname/projects/sensitive-project/.mcp-config"
      }
    }
  }
}
```

## Advanced Configuration

### Environment Variables

While TOML is recommended, environment variables can override any setting:

```bash
# Override config directory
export AIRS_MCPSERVER_FS_CONFIG_DIR="/custom/config/path"

# Override environment
export AIRS_MCPSERVER_FS_ENV="staging"

# Override nested settings (use double underscore)
export AIRS_MCPSERVER_FS__SECURITY__FILESYSTEM__ALLOWED_PATHS="~/safe/**/*"
export AIRS_MCPSERVER_FS__SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY="true"
```

### Configuration Validation

Test your configuration before deployment:

```bash
# Generate and validate configuration
airs-mcpserver-fs generate-config --output ./test-config --env development

# Test server startup (shows configuration loading)
echo '{"jsonrpc": "2.0", "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test"}}, "id": 1}' | airs-mcpserver-fs

# Check configuration logs
tail -f ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log
```

## Security Best Practices

### 1. Principle of Least Privilege
- Start with restrictive permissions and add access as needed
- Use specific file patterns instead of broad wildcards
- Regularly audit and remove unused permissions

### 2. Path Security
```toml
# ✅ Good: Specific patterns
allowed_paths = [
    "~/projects/myapp/**/*.{rs,toml,md}",
    "~/documents/work/**/*.txt",
]

# ❌ Avoid: Overly broad patterns
allowed_paths = [
    "/**/*",          # Everything on filesystem
    "~/**/*",         # Entire home directory
]
```

### 3. Environment Separation
- Use different configurations for dev/staging/production
- Never use development configurations in production
- Validate configurations before deployment

### 4. Policy Management
```toml
# Use specific policies for sensitive operations
[security.policies.config_changes]
patterns = ["**/config/**/*.toml"]
operations = ["read", "write"]
risk_level = "high"  # Higher risk = more logging
description = "Configuration changes require careful review"
```

## Troubleshooting

### Configuration Not Loading

1. **Check config directory**:
   ```bash
   echo $AIRS_MCP_FS_CONFIG_DIR
   ls -la ~/.config/airs-mcp-fs/
   ```

2. **Verify file permissions**:
   ```bash
   chmod 644 ~/.config/airs-mcp-fs/*.toml
   ```

3. **Test configuration syntax**:
   ```bash
   airs-mcpserver-fs generate-config --output /tmp/test --env development
   ```

### Path Access Issues

1. **Check denied paths** aren't blocking access
2. **Verify glob patterns** are correctly formatted
3. **Review security policies** for required operations
4. **Check logs** for path validation errors:
   ```bash
   grep "Path" ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log
   ```

### Claude Desktop Integration Issues

1. **Verify binary path** in Claude Desktop config
2. **Check environment variables** are set correctly
3. **Review server logs** for startup errors
4. **Test server manually**:
   ```bash
   airs-mcpserver-fs --help
   ```

## Migration Guide

### From Environment Variables to TOML

If you were using environment variables:

1. **Identify current variables**:
   ```bash
   env | grep AIRS_MCP_FS
   ```

2. **Convert to TOML format**:
   - `AIRS_MCP_FS__SECURITY__FILESYSTEM__ALLOWED_PATHS` → `[security.filesystem] allowed_paths = [...]`
   - `AIRS_MCP_FS__SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY` → `[security.operations] write_requires_policy = true`

3. **Update Claude Desktop config**:
   - Remove environment variables from MCP config
   - Add `AIRS_MCP_FS_CONFIG_DIR` pointing to TOML files

4. **Test the migration**:
   - Start server and verify configuration loading
   - Test file operations through Claude Desktop

## Resources

- **[Configuration Examples](./examples/config/)** - Ready-to-use configurations
- **[Claude Desktop Setup](./examples/claude-desktop/)** - Complete integration examples  
- **[API Documentation](./docs/)** - Technical reference
- **[Security Guide](./docs/security.md)** - Security best practices
- **[Performance Tuning](./docs/performance.md)** - Optimization guidelines

The TOML-based configuration system provides much cleaner configuration management, better version control support, and easier maintenance compared to environment variables while maintaining the flexibility needed for different deployment scenarios.