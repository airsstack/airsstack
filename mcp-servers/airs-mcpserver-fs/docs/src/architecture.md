# Technical Architecture

## Core Technology Stack

### **Foundation Layer**
- **Language**: Rust 2021 Edition (performance, safety, AIRS ecosystem alignment)
- **MCP Protocol**: Built on AIRS MCP foundation with STDIO transport for Claude Desktop
- **Async Runtime**: Tokio for high-performance file I/O operations
- **Security Framework**: Custom approval workflows with binary file restriction

### **Security-First Processing Stack**
- **Binary File Restriction**: Complete blocking of binary file operations for enhanced security
- **Text Processing**: Focus on source code, configuration, and documentation files
- **Format Detection**: Extension-based and content-based binary detection for rejection
- **Audit Logging**: Comprehensive tracking of security events and file operations

### **Security & Configuration**
- **Path Validation**: Canonical path resolution with allowlist/denylist patterns
- **Approval Workflows**: Human-in-the-loop confirmation for write operations
- **Configuration**: TOML-based security policies and operational settings

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Desktop                           │
│                   (MCP Client)                              │
└─────────────────────┬───────────────────────────────────────┘
                      │ STDIO Transport
                      │ JSON-RPC 2.0 Messages
┌─────────────────────▼───────────────────────────────────────┐
│                  AIRS MCP-FS                                │
│                  (MCP Server)                               │
├─────────────────────────────────────────────────────────────┤
│  Security Layer                                             │
│  ├─ Binary File Restriction (First Layer of Defense)        │
│  ├─ Path Validation & Access Control                        │
│  ├─ Human-in-the-Loop Approval Workflows                    │
│  └─ Operation Audit Logging                                 │
├─────────────────────────────────────────────────────────────┤
│  Tool Layer                                                 │
│  ├─ read_file, write_file, list_directory                   │
│  ├─ create_directory, delete_file, move_file                │
│  └─ Text-only file operations (binary files rejected)       │
├─────────────────────────────────────────────────────────────┤
│  Text Processing Engine                                     │
│  ├─ Source Code Processing (Rust, Python, JavaScript, etc.) │
│  ├─ Configuration File Processing (TOML, JSON, YAML, etc.)  │
│  ├─ Documentation Processing (Markdown, Text, etc.)         │
│  └─ Streaming Support for Large Text Files                  │
├─────────────────────────────────────────────────────────────┤
│  Filesystem Abstraction                                     │
│  ├─ Cross-Platform Path Handling                            │
│  ├─ Efficient I/O with Memory Management                    │
│  └─ File Watching & Change Detection                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                Local Filesystem                             │
│           (User's Development Environment)                  │
└─────────────────────────────────────────────────────────────┘
```

