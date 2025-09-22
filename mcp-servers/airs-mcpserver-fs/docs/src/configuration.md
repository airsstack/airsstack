# Configuration Guide

AIRS MCP-FS uses a sophisticated multi-layered configuration system that balances security, flexibility, and ease of use. This guide covers everything from basic setup to advanced security policies.

## Configuration Architecture

The configuration system follows a hierarchical approach:

```
Environment Detection → Base Configuration → Policy Application → Environment Variable Overrides
```

### Key Components

1. **Environment Detection**: Automatically detects development/staging/production environments
2. **Security Policies**: Named policies for different file types and use cases
3. **Path Validation**: Glob pattern-based path allowlists and denylists
4. **Operation Controls**: Fine-grained permissions for read/write/delete operations
5. **Binary File Restriction**: Enhanced security through complete binary file blocking

## Configuration File Locations

AIRS MCP-FS searches for configuration files in the following order:

1. **Environment Variable**: `AIRS_MCPSERVER_FS_CONFIG_DIR`
2. **User Config Directory**: `~/.config/airs-mcpserver-fs/`
3. **System Config Directory**: `/etc/airs-mcpserver-fs/`
4. **Built-in Defaults**: Secure production defaults

### Environment-Specific Files

- `development.toml` - Development environment (permissive)
- `staging.toml` - Staging environment (production-like)
- `production.toml` - Production environment (secure)
- `test.toml` - Testing environment (minimal)

## Basic Configuration Structure

```toml
# Security configuration
[security.filesystem]
allowed_paths = ["~/projects/**/*"]
denied_paths = ["**/.env*", "**/secrets/**"]

[security.operations]
read_allowed = true
write_requires_policy = false
delete_requires_explicit_allow = true

# Named security policies
[security.policies.source_code]
patterns = ["**/*.{rs,py,js,ts}"]
operations = ["read", "write"]
risk_level = "low"
description = "Source code files"

# Security configuration (binary processing disabled)
[security]
binary_processing_disabled = true
text_only_mode = true
max_file_size = 104857600  # 100MB for text files

# Server configuration
[server]
name = "airs-mcpserver-fs"
version = "1.0.0"
```

## Environment Variable Overrides

All configuration values can be overridden using environment variables with the `AIRS_MCPSERVER_FS_` prefix:

| Environment Variable | Configuration Path | Example |
|---------------------|-------------------|---------|
| `AIRS_MCPSERVER_FS_ENV` | Environment type | `development` |
| `AIRS_MCPSERVER_FS_CONFIG_DIR` | Config directory | `~/.config/airs-mcpserver-fs` |
| `AIRS_MCPSERVER_FS_LOG_DIR` | Log directory | `~/.local/share/airs-mcpserver-fs/logs` |
| `AIRS_MCPSERVER_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS` | Filesystem allowed paths | `~/projects/**/*,~/docs/**/*` |
| `AIRS_MCPSERVER_FS_SECURITY_MAX_FILE_SIZE` | Max file size | `52428800` (50MB) |

## Security Modes

AIRS MCP-FS provides three pre-configured security modes:

### Production Mode (Default)
- **Secure by default**: Minimal permissions
- **Explicit policies**: All operations require policy matches
- **Audit logging**: Comprehensive operation tracking
- **Path restrictions**: Limited to safe directories

```toml
[security.operations]
read_allowed = true
write_requires_policy = true
delete_requires_explicit_allow = true
```

### Development Mode
- **Balanced security**: Reasonable for development work
- **Broader access**: More directories allowed
- **Relaxed writes**: Writes allowed without strict policies
- **Safety nets**: Delete operations still require explicit permission

```toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",
    "~/Documents/**/*", 
    "./**/*"
]

[security.operations]
write_requires_policy = false
```

### Permissive Mode
- **Minimal restrictions**: Suitable for testing
- **Universal access**: All operations allowed
- **Testing focus**: Optimized for development testing
- **Not recommended**: For production environments

```toml
[security.policies.permissive_universal]
patterns = ["**/*"]
operations = ["read", "write", "delete", "list", "create_dir"]
risk_level = "low"
```

## Quick Configuration Examples

### Development Workstation
```toml
# ~/.config/airs-mcpserver-fs/development.toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",
    "~/Documents/**/*",
    "~/Desktop/**/*",
    "./**/*"
]

denied_paths = [
    "**/.git/**",
    "**/.env*",
    "~/.*/**"
]

[security.operations]
read_allowed = true
write_requires_policy = false
delete_requires_explicit_allow = true

[security.policies.journal_files]
patterns = ["/Users/yourusername/Documents/**/*"]
operations = ["read", "write", "list"]
risk_level = "low"
```

### Content Creation Setup
```toml
[security.policies.content_creation]
patterns = [
    "~/content/**/*.{md,txt}",
    "~/blog/**/*",
    "~/docs/**/*"
]
operations = ["read", "write", "create_dir"]
risk_level = "low"
description = "Content creation and blogging files"

[security]
# Binary processing disabled for security
binary_processing_disabled = true
text_only_mode = true
max_file_size = 209715200  # 200MB for large text files
```

### Secure Production Environment
```toml
[security.filesystem]
allowed_paths = [
    "/app/data/**/*",
    "/app/config/*.json"
]

denied_paths = [
    "/app/secrets/**",
    "**/.*/**",
    "**/*.key"
]

[security.operations]
read_allowed = true
write_requires_policy = true
delete_requires_explicit_allow = true

[security.policies.app_data]
patterns = ["/app/data/**/*.json"]
operations = ["read", "write"]
risk_level = "medium"
```

## Advanced Topics

- **[Environment Setup](./configuration/environment.md)**: Environment detection and variable configuration
- **[Security Policies](./configuration/security.md)**: Detailed security policy configuration
- **[Claude Desktop Integration](./configuration/claude_desktop.md)**: MCP client configuration
- **[Troubleshooting](./configuration/troubleshooting.md)**: Common configuration issues and solutions

## Best Practices

### Path Configuration
- Use absolute paths when possible for clarity
- Prefer specific patterns over broad wildcards
- Test path patterns before deploying to production
- Regularly review and audit allowed paths

### Security Policies
- Create named policies for different file types
- Use appropriate risk levels for audit logging
- Document policy purposes with descriptions
- Regular policy reviews and updates

### Environment Management
- Use environment-specific configuration files
- Leverage environment variables for sensitive values
- Implement proper configuration validation
- Maintain configuration documentation

## Next Steps

Once you understand the configuration basics, explore specific configuration areas:

1. **[Environment Setup](./configuration/environment.md)** - Environment detection and management
2. **[Security Policies](./configuration/security.md)** - Advanced security configuration
3. **[Claude Desktop Integration](./configuration/claude_desktop.md)** - MCP client setup
4. **[Troubleshooting](./configuration/troubleshooting.md)** - Solving configuration problems
