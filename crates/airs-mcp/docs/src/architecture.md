# Architecture

This section describes the implementation architecture of the AIRS MCP library.

## Layer Architecture

The implementation is organized in distinct layers:

```
Integration Layer (High-level APIs)
    ↓
Protocol Layer (MCP message types)
    ↓  
Transport Layer (Communication)
    ↓
JSON-RPC 2.0 Foundation
```

### Integration Layer

Provides high-level APIs for application integration:

- **McpClient**: Client-side MCP operations
- **McpServer**: Server-side MCP implementation  
- **Provider Interfaces**: Extensible server capabilities

### Protocol Layer

Implements the MCP protocol specification:

- **Message Types**: Complete MCP message definitions
- **Lifecycle Management**: Connection initialization and capabilities
- **Validation**: Request and response validation

### Transport Layer

Handles communication between clients and servers:

- **Transport Abstraction**: Generic transport interface
- **STDIO Transport**: Process-based communication
- **HTTP Transport**: RESTful communication with authentication

### JSON-RPC 2.0 Foundation

Core message system implementation:

- **Message Types**: Request, response, and notification structures
- **Serialization**: JSON encoding and decoding
- **Error Handling**: Standard JSON-RPC error types

## Module Organization

The source code is organized by functional areas:

```
src/
├── integration/     # High-level client and server APIs
├── protocol/        # MCP protocol types and validation
├── transport/       # Transport implementations
├── authentication/ # Authentication strategies
├── authorization/   # Authorization middleware
└── oauth2/         # OAuth2 implementation
```

### Key Components

- **TransportClient**: Clean request-response interface
- **MessageHandler**: Generic message processing
- **Provider Traits**: Server capability interfaces
- **Authentication**: Pluggable auth strategies
- **Authorization**: Role-based access control

## Design Principles

The implementation follows these principles:

- **Type Safety**: Comprehensive type system with validation
- **Async Native**: Built on tokio for concurrent operations
- **Modular Design**: Clear separation of concerns
- **Transport Agnostic**: Multiple transport implementations
- **Protocol Compliant**: Full MCP specification support

## Implementation Structure

```
src/
├── integration/             # High-level integration APIs
│   ├── mcp/
│   │   ├── client.rs        # McpClient implementation
│   │   └── server.rs        # McpServer implementation
│   ├── client.rs            # JsonRpc client
│   └── router.rs            # Message routing
│
├── protocol/                # MCP protocol implementation
│   ├── messages/            # Complete MCP message types
│   └── types/               # MCP type definitions
│
├── transport/               # Transport layer
│   ├── adapters/
│   │   ├── http/            # HTTP transport system
│   │   │   ├── axum/        # Axum server integration
│   │   │   ├── auth/        # HTTP authentication adapters
│   │   │   ├── sse/         # Server-Sent Events transport
│   │   │   ├── session.rs   # HTTP session management
│   │   │   └── buffer_pool.rs # Buffer pooling system
│   │   └── stdio.rs         # STDIO transport
│   ├── buffer.rs            # Buffer management
│   ├── streaming.rs         # Streaming capabilities
│   └── mcp/                 # MCP transport layer
│
├── authentication/          # Authentication system
│   ├── strategies/          # Authentication strategies
│   ├── context.rs           # Authentication context
│   └── middleware.rs        # Authentication middleware
│
├── authorization/           # Authorization framework
│   ├── policies/            # Authorization policies
│   ├── context.rs           # Authorization context
│   └── middleware.rs        # Authorization middleware
│
├── oauth2/                  # OAuth2 implementation
│   ├── lifecycle/           # Token lifecycle management
│   │   ├── cache.rs         # Token caching
│   │   ├── refresh.rs       # Token refresh
│   │   └── manager.rs       # Lifecycle manager
│   ├── middleware/          # OAuth2 middleware system
│   │   ├── axum.rs          # Axum integration
│   │   └── core.rs          # Framework-agnostic core
│   ├── validator/           # JWT & scope validation
│   │   ├── jwt.rs           # JWT validation
│   │   └── scope.rs         # Scope validation
│   └── config.rs            # OAuth2 configuration
│
├── providers/               # Provider ecosystem
│   ├── resource.rs          # Resource providers
│   ├── tool.rs              # Tool providers
│   ├── prompt.rs            # Prompt providers
│   └── logging.rs           # Logging handlers
│
└── correlation/             # Request correlation system
    ├── manager.rs           # Correlation manager
    └── types.rs             # Correlation types
```

## Major System Components

### Authentication System (`src/authentication/`)
- **Generic Strategies**: Authentication strategy abstraction
- **API Key Support**: Bearer tokens, custom headers, query parameters  
- **OAuth2 Integration**: Complete OAuth2 authentication strategy
- **Context Management**: Type-safe authentication contexts
- **Middleware Integration**: Framework-agnostic middleware system

### Authorization Framework (`src/authorization/`)
- **Policy-Based Authorization**: NoAuth, Scope-based, Binary policies
- **Method Extractors**: JSON-RPC payload and HTTP path extraction
- **Middleware**: Authorization middleware
- **Context Types**: NoAuth, Scope, Binary authorization contexts
- **Error Handling**: Structured authorization error system

### OAuth2 2.1 System (`src/oauth2/`)
- **Complete Lifecycle**: Token caching, refresh, and management
- **JWT Validation**: JWKS client with RS256 support
- **Scope Validation**: MCP method-to-scope mapping
- **Middleware Stack**: Axum integration with framework-agnostic core
- **Configuration**: Comprehensive OAuth2 configuration system

### HTTP Transport (`src/transport/adapters/http/`)
- **Axum Server**: HTTP server with full MCP support
- **Authentication Integration**: HTTP auth adapters for all strategies
- **Server-Sent Events**: Real-time streaming transport
- **Session Management**: HTTP session lifecycle and state management
- **Buffer Pooling**: Buffer management system
- **Connection Management**: Connection pooling and health checks

### Provider Ecosystem (`src/providers/`)
- **Resource Providers**: FileSystem, Configuration, Database access
- **Tool Providers**: Math operations, System commands, Text processing
- **Prompt Providers**: Analysis, Code review, Documentation templates
- **Logging Handlers**: Structured and file-based logging systems
