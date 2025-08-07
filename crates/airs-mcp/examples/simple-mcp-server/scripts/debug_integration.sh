#!/bin/bash
# Debug integration script for simple-mcp-server
# Monitors Claude Desktop integration and provides debugging information

set -e

# Source path utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils/paths.sh"

log_info "Claude Desktop Integration Debug Dashboard"

# Function to check Claude Desktop installation
check_claude_installation() {
    log_info "Checking Claude Desktop installation..."
    
    if [ -d "/Applications/Claude.app" ] || [ -d "$HOME/Applications/Claude.app" ]; then
        log_success "✅ Claude Desktop found"
        return 0
    else
        log_error "❌ Claude Desktop not found"
        log_info "Please install Claude Desktop from: https://claude.ai/download"
        return 1
    fi
}

# Function to check server binary
check_server_binary() {
    log_info "Checking simple-mcp-server binary..."
    
    if [ -f "$BINARY_PATH" ]; then
        log_success "✅ Binary found: $BINARY_PATH"
        
        # Check if binary is executable
        if [ -x "$BINARY_PATH" ]; then
            log_success "✅ Binary is executable"
        else
            log_error "❌ Binary is not executable"
            return 1
        fi
        
        # Show binary info
        log_info "Binary information:"
        ls -la "$BINARY_PATH"
        
    else
        log_error "❌ Binary not found: $BINARY_PATH"
        log_info "Please run: ./scripts/build.sh"
        return 1
    fi
}

# Function to check configuration
check_configuration() {
    log_info "Checking Claude Desktop configuration..."
    
    if [ -f "$CLAUDE_CONFIG_FILE" ]; then
        log_success "✅ Configuration file found"
        
        # Validate JSON
        if command -v python3 >/dev/null 2>&1; then
            python3 -c "
import json
try:
    with open('$CLAUDE_CONFIG_FILE', 'r') as f:
        config = json.load(f)
    print('✅ Configuration file is valid JSON')
    
    # Check for our server
    if 'mcpServers' in config and '$SERVER_NAME' in config['mcpServers']:
        server_config = config['mcpServers']['$SERVER_NAME']
        print(f'✅ $SERVER_NAME configuration found')
        print(f'   Command: {server_config.get(\"command\", \"Not specified\")}')
        
        # Verify command path
        import os
        cmd_path = server_config.get('command', '')
        if os.path.exists(cmd_path):
            print(f'✅ Command path exists: {cmd_path}')
        else:
            print(f'❌ Command path not found: {cmd_path}')
    else:
        print('❌ $SERVER_NAME not found in configuration')
except Exception as e:
    print(f'❌ Configuration error: {e}')
"
        else
            log_warning "Python3 not available - limited configuration validation"
            if grep -q "$SERVER_NAME" "$CLAUDE_CONFIG_FILE"; then
                log_success "✅ simple-mcp-server found in configuration"
            else
                log_error "❌ simple-mcp-server not found in configuration"
            fi
        fi
        
        # Show configuration
        log_info "Configuration contents:"
        echo "────────────────────────────────────────"
        cat "$CLAUDE_CONFIG_FILE"
        echo "────────────────────────────────────────"
        
    else
        log_error "❌ Configuration file not found: $CLAUDE_CONFIG_FILE"
        log_info "Please run: ./scripts/configure_claude.sh"
        return 1
    fi
}

# Function to test server manually
test_server_manually() {
    log_info "Testing server manually..."
    
    # Test basic initialization
    log_info "Sending initialization request..."
    local init_response
    init_response=$(echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | timeout 5 "$BINARY_PATH" 2>/dev/null | head -1)
    
    if echo "$init_response" | grep -q '"result"'; then
        log_success "✅ Server responds to initialization"
        
        # Check capabilities
        if echo "$init_response" | grep -q '"capabilities"'; then
            log_success "✅ Server reports capabilities"
        else
            log_warning "⚠️  No capabilities reported"
        fi
    else
        log_error "❌ Server initialization failed"
        log_info "Response: $init_response"
        return 1
    fi
}

# Function to monitor Claude Desktop logs
monitor_claude_logs() {
    local log_pattern="$HOME/Library/Logs/Claude/mcp*.log"
    
    log_info "Monitoring Claude Desktop logs..."
    log_info "Looking for log files matching: $log_pattern"
    
    # Check if log files exist
    if ls $log_pattern >/dev/null 2>&1; then
        log_success "✅ Claude Desktop MCP logs found"
        
        ask_confirmation "Monitor Claude Desktop logs in real-time? (Press Ctrl+C to stop)"
        
        log_info "Starting log monitoring..."
        echo "────────────────────────────────────────"
        tail -f $log_pattern
        
    else
        log_warning "⚠️  No Claude Desktop MCP logs found"
        log_info "This might indicate:"
        echo "  - Claude Desktop hasn't started yet"
        echo "  - No MCP servers configured"
        echo "  - Different log location"
        echo
        log_info "Try starting Claude Desktop and look for logs in:"
        echo "  $HOME/Library/Logs/Claude/"
    fi
}

# Function to check Claude Desktop process
check_claude_process() {
    log_info "Checking Claude Desktop process..."
    
    if pgrep -f "Claude" >/dev/null 2>&1; then
        log_success "✅ Claude Desktop is running"
        
        # Show process info
        log_info "Claude Desktop processes:"
        ps aux | grep -i claude | grep -v grep || true
        
    else
        log_warning "⚠️  Claude Desktop not running"
        log_info "Please start Claude Desktop to test integration"
    fi
}

# Function to show troubleshooting help
show_troubleshooting() {
    log_info "Troubleshooting Guide"
    echo
    echo "🔧 Common Issues:"
    echo
    echo "1. MCP icon not visible in Claude Desktop:"
    echo "   - Restart Claude Desktop completely"
    echo "   - Check configuration file syntax"
    echo "   - Verify binary path is correct"
    echo
    echo "2. Tools not showing up:"
    echo "   - Check server logs: tail -f /tmp/simple-mcp-server/simple-mcp-server.log"
    echo "   - Test server manually: ./scripts/test_inspector.sh"
    echo "   - Verify no stderr output from server"
    echo
    echo "3. Server connection failures:"
    echo "   - Check binary permissions: ls -la $BINARY_PATH"
    echo "   - Test server startup: timeout 3 $BINARY_PATH < /dev/null"
    echo "   - Monitor Claude Desktop logs"
    echo
    echo "4. Configuration issues:"
    echo "   - Validate JSON syntax: python3 -m json.tool $CLAUDE_CONFIG_FILE"
    echo "   - Check absolute paths are used"
    echo "   - Verify Claude Desktop config directory exists"
    echo
    log_info "For more help, run individual diagnostic commands above"
}

# Main debug function
main() {
    echo "🔍 simple-mcp-server Integration Debug Dashboard"
    echo "==============================================="
    echo
    
    # Run all checks
    check_claude_installation
    echo
    check_server_binary
    echo
    check_configuration
    echo
    test_server_manually
    echo
    check_claude_process
    echo
    
    # Offer monitoring options
    log_info "Debug Options:"
    echo "  1. Monitor Claude Desktop logs (real-time)"
    echo "  2. Show troubleshooting guide"
    echo "  3. Exit"
    echo
    
    read -p "Choose option (1-3): " -n 1 -r
    echo
    
    case $REPLY in
        1)
            monitor_claude_logs
            ;;
        2)
            show_troubleshooting
            ;;
        3)
            log_info "Debug session ended"
            ;;
        *)
            log_warning "Invalid option"
            ;;
    esac
}

# Run main function
main "$@"
