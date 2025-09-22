# Quick Start Guide

Get up and running with AIRS MCP Server - Filesystem in under 5 minutes.

## Prerequisites

- **Rust**: Version 1.88.0 or later
- **Claude Desktop**: Latest version with MCP support
- **Operating System**: macOS, Linux, or Windows

## Installation

### Option 1: From Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/rstlix0x0/airs.git
cd airs

# Build the binary
cargo build --release --bin airs-mcpserver-fs

# The binary will be at: target/release/airs-mcpserver-fs
```

### Option 2: Using Cargo
```bash
cargo install --path mcp-servers/airs-mcpserver-fs
```

## Basic Setup

### Step 1: Generate Configuration
```bash
# Generate development configuration
airs-mcpserver-fs generate-config

# This creates ~/.config/airs-mcpserver-fs/development.toml
```

### Step 2: Configure Claude Desktop

Add to your Claude Desktop MCP configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "airs-mcpserver-fs": {
      "command": "/path/to/airs-mcpserver-fs",
      "env": {
        "AIRS_MCPSERVER_FS_ENV": "development",
        "AIRS_MCPSERVER_FS_CONFIG_DIR": "/Users/yourusername/.config/airs-mcpserver-fs",
        "AIRS_MCPSERVER_FS_LOG_DIR": "/Users/yourusername/.local/share/airs-mcpserver-fs/logs"
      }
    }
  }
}
```

**Important**: Replace `/path/to/airs-mcpserver-fs` with the actual path to your binary and `yourusername` with your actual username.

### Step 3: Restart Claude Desktop

Restart Claude Desktop to load the new MCP server configuration.

## Test Your Setup

Once Claude Desktop restarts, try these commands:

```
User: "List the files in my Documents directory"
Claude: *uses list_directory tool* → shows your Documents contents

User: "Read my project's README.md file"
Claude: *uses read_file tool* → displays the README content

User: "Create a new file called hello.txt with 'Hello World' in my Documents"
Claude: *uses write_file tool* → creates the file with approval prompt
```

## Common Issues

### "Permission Denied" Errors
- **Cause**: Path not in allowed_paths configuration
- **Solution**: Edit your configuration file to include the required paths

### "Security validation failed"
- **Cause**: File pattern not matching security policies
- **Solution**: Check glob patterns in your configuration file

### "Invalid server response"
- **Cause**: Environment variables not set correctly
- **Solution**: Verify AIRS_MCPSERVER_FS_ENV and config directory paths

## Next Steps

- **[Configuration Guide](./configuration.md)**: Customize security policies and file access
- **[Security Policies](./configuration/security.md)**: Understand the security model
- **[Claude Desktop Integration](./configuration/claude_desktop.md)**: Advanced integration options
- **[Troubleshooting](./configuration/troubleshooting.md)**: Solve common problems

## Development Configuration Example

For development work, your configuration should include your project directories:

```toml
# ~/.config/airs-mcpserver-fs/development.toml
[security.filesystem]
allowed_paths = [
    "~/projects/**/*",           # All your projects
    "~/Documents/**/*",          # Documents directory
    "~/Desktop/**/*",            # Desktop files
    "./**/*"                     # Current directory when running from project
]

[security.operations]
read_allowed = true
write_requires_policy = false    # Allow writes in development
delete_requires_explicit_allow = true

# Named policies for different file types
[security.policies.journal_files]
patterns = ["/Users/yourusername/Documents/**/*"]
operations = ["read", "write", "list"]
risk_level = "low"
description = "Personal journal and document files"
```

This configuration allows Claude to work with your development files while maintaining reasonable security boundaries.
