#!/bin/bash
# Test runner script for HTTP API Key integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$SCRIPT_DIR/venv"

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

# Run the tests
echo "🚀 Running HTTP API Key integration tests..."
cd "$PROJECT_DIR"

# Set environment variables for testing
export PYTHONPATH="$PROJECT_DIR:$PYTHONPATH"
export RUST_LOG="info,airs_mcp=debug"

# Run main integration tests with verbose output
echo "📋 Running main integration tests..."
pytest tests/test_http_apikey_integration.py -v -s --tb=short

echo ""
echo "🔥 Running stress tests and edge case validation..."
pytest tests/test_stress_validation.py -v -s --tb=short

echo ""
echo "✅ All tests completed!"
echo ""
echo "To run tests manually:"
echo "  cd $PROJECT_DIR"
echo "  source tests/venv/bin/activate"
echo "  pytest tests/ -v  # Run all tests"
echo "  pytest tests/test_http_apikey_integration.py -v  # Main tests only"
echo "  pytest tests/test_stress_validation.py -v  # Stress tests only"