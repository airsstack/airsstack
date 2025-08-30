# Configuration Troubleshooting

This section provides comprehensive troubleshooting guidance for common AIRS MCP-FS configuration issues, complete with diagnostic steps and solutions.

## Quick Diagnostic Checklist

When experiencing issues, run through this checklist:

1. **✅ Binary Accessibility**: Can you run `airs-mcp-fs --help`?
2. **✅ Configuration File**: Does your config file exist and have correct syntax?
3. **✅ Environment Detection**: Is `AIRS_MCP_FS_ENV` set correctly?
4. **✅ Path Permissions**: Are your allowed paths accessible?
5. **✅ Claude Desktop Config**: Is the JSON configuration syntactically correct?
6. **✅ Log Files**: Are there error messages in the log files?

## Configuration Loading Issues

### Problem: Configuration File Not Found

**Symptoms**:
```
📋 Configuration loaded from production environment
   Configuration files: []
   Environment variables: 0 overrides
   
⚠️  Using built-in defaults - no configuration file found
```

**Diagnostic Steps**:
```bash
# Check if environment is detected correctly
echo $AIRS_MCP_FS_ENV

# Check if config directory exists
ls -la ~/.config/airs-mcp-fs/

# Check if config file exists for your environment
ls -la ~/.config/airs-mcp-fs/development.toml
```

**Solutions**:

1. **Generate Missing Configuration**
   ```bash
   # Generate development configuration
   airs-mcp-fs generate-config --env development

   # Generate for specific environment
   airs-mcp-fs generate-config --env production --output ~/.config/airs-mcp-fs
   ```

2. **Set Explicit Config Directory**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "env": {
           "AIRS_MCP_FS_CONFIG_DIR": "/Users/username/.config/airs-mcp-fs"
         }
       }
     }
   }
   ```

3. **Create Manual Configuration**
   ```bash
   # Create config directory
   mkdir -p ~/.config/airs-mcp-fs

   # Create basic development configuration
   cat > ~/.config/airs-mcp-fs/development.toml << 'EOF'
   [security.filesystem]
   allowed_paths = ["~/Documents/**/*", "~/projects/**/*"]
   
   [security.operations]
   read_allowed = true
   write_requires_policy = false
   delete_requires_explicit_allow = true
   EOF
   ```

### Problem: Configuration Syntax Errors

**Symptoms**:
```
❌ Configuration loading failed: TOML parse error at line 5, column 1
```

**Diagnostic Steps**:
```bash
# Validate TOML syntax
python3 -c "import tomllib; tomllib.load(open('~/.config/airs-mcp-fs/development.toml', 'rb'))"

# Or use a TOML validator
toml-validator ~/.config/airs-mcp-fs/development.toml
```

**Solutions**:

1. **Fix Common TOML Errors**
   ```toml
   # ❌ Missing quotes around strings
   allowed_paths = [~/Documents/**/*]
   
   # ✅ Correct string quoting
   allowed_paths = ["~/Documents/**/*"]
   
   # ❌ Invalid section header
   [security][filesystem]
   
   # ✅ Correct nested section
   [security.filesystem]
   ```

2. **Validate Configuration Structure**
   ```bash
   # Test configuration loading manually
   RUST_LOG=debug airs-mcp-fs 2>&1 | grep -i config
   ```

## Permission and Security Issues

### Problem: "Security validation failed: Access denied"

**Symptoms**:
```
❌ Security validation failed: Access denied for path: /Users/username/Documents/file.txt
```

**Diagnostic Steps**:
```bash
# Check current configuration
cat ~/.config/airs-mcp-fs/development.toml | grep -A 5 allowed_paths

# Test path matching manually
RUST_LOG=debug airs-mcp-fs 2>&1 | grep -i "path validation"
```

**Root Causes and Solutions**:

1. **Path Not in Allowed Paths**
   ```toml
   # ❌ Path not included
   [security.filesystem]
   allowed_paths = ["~/projects/**/*"]
   
   # ✅ Add required path
   [security.filesystem]
   allowed_paths = [
       "~/projects/**/*",
       "~/Documents/**/*"  # Add this line
   ]
   ```

2. **Glob Pattern Doesn't Match**
   ```toml
   # ❌ Pattern doesn't match directory itself
   allowed_paths = ["~/Documents/**/*"]  # Matches contents but not directory
   
   # ✅ Include both directory and contents
   allowed_paths = [
       "~/Documents",        # Directory itself
       "~/Documents/**/*"    # Directory contents
   ]
   ```

3. **Denied Paths Taking Precedence**
   ```toml
   # Check if path is in denied_paths
   [security.filesystem]
   allowed_paths = ["~/Documents/**/*"]
   denied_paths = ["~/.*/**"]  # This might block ~/Documents
   
   # ✅ Make denied paths more specific
   denied_paths = [
       "**/.git/**",
       "**/.env*",
       "~/Library/**"  # More specific than ~/.*/**
   ]
   ```

### Problem: "Write operation requires policy"

**Symptoms**:
```
❌ Write operation denied: No matching policy found for /Users/username/Documents/file.txt
```

**Diagnostic Steps**:
```bash
# Check operation configuration
cat ~/.config/airs-mcp-fs/development.toml | grep -A 5 operations

# Check security policies
cat ~/.config/airs-mcp-fs/development.toml | grep -A 10 policies
```

**Solutions**:

1. **Disable Policy Requirement for Development**
   ```toml
   [security.operations]
   read_allowed = true
   write_requires_policy = false  # Allow writes without policy
   delete_requires_explicit_allow = true
   ```

2. **Create Appropriate Security Policy**
   ```toml
   [security.policies.document_files]
   patterns = ["~/Documents/**/*.{txt,md,doc}"]
   operations = ["read", "write"]
   risk_level = "low"
   description = "Personal document files"
   ```

3. **Use Environment Variables Override**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "env": {
           "AIRS_MCP_FS_SECURITY_OPERATIONS_WRITE_REQUIRES_POLICY": "false"
         }
       }
     }
   }
   ```

## Environment Detection Issues

### Problem: Wrong Environment Detected

**Symptoms**:
```
📋 Configuration loaded from production environment
```
(When you expected development)

**Diagnostic Steps**:
```bash
# Check environment variables
echo "AIRS_MCP_FS_ENV: $AIRS_MCP_FS_ENV"
echo "NODE_ENV: $NODE_ENV"
echo "ENVIRONMENT: $ENVIRONMENT"

# Check if running in debug mode
cargo --version
rustc --version
```

**Solutions**:

1. **Set Explicit Environment Variable**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "env": {
           "AIRS_MCP_FS_ENV": "development"
         }
       }
     }
   }
   ```

2. **Check for Conflicting Environment Variables**
   ```bash
   # Unset conflicting variables
   unset NODE_ENV
   unset ENVIRONMENT
   
   # Set AIRS-specific variable
   export AIRS_MCP_FS_ENV=development
   ```

## Claude Desktop Integration Issues

### Problem: MCP Server Not Loading

**Symptoms**: Claude Desktop doesn't show filesystem tools available

**Diagnostic Steps**:
```bash
# Check if binary exists and is executable
ls -la /path/to/airs-mcp-fs
file /path/to/airs-mcp-fs

# Test binary directly
/path/to/airs-mcp-fs --help

# Validate JSON configuration
python3 -m json.tool < "~/Library/Application Support/Claude/claude_desktop_config.json"
```

**Solutions**:

1. **Fix Binary Path**
   ```bash
   # Find correct binary path
   which airs-mcp-fs
   
   # Or use absolute path
   realpath target/release/airs-mcp-fs
   ```

2. **Fix JSON Configuration**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "command": "/correct/absolute/path/to/airs-mcp-fs",
         "env": {
           "AIRS_MCP_FS_ENV": "development"
         }
       }
     }
   }
   ```

3. **Check Binary Permissions**
   ```bash
   # Make binary executable
   chmod +x /path/to/airs-mcp-fs
   
   # Check architecture compatibility
   file /path/to/airs-mcp-fs
   ```

### Problem: "Invalid server response" Errors

**Symptoms**: Claude Desktop shows MCP communication errors

**Diagnostic Steps**:
```bash
# Check log files
tail -f ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log

# Test server communication manually
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | /path/to/airs-mcp-fs
```

**Solutions**:

1. **Check Log Directory Permissions**
   ```bash
   # Create log directory if it doesn't exist
   mkdir -p ~/.local/share/airs-mcp-fs/logs
   
   # Set proper permissions
   chmod 755 ~/.local/share/airs-mcp-fs/logs
   ```

2. **Set Explicit Log Directory**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "env": {
           "AIRS_MCP_FS_LOG_DIR": "/Users/username/.local/share/airs-mcp-fs/logs"
         }
       }
     }
   }
   ```

## Path and Glob Pattern Issues

### Problem: Glob Patterns Not Matching Expected Files

**Symptoms**: Files you expect to be accessible are denied

**Diagnostic Steps**:
```bash
# Test glob patterns manually
find ~/Documents -name "*.txt" | head -5

# Check pattern syntax
echo "Pattern: ~/Documents/**/*.txt"
echo "Test file: ~/Documents/notes/todo.txt"
```

**Pattern Testing Tool**:
```bash
# Create a simple pattern tester
cat > test_pattern.py << 'EOF'
import glob
import sys

pattern = sys.argv[1]
test_path = sys.argv[2]

matches = glob.glob(pattern, recursive=True)
print(f"Pattern: {pattern}")
print(f"Test path: {test_path}")
print(f"Matches: {test_path in matches}")
print(f"All matches: {matches[:10]}")  # Show first 10 matches
EOF

python3 test_pattern.py "~/Documents/**/*.txt" "~/Documents/notes/todo.txt"
```

**Solutions**:

1. **Fix Common Glob Pattern Issues**
   ```toml
   # ❌ Missing recursive wildcard
   allowed_paths = ["~/Documents/*.txt"]  # Only matches direct children
   
   # ✅ Use recursive wildcard
   allowed_paths = ["~/Documents/**/*.txt"]  # Matches all subdirectories
   
   # ❌ Forgetting directory access
   allowed_paths = ["~/Documents/**/*"]  # Matches contents but not directory
   
   # ✅ Include directory and contents
   allowed_paths = [
       "~/Documents",        # Directory itself
       "~/Documents/**/*"    # Directory contents
   ]
   ```

2. **Test Different Pattern Approaches**
   ```toml
   # Specific file types
   allowed_paths = ["~/Documents/**/*.{txt,md,doc,pdf}"]
   
   # All files in specific directories
   allowed_paths = ["~/Documents/projects/**/*"]
   
   # Multiple directory patterns
   allowed_paths = [
       "~/Documents/**/*",
       "~/projects/**/*",
       "~/Desktop/**/*"
   ]
   ```

## File Size and Binary Processing Issues

### Problem: "File too large" Errors

**Symptoms**:
```
❌ File too large: 150MB exceeds maximum size of 100MB
```

**Solutions**:

1. **Increase File Size Limit**
   ```toml
   [binary]
   max_file_size = 209715200  # 200MB
   ```

2. **Use Environment Variable Override**
   ```json
   {
     "mcpServers": {
       "airs-mcp-fs": {
         "env": {
           "AIRS_MCP_FS_BINARY_MAX_FILE_SIZE": "209715200"
         }
       }
     }
   }
   ```

### Problem: Binary Processing Failures

**Symptoms**: Images or PDFs not processing correctly

**Diagnostic Steps**:
```bash
# Check if binary processing is enabled
cat ~/.config/airs-mcp-fs/development.toml | grep -A 5 binary

# Test file format
file ~/Documents/image.jpg
```

**Solutions**:

1. **Enable Binary Processing**
   ```toml
   [binary]
   enable_image_processing = true
   enable_pdf_processing = true
   ```

2. **Check File Format Support**
   ```toml
   # Supported image formats: JPEG, PNG, GIF, WebP, TIFF, BMP
   # Supported document formats: PDF
   ```

## Advanced Debugging

### Enable Debug Logging

```json
{
  "mcpServers": {
    "airs-mcp-fs": {
      "env": {
        "RUST_LOG": "debug",
        "AIRS_MCP_FS_ENV": "development"
      }
    }
  }
}
```

This provides detailed logging of:
- Configuration loading process
- Environment detection logic
- Security validation decisions
- Path pattern matching
- Policy evaluation results

### Log Analysis

```bash
# Monitor logs in real-time
tail -f ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log

# Search for specific errors
grep -i "error\|denied\|failed" ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log

# Filter security-related logs
grep -i "security\|validation\|policy" ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log
```

### Configuration Validation Tool

```bash
# Create configuration validator script
cat > validate_config.sh << 'EOF'
#!/bin/bash

CONFIG_FILE="$1"
if [ -z "$CONFIG_FILE" ]; then
    echo "Usage: $0 <config_file>"
    exit 1
fi

echo "Validating configuration: $CONFIG_FILE"

# Check file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file not found"
    exit 1
fi

# Check TOML syntax
if python3 -c "import tomllib; tomllib.load(open('$CONFIG_FILE', 'rb'))" 2>/dev/null; then
    echo "✅ TOML syntax valid"
else
    echo "❌ TOML syntax error"
    python3 -c "import tomllib; tomllib.load(open('$CONFIG_FILE', 'rb'))"
    exit 1
fi

# Test with AIRS MCP-FS
TEMP_ENV=$(mktemp)
echo "AIRS_MCP_FS_CONFIG_DIR=$(dirname "$CONFIG_FILE")" > "$TEMP_ENV"
echo "AIRS_MCP_FS_ENV=$(basename "$CONFIG_FILE" .toml)" >> "$TEMP_ENV"

if env -i bash -c "source $TEMP_ENV && timeout 5s airs-mcp-fs" 2>/dev/null; then
    echo "✅ Configuration loads successfully"
else
    echo "❌ Configuration loading failed"
    echo "Check logs for details:"
    echo "  tail ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log"
fi

rm "$TEMP_ENV"
EOF

chmod +x validate_config.sh

# Use the validator
./validate_config.sh ~/.config/airs-mcp-fs/development.toml
```

## Getting Help

### Log Collection for Support

```bash
# Collect diagnostic information
cat > collect_diagnostics.sh << 'EOF'
#!/bin/bash

echo "AIRS MCP-FS Diagnostic Information"
echo "=================================="
echo "Date: $(date)"
echo "OS: $(uname -a)"
echo ""

echo "Environment Variables:"
echo "AIRS_MCP_FS_ENV: $AIRS_MCP_FS_ENV"
echo "AIRS_MCP_FS_CONFIG_DIR: $AIRS_MCP_FS_CONFIG_DIR"
echo "AIRS_MCP_FS_LOG_DIR: $AIRS_MCP_FS_LOG_DIR"
echo ""

echo "Binary Information:"
which airs-mcp-fs
airs-mcp-fs --version 2>/dev/null || echo "Binary not found or not executable"
echo ""

echo "Configuration Files:"
find ~/.config/airs-mcp-fs -name "*.toml" 2>/dev/null | head -5
echo ""

echo "Recent Log Entries:"
tail -20 ~/.local/share/airs-mcp-fs/logs/airs-mcp-fs.log 2>/dev/null || echo "No log file found"
EOF

chmod +x collect_diagnostics.sh
./collect_diagnostics.sh > diagnostics.txt
```

### Common Support Requests

1. **Configuration Help**: Share your configuration file (remove sensitive paths)
2. **Error Logs**: Include recent log entries with error messages
3. **Environment Info**: Share environment detection and variable settings
4. **Claude Desktop Config**: Share MCP server configuration (remove sensitive paths)

## Related Sections

- **[Configuration Overview](./overview.md)**: Understanding the configuration system
- **[Environment Setup](./environment.md)**: Environment-specific configuration
- **[Security Policies](./security.md)**: Security policy troubleshooting
- **[Claude Desktop Integration](./claude_desktop.md)**: MCP integration issues
