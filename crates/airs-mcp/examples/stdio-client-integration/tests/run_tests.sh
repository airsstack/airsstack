#!/bin/bash
# Test runner script for STDIO MCP Client Integration tests

echo "🧪 STDIO MCP Client Integration Test Suite"
echo "=========================================="

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "❌ Virtual environment not found. Please run setup first:"
    echo "   python3 -m venv venv"
    echo "   source venv/bin/activate"
    echo "   pip install -r requirements.txt"
    exit 1
fi

# Check if in virtual environment
if [ -z "$VIRTUAL_ENV" ]; then
    echo "🔧 Activating virtual environment..."
    source venv/bin/activate
fi

# Build Rust binaries first
echo "🔨 Building Rust binaries..."
cd ..
cargo build --quiet
if [ $? -ne 0 ]; then
    echo "❌ Failed to build Rust binaries"
    exit 1
fi
cd tests

echo "✅ Rust binaries built successfully"
echo ""

# Run tests based on argument
case "$1" in
    "integration"|"int")
        echo "🧪 Running Integration Tests..."
        python test_client_integration.py
        ;;
    "transport"|"trans")
        echo "🧪 Running Transport Tests..."
        python test_transport.py
        ;;
    "error"|"err")
        echo "🧪 Running Error Scenario Tests..."
        python test_error_scenarios.py
        ;;
    "pytest"|"py")
        echo "🧪 Running All Tests with pytest..."
        python -m pytest -v
        ;;
    "quick"|"q")
        echo "🧪 Running Quick Test Suite..."
        python test_client_integration.py
        ;;
    *)
        echo "🧪 Running All Tests..."
        echo ""
        echo "📋 1. Integration Tests"
        echo "----------------------"
        python test_client_integration.py
        echo ""
        echo "📋 2. Transport Tests"
        echo "--------------------"
        python test_transport.py
        echo ""
        echo "📋 3. Error Scenario Tests"
        echo "-------------------------"
        python test_error_scenarios.py
        echo ""
        echo "🎉 All test suites completed!"
        ;;
esac