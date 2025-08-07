#!/bin/bash
# Claude Desktop configuration script for simple-mcp-server
# Safely manages Claude Desktop configuration with backup

set -e

# Source path utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils/paths.sh"

log_info "Configuring Claude Desktop for simple-mcp-server"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    log_error "Binary not found at: $BINARY_PATH"
    log_info "Please run: ./scripts/build.sh first"
    exit 1
fi

# Function to backup existing config
backup_config() {
    if [ -f "$CLAUDE_CONFIG_FILE" ]; then
        local backup_file="${CLAUDE_CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$CLAUDE_CONFIG_FILE" "$backup_file"
        log_success "Backed up existing config to: $backup_file"
    else
        log_info "No existing Claude Desktop config found - will create new one"
    fi
}

# Function to create Claude Desktop directory
ensure_claude_dir() {
    if [ ! -d "$CLAUDE_CONFIG_DIR" ]; then
        log_info "Creating Claude Desktop config directory..."
        mkdir -p "$CLAUDE_CONFIG_DIR"
    fi
}

# Function to generate server configuration
generate_config() {
    local config_json
    
    # Check if existing config exists
    if [ -f "$CLAUDE_CONFIG_FILE" ]; then
        log_info "Merging with existing Claude Desktop configuration"
        
        # Use Python to merge JSON if available
        if command -v python3 >/dev/null 2>&1; then
            python3 -c "
import json
import sys

# Read existing config
try:
    with open('$CLAUDE_CONFIG_FILE', 'r') as f:
        existing = json.load(f)
except:
    existing = {}

# Ensure mcpServers exists
if 'mcpServers' not in existing:
    existing['mcpServers'] = {}

# Add our server
existing['mcpServers']['$SERVER_NAME'] = {
    'command': '$BINARY_PATH'
}

# Write back
with open('$CLAUDE_CONFIG_FILE', 'w') as f:
    json.dump(existing, f, indent=2)

print('Configuration merged successfully')
"
        else
            log_warning "Python3 not available - using basic JSON merge"
            # Simple append if mcpServers exists, otherwise create new
            if grep -q "mcpServers" "$CLAUDE_CONFIG_FILE" 2>/dev/null; then
                log_error "Manual merge required - Python3 not available for JSON manipulation"
                log_info "Please manually add this to your Claude Desktop config:"
                echo "\"$SERVER_NAME\": {"
                echo "  \"command\": \"$BINARY_PATH\""
                echo "}"
                exit 1
            fi
        fi
    else
        log_info "Creating new Claude Desktop configuration"
        config_json="{
  \"mcpServers\": {
    \"$SERVER_NAME\": {
      \"command\": \"$BINARY_PATH\"
    }
  }
}"
        echo "$config_json" > "$CLAUDE_CONFIG_FILE"
    fi
}

# Function to validate configuration
validate_config() {
    log_info "Validating configuration file..."
    
    if command -v python3 >/dev/null 2>&1; then
        python3 -c "
import json
try:
    with open('$CLAUDE_CONFIG_FILE', 'r') as f:
        config = json.load(f)
    print('✅ Configuration file is valid JSON')
    if 'mcpServers' in config and '$SERVER_NAME' in config['mcpServers']:
        print('✅ simple-mcp-server configuration found')
    else:
        print('❌ simple-mcp-server configuration not found')
        exit(1)
except Exception as e:
    print(f'❌ Configuration validation failed: {e}')
    exit(1)
"
    else
        # Basic validation without Python
        if [ -f "$CLAUDE_CONFIG_FILE" ]; then
            log_success "Configuration file created"
        else
            log_error "Configuration file creation failed"
            exit 1
        fi
    fi
}

# Function to show configuration
show_config() {
    log_info "Current Claude Desktop configuration:"
    echo "────────────────────────────────────────"
    if command -v python3 >/dev/null 2>&1; then
        python3 -c "
import json
with open('$CLAUDE_CONFIG_FILE', 'r') as f:
    config = json.load(f)
print(json.dumps(config, indent=2))
"
    else
        cat "$CLAUDE_CONFIG_FILE"
    fi
    echo "────────────────────────────────────────"
}

# Main configuration process
main() {
    log_info "Claude Desktop Configuration Process"
    echo
    log_info "This will modify your Claude Desktop configuration to include simple-mcp-server"
    log_info "Config file: $CLAUDE_CONFIG_FILE"
    log_info "Server binary: $BINARY_PATH"
    echo
    
    ask_confirmation "Proceed with Claude Desktop configuration?"
    
    # Ensure directory exists
    ensure_claude_dir
    
    # Backup existing config
    backup_config
    
    # Generate new configuration
    generate_config
    
    # Validate configuration
    validate_config
    
    # Show final configuration
    show_config
    
    log_success "Claude Desktop configuration completed!"
    echo
    log_warning "Important next steps:"
    echo "  1. Restart Claude Desktop completely"
    echo "  2. Look for the MCP plug icon in the chat input area"
    echo "  3. Click the icon to see available tools"
    echo
    log_info "To test the integration:"
    echo "  ./scripts/debug_integration.sh  # Monitor integration status"
    echo "  ./scripts/integrate.sh          # Full integration test"
}

# Run main function
main "$@"
