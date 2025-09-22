# Environment Setup

AIRS MCP-FS uses environment-aware configuration to automatically adapt to different deployment contexts. This section covers environment detection, configuration, and best practices.

## Environment Types

AIRS MCP-FS supports four distinct environments, each with different security and operational defaults:

### Development Environment
**Purpose**: Local development and testing  
**Security**: Balanced - permissive enough for productivity, secure enough for safety  
**Configuration File**: `development.toml`

```toml
# development.toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",      # All project directories
    "~/Documents/**/*",     # Personal documents
    "~/Desktop/**/*",       # Desktop files
    "./**/*"               # Current working directory
]

[security.operations]
read_allowed = true
write_requires_policy = false    # Allow writes for development
delete_requires_explicit_allow = true
```

### Staging Environment
**Purpose**: Pre-production testing and validation  
**Security**: Production-like with slightly relaxed monitoring  
**Configuration File**: `staging.toml`

```toml
# staging.toml
[security.filesystem]
allowed_paths = [
    "/app/staging/**/*",
    "/tmp/staging-data/**/*"
]

[security.operations]
read_allowed = true
write_requires_policy = true     # Require policies like production
delete_requires_explicit_allow = true
```

### Production Environment
**Purpose**: Live deployment with maximum security  
**Security**: Secure by default, minimal permissions  
**Configuration File**: `production.toml`

```toml
# production.toml
[security.filesystem]
allowed_paths = [
    "/app/data/**/*.json",       # Only specific data files
    "/app/config/app.toml"       # Only application config
]

denied_paths = [
    "/app/secrets/**",           # Never access secrets
    "**/*.key",                  # No key files
    "**/.env*"                   # No environment files
]

[security.operations]
read_allowed = true
write_requires_policy = true     # All writes need policies
delete_requires_explicit_allow = true
```

### Test Environment
**Purpose**: Unit and integration testing  
**Security**: Minimal restrictions for test execution  
**Configuration File**: `test.toml`

```toml
# test.toml
[security.filesystem]
allowed_paths = ["**/*"]         # Allow all paths for testing

[security.operations]
read_allowed = true
write_requires_policy = false    # No restrictions for tests
delete_requires_explicit_allow = false
```

## Environment Detection

AIRS MCP-FS uses multiple strategies to detect the current environment:

### Environment Variable Detection

Checked in priority order:

1. **`AIRS_MCPSERVER_FS_ENV`** - Primary environment variable
   ```bash
   export AIRS_MCPSERVER_FS_ENV=development
   ```

2. **`NODE_ENV`** - Node.js ecosystem compatibility
   ```bash
   export NODE_ENV=production
   ```

3. **`ENVIRONMENT`** - Generic environment variable
   ```bash
   export ENVIRONMENT=staging
   ```

### Automatic Detection

When no environment variables are set:

```rust
// Automatic environment detection logic
if cfg!(test) {
    ConfigEnvironment::Test
} else if cfg!(debug_assertions) {
    ConfigEnvironment::Development
} else {
    ConfigEnvironment::Production
}
```

- **Test runs**: Automatically use `test` environment
- **Debug builds**: Default to `development`
- **Release builds**: Default to `production`

## Configuration File Management

### File Location Priority

AIRS MCP-FS searches for configuration files in order:

1. **Environment Variable Path**
   ```bash
   export AIRS_MCPSERVER_FS_CONFIG_DIR=~/.config/airs-mcpserver-fs
   # Looks for: ~/.config/airs-mcpserver-fs/development.toml
   ```

2. **User Configuration Directory**
   ```
   ~/.config/airs-mcpserver-fs/development.toml
   ```

3. **System Configuration Directory**
   ```
   /etc/airs-mcpserver-fs/development.toml
   ```

4. **Built-in Defaults**
   ```
   Compiled defaults for the detected environment
   ```

### File Naming Convention

Environment-specific configuration files follow this pattern:

- `development.toml` - Development environment
- `staging.toml` - Staging environment
- `production.toml` - Production environment
- `test.toml` - Testing environment

## Environment Variable Overrides

All configuration values can be overridden using environment variables:

### Common Environment Variables

```bash
# Core environment setup
export AIRS_MCPSERVER_FS_ENV=development
export AIRS_MCPSERVER_FS_CONFIG_DIR=~/.config/airs-mcpserver-fs
export AIRS_MCPSERVER_FS_LOG_DIR=~/.local/share/airs-mcpserver-fs/logs

# Security overrides
export AIRS_MCPSERVER_FS_SECURITY_OPERATIONS_READ_ALLOWED=true
export AIRS_MCPSERVER_FS_SECURITY_OPERATIONS_WRITE_REQUIRES_POLICY=false

# File access overrides
export AIRS_MCPSERVER_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS="~/projects/**/*,~/docs/**/*"

# Binary processing overrides
export AIRS_MCPSERVER_FS_BINARY_MAX_FILE_SIZE=52428800  # 50MB
export AIRS_MCPSERVER_FS_BINARY_ENABLE_IMAGE_PROCESSING=true
```

### Variable Naming Convention

Environment variables follow this pattern:
```
AIRS_MCPSERVER_FS_{SECTION}_{SUBSECTION}_{SETTING}
```

Examples:
- `security.filesystem.allowed_paths` → `AIRS_MCPSERVER_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS`
- `binary.max_file_size` → `AIRS_MCPSERVER_FS_BINARY_MAX_FILE_SIZE`
- `server.name` → `AIRS_MCPSERVER_FS_SERVER_NAME`

## Environment-Specific Examples

### Development Workstation Setup

```bash
# ~/.bashrc or ~/.zshrc
export AIRS_MCPSERVER_FS_ENV=development
export AIRS_MCPSERVER_FS_CONFIG_DIR=~/.config/airs-mcpserver-fs
export AIRS_MCPSERVER_FS_LOG_DIR=~/.local/share/airs-mcpserver-fs/logs

# Allow broader access for development
export AIRS_MCPSERVER_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS="~/projects/**/*,~/Documents/**/*,~/Desktop/**/*,./**/*"
```

Configuration file (`~/.config/airs-mcpserver-fs/development.toml`):
```toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",
    "~/Documents/**/*",
    "~/Desktop/**/*",
    "./**/*"
]

[security.operations]
read_allowed = true
write_requires_policy = false
delete_requires_explicit_allow = true

[security.policies.development_files]
patterns = ["~/projects/**/*.{rs,py,js,ts,md}"]
operations = ["read", "write", "create_dir"]
risk_level = "low"
description = "Development source files"
```

### CI/CD Pipeline Setup

```yaml
# .github/workflows/test.yml
env:
  AIRS_MCPSERVER_FS_ENV: test
  AIRS_MCPSERVER_FS_SECURITY_OPERATIONS_WRITE_REQUIRES_POLICY: false
  AIRS_MCPSERVER_FS_SECURITY_OPERATIONS_DELETE_REQUIRES_EXPLICIT_ALLOW: false
```

### Docker Production Setup

```dockerfile
# Dockerfile
ENV AIRS_MCPSERVER_FS_ENV=production
ENV AIRS_MCPSERVER_FS_CONFIG_DIR=/app/config
ENV AIRS_MCPSERVER_FS_LOG_DIR=/app/logs
ENV AIRS_MCPSERVER_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS="/app/data/**/*"
```

Production configuration (`/app/config/production.toml`):
```toml
[security.filesystem]
allowed_paths = ["/app/data/**/*.json"]
denied_paths = ["/app/secrets/**", "**/*.key", "**/.env*"]

[security.operations]
read_allowed = true
write_requires_policy = true
delete_requires_explicit_allow = true

[security.policies.app_data]
patterns = ["/app/data/**/*.json"]
operations = ["read", "write"]
risk_level = "medium"
description = "Application data files"
```

## Environment Validation

AIRS MCP-FS validates environment configuration at startup:

### Validation Checks

1. **Environment Consistency**: Warns if environment settings don't match detected environment
2. **Security Validation**: Checks for potential security issues in permissive environments
3. **Path Validation**: Ensures all configured paths are accessible
4. **Policy Validation**: Verifies security policies are properly configured

### Validation Output Example

```
📋 Configuration loaded from development environment
   Configuration files: ["/Users/username/.config/airs-mcpserver-fs/development.toml"]
   Environment variables: 3 overrides
   
✅ Environment validation passed
   - Security policies: 4 active policies
   - Allowed paths: 4 patterns validated
   - Risk assessment: Low risk configuration
```

## Environment Migration

### Development to Staging

When promoting to staging:

1. **Review Security**: Ensure policies are appropriate for staging
2. **Update Paths**: Change paths from local development to staging paths
3. **Enable Monitoring**: Increase logging and audit levels
4. **Test Configuration**: Validate configuration with staging data

### Staging to Production

When promoting to production:

1. **Security Audit**: Complete security review of all policies
2. **Minimal Permissions**: Reduce allowed paths to absolute minimum
3. **Enable Auditing**: Full audit logging and monitoring
4. **Backup Configuration**: Maintain configuration backups

## Troubleshooting Environment Issues

### Common Environment Problems

1. **Wrong Environment Detected**
   ```bash
   # Check environment detection
   echo $AIRS_MCPSERVER_FS_ENV
   
   # Set explicitly
   export AIRS_MCPSERVER_FS_ENV=development
   ```

2. **Configuration File Not Found**
   ```bash
   # Check file existence
   ls -la ~/.config/airs-mcpserver-fs/
   
   # Generate missing configuration
   airs-mcpserver-fs generate-config --env development
   ```

3. **Permission Denied in Environment**
   ```bash
   # Check allowed paths in configuration
   cat ~/.config/airs-mcpserver-fs/development.toml | grep allowed_paths
   
   # Add required paths to configuration
   ```

### Environment Debugging

Enable debug logging to troubleshoot environment issues:

```bash
export RUST_LOG=debug
export AIRS_MCPSERVER_FS_ENV=development
airs-mcpserver-fs
```

This will show detailed information about:
- Environment detection process
- Configuration file loading
- Security policy evaluation
- Path validation results

## Best Practices

### Environment Separation

1. **Use Different Configurations**: Each environment should have its own configuration file
2. **Environment Variables**: Use environment variables for environment-specific values
3. **Version Control**: Keep configuration files in version control with environment branches
4. **Documentation**: Document environment-specific requirements and constraints

### Security Across Environments

1. **Progressive Security**: Each environment should be more secure than the previous
2. **Regular Reviews**: Periodically review environment configurations
3. **Audit Trails**: Maintain audit logs for all environment changes
4. **Testing**: Test security policies in staging before production

### Configuration Management

1. **Consistent Naming**: Use consistent naming across environments
2. **Template-Based**: Use configuration templates for consistency
3. **Validation**: Validate configurations before deployment
4. **Rollback Plans**: Maintain previous configurations for rollback

## Related Sections

- **[Configuration Overview](./overview.md)**: Overall configuration system architecture
- **[Security Policies](./security.md)**: Detailed security configuration
- **[Claude Desktop Integration](./claude_desktop.md)**: MCP client environment setup
- **[Troubleshooting](./troubleshooting.md)**: Environment-specific troubleshooting
