# Scripts Directory

This directory contains utility scripts for setting up and managing the HTTP OAuth2 Client Integration example.

## Setup Scripts

### `setup_keys.sh` (Shell Script)
Generates cryptographic keys needed for JWT signing and verification in the OAuth2 flow.

**Usage:**
```bash
./scripts/setup_keys.sh
```

**Requirements:**
- OpenSSL installed and accessible in PATH
- Write permissions in the example directory

### `setup_keys.py` (Python Script)
Python alternative to the shell script, provides the same functionality with better cross-platform compatibility.

**Usage:**
```bash
python3 scripts/setup_keys.py
# or if executable:
./scripts/setup_keys.py
```

**Requirements:**
- Python 3.6+
- OpenSSL installed and accessible in PATH
- Write permissions in the example directory

## Generated Files

Both scripts generate the following files in the `test_keys/` directory:

- `private_key.pem` - RSA private key (2048-bit) for JWT signing
- `public_key.pem` - RSA public key for JWT verification

## Security Notes

⚠️ **Important Security Considerations:**

1. **Development Only**: These keys are for testing and development purposes only
2. **Never Commit**: Private keys should never be committed to version control
3. **File Permissions**: Private key is set to read-only for owner (600 permissions)
4. **Production**: In production environments, use proper key management systems (AWS KMS, Azure Key Vault, HashiCorp Vault, etc.)

## Platform Compatibility

- **macOS/Linux**: Both shell and Python scripts work natively
- **Windows**: Use Git Bash, WSL, or Python script with OpenSSL installed
- **CI/CD**: Python script is recommended for automated environments

## Future Scripts

This directory can be extended with additional utility scripts such as:

- Test data generation scripts
- Environment setup scripts
- Development server startup scripts
- Integration test runners