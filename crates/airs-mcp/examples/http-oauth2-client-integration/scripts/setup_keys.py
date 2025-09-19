#!/usr/bin/env python3
"""
Setup script for HTTP OAuth2 Client Integration Example
Generates cryptographic keys needed for JWT signing and verification
"""

import os
import subprocess
import sys
from pathlib import Path


def check_openssl():
    """Check if OpenSSL is available"""
    try:
        subprocess.run(["openssl", "version"], capture_output=True, check=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def generate_keys():
    """Generate RSA private and public keys"""
    # Get script directory and go up one level to example root
    script_dir = Path(__file__).parent
    example_root = script_dir.parent
    
    # Change to example root directory
    os.chdir(example_root)
    
    # Create test_keys directory
    keys_dir = Path("test_keys")
    keys_dir.mkdir(exist_ok=True)
    
    private_key_path = keys_dir / "private_key.pem"
    public_key_path = keys_dir / "public_key.pem"
    
    print("🔑 Setting up cryptographic keys for OAuth2 JWT tokens...")
    
    # Check OpenSSL availability
    if not check_openssl():
        print("❌ Error: OpenSSL is required but not installed.")
        print("   Please install OpenSSL and try again.")
        print("   - macOS: brew install openssl")
        print("   - Ubuntu/Debian: sudo apt-get install openssl")
        print("   - Windows: Download from https://slproweb.com/products/Win32OpenSSL.html")
        print("   - Or use Git Bash/WSL with OpenSSL installed")
        sys.exit(1)
    
    try:
        # Generate RSA private key (2048-bit)
        print("📝 Generating RSA private key (2048-bit)...")
        subprocess.run([
            "openssl", "genrsa", 
            "-out", str(private_key_path), 
            "2048"
        ], check=True, capture_output=True)
        
        # Extract public key from private key
        print("🔓 Extracting public key...")
        subprocess.run([
            "openssl", "rsa",
            "-in", str(private_key_path),
            "-pubout", 
            "-out", str(public_key_path)
        ], check=True, capture_output=True)
        
        # Set appropriate permissions
        os.chmod(private_key_path, 0o600)  # Read-write for owner only
        os.chmod(public_key_path, 0o644)   # Read for everyone, write for owner
        
        print("✅ Cryptographic keys generated successfully!")
        print("")
        print("📁 Generated files:")
        print(f"   - {private_key_path} (RSA private key for JWT signing)")
        print(f"   - {public_key_path} (RSA public key for JWT verification)")
        print("")
        print("⚠️  Security Note:")
        print("   - Private key is set to read-only for owner (600)")
        print("   - These keys are for TESTING/DEVELOPMENT only")
        print("   - Never commit private keys to version control")
        print("   - In production, use proper key management systems")
        print("")
        print("🚀 You can now run the OAuth2 integration example!")
        print("   cargo run --bin http-oauth2-mock-server")
        print("   cargo run --bin http-mcp-mock-server")
        print("   cargo run --bin http-oauth2-client")
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Error generating keys: {e}")
        print("   Please check that OpenSSL is properly installed and accessible.")
        sys.exit(1)


def main():
    """Main function"""
    print("🔐 OAuth2 Integration Cryptographic Key Setup")
    print("=" * 50)
    
    generate_keys()


if __name__ == "__main__":
    main()