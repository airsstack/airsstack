# simple-mcp-server Integration Scripts

This directory contains scripts for building, testing, and integrating the simple-mcp-server with Claude Desktop.

## Quick Start

For complete automated integration:
```bash
./scripts/integrate.sh
```

## Individual Scripts

### 🔨 Build Script
```bash
./scripts/build.sh
```
- Builds optimized release binary
- Verifies binary functionality
- Always builds in release mode
- **Requires confirmation** (compilation operation)

### 🧪 MCP Inspector Testing
```bash
./scripts/test_inspector.sh
```
- Tests server with MCP Inspector
- Comprehensive positive/negative test cases
- Verifies STDIO compliance (no stderr output)
- Interactive browser-based testing
- **Automated** (no confirmation required)

### ⚙️ Claude Desktop Configuration
```bash
./scripts/configure_claude.sh
```
- Configures Claude Desktop integration
- Backs up existing configuration
- Merges with existing MCP servers
- Validates JSON configuration
- **Requires confirmation** (modifies system config)

### 🔍 Debug Integration
```bash
./scripts/debug_integration.sh
```
- Real-time debugging dashboard
- Monitors Claude Desktop logs
- Verifies configuration and binary
- Tests server connectivity
- **Automated** (no confirmation required)

### 🚀 Master Integration
```bash
./scripts/integrate.sh
```
- Complete end-to-end integration
- Orchestrates all phases
- Includes verification steps
- **Requires confirmation** (multiple system changes)

## File Structure

```
scripts/
├── build.sh              # Build optimized binary
├── test_inspector.sh     # Comprehensive testing
├── configure_claude.sh   # Claude Desktop setup
├── debug_integration.sh  # Debug dashboard
├── integrate.sh          # Master orchestration
├── utils/
│   └── paths.sh          # Centralized path definitions
└── README.md             # This documentation
```

## Key Paths

- **Project Root**: `/Users/hiraq/Projects/rstlix0x0/airs/crates/airs-mcp/examples/simple-mcp-server`
- **Binary Path**: `target/release/simple-mcp-server`
- **Log Directory**: `/tmp/simple-mcp-server/`
- **Claude Config**: `~/Library/Application Support/Claude/claude_desktop_config.json`

## Prerequisites

- **Rust/Cargo**: For building the server
- **Node.js/npx**: For MCP Inspector testing
- **Claude Desktop**: For integration testing
- **Python3**: For JSON configuration management (optional but recommended)

## Operation Types

### Automated Operations (No Confirmation)
- MCP Inspector testing
- Log monitoring
- Status verification
- Debug information gathering

### Operations Requiring Confirmation
- Binary compilation (`build.sh`)
- Configuration modification (`configure_claude.sh`)
- Claude Desktop restart (`integrate.sh`)
- Master integration process (`integrate.sh`)

## Testing Strategy

### Positive Test Cases
- ✅ Tool execution with valid parameters
- ✅ Resource access with existing URIs
- ✅ Prompt generation with all arguments
- ✅ Server initialization and capabilities
- ✅ Protocol compliance verification

### Negative Test Cases
- ❌ Invalid JSON-RPC requests
- ❌ Malformed tool parameters
- ❌ Non-existent resource access
- ❌ Missing prompt arguments
- ❌ Protocol violation handling

## Troubleshooting

### Common Issues

1. **MCP icon not visible**:
   - Run `./scripts/debug_integration.sh`
   - Check configuration: `cat ~/Library/Application\ Support/Claude/claude_desktop_config.json`
   - Restart Claude Desktop completely

2. **Tools not working**:
   - Test server: `./scripts/test_inspector.sh`
   - Check logs: `tail -f /tmp/simple-mcp-server/simple-mcp-server.log`
   - Verify no stderr output

3. **Build failures**:
   - Ensure Rust is installed: `cargo --version`
   - Clean build: `cargo clean && cargo build --release`

4. **Configuration errors**:
   - Validate JSON: `python3 -m json.tool ~/Library/Application\ Support/Claude/claude_desktop_config.json`
   - Restore backup if needed

### Debug Commands

```bash
# Monitor server logs
tail -f /tmp/simple-mcp-server/simple-mcp-server.log

# Test server manually
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | ./target/release/simple-mcp-server

# Check Claude Desktop logs
tail -f ~/Library/Logs/Claude/mcp*.log

# Verify configuration
python3 -m json.tool ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

## Safety Features

- **Automatic backups** of Claude Desktop configuration
- **Confirmation prompts** for sensitive operations
- **Error recovery** with user guidance
- **Path validation** before operations
- **JSON syntax validation** for configurations

## Integration Workflow

1. **Prerequisites Check** → Verify Rust, Node.js, Claude Desktop
2. **Build Phase** → Compile optimized release binary
3. **Inspector Testing** → Validate server functionality
4. **Configuration** → Set up Claude Desktop integration
5. **Integration Test** → Verify end-to-end functionality
6. **Monitoring** → Debug and troubleshoot issues

For questions or issues, run `./scripts/debug_integration.sh` for comprehensive diagnostics.
