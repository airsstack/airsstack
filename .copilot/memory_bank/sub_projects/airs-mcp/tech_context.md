# tech_context.md

## Technologies Used
- Rust
- AIRS workspace
- Model Context Protocol (MCP) - **2025-03-26 specification compliance**
- Tokio (async runtime) - **Multi-runtime architecture patterns**
- Serde/Serde JSON (serialization)
- DashMap (concurrent hashmap)
- Bytes (zero-copy buffer management) ✅ NEW
- BytesMut (mutable buffer operations) ✅ NEW
- Thiserror (structured error handling)
- **HTTP Streamable Transport**: hyper/axum for single-endpoint architecture ✅ PRIORITY
- **OAuth 2.1 Security**: oauth2 crate for enterprise authentication ✅ REQUIRED
- **Connection Pooling**: deadpool for production resource management ✅ PERFORMANCE
- Optional: OAuth2, Rustls (feature-gated)

## MCP Protocol Evolution - CRITICAL UPDATE
- **HTTP Streamable Transport**: Official replacement for HTTP+SSE (March 2025)
- **Single Endpoint**: `/mcp` endpoint with dynamic response modes
- **Session Management**: `Mcp-Session-Id` headers with reconnection support
- **Legacy SSE**: Deprecated but supported for backward compatibility
- **OAuth 2.1 Mandatory**: MCP Protocol Revision 2025-06-18 enhanced security requirements
- **Universal PKCE**: Mandatory for ALL clients including confidential clients
- **Resource Indicators**: RFC 8707 mandatory for token binding and phishing protection
- **Protected Resource Metadata**: RFC 9728 for authorization server discovery

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
