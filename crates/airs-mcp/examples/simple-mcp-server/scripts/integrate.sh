#!/bin/bash
# Master integration script for simple-mcp-server
# Orchestrates the complete integration process

set -e

# Source path utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils/paths.sh"

log_info "simple-mcp-server Master Integration Script"
echo "============================================="

# Function to check prerequisites
check_prerequisites() {
    log_info "Phase 1: Checking prerequisites..."
    
    local all_good=true
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_error "❌ Not in simple-mcp-server project directory"
        all_good=false
    else
        log_success "✅ Project directory verified"
    fi
    
    # Check for required tools
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "❌ Cargo not found - Rust installation required"
        all_good=false
    else
        log_success "✅ Cargo found"
    fi
    
    if ! command -v npx >/dev/null 2>&1; then
        log_error "❌ npx not found - Node.js installation required"
        all_good=false
    else
        log_success "✅ npx found"
    fi
    
    # Check Claude Desktop
    if [ -d "/Applications/Claude.app" ] || [ -d "$HOME/Applications/Claude.app" ]; then
        log_success "✅ Claude Desktop found"
    else
        log_warning "⚠️  Claude Desktop not found"
        log_info "Please install from: https://claude.ai/download"
        all_good=false
    fi
    
    if [ "$all_good" = false ]; then
        log_error "Prerequisites not met - please install missing components"
        exit 1
    fi
    
    log_success "Prerequisites check completed!"
}

# Function to build server
build_server() {
    log_info "Phase 2: Building simple-mcp-server..."
    
    if [ -f "$BINARY_PATH" ]; then
        log_info "Binary already exists. Rebuild?"
        read -p "Rebuild binary? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping build - using existing binary"
            return 0
        fi
    fi
    
    # Run build script
    "$SCRIPTS_DIR/build.sh"
}

# Function to test with MCP Inspector
test_inspector() {
    log_info "Phase 3: Testing with MCP Inspector..."
    
    ask_confirmation "Test the server with MCP Inspector? (Recommended)"
    
    # Run inspector test
    "$SCRIPTS_DIR/test_inspector.sh"
    
    echo
    log_info "Did the MCP Inspector tests pass?"
    read -p "Continue with Claude Desktop integration? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Integration cancelled - please fix MCP Inspector issues first"
        exit 1
    fi
}

# Function to configure Claude Desktop
configure_claude() {
    log_info "Phase 4: Configuring Claude Desktop..."
    
    # Run configuration script
    "$SCRIPTS_DIR/configure_claude.sh"
}

# Function to restart Claude Desktop
restart_claude() {
    log_info "Phase 5: Restarting Claude Desktop..."
    
    ask_confirmation "Restart Claude Desktop to load new configuration?"
    
    # Kill Claude Desktop if running
    if pgrep -f "Claude" >/dev/null 2>&1; then
        log_info "Stopping Claude Desktop..."
        pkill -f "Claude" || true
        sleep 2
    fi
    
    # Start Claude Desktop
    log_info "Starting Claude Desktop..."
    if [ -d "/Applications/Claude.app" ]; then
        open "/Applications/Claude.app"
    elif [ -d "$HOME/Applications/Claude.app" ]; then
        open "$HOME/Applications/Claude.app"
    else
        log_error "Claude Desktop application not found"
        return 1
    fi
    
    log_success "Claude Desktop restarted"
    log_info "Please wait a few seconds for Claude Desktop to fully initialize..."
    sleep 5
}

# Function to verify integration
verify_integration() {
    log_info "Phase 6: Verifying integration..."
    
    log_info "Integration verification checklist:"
    echo
    echo "In Claude Desktop, please check:"
    echo "  1. 🔌 MCP plug icon appears in the chat input area (bottom-right)"
    echo "  2. 🛠️  Click the plug icon - you should see 'simple-mcp-server' tools:"
    echo "       - Add Numbers"
    echo "       - Greet User"
    echo "  3. 📝 Test a tool by asking: 'Can you add 5 and 3 for me?'"
    echo "  4. 📋 Test a resource by asking: 'What's in the example file?'"
    echo
    
    read -p "Did you see the MCP plug icon and tools? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_success "🎉 Integration successful!"
        echo
        log_info "You can now use simple-mcp-server tools in Claude Desktop:"
        echo "  - Ask Claude to add numbers"
        echo "  - Ask Claude to greet someone"
        echo "  - Ask Claude to read the example files"
        echo "  - Ask Claude to review code or explain concepts"
        echo
        log_info "For debugging, run: ./scripts/debug_integration.sh"
        return 0
    else
        log_error "Integration verification failed"
        return 1
    fi
}

# Function to handle integration failure
handle_failure() {
    log_error "Integration failed during verification"
    echo
    log_info "Troubleshooting options:"
    echo "  1. Run debug dashboard: ./scripts/debug_integration.sh"
    echo "  2. Check server logs: tail -f /tmp/simple-mcp-server/simple-mcp-server.log"
    echo "  3. Test server manually: ./scripts/test_inspector.sh"
    echo "  4. Verify configuration: cat '$CLAUDE_CONFIG_FILE'"
    echo
    
    read -p "Run debug dashboard now? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        "$SCRIPTS_DIR/debug_integration.sh"
    fi
}

# Function to show completion summary
show_summary() {
    echo
    echo "=========================================="
    log_success "simple-mcp-server Integration Complete!"
    echo "=========================================="
    echo
    log_info "What was accomplished:"
    echo "  ✅ Built optimized release binary"
    echo "  ✅ Tested with MCP Inspector"
    echo "  ✅ Configured Claude Desktop"
    echo "  ✅ Verified integration"
    echo
    log_info "Available tools in Claude Desktop:"
    echo "  🧮 Add Numbers - Mathematical calculations"
    echo "  👋 Greet User - Personalized greetings"
    echo "  📄 File Resources - Read example files"
    echo "  📝 Code Review - Generate code review prompts"
    echo "  🎓 Explain Concept - Technical explanations"
    echo
    log_info "Useful commands:"
    echo "  ./scripts/debug_integration.sh  # Debug any issues"
    echo "  ./scripts/test_inspector.sh     # Re-test server"
    echo "  tail -f /tmp/simple-mcp-server/simple-mcp-server.log  # Monitor logs"
    echo
    log_success "Happy prompting! 🚀"
}

# Main integration process
main() {
    echo "🚀 Starting complete simple-mcp-server integration process"
    echo
    log_warning "This script will:"
    echo "  1. Check prerequisites"
    echo "  2. Build the server binary"
    echo "  3. Test with MCP Inspector"
    echo "  4. Configure Claude Desktop"
    echo "  5. Restart Claude Desktop"
    echo "  6. Verify integration"
    echo
    
    ask_confirmation "Begin complete integration process?"
    
    # Execute phases
    check_prerequisites
    echo
    
    build_server
    echo
    
    test_inspector
    echo
    
    configure_claude
    echo
    
    restart_claude
    echo
    
    if verify_integration; then
        show_summary
    else
        handle_failure
    fi
}

# Run main function
main "$@"
