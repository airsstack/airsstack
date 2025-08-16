# tech_context.md

## Technologies Used
- Rust 1.88.0+ (MSRV with async/await support)
- AIRS workspace architecture
- Model Context Protocol (MCP) - **2025-03-26 specification compliance**
- **Current Implementation Stack**:
  - Tokio (async runtime) - **Multi-runtime architecture patterns**
  - Serde/Serde JSON (serialization)
  - DashMap (concurrent hashmap)
  - Bytes (zero-copy buffer management) ✅ IMPLEMENTED
  - BytesMut (mutable buffer operations) ✅ IMPLEMENTED
  - Thiserror (structured error handling)
- **Remote Server Dependencies (PLANNED)**:
  - **hyper/axum**: HTTP Streamable server implementation with async/await support
  - **oauth2 crate**: OAuth 2.1 Protected Resource Metadata compliance  
  - **deadpool**: Production-grade connection pooling for session management
  - **crossbeam-queue**: Lock-free patterns for performance optimization
  - **tokio**: Enhanced async runtime for all transport operations
  - **serde**: JSON serialization for MCP protocol messages

## Remote Server Technology Stack (8-Week Implementation Plan)
**Phase 1 Technologies (Weeks 1-3)**:
- **hyper**: HTTP/1.1 and HTTP/2 server foundation
- **axum**: Web framework for `/mcp` endpoint implementation
- **tokio**: Async runtime for session management and request handling
- **serde_json**: Dynamic JSON response mode selection

**Phase 2 Technologies (Weeks 4-6)**:
- **oauth2**: OAuth 2.1 + PKCE implementation with S256 challenge method
- **jsonwebtoken**: JWT token validation and parsing
- **ring**: Cryptographic operations for secure token handling
- **deadpool**: Connection pooling for authorization server communication

**Phase 3 Technologies (Weeks 7-8)**:
- **metrics**: Performance monitoring and metrics collection
- **tracing**: Structured logging for production observability
- **tower**: Middleware for rate limiting and request processing
- **crossbeam**: Lock-free data structures for high-performance operation

## MCP Protocol Evolution - CRITICAL UPDATE
- **HTTP Streamable Transport**: Official replacement for HTTP+SSE (March 2025)
- **Single Endpoint**: `/mcp` endpoint with dynamic response modes
- **Session Management**: `Mcp-Session-Id` headers with reconnection support
- **Legacy SSE**: Deprecated but supported for backward compatibility
- **OAuth 2.1 Mandatory**: MCP Protocol Revision 2025-06-18 enhanced security requirements
- **Universal PKCE**: Mandatory for ALL clients including confidential clients
- **Resource Indicators**: RFC 8707 mandatory for token binding and phishing protection
- **Protected Resource Metadata**: RFC 9728 for authorization server discovery

## Standards Compliance Documentation
**OAuth 2.1 RFC Specifications**: `oauth2_rfc_specifications.md`
- RFC 9728: OAuth 2.0 Protected Resource Metadata (complete implementation guide)
- RFC 7636: Proof Key for Code Exchange (PKCE) with S256 method requirements
- RFC 8707: Resource Indicators for OAuth 2.0 (prevents confused deputy attacks)
- RFC 6749: OAuth 2.0 Authorization Framework (core authorization flows)

**MCP Protocol Specification**: `mcp_official_specification.md`
- MCP 2025-06-18: Current specification with OAuth 2.1 integration requirements
- JSON-RPC 2.0: Base protocol for MCP message format
- Security Architecture: Client-host-server isolation boundaries
- OAuth Integration: Mandatory HTTP transport authentication requirements
- Implementation Patterns: Token audience validation, scope mapping, PKCE integration

## Development Setup
- Built as part of AIRS workspace
- Follows workspace build instructions
- **Performance Targets**: 50,000+ concurrent connections, sub-millisecond latency
- **Production Standards**: Enterprise-scale deployment patterns

## Technical Constraints
- Must comply with MIT OR Apache-2.0 licensing
- Adhere to AIRS documentation and architecture standards
- **MCP 2025-03-26 specification compliance** ✅ MANDATORY
- **OAuth 2.1 implementation** for enterprise security ✅ REQUIRED
- Feature flags for transport and security options
- Security audit framework required (static/dynamic analysis, compliance, vulnerability scanning)
