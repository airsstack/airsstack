# Claude Desktop Integration Examples

This directory contains example configurations and setup instructions for integrating AIRS MCP-FS with Claude Desktop.

## Quick Setup

1. **Choose a configuration**: Copy one of the configuration files from `../config/` that matches your security needs:
   - `claude-desktop.toml` - Balanced security for general development
   - `secure.toml` - High security for sensitive environments
   - `educational.toml` - Permissive for learning and tutorials
   - `development.toml` - Very permissive for local development

2. **Place the configuration**: Put your chosen configuration file in a dedicated directory:
   ```bash
   mkdir -p ~/.config/airs-mcp-fs
   cp ../config/claude-desktop.toml ~/.config/airs-mcp-fs/config.toml
   ```

3. **Configure Claude Desktop**: Add the MCP server configuration to Claude Desktop's `claude_desktop_config.json`:

   **Using TOML configuration** (recommended):
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "command": "/path/to/airs-mcpserver-fs",
         "env": {
           "AIRS_MCP_FS_CONFIG_DIR": "/Users/yourname/.config/airs-mcp-fs"
         }
       }
     }
   }
   ```

   **Using direct environment variables** (legacy):
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "command": "/path/to/airs-mcpserver-fs",
         "env": {
           "AIRS_MCP_FS__SECURITY__FILESYSTEM__ALLOWED_PATHS": "~/Projects/**/*,~/Documents/**/*.md"
         }
       }
     }
   }
   ```

## Configuration Options

### TOML-based Configuration (Recommended)

The TOML approach provides:
- ✅ **Readable configuration**: Easy to understand and modify
- ✅ **Version control friendly**: Can be committed to project repositories
- ✅ **Environment-specific**: Different configs for dev/staging/production
- ✅ **Rich security policies**: Complex rules for different file types
- ✅ **Documentation**: Comments and descriptions for all settings

Set the config directory with:
```bash
export AIRS_MCP_FS_CONFIG_DIR="/path/to/your/config/directory"
```

### Environment Variables (Legacy)

Environment variables are still supported for:
- Docker containers and cloud deployments
- CI/CD pipelines
- Simple overrides of TOML settings

Variable format uses double underscores for nesting:
```bash
AIRS_MCP_FS__SECURITY__FILESYSTEM__ALLOWED_PATHS="~/Projects/**/*"
AIRS_MCP_FS__SECURITY__OPERATIONS__WRITE_REQUIRES_POLICY="false"
```

## Security Considerations

1. **Path Configuration**: Always use the most restrictive paths possible for your use case
2. **Regular Review**: Periodically review and update allowed paths
3. **Environment Separation**: Use different configurations for development vs production
4. **Audit Logging**: Monitor the server logs for security events

## Troubleshooting

### Configuration Not Loading
- Check that `AIRS_MCP_FS_CONFIG_DIR` points to the correct directory
- Verify the TOML file syntax with `cargo run --bin airs-mcpserver-fs -- generate-config --env development`
- Check the server logs for configuration errors

### Path Access Denied
- Verify the path patterns in your `allowed_paths` configuration
- Check that `denied_paths` isn't blocking your intended access
- Review the security policies for your file types

### Claude Desktop Connection Issues
- Ensure the binary path in `claude_desktop_config.json` is correct
- Check that the server starts successfully: `./airs-mcpserver-fs --help`
- Review Claude Desktop's logs for connection errors

## Example Files

- `claude_desktop_config.json` - Complete Claude Desktop configuration example
- `docker-compose.yml` - Docker deployment example
- `systemd.service` - Linux service configuration example