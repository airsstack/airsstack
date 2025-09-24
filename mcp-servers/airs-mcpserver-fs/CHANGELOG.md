# Changelog

All notable changes to airs-mcpserver-fs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Performance optimizations and additional security features

## [0.1.2] - 2025-09-24

### Changed
- **Dependency update**: Upgraded airs-mcp from 0.2.2 to 0.2.3
- **Enhanced security management**: Now uses airs-mcp v0.2.3 with comprehensive security audit configuration
- **Latest RSA version**: Updated to RSA 0.10.0-rc.8 (latest available) in development dependencies

### Security
- **Comprehensive security audit**: All known vulnerabilities properly documented and isolated to non-production code
- **Zero production vulnerabilities**: Clean security posture for all production code paths
- **Risk management**: Structured approach to security dependency management

## [0.1.1] - 2025-09-24

### Changed
- **Dependency update**: Upgraded airs-mcp from 0.2.1 to 0.2.2
- **Security enhancement**: Now uses airs-mcp v0.2.2 with RSA vulnerability fix (RUSTSEC-2023-0071 resolved)

### Security
- **Improved security posture**: Indirect resolution of RSA Marvin Attack vulnerability through upstream dependency update

## [0.1.0] - 2025-09-24

### Added
- **Initial release** of airs-mcpserver-fs as dedicated MCP filesystem server
- **Architectural migration** from crates/airs-mcp-fs to mcp-servers/airs-mcpserver-fs
- **Complete filesystem operations** through MCP protocol (read, write, list, create, delete, move, copy)
- **Security-first design** with 5-layer security validation system
- **Binary file restriction** with comprehensive content and extension-based detection
- **Human-in-the-loop approval** workflows for write/delete operations
- **Multi-environment configuration** system (development, staging, production)
- **Claude Desktop integration** with step-by-step setup guide
- **Comprehensive audit logging** with correlation IDs and security event tracking
- **Path validation and sanitization** with unicode normalization and traversal prevention
- **Performance optimization** with sub-100ms response times

### Security
- **Zero production vulnerabilities** - All security audits passed
- **5-layer security framework** - Path validation, permissions, policies, approval, audit
- **Binary file blocking** - Prevents entire classes of binary-based attacks
- **Memory safety** - Rust-based implementation with comprehensive error handling
- **Audit compliance** - Complete operation logging with security event tracking

### Technical Implementation
- **188 comprehensive tests** (146 unit + 17 integration + 25 doc tests)
- **Zero compilation warnings** - Professional code quality maintained
- **Workspace integration** - Clean dependency management with airs-mcp v0.2.1
- **Modular architecture** - Separation of concerns with clean interfaces
- **Cross-platform support** - Windows, macOS, and Linux compatibility

### Dependencies
- **airs-mcp v0.2.1** - Foundation MCP protocol implementation
- **Security dependencies** - Modern cryptography and validation libraries
- **Testing framework** - Comprehensive test coverage with multiple test types
