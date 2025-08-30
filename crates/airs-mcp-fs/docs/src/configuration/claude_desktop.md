# Claude Desktop Integration

This section covers comprehensive integration of AIRS MCP-FS with Claude Desktop, including configuration, troubleshooting, and advanced usage patterns.

## MCP Server Configuration

Claude Desktop uses JSON configuration to define MCP servers. AIRS MCP-FS integrates as a standard MCP server with additional environment variable support.

### Basic Configuration

Add AIRS MCP-FS to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`  
**Linux**: `~/.config/claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/path/to/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development"
      }
    }
  }
}
```

### Complete Configuration

For full control over AIRS MCP-FS behavior:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/Users/username/path/to/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/username/.config/airs-mcp-fs",
        "AIRS_MCP_FS_LOG_DIR": "/Users/username/.local/share/airs-mcp-fs/logs",
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Environment Variable Configuration

AIRS MCP-FS behavior is controlled through environment variables in the Claude Desktop configuration:

### Core Environment Variables

| Variable | Purpose | Example | Required |
|----------|---------|---------|----------|
| `AIRS_MCP_FS_ENV` | Environment type | `development` | Yes |
| `AIRS_MCP_FS_CONFIG_DIR` | Configuration directory | `~/.config/airs-mcp-fs` | Recommended |
| `AIRS_MCP_FS_LOG_DIR` | Log output directory | `~/.local/share/airs-mcp-fs/logs` | Recommended |
| `RUST_LOG` | Logging level | `info`, `debug` | Optional |

### Security Override Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS` | Override allowed paths | `~/projects/**/*,~/docs/**/*` |
| `AIRS_MCP_FS_SECURITY_OPERATIONS_WRITE_REQUIRES_POLICY` | Control write policy | `false` |
| `AIRS_MCP_FS_BINARY_MAX_FILE_SIZE` | Max file size | `52428800` (50MB) |

## Configuration Examples

### Development Workstation

Ideal for local development with broad file access:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/developer/.config/airs-mcp-fs",
        "AIRS_MCP_FS_LOG_DIR": "/Users/developer/.local/share/airs-mcp-fs/logs",
        "AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS": "/Users/developer/projects/**/*,/Users/developer/Documents/**/*,/Users/developer/Desktop/**/*",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Content Creation Setup

Optimized for writing, blogging, and content management:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/writer/.config/airs-mcp-fs",
        "AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS": "/Users/writer/writing/**/*,/Users/writer/blog/**/*,/Users/writer/assets/**/*",
        "AIRS_MCP_FS_BINARY_ENABLE_IMAGE_PROCESSING": "true",
        "AIRS_MCP_FS_BINARY_MAX_FILE_SIZE": "209715200"
      }
    }
  }
}
```

### Secure Production Environment

Minimal permissions for production use:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/opt/airs-mcp-fs/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "production",
        "AIRS_MCP_FS_CONFIG_DIR": "/etc/airs-mcp-fs",
        "AIRS_MCP_FS_LOG_DIR": "/var/log/airs-mcp-fs",
        "AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS": "/app/data/**/*.json",
        "AIRS_MCP_FS_SECURITY_OPERATIONS_WRITE_REQUIRES_POLICY": "true",
        "RUST_LOG": "warn"
      }
    }
  }
}
```

## Integration Workflow

### Initial Setup Process

1. **Install AIRS MCP-FS**
   ```bash
   cargo build --release --bin airs-mcp-fs
   ```

2. **Generate Configuration**
   ```bash
   airs-mcp-fs generate-config --env development
   ```

3. **Configure Claude Desktop**
   ```bash
   # Edit Claude Desktop configuration
   nano "~/Library/Application Support/Claude/claude_desktop_config.json"
   ```

4. **Restart Claude Desktop**
   - Quit Claude Desktop completely
   - Restart to load new MCP server configuration

5. **Verify Integration**
   ```
   User: "List the files in my current directory"
   Claude: *uses list_directory tool* → shows directory contents
   ```

### Configuration Validation

After setup, validate your configuration:

1. **Check Configuration Loading**
   ```bash
   # Run AIRS MCP-FS manually to check configuration
   AIRS_MCP_FS_ENV=development /path/to/airs-mcp-fs
   ```

2. **Verify File Access**
   ```
   User: "Can you read my README.md file?"
   Claude: *uses read_file tool* → displays README content
   ```

3. **Test Security Policies**
   ```
   User: "Try to read a file in a restricted directory"
   Claude: *attempts access* → should receive permission denied
   ```

## Advanced Integration Patterns

### Multi-Environment Setup

Configure different AIRS MCP-FS instances for different environments:

```json
{
  "mcpServers": {
    "airs-mcp-fs-dev": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/dev/.config/airs-mcp-fs-dev"
      }
    },
    "airs-mcp-fs-prod": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "production",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/dev/.config/airs-mcp-fs-prod"
      }
    }
  }
}
```

### Project-Specific Configuration

Use different configurations for different projects:

```json
{
  "mcpServers": {
    "airs-mcp-fs-project-a": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS": "/Users/dev/project-a/**/*"
      }
    },
    "airs-mcp-fs-project-b": {
      "command": "/usr/local/bin/airs-mcp-fs", 
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_SECURITY_FILESYSTEM_ALLOWED_PATHS": "/Users/dev/project-b/**/*"
      }
    }
  }
}
```

## MCP Tool Usage

Once configured, Claude Desktop can use AIRS MCP-FS tools:

### Available Tools

| Tool | Purpose | Example Usage |
|------|---------|---------------|
| `read_file` | Read file contents | "Read my package.json file" |
| `write_file` | Create or update files | "Create a new component file" |
| `list_directory` | Browse directories | "List files in my src directory" |
| `create_directory` | Create directories | "Create a new folder structure" |
| `delete_file` | Remove files | "Delete the old backup files" |
| `move_file` | Move/rename files | "Rename this file to main.rs" |
| `copy_file` | Copy files | "Make a backup copy of this file" |

### Example Interactions

#### Reading Project Files
```
User: "Analyze the structure of my Rust project"
Claude: *uses list_directory and read_file tools*
→ Examines Cargo.toml, src/ directory, and main source files
→ Provides comprehensive project analysis
```

#### Creating New Files
```
User: "Create a new React component for user authentication"
Claude: *uses write_file tool*
→ Creates src/components/AuthComponent.tsx
→ Includes proper TypeScript interfaces and React hooks
```

#### File Organization
```
User: "Organize my Downloads folder by file type"
Claude: *uses list_directory, create_directory, and move_file tools*
→ Creates folders by file type
→ Moves files into appropriate directories
→ Provides summary of organization
```

## Troubleshooting Integration

### Common Integration Issues

#### MCP Server Not Loading

**Symptoms**: Claude Desktop doesn't show filesystem tools available

**Solutions**:
1. **Check Binary Path**
   ```bash
   # Verify binary exists and is executable
   ls -la /path/to/airs-mcp-fs
   chmod +x /path/to/airs-mcp-fs
   ```

2. **Validate JSON Configuration**
   ```bash
   # Check JSON syntax
   cat "~/Library/Application Support/Claude/claude_desktop_config.json" | python -m json.tool
   ```

3. **Check Logs**
   ```bash
   # Check AIRS MCP-FS logs
   tail -f ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log
   ```

#### Permission Denied Errors

**Symptoms**: "Security validation failed: Access denied"

**Solutions**:
1. **Check Allowed Paths**
   ```toml
   # Verify path is in allowed_paths
   [security.filesystem]
   allowed_paths = ["/Users/username/Documents/**/*"]
   ```

2. **Check Glob Patterns**
   ```toml
   # Ensure patterns include both directory and contents
   allowed_paths = [
       "/Users/username/Documents",      # Directory itself
       "/Users/username/Documents/**/*"  # Directory contents
   ]
   ```

3. **Review Security Policies**
   ```toml
   # Check if operation is allowed by policy
   [security.policies.documents]
   patterns = ["/Users/username/Documents/**/*"]
   operations = ["read", "write", "list"]
   ```

#### Configuration Not Loading

**Symptoms**: AIRS MCP-FS using default configuration instead of custom

**Solutions**:
1. **Verify Environment Variables**
   ```json
   {
     "env": {
       "AIRS_MCP_FS_ENV": "development",
       "AIRS_MCP_FS_CONFIG_DIR": "/correct/path/to/config"
     }
   }
   ```

2. **Check File Existence**
   ```bash
   # Verify configuration file exists
   ls -la ~/.config/airs-mcp-fs/development.toml
   ```

3. **Validate Configuration Syntax**
   ```bash
   # Test configuration loading
   AIRS_MCP_FS_ENV=development /path/to/airs-mcp-fs
   ```

### Debug Mode

Enable debug logging for detailed troubleshooting:

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/usr/local/bin/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "RUST_LOG": "debug"
      }
    }
  }
}
```

This provides detailed logging of:
- Configuration loading process
- Security validation decisions
- MCP protocol communication
- File operation execution

## Integration Best Practices

### Security Best Practices

1. **Environment-Specific Configuration**: Use different configurations for different environments
2. **Minimal Permissions**: Grant only necessary file access permissions
3. **Regular Updates**: Keep AIRS MCP-FS updated for security fixes
4. **Audit Logging**: Monitor file access patterns and unusual activity

### Configuration Management

1. **Version Control**: Keep configuration files in version control
2. **Documentation**: Document environment-specific settings
3. **Testing**: Test configuration changes before deploying
4. **Backup**: Maintain backup configurations for rollback

### Performance Optimization

1. **Specific Paths**: Use specific path patterns to reduce security checking overhead
2. **File Size Limits**: Set appropriate file size limits for your use case
3. **Binary Processing**: Disable unused binary processing features
4. **Log Levels**: Use appropriate log levels (avoid debug in production)

### Maintenance

1. **Regular Reviews**: Periodically review and update configurations
2. **Log Monitoring**: Monitor logs for errors and security issues
3. **Performance Monitoring**: Track file operation performance
4. **Update Planning**: Plan for AIRS MCP-FS updates and migrations

## Related Sections

- **[Configuration Overview](./overview.md)**: Overall configuration architecture
- **[Environment Setup](./environment.md)**: Environment-specific configuration
- **[Security Policies](./security.md)**: Security policy configuration
- **[Troubleshooting](./troubleshooting.md)**: Detailed troubleshooting guide
