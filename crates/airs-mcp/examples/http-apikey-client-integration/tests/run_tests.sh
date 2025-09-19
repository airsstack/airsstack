#!/bin/bash
# Test runner script for HTTP API Key client integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VENV_DIR="$SCRIPT_DIR/venv"

# Parse command line arguments
SUITE="all"
COMPREHENSIVE="false"

while [[ $# -gt 0 ]]; do
    case $1 in
        --suite)
            SUITE="$2"
            shift 2
            ;;
        --comprehensive)
            COMPREHENSIVE="true"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --suite SUITE        Test suite to run: mock, production, stress, all (default: all)"
            echo "  --comprehensive      Use comprehensive test runner with detailed reporting"
            echo "  --help              Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Run all tests with standard runner"
            echo "  $0 --comprehensive           # Run all tests with comprehensive reporting"
            echo "  $0 --suite mock              # Run only mock server tests"
            echo "  $0 --suite stress --comprehensive  # Run stress tests with reporting"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
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

# Build the Rust binaries
echo "🔨 Building HTTP client and mock server binaries..."
cd "$PROJECT_DIR"
cargo build --bin http-apikey-client --bin http-mock-server

# Set environment variables for testing
export PYTHONPATH="$PROJECT_DIR:$PYTHONPATH"
export RUST_LOG="info,airs_mcp=debug"

if [ "$COMPREHENSIVE" = "true" ]; then
    echo "🚀 Running comprehensive HTTP API Key client integration tests..."
    echo "Suite: $SUITE"
    echo ""
    
    cd "$SCRIPT_DIR"
    python run_comprehensive_tests.py --suite "$SUITE"
else
    echo "🚀 Running HTTP API Key client integration tests..."
    echo "Suite: $SUITE"
    echo ""
    
    cd "$SCRIPT_DIR"
    
    # Run tests based on suite selection
    case $SUITE in
        mock)
            echo "📋 Running mock server integration tests..."
            pytest test_http_client_mock_integration.py -v -s --tb=short
            ;;
        production)
            echo "📋 Running production server integration tests..."
            pytest test_http_client_production_integration.py -v -s --tb=short
            ;;
        stress)
            echo "🔥 Running stress tests and edge case validation..."
            pytest test_stress_validation.py -v -s --tb=short
            ;;
        all)
            echo "📋 Running mock server integration tests..."
            pytest test_http_client_mock_integration.py -v -s --tb=short
            
            echo ""
            echo "📋 Running production server integration tests..."
            pytest test_http_client_production_integration.py -v -s --tb=short
            
            echo ""
            echo "🔥 Running stress tests and edge case validation..."
            pytest test_stress_validation.py -v -s --tb=short
            ;;
        *)
            echo "❌ Unknown test suite: $SUITE"
            echo "Valid suites: mock, production, stress, all"
            exit 1
            ;;
    esac
fi

echo ""
echo "✅ All tests completed!"
echo ""