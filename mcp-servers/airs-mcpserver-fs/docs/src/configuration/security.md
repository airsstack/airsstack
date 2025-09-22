# Security Policies

AIRS MCP-FS implements a comprehensive security model based on named policies, path validation, and operation controls. This section covers advanced security configuration and best practices.

## Security Architecture

The security system uses multiple layers of protection:

```
Request → Path Validation → Policy Matching → Operation Authorization → Audit Logging
```

### Security Components

1. **Path Validation**: Glob pattern-based allowlists and denylists
2. **Named Policies**: Fine-grained control for specific file types
3. **Operation Controls**: Global permissions for read/write/delete operations
4. **Risk Assessment**: Risk-level categorization for audit and monitoring
5. **Audit Logging**: Comprehensive operation tracking

## Path-Based Security

### Allowed Paths

Allowed paths define what directories and files AIRS MCP-FS can access:

```toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",              # All files in projects directory
    "~/Documents/**/*.{md,txt}",    # Only markdown and text in Documents
    "./**/*",                       # Current directory and subdirectories
    "/tmp/airs-mcp-fs/**/*"        # Temporary files
]
```

### Denied Paths

Denied paths take precedence over allowed paths and prevent access to sensitive locations:

```toml
[security.filesystem]
denied_paths = [
    "**/.git/**",                   # Git repositories
    "**/.env*",                     # Environment files
    "~/.*/**",                      # Hidden directories (except specific allowlist)
    "**/id_rsa*",                   # SSH keys
    "**/credentials*",              # Credential files
    "**/secrets*",                  # Secret files
    "**/*.key",                     # Private key files
    "/etc/passwd",                  # System password file
    "/etc/shadow"                   # System shadow file
]
```

### Glob Pattern Syntax

AIRS MCP-FS uses standard glob patterns for path matching:

| Pattern | Description | Example |
|---------|-------------|---------|
| `*` | Matches any character except `/` | `*.txt` matches `file.txt` |
| `**` | Matches any character including `/` | `**/src/**` matches any src directory |
| `?` | Matches single character | `file?.txt` matches `file1.txt` |
| `[...]` | Character class | `*.[jt]s` matches `.js` and `.ts` |
| `{...}` | Alternation | `*.{md,txt}` matches `.md` and `.txt` |

### Path Validation Examples

```toml
# Development workstation
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",          # All project files
    "~/Documents/**/*",         # All documents
    "~/Desktop/**/*",           # Desktop files
    "./**/*"                    # Current working directory
]

denied_paths = [
    "**/.git/**",               # No Git internals
    "**/.env*",                 # No environment files
    "~/.*/**"                   # No hidden directories
]
```

```toml
# Content creation setup
[security.filesystem]
allowed_paths = [
    "~/writing/**/*.{md,txt}",  # Writing files only
    "~/blog/**/*",              # Blog content
    "~/assets/images/**/*.{jpg,png,gif}"  # Image assets
]

denied_paths = [
    "**/drafts/private/**",     # Private drafts
    "**/*.backup"               # Backup files
]
```

## Named Security Policies

Named policies provide fine-grained control over specific file types and use cases:

### Policy Structure

```toml
[security.policies.policy_name]
patterns = ["**/*.{ext1,ext2}"]     # File patterns this policy covers
operations = ["read", "write"]       # Allowed operations
risk_level = "low"                   # Risk category for auditing
description = "Policy description"   # Human-readable description
```

### Available Operations

| Operation | Description | Usage |
|-----------|-------------|-------|
| `read` | Read file contents | Reading source code, documents |
| `write` | Create or modify files | Creating new files, editing existing |
| `delete` | Remove files or directories | Cleanup, refactoring |
| `list` | List directory contents | Directory browsing |
| `create_dir` | Create directories | Project setup, organization |
| `move` | Move/rename files | File organization |
| `copy` | Copy files | Backup, duplication |

### Risk Levels

Risk levels control audit logging and monitoring intensity:

| Level | Description | Use Cases | Logging |
|-------|-------------|-----------|---------|
| `low` | Normal operations | Source code, documentation | Basic |
| `medium` | Moderate risk | Configuration files, scripts | Standard |
| `high` | Elevated risk | System files, databases | Detailed |
| `critical` | High-risk operations | Security files, credentials | Comprehensive |

### Built-in Policies

AIRS MCP-FS includes several built-in security policies:

#### Source Code Policy

```toml
[security.policies.source_code]
patterns = [
    "**/*.{rs,py,js,ts,jsx,tsx}",
    "**/*.{c,cpp,h,hpp}",
    "**/*.{java,kt,scala}",
    "**/*.{go,rb,php,swift}"
]
operations = ["read", "write"]
risk_level = "low"
description = "Source code files - safe for development"
```

#### Documentation Policy

```toml
[security.policies.documentation]
patterns = [
    "**/*.{md,txt,rst}",
    "**/README*",
    "**/CHANGELOG*",
    "**/LICENSE*"
]
operations = ["read", "write"]
risk_level = "low"
description = "Documentation files - safe for editing"
```

#### Configuration Files Policy

```toml
[security.policies.config_files]
patterns = [
    "**/Cargo.toml",
    "**/*.{json,yaml,yml,toml}",
    "**/*.{xml,ini,conf}"
]
operations = ["read", "write"]
risk_level = "medium"
description = "Configuration files - moderate risk"
```

#### Build Artifacts Policy

```toml
[security.policies.build_artifacts]
patterns = [
    "**/target/**",
    "**/dist/**",
    "**/build/**",
    "**/*.{tmp,bak,log}"
]
operations = ["read", "delete"]
risk_level = "low"
description = "Build artifacts and temporary files - safe to clean"
```

## Operation Controls

Global operation controls provide default permissions:

```toml
[security.operations]
read_allowed = true                      # Allow read operations globally
write_requires_policy = false           # Writes need matching policy
delete_requires_explicit_allow = true   # Deletes need explicit permission
create_dir_allowed = true               # Allow directory creation
```

### Operation Control Examples

#### Development Mode
```toml
[security.operations]
read_allowed = true
write_requires_policy = false           # Allow writes for development
delete_requires_explicit_allow = true   # Still require explicit delete
create_dir_allowed = true
```

#### Production Mode
```toml
[security.operations]
read_allowed = true
write_requires_policy = true            # All writes need policies
delete_requires_explicit_allow = true   # All deletes need explicit permission
create_dir_allowed = false              # No directory creation
```

## Custom Security Policies

### Content Creation Policy

```toml
[security.policies.content_creation]
patterns = [
    "~/blog/**/*.{md,txt}",
    "~/articles/**/*.{md,txt}",
    "~/drafts/**/*.{md,txt}"
]
operations = ["read", "write", "create_dir"]
risk_level = "low"
description = "Content creation and blogging files"
```

### Image Processing Policy

```toml
[security.policies.image_assets]
patterns = [
    "~/assets/images/**/*.{jpg,jpeg,png,gif,webp}",
    "~/photos/**/*.{jpg,jpeg,png,raw}",
    "~/screenshots/**/*.png"
]
operations = ["read", "write", "copy"]
risk_level = "low"
description = "Image files for processing and optimization"
```

### Database Files Policy

```toml
[security.policies.database_files]
patterns = [
    "~/data/**/*.{db,sqlite,sqlite3}",
    "~/backups/**/*.sql"
]
operations = ["read", "write"]
risk_level = "high"
description = "Database files - handle with care"
```

### Sensitive Configuration Policy

```toml
[security.policies.sensitive_config]
patterns = [
    "**/config/production/**",
    "**/*_secrets.toml",
    "**/api_keys.json"
]
operations = ["read"]                    # Read-only for sensitive configs
risk_level = "critical"
description = "Sensitive configuration files - read-only access"
```

## Environment-Specific Security

### Development Environment

```toml
# development.toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",
    "~/Documents/**/*",
    "./**/*"
]

[security.operations]
read_allowed = true
write_requires_policy = false
delete_requires_explicit_allow = true

[security.policies.dev_files]
patterns = ["~/dev-projects/**/*"]
operations = ["read", "write", "delete", "create_dir"]
risk_level = "low"
description = "Development files - permissive access"
```

### Staging Environment

```toml
# staging.toml
[security.filesystem]
allowed_paths = [
    "/app/staging/**/*",
    "/tmp/staging-data/**/*"
]

[security.operations]
read_allowed = true
write_requires_policy = true            # Require policies
delete_requires_explicit_allow = true

[security.policies.staging_data]
patterns = ["/app/staging/**/*.json"]
operations = ["read", "write"]
risk_level = "medium"
description = "Staging environment data files"
```

### Production Environment

```toml
# production.toml
[security.filesystem]
allowed_paths = ["/app/data/**/*.json"]
denied_paths = ["/app/secrets/**", "**/*.key"]

[security.operations]
read_allowed = true
write_requires_policy = true
delete_requires_explicit_allow = true

[security.policies.production_data]
patterns = ["/app/data/**/*.json"]
operations = ["read", "write"]
risk_level = "high"
description = "Production data files - high security"
```

## Security Validation

AIRS MCP-FS performs comprehensive security validation:

### Path Traversal Protection

Protection against directory traversal attacks:

```
❌ Blocked: ../../../etc/passwd
❌ Blocked: ..\..\..\..\windows\system32
❌ Blocked: /app/data/../secrets/key.pem
✅ Allowed: ~/projects/my-app/src/main.rs
```

### Policy Validation

Ensures security policies are correctly configured:

- **Pattern Validation**: Glob patterns are syntactically correct
- **Operation Validation**: Operations are recognized and valid
- **Risk Level Validation**: Risk levels are properly categorized
- **Consistency Checks**: No conflicting policies or permissions

### Real-time Security Monitoring

```
🔍 Security audit: high_risk_operation
   File: /app/config/database.toml
   Operation: write
   Policy: sensitive_config
   Risk Level: critical
   Result: ✅ Allowed (explicit policy match)
```

## Audit Logging

Comprehensive security audit logging:

### Log Levels by Risk

- **Low Risk**: Basic operation logging
- **Medium Risk**: Detailed operation logging with context
- **High Risk**: Comprehensive logging with full request details
- **Critical Risk**: Maximum logging with security analysis

### Audit Log Example

```json
{
  "timestamp": "2025-08-30T10:30:00Z",
  "operation": "write",
  "file_path": "/app/config/database.toml",
  "policy_matched": "sensitive_config",
  "risk_level": "critical",
  "result": "allowed",
  "user_context": "mcp_client",
  "request_id": "req_123456"
}
```

## Security Best Practices

### Policy Design

1. **Principle of Least Privilege**: Grant minimal necessary permissions
2. **Specific Patterns**: Use precise glob patterns instead of broad wildcards
3. **Risk Categorization**: Properly categorize operations by risk level
4. **Regular Reviews**: Periodically review and update policies

### Path Configuration

1. **Explicit Allowlists**: Use explicit allowed paths instead of permissive patterns
2. **Comprehensive Denylists**: Block access to sensitive directories
3. **Environment Separation**: Different path restrictions for different environments
4. **Path Testing**: Test path patterns before deployment

### Operation Security

1. **Write Protection**: Require policies for write operations in production
2. **Delete Confirmation**: Always require explicit permission for deletions
3. **Operation Auditing**: Log all security-relevant operations
4. **Failure Handling**: Secure failure modes that deny access by default

### Monitoring and Maintenance

1. **Regular Audits**: Review security logs and access patterns
2. **Policy Updates**: Keep security policies current with changing requirements
3. **Incident Response**: Have procedures for security incidents
4. **Documentation**: Maintain up-to-date security documentation

## Troubleshooting Security Issues

### Common Security Problems

1. **Access Denied Errors**
   - Check allowed_paths configuration
   - Verify no denied_paths patterns match
   - Ensure appropriate policy exists for file type

2. **Policy Not Matching**
   - Test glob patterns with intended file paths
   - Check policy operation permissions
   - Verify risk level is appropriate

3. **Environment Security Conflicts**
   - Review environment-specific configurations
   - Check environment variable overrides
   - Validate policy inheritance between environments

### Security Debugging

Enable security debugging for detailed analysis:

```bash
export RUST_LOG=debug
export AIRS_MCPSERVER_FS_ENV=development
airs-mcp-fs
```

This provides detailed logging of:
- Path validation decisions
- Policy matching process
- Security check results
- Risk assessment details

## Related Sections

- **[Configuration Overview](./overview.md)**: Overall security architecture
- **[Environment Setup](./environment.md)**: Environment-specific security
- **[Claude Desktop Integration](./claude_desktop.md)**: Client security considerations
- **[Troubleshooting](./troubleshooting.md)**: Security problem resolution
