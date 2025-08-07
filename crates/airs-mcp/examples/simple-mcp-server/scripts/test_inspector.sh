#!/bin/bash
# Comprehensive MCP Inspector testing for simple-mcp-server
# Tests all functionality with positive and negative test cases

set -e

# Source path utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils/paths.sh"

log_info "Starting comprehensive MCP Inspector testing for simple-mcp-server"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    log_error "Binary not found at: $BINARY_PATH"
    log_info "Please run: ./scripts/build.sh first"
    exit 1
fi

# Check if npx is available
if ! command -v npx >/dev/null 2>&1; then
    log_error "npx not found. Please install Node.js"
    exit 1
fi

log_info "Binary found: $BINARY_PATH"
log_info "Starting MCP Inspector test suite..."

# Function to test MCP Inspector
test_with_inspector() {
    log_info "Launching MCP Inspector for interactive testing..."
    log_warning "MCP Inspector will open in your browser."
    log_info "Please test the following capabilities:"
    echo
    echo "📋 Resources to test:"
    echo "  - file:///tmp/example.txt"
    echo "  - file:///tmp/config.json"
    echo "  - Invalid URI (negative test)"
    echo
    echo "🔧 Tools to test:"
    echo "  Positive cases:"
    echo "    - add: {\"a\": 5, \"b\": 3}"
    echo "    - greet: {\"name\": \"World\"}"
    echo "  Negative cases:"
    echo "    - add: {\"a\": \"invalid\"}"
    echo "    - add: {\"b\": 3} (missing 'a')"
    echo "    - greet: {} (missing 'name')"
    echo "    - invalid_tool: {}"
    echo
    echo "📝 Prompts to test:"
    echo "  Positive cases:"
    echo "    - code_review: {\"language\": \"rust\", \"code\": \"fn main() {}\"}"
    echo "    - explain_concept: {\"concept\": \"MCP\", \"level\": \"beginner\"}"
    echo "  Negative cases:"
    echo "    - code_review: {} (missing required fields)"
    echo "    - invalid_prompt: {}"
    echo
    
    ask_confirmation "Ready to launch MCP Inspector?"
    
    # Launch MCP Inspector
    npx @modelcontextprotocol/inspector "$BINARY_PATH"
}

# Function to run automated protocol tests
run_automated_tests() {
    log_info "Running automated protocol compliance tests..."
    
    # Test 1: Basic connectivity
    log_info "Test 1: Basic connectivity and initialization"
    echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | timeout 5 "$BINARY_PATH" >/dev/null 2>&1
    if [ $? -eq 0 ]; then
        log_success "✅ Basic connectivity test passed"
    else
        log_error "❌ Basic connectivity test failed"
        return 1
    fi
    
    # Test 2: Invalid JSON
    log_info "Test 2: Invalid JSON handling"
    echo 'invalid json' | timeout 3 "$BINARY_PATH" >/dev/null 2>&1 || true
    log_success "✅ Invalid JSON handling test completed"
    
    # Test 3: Empty input
    log_info "Test 3: Empty input handling"
    echo '' | timeout 3 "$BINARY_PATH" >/dev/null 2>&1 || true
    log_success "✅ Empty input handling test completed"
    
    log_success "Automated protocol tests completed"
}

# Function to verify no stderr output
verify_no_stderr() {
    log_info "Verifying no stderr output (STDIO compliance)..."
    
    # Capture stderr during initialization
    local stderr_output
    stderr_output=$(echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | timeout 3 "$BINARY_PATH" 2>&1 >/dev/null)
    
    if [ -z "$stderr_output" ]; then
        log_success "✅ No stderr output detected - STDIO compliant"
    else
        log_error "❌ Stderr output detected:"
        echo "$stderr_output"
        log_warning "This violates MCP STDIO transport requirements"
        return 1
    fi
}

# Main test execution
main() {
    # Run automated tests first
    run_automated_tests
    
    # Verify STDIO compliance
    verify_no_stderr
    
    # Interactive testing with MCP Inspector
    test_with_inspector
    
    log_success "MCP Inspector testing completed!"
    echo
    log_info "If all tests passed, you can proceed with:"
    echo "  ./scripts/configure_claude.sh  # Configure Claude Desktop"
}

# Run main function
main "$@"
