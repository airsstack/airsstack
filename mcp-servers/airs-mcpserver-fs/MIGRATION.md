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

## Troubleshooting

### Side-by-Side Configuration Comparison

**Legacy Configuration (airs-mcp-fs):**
```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "command": "/path/to/airs/target/release/airs-mcp-fs",
      "args": ["serve"],
      "env": {
        "AIRS_MCP_FS_ENV": "development",
        "AIRS_MCP_FS_CONFIG_DIR": "/Users/username/.config/airs-mcp-fs",
        "AIRS_MCP_FS_ROOT_DIR": "/Users/username/projects",
        "AIRS_MCP_FS_ALLOWED_PATHS": "/Users/username/projects,/Users/username/docs"
      }
    }
  }
}
```

**New Configuration (airs-mcpserver-fs):**
```json
{
  "mcpServers": {
    "airs-mcpserver-fs": {
      "command": "/path/to/airs/target/release/airs-mcpserver-fs",
      "args": ["serve"],
      "env": {
        "AIRS_MCPSERVER_FS_ENV": "development",
        "AIRS_MCPSERVER_FS_CONFIG_DIR": "/Users/username/.config/airs-mcpserver-fs",
        "AIRS_MCPSERVER_FS_ROOT_DIR": "/Users/username/projects",
        "AIRS_MCPSERVER_FS_ALLOWED_PATHS": "/Users/username/projects,/Users/username/docs"
      }
    }
  }
}
```

### Common Migration Issues

#### Issue: "Binary not found" error
**Symptoms:** Claude Desktop shows connection error or "command not found"
**Solution:**
1. Verify binary exists: `ls -la target/release/airs-mcpserver-fs`
2. Check permissions: `chmod +x target/release/airs-mcpserver-fs`
3. Use absolute path in Claude config: `/full/path/to/airs/target/release/airs-mcpserver-fs`

#### Issue: Environment variables not recognized
**Symptoms:** Server starts but can't access expected directories
**Solution:**
1. Double-check variable name conversion: `AIRS_MCP_FS_*` → `AIRS_MCPSERVER_FS_*`
2. Verify environment variable values with: `echo $AIRS_MCPSERVER_FS_ROOT_DIR`
3. Test with minimal config (no env vars) first

#### Issue: Claude Desktop can't connect to server
**Symptoms:** "Failed to connect to MCP server" in Claude Desktop
**Solution:**
1. Test server manually: `./target/release/airs-mcpserver-fs --help`
2. Check Claude Desktop logs: `~/Library/Logs/Claude/`
3. Try temporary config with minimal settings
4. Verify JSON syntax in claude_desktop_config.json

#### Issue: Configuration directory not found
**Symptoms:** Server complains about missing config files
**Solution:**
1. Create new config directory: `mkdir -p ~/.config/airs-mcpserver-fs`
2. Copy existing config: `cp -r ~/.config/airs-mcp-fs/* ~/.config/airs-mcpserver-fs/`
3. Or keep existing path: Use `AIRS_MCPSERVER_FS_CONFIG_DIR=~/.config/airs-mcp-fs`

### Validation Commands

**Test new binary:**
```bash
# Build and verify
cargo build --release --package airs-mcpserver-fs
./target/release/airs-mcpserver-fs --help

# Test startup (should show deprecation warning for comparison)
echo 'exit' | ./target/release/airs-mcp-fs serve
echo 'exit' | ./target/release/airs-mcpserver-fs serve
```

**Verify environment:**
```bash
# Check all new environment variables
env | grep AIRS_MCPSERVER_FS_

# Compare with legacy (should be empty after migration)
env | grep AIRS_MCP_FS_
```

### Debugging Steps

1. **Isolate the Issue:**
   - Does legacy version still work? `./target/release/airs-mcp-fs --help`
   - Does new binary build? `cargo build --release --package airs-mcpserver-fs`
   - Can you run new binary? `./target/release/airs-mcpserver-fs --help`

2. **Test Configuration:**
   - Validate JSON syntax: Use online JSON validator or `python -m json.tool claude_desktop_config.json`
   - Check file permissions: `ls -la ~/Library/Application\ Support/Claude/claude_desktop_config.json`

3. **Progressive Testing:**
   - Start with minimal config (just command path)
   - Add environment variables one by one
   - Test each change with Claude Desktop restart

### Performance Validation

**Expected identical performance:**
```bash
# Time comparison (should be nearly identical)
time ./target/release/airs-mcp-fs --help
time ./target/release/airs-mcpserver-fs --help

# Memory usage (using Activity Monitor or htop)
# Response times should remain sub-100ms
```

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
**A:** The legacy version will be maintained until **December 31, 2025** to ensure smooth transition. Here's the timeline:

**Migration Timeline:**
- **Phase 1 (September 2025)**: New architecture available, both versions supported
- **Phase 2 (October-November 2025)**: Migration notices active, documentation updated
- **Phase 3 (December 2025)**: Final month for migration, increased deprecation warnings
- **Phase 4 (January 1, 2026)**: Legacy version removed from workspace

**Deprecation Notices:**
- Legacy binary shows migration notice on startup
- Legacy README prominently displays migration information
- Documentation consistently points to new version

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