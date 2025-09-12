# [TASK-030] - HTTP Transport Zero-Dyn Architecture Refactoring

**Status:** pending  
**Added:** 2025-09-12  
**Updated:** 2025-09-12

## Original Request
Complete architectural refactoring of HTTP transport to eliminate all `dyn` patterns, implement zero-cost generic abstractions, remove dual-layer JSON-RPC processing, and ensure compatibility with `McpServer<T: Transport>` abstraction layer.

## Thought Process
Through detailed architectural analysis, we identified several critical issues with the current HTTP transport implementation:

1. **Dual MCP Handling Paths**: Current system has unused `mcp_handler: Option<Arc<dyn McpRequestHandler>>` alongside active `mcp_handlers: Arc<McpHandlers>` causing architectural confusion
2. **Unnecessary JSON-RPC Layer**: HTTP → JSON-RPC → mcp_operations.rs creates triple processing overhead
3. **Dynamic Dispatch Overhead**: Multiple `Arc<dyn Trait>` patterns violate workspace standards (§5.1)
4. **Code Duplication**: `handlers.rs` and `mcp_operations.rs` contain duplicate MCP logic
5. **McpServer Integration Gap**: HTTP transport must implement `Transport` trait for high-level `McpServer` wrapper

**Architectural Decision**: Transform to direct McpRequestHandler pattern with associated types, eliminate legacy components, and maintain authentication at engine layer.

## Implementation Plan

### Phase 1: Core Trait Redesign with Associated Types
- [ ] **HttpEngine Trait Refactor**: Replace `Arc<dyn McpRequestHandler>` with `type Handler: McpRequestHandler`
- [ ] **McpRequestHandler Generic**: Create `AxumMcpRequestHandler<R, T, P, L>` with generic provider types
- [ ] **Default Provider Types**: Implement `NoResourceProvider`, `NoToolProvider`, etc. for zero-cost defaults
- [ ] **Error Mapping**: Ensure `HttpEngineError` properly converts to/from `TransportError`

### Phase 2: Direct MCP Handler Implementation  
- [ ] **Create AxumMcpRequestHandler**: Direct HTTP → MCP processing without JSON-RPC intermediary
- [ ] **Migrate MCP Logic**: Move all logic from `mcp_operations.rs` into `AxumMcpRequestHandler` methods
- [ ] **Generic Builder Pattern**: `AxumMcpRequestHandlerBuilder<R, T, P, L>` with type-safe provider injection
- [ ] **HTTP Request/Response Types**: Define proper `HttpRequest`/`HttpResponse` structs

### Phase 3: AxumHttpServer Simplification
- [ ] **Remove Legacy Fields**: Eliminate `mcp_handlers` from `ServerState`, use direct `mcp_handler` storage
- [ ] **Update Constructor**: Remove `McpHandlers` parameter, inject via `register_mcp_handler()`
- [ ] **Simplify Router**: Update `create_router()` to use `Extension<AxumMcpRequestHandler>`
- [ ] **Direct Handler Usage**: Simplify `handle_mcp_request()` to delegate directly to handler

### Phase 4: Generic HttpTransport & Builder
- [ ] **Generic HttpTransport**: `HttpTransport<E: HttpEngine>` with associated handler type
- [ ] **Transport Trait Implementation**: Implement `Transport` for `McpServer` compatibility
- [ ] **Generic Builder**: `HttpTransportBuilder<E>` with engine-specific configuration methods
- [ ] **Pre-configured Builders**: OAuth2, custom auth builder methods for common patterns

### Phase 5: Authentication Integration
- [ ] **Engine-Layer Auth**: Keep existing AxumHttpServer authentication builder methods
- [ ] **Builder Delegation**: HttpTransportBuilder delegates auth config to engine builders
- [ ] **Factory Methods**: `with_oauth2_engine()`, `with_custom_auth_engine()` for common patterns
- [ ] **Manual Configuration**: Direct engine access for advanced scenarios

### Phase 6: Legacy Component Removal & Integration
- [ ] **Delete Files**: Remove `mcp_operations.rs`, `mcp_handlers.rs`
- [ ] **Update Examples**: Modernize all HTTP examples to use new architecture
- [ ] **McpServer Integration**: Ensure full compatibility with `McpServer<HttpTransport<E>>`
- [ ] **Documentation Update**: Update all documentation to reflect new architecture

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | HttpEngine trait with associated Handler type | not_started | 2025-09-12 | Remove Arc<dyn McpRequestHandler>, add type Handler |
| 1.2 | Generic AxumMcpRequestHandler with provider types | not_started | 2025-09-12 | Zero-cost generic abstraction over providers |
| 1.3 | Default provider implementations | not_started | 2025-09-12 | NoResourceProvider, NoToolProvider, etc. |
| 2.1 | Direct MCP processing without JSON-RPC layer | not_started | 2025-09-12 | Eliminate triple processing overhead |
| 2.2 | Migrate logic from mcp_operations.rs | not_started | 2025-09-12 | Move all MCP method implementations |
| 2.3 | Generic builder pattern for handler | not_started | 2025-09-12 | Type-safe provider injection |
| 3.1 | Remove McpHandlers from ServerState | not_started | 2025-09-12 | Simplify state to direct handler reference |
| 3.2 | Update AxumHttpServer constructor | not_started | 2025-09-12 | Remove MCP dependencies from constructor |
| 3.3 | Simplify router and handlers | not_started | 2025-09-12 | Direct delegation to AxumMcpRequestHandler |
| 4.1 | Generic HttpTransport implementation | not_started | 2025-09-12 | HttpTransport<E: HttpEngine> |
| 4.2 | Transport trait implementation | not_started | 2025-09-12 | McpServer compatibility |
| 4.3 | Generic HttpTransportBuilder | not_started | 2025-09-12 | Engine-agnostic builder pattern |
| 5.1 | Preserve AxumHttpServer auth builders | not_started | 2025-09-12 | Keep with_oauth2_authorization, etc. |
| 5.2 | HttpTransportBuilder auth delegation | not_started | 2025-09-12 | Delegate to engine-specific builders |
| 5.3 | Pre-configured engine builders | not_started | 2025-09-12 | Common OAuth2, custom auth patterns |
| 6.1 | Delete legacy components | not_started | 2025-09-12 | Remove mcp_operations.rs, mcp_handlers.rs |
| 6.2 | Update examples and documentation | not_started | 2025-09-12 | Modernize all HTTP examples |
| 6.3 | Validate McpServer integration | not_started | 2025-09-12 | Test full application flow |

## Progress Log

### 2025-09-12
- Created task with comprehensive architectural analysis
- Documented zero-dyn architecture decisions
- Identified integration requirements with McpServer abstraction
- Defined 6-phase implementation plan with 18 subtasks
- Established workspace standards compliance requirements (§2.1, §3.2, §4.3, §5.1)

## Architectural Decisions Captured

### Zero-Dyn Architecture Pattern
- **Associated Types**: `trait HttpEngine { type Handler: McpRequestHandler; }`
- **Generic Constraints**: `HttpTransport<E: HttpEngine>` instead of `Box<dyn Trait>`
- **Concrete Storage**: Direct `AxumMcpRequestHandler` storage, no dynamic dispatch
- **Provider Generics**: `AxumMcpRequestHandler<R, T, P, L>` with default types

### Direct MCP Integration
- **Eliminate**: JSON-RPC intermediary, `mcp_operations.rs`, `McpHandlers`
- **Flow**: HTTP Request → AxumMcpRequestHandler → HTTP Response
- **Benefits**: Single processing path, reduced allocations, type safety

### Engine-Layer Authentication
- **Principle**: Authentication/authorization remains at concrete engine implementation
- **HttpEngine**: Core lifecycle only (bind, start, shutdown, register_mcp_handler)
- **AxumHttpServer**: OAuth2, custom auth via existing builder patterns
- **HttpTransportBuilder**: Delegates engine-specific configuration

### McpServer Integration
- **Requirement**: `HttpTransport<E>` must implement `Transport` trait
- **Application Flow**: HttpTransportBuilder → HttpTransport → McpServer → start()
- **Configuration**: Providers → Handler → Transport → Server → Lifecycle
- **Compatibility**: Full integration with existing `McpServer<T: Transport>` abstraction

### Usage Examples Defined
```rust
// Simple usage
let mut transport = HttpTransportBuilder::with_default_engine().build().await?;
let handler = AxumMcpRequestHandlerBuilder::new().build();
transport.register_mcp_handler(handler);
let server = McpServer::new(transport);
server.start().await?;

// OAuth2 usage  
let mut transport = HttpTransportBuilder::with_oauth2_engine(
    connection_manager, oauth2_adapter, auth_config
).await?.build().await?;
let handler = AxumMcpRequestHandlerBuilder::new()
    .with_resource_provider(provider).build();
transport.register_mcp_handler(handler);
let server = McpServer::new(transport);
server.start().await?;
```

### Quality Requirements
- **Zero Warnings**: `cargo check --workspace`, `cargo clippy --workspace`
- **All Tests Pass**: `cargo test --workspace`
- **Workspace Standards**: §2.1 (3-layer imports), §3.2 (chrono), §4.3 (mod.rs), §5.1 (no dyn)
- **Backward Compatibility**: Existing authentication patterns preserved
- **Documentation**: Complete API documentation and examples