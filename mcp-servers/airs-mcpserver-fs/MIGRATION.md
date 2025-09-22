# Migration Guide: airs-mcp-fs → airs-mcpserver-fs

This guide helps you migrate from the legacy `airs-mcp-fs` project to the new `airs-mcpserver-fs` architecture.

## Overview

The AIRS MCP Filesystem server has been migrated from `crates/airs-mcp-fs` to `mcp-servers/airs-mcpserver-fs` to establish proper separation between core MCP libraries and MCP server implementations. This migration provides:

- **Clean Architecture**: Proper separation of concerns
- **Ecosystem Foundation**: Prepared for additional MCP servers
- **Zero Functional Changes**: All functionality preserved
- **Improved Organization**: Better project structure

## Quick Migration (5 minutes)

### 1. Update Binary Path

**Old path:**
```bash
/path/to/airs/target/release/airs-mcp-fs
```

**New path:**
```bash
/path/to/airs/target/release/airs-mcpserver-fs
```

### 2. Update Claude Desktop Configuration

**Before:**
```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/path/to/airs-mcp-fs",
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/yourusername/.config/airs-mcp-fs"
      }
    }
  }
}
```

**After:**
```json
{
  "mcpServers": {
    "airs-mcpserver-fs": {
      "command": "/path/to/airs-mcpserver-fs",
      "env": {
        "AIRS_MCPSERVER_FS_ENV": "development",
        "AIRS_MCPSERVER_FS_CONFIG_DIR": "/Users/yourusername/.config/airs-mcpserver-fs"
      }
    }
  }
}
```

### 3. Update Configuration Directory (Optional)

You can keep using your existing configuration directory, or migrate to the new naming convention:

**Option 1: Keep existing (no changes required)**
```bash
# Your existing config continues to work
~/.config/airs-mcp-fs/
```

**Option 2: Migrate to new naming**
```bash
# Move configuration to new location
mv ~/.config/airs-mcp-fs ~/.config/airs-mcpserver-fs

# Update environment variable
export AIRS_MCPSERVER_FS_CONFIG_DIR=~/.config/airs-mcpserver-fs
```

### 4. Build and Test

```bash
# Build the new binary
cd /path/to/airs
cargo build --release --bin airs-mcpserver-fs

# Test the new setup
# Restart Claude Desktop and verify functionality
```

## Detailed Migration Steps

### Environment Variables Migration

All environment variables have been updated with the new prefix:

| Old Variable | New Variable |
|--------------|--------------|
| `AIRS_MCP_FS_ENV` | `AIRS_MCPSERVER_FS_ENV` |
| `AIRS_MCP_FS_CONFIG_DIR` | `AIRS_MCPSERVER_FS_CONFIG_DIR` |
| `AIRS_MCP_FS_LOG_DIR` | `AIRS_MCPSERVER_FS_LOG_DIR` |
| `AIRS_MCP_FS_SECURITY_*` | `AIRS_MCPSERVER_FS_SECURITY_*` |

### Configuration File Migration

#### Option 1: Update Configuration Files
```bash
# Update your existing config files to use new binary name
sed -i 's/airs-mcp-fs/airs-mcpserver-fs/g' ~/.config/airs-mcp-fs/*.toml
```

#### Option 2: Use Legacy Configuration
Your existing configuration files will continue to work without changes. The new binary maintains full backward compatibility.

### Shell Scripts and Automation

Update any shell scripts or automation that references the old binary:

```bash
# Old
./target/release/airs-mcp-fs --config development.toml

# New  
./target/release/airs-mcpserver-fs --config development.toml
```

## Validation Steps

After migration, verify everything works correctly:

### 1. Build Test
```bash
cargo build --release --bin airs-mcpserver-fs
echo "Build status: $?"  # Should be 0
```

### 2. Configuration Test
```bash
./target/release/airs-mcpserver-fs --help
# Should show help without errors
```

### 3. Claude Desktop Integration Test
1. Restart Claude Desktop
2. Test basic commands:
   - "List files in my current directory"
   - "Read the contents of README.md"
   - "Create a test file"

### 4. Performance Validation
- Response times should remain sub-100ms
- Memory usage should be unchanged
- All security features should work identically

## Rollback Plan

If you encounter issues, you can easily rollback:

### 1. Restore Claude Desktop Configuration
```bash
# Restore your backup of claude_desktop_config.json
cp claude_desktop_config.json.backup ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

### 2. Use Legacy Binary
```bash
# Build and use the legacy version
cargo build --release --bin airs-mcp-fs
# Update Claude config to point to old binary
```

### 3. Restore Environment Variables
```bash
# Reset to legacy environment variables
export AIRS_MCP_FS_ENV=development
export AIRS_MCP_FS_CONFIG_DIR=~/.config/airs-mcp-fs
```

## Backward Compatibility

The workspace maintains both versions during the transition period:

- **Legacy**: `crates/airs-mcp-fs` (preserved for compatibility)
- **New**: `mcp-servers/airs-mcpserver-fs` (recommended for new installations)

Both versions:
- ✅ Provide identical functionality
- ✅ Use the same configuration format
- ✅ Maintain the same performance characteristics
- ✅ Support the same security features

## FAQ

### Q: Will my existing configuration work?
**A:** Yes! The new binary maintains full backward compatibility with existing configuration files and directories.

### Q: Do I need to migrate my configuration directory?
**A:** No, it's optional. Your existing `~/.config/airs-mcp-fs/` directory will continue to work. Migration to `~/.config/airs-mcpserver-fs/` is recommended but not required.

### Q: What if I have custom automation or scripts?
**A:** Update any hardcoded paths to use `airs-mcpserver-fs` instead of `airs-mcp-fs`. The command-line interface remains identical.

### Q: Can I run both versions simultaneously?
**A:** Yes, during the transition period you can have both configured in Claude Desktop with different names (e.g., "airs-mcp-fs-legacy" and "airs-mcpserver-fs").

### Q: When will the legacy version be deprecated?
**A:** The legacy version will be maintained for at least 6 months to ensure smooth transition. Deprecation timeline will be announced with advance notice.

## Support

If you encounter issues during migration:

1. **Check the troubleshooting section** in the main README
2. **Verify your configuration** against the examples in this guide
3. **Test with the legacy version** to isolate migration-specific issues
4. **Report issues** with detailed steps to reproduce

## Migration Checklist

- [ ] Build new binary: `cargo build --release --bin airs-mcpserver-fs`
- [ ] Update Claude Desktop configuration JSON
- [ ] Update environment variables (if using)
- [ ] Update any scripts or automation
- [ ] Test basic functionality with Claude Desktop
- [ ] Verify performance characteristics
- [ ] Update bookmarks/documentation to point to new binary
- [ ] (Optional) Migrate configuration directory
- [ ] Create backup of working legacy configuration

**Estimated migration time:** 5-15 minutes depending on complexity of your setup.

## What's Next

After successful migration, you're ready to take advantage of:

- **Future MCP servers** that will be added to the `mcp-servers/` directory
- **Improved documentation** with clearer examples and guides
- **Better project organization** that scales with the AIRS ecosystem

Welcome to the new AIRS MCP Server architecture! 🚀