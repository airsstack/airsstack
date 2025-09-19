#!/bin/bash
# Test runner script for HTTP OAuth2 client integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$SCRIPT_DIR/venv"

# Parse command line arguments
SUITE="all"
VERBOSE=""
PARALLEL=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --suite)
            SUITE="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE="-v -s"
            shift
            ;;
        --parallel|-j)
            PARALLEL="-n auto"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "OPTIONS:"
            echo "  --suite SUITE     Run specific test suite (basic|integration|comprehensive|all)"
            echo "  --verbose, -v     Run with verbose output"
            echo "  --parallel, -j    Run tests in parallel"
            echo "  --help, -h        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "🔧 Setting up Python test environment..."

# Create virtual environment if it doesn't exist
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv "$VENV_DIR"
fi

# Activate virtual environment
echo "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

# Install dependencies
echo "Installing Python dependencies..."
pip install --upgrade pip
pip install -r "$PROJECT_DIR/tests/requirements.txt"

echo "📋 Environment setup complete!"
echo "Python version: $(python --version)"
echo "Pytest version: $(pytest --version)"
echo ""

echo "🏗️  Building binaries..."

# Build all required binaries
echo "Building OAuth2 client..."
cargo build --bin http-oauth2-client
if [ $? -ne 0 ]; then
    echo "❌ Failed to build OAuth2 client"
    exit 1
fi

echo "Building OAuth2 mock server..."
cargo build --bin http-oauth2-mock-server
if [ $? -ne 0 ]; then
    echo "❌ Failed to build OAuth2 mock server"
    exit 1
fi

echo "Building MCP mock server..."
cargo build --bin http-mcp-mock-server
if [ $? -ne 0 ]; then
    echo "❌ Failed to build MCP mock server"
    exit 1
fi

echo "✅ All binaries built successfully!"
echo ""

# Set environment variables
export RUST_LOG="info,airs_mcp=debug"
export PROJECT_DIR="$PROJECT_DIR"

echo "🧪 Running tests..."

# Determine which tests to run
case $SUITE in
    basic)
        echo "Running basic OAuth2 tests..."
        pytest "$PROJECT_DIR/tests/test_oauth2_basic.py" $VERBOSE $PARALLEL
        ;;
    integration)
        echo "Running OAuth2 integration tests..."
        pytest "$PROJECT_DIR/tests/test_oauth2_integration.py" $VERBOSE $PARALLEL
        ;;
    comprehensive)
        echo "Running comprehensive OAuth2 tests..."
        pytest "$PROJECT_DIR/tests/test_oauth2_comprehensive.py" $VERBOSE $PARALLEL
        ;;
    all)
        echo "Running all OAuth2 client integration tests..."
        pytest "$PROJECT_DIR/tests/" $VERBOSE $PARALLEL
        ;;
    *)
        echo "❌ Unknown test suite: $SUITE"
        echo "Valid suites: basic, integration, comprehensive, all"
        exit 1
        ;;
esac

echo ""
echo "🎉 Test execution completed!"