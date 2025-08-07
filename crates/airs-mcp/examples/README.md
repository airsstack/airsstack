# AIRS-MCP Examples

This directory contains example implementations demonstrating how to use the `airs-mcp` library.

## Examples

### [simple-mcp-server](./simple-mcp-server/)

A comprehensive example demonstrating the core features of the MCP (Model Context Protocol) implementation:

- **Resources**: File system resource provider
- **Tools**: Calculator and greeting tools  
- **Prompts**: Code review and concept explanation prompts
- **Testing Scripts**: Python and shell scripts for manual testing

**To run:**
```bash
cd examples/simple-mcp-server
cargo run
```

**To test:**
```bash
cd examples/simple-mcp-server
./test_simple.sh
```

## Creating New Examples

Each example should be a self-contained Rust project with its own `Cargo.toml`:

1. Create a new directory: `mkdir examples/my-example`
2. Add standalone `Cargo.toml` with `airs-mcp` dependency
3. Include documentation and testing scripts
4. Add description to this README

## Structure

```
examples/
├── README.md                    # This file
├── simple-mcp-server/          # Basic MCP server implementation
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── README.md
│   └── test_*.{sh,py}          # Testing scripts
└── future-example/             # Template for new examples
    ├── Cargo.toml
    ├── src/main.rs
    └── README.md
```

Each example demonstrates specific aspects of the `airs-mcp` library and serves as both documentation and reference implementation.
