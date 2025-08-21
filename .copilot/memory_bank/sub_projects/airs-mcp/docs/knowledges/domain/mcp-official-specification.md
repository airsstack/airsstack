# MCP Official Specification Protocol - TASK014 Reference Documentation

This document contains the official Model Context Protocol (MCP) specification requirements for implementing OAuth 2.1 authentication in TASK014.

## Protocol Overview

**MCP Version**: 2025-06-18 (Current)  
**Base Protocol**: JSON-RPC 2.0  
**Authorization**: OAuth 2.1 with selected subset of features  
**Security**: Mandatory for HTTP-based transports

## MCP Architecture

### Core Components
```
Host Process (Container & Coordinator)
├── Multiple Client Instances
│   ├── 1:1 relationship with servers
│   ├── Protocol negotiation & capability exchange
│   ├── Bidirectional message routing
│   └── Security boundary maintenance
└── Security Policy Enforcement
    ├── User authorization decisions
    ├── Context aggregation control
    └── Connection permission management
```

### Design Principles Critical for TASK014
1. **Server Isolation**: Servers cannot read full conversations or "see into" other servers
2. **Host-Controlled Security**: Host enforces all security boundaries and consent requirements
3. **Capability Negotiation**: Explicit declaration of supported features during initialization
4. **Progressive Enhancement**: Core protocol + optional capabilities as needed

## MCP Authorization Framework

### Mandatory Requirements for HTTP Transports

#### Standards Compliance
- **OAuth 2.1**: IETF Draft (draft-ietf-oauth-v2-1-13) - Selected subset implementation
- **RFC 8414**: OAuth 2.0 Authorization Server Metadata (MANDATORY)
- **RFC 7591**: Dynamic Client Registration Protocol (SHOULD support)  
- **RFC 9728**: OAuth 2.0 Protected Resource Metadata (MANDATORY)
- **RFC 8707**: Resource Indicators for OAuth 2.0 (MANDATORY)

#### OAuth 2.1 Roles in MCP Context
```rust
// MCP Server → OAuth 2.1 Resource Server
impl OAuth2ResourceServer for McpServer {
    // Accepts and validates access tokens
    // Responds to protected resource requests
    // Implements RFC 9728 metadata endpoint
}

// MCP Client → OAuth 2.1 Client  
impl OAuth2Client for McpClient {
    // Makes protected resource requests on behalf of user
    // Implements RFC 8707 resource indicators
    // Handles dynamic client registration (RFC 7591)
}

// Authorization Server → OAuth 2.1 Authorization Server
// (External entity - implementation details beyond MCP scope)
```

### Critical Implementation Requirements

#### 1. Protected Resource Metadata (RFC 9728) - MANDATORY
```http
GET /.well-known/oauth-protected-resource HTTP/1.1
Host: mcp.example.com

HTTP/1.1 200 OK
Content-Type: application/json

{
  "resource": "https://mcp.example.com",
  "authorization_servers": ["https://auth.example.com"],
  "scopes_supported": ["mcp:tools:execute", "mcp:resources:read", "mcp:prompts:read"],
  "bearer_methods_supported": ["header"],
  "resource_name": "MCP Resource Server"
}
```

#### 2. WWW-Authenticate Header (RFC 9728) - MANDATORY
```http
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer resource_metadata="https://mcp.example.com/.well-known/oauth-protected-resource"
```

#### 3. Resource Indicators (RFC 8707) - MANDATORY
```http
# Authorization Request
GET /oauth/authorize?
  response_type=code&
  client_id=mcp-client&
  resource=https%3A%2F%2Fmcp.example.com&
  state=xyz&
  code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&
  code_challenge_method=S256

# Token Request
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=abc123&
resource=https%3A%2F%2Fmcp.example.com&
code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
```

#### 4. Access Token Usage - MANDATORY
```http
GET /mcp HTTP/1.1
Host: mcp.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

**Token Requirements:**
- MUST use Authorization header (Bearer scheme)
- MUST NOT include tokens in URI query string
- MUST include authorization in EVERY HTTP request (even within same session)
- MUST validate audience restriction to specific MCP server

### Canonical MCP Server URI Format

**Valid Examples:**
- `https://mcp.example.com/mcp`
- `https://mcp.example.com`
- `https://mcp.example.com:8443`
- `https://mcp.example.com/server/mcp`

**Invalid Examples:**
- `mcp.example.com` (missing scheme)
- `https://mcp.example.com#fragment` (contains fragment)

**Consistency Rule**: Use form without trailing slash unless semantically significant.

## MCP Security Requirements for TASK014

### Critical Security Principles

#### 1. User Consent and Control
- Users MUST explicitly consent to all data access and operations
- Users MUST retain control over shared data and authorized actions
- Clear UIs REQUIRED for reviewing and authorizing activities

#### 2. Tool Safety
- Tools represent arbitrary code execution - treat with appropriate caution
- Hosts MUST obtain explicit user consent before invoking any tool
- Tool descriptions/annotations considered untrusted unless from trusted server

#### 3. Token Audience Binding (CRITICAL)
```rust
impl McpServer {
    fn validate_access_token(&self, token: &str) -> Result<AuthContext> {
        // MANDATORY: Validate token audience matches this MCP server
        let claims = self.jwt_validator.validate(token)?;
        
        if claims.audience != self.canonical_uri {
            return Err(AuthError::InvalidAudience);
        }
        
        // MANDATORY: Verify token issued specifically for this resource
        if !claims.aud.contains(&self.canonical_uri) {
            return Err(AuthError::WrongAudience);
        }
        
        Ok(AuthContext::from(claims))
    }
}
```

#### 4. Token Passthrough Prevention (CRITICAL)
- MCP servers MUST NOT pass through received tokens to upstream APIs
- MCP servers MUST use separate tokens for upstream API calls
- Each token MUST be bound to its intended resource only

#### 5. PKCE Implementation (MANDATORY)
```rust
impl McpClient {
    fn initiate_auth_flow(&self) -> AuthFlow {
        // MANDATORY: Implement PKCE for authorization code protection
        let (code_verifier, code_challenge) = generate_pkce_s256();
        
        AuthFlow {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256", // MANDATORY - S256 only
            resource: self.mcp_server_canonical_uri.clone(),
            state: generate_state(),
        }
    }
}
```

### Error Handling Requirements

#### HTTP Status Codes
| Code | Status | Usage |
|------|--------|-------|
| 401 | Unauthorized | Authorization required or token invalid |
| 403 | Forbidden | Invalid scopes or insufficient permissions |
| 400 | Bad Request | Malformed authorization request |

#### OAuth Error Responses (RFC 6749)
- `invalid_request`: Malformed request
- `invalid_client`: Client authentication failed
- `invalid_grant`: Authorization grant invalid/expired
- `unauthorized_client`: Client not authorized
- `unsupported_grant_type`: Grant type not supported
- `invalid_scope`: Requested scope invalid/unknown

## TASK014 MCP Integration Requirements

### Middleware Integration with MCP Protocol

```rust
// MCP Server with OAuth 2.1 middleware
impl AxumHttpServer {
    fn create_mcp_routes(&self) -> Router {
        Router::new()
            // Core MCP endpoints
            .route("/mcp", post(handle_mcp_request))
            .route("/health", get(handle_health))
            
            // OAuth 2.1 metadata endpoints (RFC 9728)
            .route("/.well-known/oauth-protected-resource", get(handle_metadata))
            
            // OAuth 2.1 middleware stack
            .layer(oauth_middleware_layer(self.oauth_config.clone()))
            .layer(session_middleware_layer(self.session_manager.clone()))
            .layer(rate_limiting_middleware())
    }
    
    async fn handle_mcp_request(
        State(server): State<Arc<AxumHttpServer>>,
        auth_context: AuthContext, // Injected by OAuth middleware
        Json(request): Json<JsonRpcRequest>,
    ) -> Result<Json<JsonRpcResponse>> {
        // MCP request processing with authenticated context
        server.process_mcp_request(request, auth_context).await
    }
}
```

### MCP Scope Mapping

```rust
// MCP method to OAuth scope mapping
const MCP_SCOPES: &[(&str, &str)] = &[
    ("tools/call", "mcp:tools:execute"),
    ("tools/list", "mcp:tools:read"),
    ("resources/read", "mcp:resources:read"),
    ("resources/list", "mcp:resources:list"),  
    ("resources/subscribe", "mcp:resources:subscribe"),
    ("prompts/get", "mcp:prompts:read"),
    ("prompts/list", "mcp:prompts:list"),
    ("logging/setLevel", "mcp:logging:configure"),
];

impl OAuth2Security {
    fn validate_mcp_method_scope(&self, method: &str, token_scopes: &[String]) -> bool {
        if let Some((_, required_scope)) = MCP_SCOPES.iter().find(|(m, _)| *m == method) {
            token_scopes.contains(&required_scope.to_string())
        } else {
            false // Unknown method requires explicit authorization
        }
    }
}
```

### Success Criteria for MCP Compliance

1. **✅ MCP Protocol Compliance**: 100% adherence to MCP 2025-06-18 specification
2. **✅ OAuth 2.1 Integration**: Full RFC compliance with MCP-specific requirements
3. **✅ Security Boundaries**: Proper token audience validation and session isolation
4. **✅ User Consent Flows**: Explicit authorization for all MCP operations
5. **✅ Capability Negotiation**: OAuth capabilities properly declared during MCP initialization
6. **✅ Transport Compatibility**: HTTP transport with proper authentication headers

## Implementation Summary for TASK014

The OAuth 2.1 middleware implementation MUST:

1. **Implement RFC 9728**: Protected resource metadata endpoint with MCP scope support
2. **Implement RFC 8707**: Resource indicators for MCP server identification  
3. **Integrate with MCP**: Scope validation for all MCP methods
4. **Maintain Security**: Token audience validation, no token passthrough
5. **Preserve MCP Architecture**: Client-host-server isolation boundaries
6. **Support Dynamic Registration**: RFC 7591 for automatic client onboarding

This creates a **production-ready OAuth 2.1 authentication layer** that fully respects MCP protocol design principles while providing enterprise-grade security for HTTP-based MCP deployments.
