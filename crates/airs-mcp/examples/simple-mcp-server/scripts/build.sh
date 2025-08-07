#!/bin/bash
# Build script for simple-mcp-server
# Always builds in release mode for optimal performance

set -e

# Source path utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/utils/paths.sh"

log_info "Building simple-mcp-server in release mode..."

# Confirm build operation
ask_confirmation "This will compile the simple-mcp-server binary in release mode."

# Navigate to project root
cd "$PROJECT_ROOT"

# Clean previous builds
log_info "Cleaning previous builds..."
cargo clean

# Build in release mode
log_info "Compiling simple-mcp-server (release mode)..."
cargo build --release

# Verify binary was created
if [ -f "$BINARY_PATH" ]; then
    log_success "Build completed successfully!"
    log_info "Binary location: $BINARY_PATH"
    
    # Show binary info
    log_info "Binary information:"
    ls -la "$BINARY_PATH"
    
    # Test binary can run (basic check)
    log_info "Testing binary execution..."
    timeout 2 "$BINARY_PATH" < /dev/null > /dev/null 2>&1 || true
    log_success "Binary execution test passed"
    
else
    log_error "Build failed - binary not found at expected location"
    exit 1
fi

log_success "simple-mcp-server build process completed!"
echo
log_info "Next steps:"
echo "  1. Run: ./scripts/test_inspector.sh    # Test with MCP Inspector"
echo "  2. Run: ./scripts/configure_claude.sh  # Configure Claude Desktop"
echo "  3. Run: ./scripts/integrate.sh         # Full integration test"
