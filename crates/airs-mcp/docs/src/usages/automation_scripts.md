# Automation Scripts

*Production-grade automation infrastructure for seamless Claude Desktop integration*

## Overview

AIRS MCP includes a comprehensive automation infrastructure that transforms Claude Desktop integration from a complex manual process into a single-command operation. The script suite handles everything from building optimized binaries to real-time debugging and monitoring.

> **Production Ready**: All scripts have been battle-tested in production environments with comprehensive error handling and safety measures.

## Complete Script Suite

### 🚀 Master Integration Script

**`./scripts/integrate.sh`** - Complete end-to-end integration

```bash
# Single command for complete integration
./scripts/integrate.sh
```

**What it does:**
- Orchestrates the entire integration workflow
- Builds optimized release binary
- Runs comprehensive MCP Inspector tests
- Configures Claude Desktop with safety backups
- Verifies end-to-end functionality
- Provides real-time debugging dashboard

**Features:**
- ✅ **Safety First**: Automatic configuration backups with timestamp recovery
- ✅ **Comprehensive Testing**: MCP Inspector validation before Claude integration
- ✅ **Error Recovery**: Intelligent error handling with clear recovery instructions
- ✅ **User Confirmation**: Prompts for sensitive operations (build, config changes)

### 🔨 Build Script

**`./scripts/build.sh`** - Optimized binary compilation

```bash
# Build production-ready binary
./scripts/build.sh
```

**Features:**
- Always builds in release mode for optimal performance
- Verifies binary functionality after compilation
- Asks for user confirmation before compilation
- Provides clear success/failure feedback

### 🧪 Testing Script

**`./scripts/test_inspector.sh`** - Comprehensive MCP validation

```bash
# Run complete MCP Inspector test suite
./scripts/test_inspector.sh
```

**Testing Coverage:**
- ✅ **Schema Validation**: 100% MCP 2024-11-05 specification compliance
- ✅ **Positive Test Cases**: All standard MCP operations
- ✅ **Negative Test Cases**: Error handling and edge cases
- ✅ **STDIO Compliance**: Ensures no stderr output contamination
- ✅ **Interactive Browser Testing**: Visual validation with MCP Inspector UI

**What gets tested:**
```
✓ Server initialization and capabilities
✓ Tool listing and execution
✓ Resource discovery and access
✓ Prompt listing and retrieval
✓ Error handling and recovery
✓ Protocol compliance validation
✓ Connection lifecycle management
```

### ⚙️ Configuration Script

**`./scripts/configure_claude.sh`** - Claude Desktop setup

```bash
# Configure Claude Desktop integration
./scripts/configure_claude.sh
```

**Safety Features:**
- ✅ **Automatic Backups**: Creates timestamped config backups
- ✅ **JSON Validation**: Validates configuration before applying
- ✅ **Merge Strategy**: Preserves existing MCP servers
- ✅ **Path Verification**: Ensures binary paths are absolute and valid
- ✅ **User Confirmation**: Asks before modifying system configuration

**Configuration Details:**
- Locates Claude Desktop config file automatically
- Uses absolute paths for binary references
- Sets appropriate environment variables
- Validates configuration syntax

### 🔍 Debug Script

**`./scripts/debug_integration.sh`** - Real-time debugging dashboard

```bash
# Launch debugging dashboard
./scripts/debug_integration.sh
```

**Debugging Features:**
- ✅ **Real-time Log Monitoring**: Tails server logs with syntax highlighting
- ✅ **Configuration Verification**: Validates Claude Desktop configuration
- ✅ **Binary Testing**: Tests server connectivity and responsiveness
- ✅ **System Status**: Checks all prerequisites and dependencies
- ✅ **Interactive Monitoring**: Live updates with formatted output

**Debug Information:**
```
✓ Claude Desktop configuration status
✓ Server binary location and permissions
✓ Log file monitoring and analysis
✓ Connection testing and validation
✓ System prerequisites verification
```

## Script Features & Safety

### Confirmation Strategy

Scripts follow a **user-first approach** for sensitive operations:

```bash
# Operations requiring confirmation:
- Binary compilation (build.sh)
- Configuration modification (configure_claude.sh)
- Complete integration (integrate.sh)

# Automated operations:
- Testing (test_inspector.sh)
- Debugging (debug_integration.sh)
- Log monitoring and status checks
```

### Error Recovery

All scripts implement **ask-first error handling**:

1. **Detect Error**: Comprehensive error detection with specific error codes
2. **User Notification**: Clear explanation of what went wrong
3. **Recovery Options**: Specific instructions for manual recovery
4. **Safe Defaults**: Conservative choices that preserve existing state

### Logging Strategy

- ✅ **Terminal Output**: All script progress displayed to terminal
- ✅ **Server Logs**: File-based logging for server operations (`/tmp/simple-mcp-server/`)
- ✅ **No Contamination**: STDIO transport compliance (no stderr output from server)
- ✅ **Structured Logging**: JSON-formatted logs with proper levels

## Prerequisites

The scripts automatically verify all prerequisites:

```bash
# Required tools (automatically checked):
✓ Rust/Cargo - For building the server
✓ Node.js/npx - For MCP Inspector testing  
✓ Claude Desktop - For integration testing
✓ Python3 - For JSON configuration management (optional)
```

## Integration Workflow

The complete integration follows this proven workflow:

```
1. Prerequisites Check → Verify all required tools
2. Build Phase → Compile optimized release binary  
3. Inspector Testing → Comprehensive MCP validation
4. Configuration → Claude Desktop setup with backups
5. Integration Verification → End-to-end testing
6. Debug Dashboard → Real-time monitoring setup
```

## File Structure

```
scripts/
├── integrate.sh           # Master orchestration script
├── build.sh              # Optimized binary building
├── test_inspector.sh     # MCP Inspector testing
├── configure_claude.sh   # Claude Desktop configuration
├── debug_integration.sh  # Debug dashboard
├── utils/
│   └── paths.sh          # Centralized path definitions
└── README.md             # Detailed script documentation
```

## Path Configuration

All scripts use centralized path management through `utils/paths.sh`:

```bash
# Key paths (automatically detected):
PROJECT_ROOT="/path/to/airs/crates/airs-mcp/examples/simple-mcp-server"
BINARY_PATH="target/release/simple-mcp-server"
LOG_DIR="/tmp/simple-mcp-server/"
CLAUDE_CONFIG="~/Library/Application Support/Claude/claude_desktop_config.json"
```

## Quick Reference

```bash
# Complete integration (recommended)
./scripts/integrate.sh

# Individual operations
./scripts/build.sh                    # Build binary
./scripts/test_inspector.sh          # Test with MCP Inspector
./scripts/configure_claude.sh        # Configure Claude Desktop  
./scripts/debug_integration.sh       # Debug dashboard

# Get help
./scripts/integrate.sh --help
```

## Troubleshooting

If you encounter issues, the scripts provide specific guidance:

1. **Build Failures**: Check Rust installation and dependencies
2. **Test Failures**: Review MCP Inspector output for specific errors
3. **Configuration Issues**: Verify Claude Desktop installation and permissions
4. **Integration Problems**: Use debug dashboard for real-time diagnostics

**Debug Command**: `./scripts/debug_integration.sh` provides comprehensive diagnostics for all common issues.
