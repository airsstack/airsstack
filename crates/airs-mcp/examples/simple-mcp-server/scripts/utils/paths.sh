#!/bin/bash
# Centralized path definitions for simple-mcp-server

# Project paths
PROJECT_ROOT="/Users/hiraq/Projects/rstlix0x0/airs/crates/airs-mcp/examples/simple-mcp-server"
BINARY_PATH="${PROJECT_ROOT}/target/release/simple-mcp-server"
SCRIPTS_DIR="${PROJECT_ROOT}/scripts"

# System paths
CLAUDE_CONFIG_DIR="$HOME/Library/Application Support/Claude"
CLAUDE_CONFIG_FILE="${CLAUDE_CONFIG_DIR}/claude_desktop_config.json"
LOG_DIR="/tmp/simple-mcp-server"

# Server identification
SERVER_NAME="simple-mcp-server"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Confirmation function
ask_confirmation() {
    local message="$1"
    log_warning "$message"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Operation cancelled."
        exit 1
    fi
}
