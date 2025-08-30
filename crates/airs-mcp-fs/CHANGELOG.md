# Changelog

All notable changes to AIRS MCP-FS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Planning for future releases

## [0.1.0] - 2025-08-30

### Added
- Initial release of AIRS MCP-FS with security-first architecture
- Complete filesystem operations through MCP protocol (text files only)
- **Binary file processing completely disabled for enhanced security**
- Enhanced security framework with binary file restriction as first layer of validation
- Comprehensive binary file detection (extension-based and content-based)
- Multi-environment configuration system with security-first defaults
- Security policies and path validation
- Human-in-the-loop approval workflows
- Comprehensive audit logging with binary file rejection tracking
- Text-only file operations for maximum security
- Configuration simplified with `binary_processing_disabled = true` by default

### Security
- **Attack surface reduction**: Eliminated entire classes of binary-based security vulnerabilities
- **Memory safety**: Prevented buffer overflows and memory corruption from binary parsing
- **Malware prevention**: Blocked execution of potentially malicious binary content
- **Resource protection**: Eliminated resource exhaustion from complex binary processing
- **Enterprise-grade security**: Production-ready security framework from initial release

### Technical Implementation
- 5-layer security validation system with binary restriction as first layer
- Text-only processing focused on development workflows
- Support for source code, configuration, documentation, and data files
- Comprehensive error handling and security validation
- 191 test suite including 3 dedicated binary rejection tests
