# Technical Architecture

## Core Technology Stack

### **Foundation Layer**
- **Language**: Rust 2021 Edition (performance, safety, AIRS ecosystem alignment)
- **MCP Protocol**: Built on AIRS MCP foundation with STDIO transport for Claude Desktop
- **Async Runtime**: Tokio for high-performance file I/O operations
- **Security Framework**: Custom approval workflows with configurable policies

### **Binary Processing Stack**
- **Image Processing**: `image` crate with format detection and transformation
- **PDF Processing**: `pdf` and `pdf-extract` crates for text/image extraction
- **Format Detection**: `infer` crate for magic number-based file type identification
- **Compression**: LZ4 for efficient large file handling

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
│  ├─ Path Validation & Access Control                        │
│  ├─ Human-in-the-Loop Approval Workflows                    │
│  └─ Operation Audit Logging                                 │
├─────────────────────────────────────────────────────────────┤
│  Tool Layer                                                 │
│  ├─ read_file, write_file, list_directory                   │
│  ├─ create_directory, delete_file, move_file                │
│  └─ read_binary, write_binary, extract_content              │
├─────────────────────────────────────────────────────────────┤
│  Binary Processing Engine                                   │
│  ├─ Image Processing (resize, thumbnail, metadata)          │
│  ├─ PDF Processing (text extraction, image extraction)      │
│  ├─ Format Detection & Validation                           │
│  └─ Compression & Streaming for Large Files                 │
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

