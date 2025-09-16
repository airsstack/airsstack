#!/bin/bash

# OAuth2 MCP Integration Test Setup Script
# This script sets up the Python environment for running integration tests

set -e  # Exit on any error

echo "🔧 OAuth2 MCP Integration Test Setup"
echo "====================================="

# Check if we're in the tests directory
if [[ ! -f "run_tests.py" ]]; then
    echo "❌ Error: This script must be run from the tests/ directory"
    echo "   Run: cd tests && ./setup.sh"
    exit 1
fi

# Check Python version
echo "🐍 Checking Python installation..."
python3 --version || {
    echo "❌ Error: Python 3 is required but not found"
    echo "   Please install Python 3.8 or higher"
    exit 1
}

# Create virtual environment if it doesn't exist
if [[ ! -d "venv" ]]; then
    echo "📦 Creating Python virtual environment..."
    python3 -m venv venv
    echo "✅ Virtual environment created in venv/"
else
    echo "✅ Virtual environment already exists"
fi

# Activate virtual environment and install dependencies
echo "📚 Installing dependencies..."
source venv/bin/activate

pip install --upgrade pip
pip install -r requirements.txt

echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "To use the test environment:"
echo "  1. Activate: source venv/bin/activate"
echo "  2. Run tests: python run_tests.py basic"
echo "  3. Deactivate: deactivate"
echo ""
echo "Quick test command:"
echo "  source venv/bin/activate && python run_tests.py basic"