# Legacy Integration Layer Backup

This directory contains a backup of the legacy integration layer that was replaced during Task 028 Module Consolidation Refactoring.

## Backup Date
September 9, 2025

## Contents

### Legacy Files (Removed)
- `client.rs` - Legacy JsonRpcClient using old transport::traits::Transport
- `server.rs` - Legacy JsonRpcServer implementation  
- `handler.rs` - Legacy handler traits and implementations
- `router.rs` - Legacy message routing system
- `error.rs` - Integration error types (some parts kept/merged)
- `mod.rs` - Legacy module exports

### Modern Implementation (Moved to /integration/)
- `mcp/client.rs` → `/integration/client.rs` - Modern McpClient
- `mcp/server.rs` → `/integration/server.rs` - Modern McpServer
- `mcp/error.rs` → `/integration/error.rs` - MCP-specific errors
- `mcp/constants.rs` → `/integration/constants.rs` - MCP protocol constants
- `mcp/mod.rs` → `/integration/mod.rs` - Modern module exports

## Architecture Changes

### Before (Legacy)
```
integration/
├── client.rs        # JsonRpcClient (legacy transport)
├── server.rs        # JsonRpcServer (legacy transport)
├── handler.rs       # Generic handlers
├── router.rs        # Message routing
└── mcp/            # MCP-specific layer
    ├── client.rs    # McpClient (modern transport)
    └── server.rs    # McpServer (modern transport)
```

### After (Modern)
```
integration/
├── client.rs        # McpClient (modern transport) 
├── server.rs        # McpServer (modern transport)
├── error.rs         # MCP-specific errors
├── constants.rs     # MCP protocol constants
└── mod.rs           # Clean exports
```

## Transport Pattern Migration

### Legacy Pattern (Removed)
- `transport::traits::Transport` with `send()`/`receive()` methods
- Manual correlation management with `CorrelationManager`
- Request/response pairing in background tasks
- Generic JSON-RPC operations

### Modern Pattern (Now Used)
- `protocol::transport::Transport` with `MessageHandler` callbacks
- Event-driven architecture with async message handling
- Built-in correlation via protocol layer
- MCP-specific operations with type safety

## Recovery Instructions

If any legacy logic needs to be referenced:
1. Check the backed up files for implementation patterns
2. Adapt the logic to use the modern protocol transport
3. Use MCP-specific types instead of generic JSON-RPC
4. Follow the event-driven MessageHandler pattern

## Integration Points Changed

- All imports changed from `crate::transport::Transport` to `crate::protocol::transport::Transport`
- Client creation pattern changed from generic JSON-RPC to MCP-specific builders
- Error handling changed from `IntegrationError` to `McpError` for MCP operations
- Message handling changed from manual correlation to `MessageHandler` callbacks

This backup preserves the complete legacy implementation for reference during the transition period.
