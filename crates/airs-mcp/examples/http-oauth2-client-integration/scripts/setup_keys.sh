#!/bin/bash

# Setup script for HTTP OAuth2 Client Integration Example
# Generates cryptographic keys needed for JWT signing and verification

set -e

echo "🔑 Setting up cryptographic keys for OAuth2 JWT tokens..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Go up one level to the example root directory
EXAMPLE_ROOT="$(dirname "$SCRIPT_DIR")"

# Change to example root directory
cd "$EXAMPLE_ROOT"

# Create test_keys directory if it doesn't exist
mkdir -p test_keys

# Check if OpenSSL is available
if ! command -v openssl &> /dev/null; then
    echo "❌ Error: OpenSSL is required but not installed."
    echo "   Please install OpenSSL and try again."
    echo "   - macOS: brew install openssl"
    echo "   - Ubuntu/Debian: sudo apt-get install openssl"
    echo "   - Windows: Use Git Bash or WSL with OpenSSL installed"
    exit 1
fi

# Generate RSA private key (2048-bit)
echo "📝 Generating RSA private key (2048-bit)..."
openssl genrsa -out test_keys/private_key.pem 2048

# Extract public key from private key
echo "🔓 Extracting public key..."
openssl rsa -in test_keys/private_key.pem -pubout -out test_keys/public_key.pem

# Set appropriate permissions (private key should be readable only by owner)
chmod 600 test_keys/private_key.pem
chmod 644 test_keys/public_key.pem

echo "✅ Cryptographic keys generated successfully!"
echo ""
echo "📁 Generated files:"
echo "   - test_keys/private_key.pem (RSA private key for JWT signing)"
echo "   - test_keys/public_key.pem (RSA public key for JWT verification)"
echo ""
echo "⚠️  Security Note:"
echo "   - Private key is set to read-only for owner (600)"
echo "   - These keys are for TESTING/DEVELOPMENT only"
echo "   - Never commit private keys to version control"
echo "   - In production, use proper key management systems"
echo ""
echo "🚀 You can now run the OAuth2 integration example!"
echo "   cargo run --bin http-oauth2-mock-server"
echo "   cargo run --bin http-mcp-mock-server"
echo "   cargo run --bin http-oauth2-client"