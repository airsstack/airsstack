# Documentation & Community

## **Documentation Strategy**

### **Multi-Audience Documentation**
1. **Developer Documentation**: API reference, architecture guides, examples
2. **User Documentation**: Installation, configuration, usage tutorials
3. **Administrator Documentation**: Security configuration, deployment guides
4. **Contributor Documentation**: Development setup, contribution guidelines

### **Interactive Examples**
```markdown
# AIRS MCP-FS Usage Examples

## Basic File Operations

### Reading Files
```rust
// Through MCP tool call
{
  "name": "read_file",
  "arguments": {
    "path": "./README.md",
    "encoding": "utf8"
  }
}
```

## Writing Files with Approval
```rust
// The system will prompt for approval before writing
{
  "name": "write_file", 
  "arguments": {
    "path": "./new-feature.md",
    "content": "# New Feature\n\nThis describes...",
    "create_directories": true
  }
}
```

## Image Processing
```rust
// Process image with thumbnail generation
{
  "name": "read_binary_advanced",
  "arguments": {
    "path": "./assets/logo.png",
    "processing_options": {
      "image_options": {
        "generate_thumbnail": true,
        "max_dimension": 800,
        "extract_metadata": true
      }
    }
  }
}
```

## **Community Building**
```markdown
# Contributing to AIRS MCP-FS

## Getting Started
1. Fork the repository
2. Set up development environment
3. Run the test suite
4. Make your changes
5. Submit a pull request

## Development Environment
```bash
# Clone and setup
git clone https://github.com/airsstack/airsstack.git
cd airs/mcp-servers/airs-mcpserver-fs

# Install dependencies
cargo build

# Run tests
cargo test

# Run with development config
cargo run -- --config ./dev-config.toml
```
