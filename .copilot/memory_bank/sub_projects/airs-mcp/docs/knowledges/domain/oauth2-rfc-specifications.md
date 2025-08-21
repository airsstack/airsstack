# OAuth 2.1 RFC Specifications - TASK014 Reference Documentation

This document contains essential RFC specifications required for implementing OAuth 2.1 enterprise authentication in TASK014.

## RFC 9728: OAuth 2.0 Protected Resource Metadata

**Status**: Mandatory for TASK014  
**Purpose**: Enables protected resources to publish metadata about their configuration, including supported scopes, authorization servers, and security requirements.

### Key Implementation Requirements

#### Metadata Endpoint
- **Location**: `/.well-known/oauth-protected-resource`
- **Method**: HTTP GET
- **Content-Type**: `application/json`

#### Required Metadata Parameters
```json
{
  "resource": "https://mcp.example.com",
  "authorization_servers": ["https://auth.example.com"],
  "scopes_supported": ["mcp:tools:execute", "mcp:resources:read", "mcp:prompts:read"],
  "bearer_methods_supported": ["header"],
  "resource_name": "MCP Resource Server"
}
```

#### Critical Implementation Details
1. **Resource Identifier Validation**: The `resource` value MUST be identical to the protected resource's identifier
2. **TLS Requirements**: MUST use TLS for all metadata requests
3. **WWW-Authenticate Integration**: Can dynamically indicate metadata URL changes
4. **Audience Restriction**: SHOULD audience-restrict issued access tokens

### Security Considerations for TASK014
- **Impersonation Prevention**: Resource identifier validation prevents metadata spoofing
- **Audience-Restricted Tokens**: RECOMMENDED when multiple resource servers exist
- **Certificate Validation**: MUST perform TLS certificate checking per RFC 9525

---

## RFC 7636: Proof Key for Code Exchange (PKCE)

**Status**: Mandatory for TASK014 - Universal PKCE requirement  
**Purpose**: Prevents authorization code interception attacks, especially for public clients.

### Key Implementation Requirements

#### S256 Method (Mandatory)
```rust
// Code verifier generation (43-128 characters)
let code_verifier = generate_random_string(43); // Base64url, high entropy

// Code challenge generation (REQUIRED: S256 method)
let code_challenge = BASE64URL-ENCODE(SHA256(ASCII(code_verifier)));
```

#### Authorization Request Parameters
- `code_challenge`: Base64url-encoded SHA256 hash of code verifier
- `code_challenge_method`: MUST be "S256" (plain method discouraged)

#### Token Request Parameters
- `code_verifier`: Original random string used to generate challenge

### Critical Security Requirements
1. **Minimum Entropy**: Code verifier MUST have 256 bits of entropy
2. **S256 Mandatory**: Clients MUST use S256 method if capable
3. **No Downgrade**: Clients MUST NOT downgrade from S256 to plain
4. **Server Validation**: Server MUST verify code_challenge matches transformed code_verifier

### Implementation for TASK014
```rust
impl OAuth2Security {
    fn generate_pkce_challenge() -> (String, String) {
        // Generate 32-byte random sequence (256 bits entropy)
        let verifier = generate_code_verifier();
        let challenge = BASE64URL-ENCODE(SHA256(ASCII(verifier)));
        (verifier, challenge)
    }
    
    fn validate_pkce(code_verifier: &str, stored_challenge: &str) -> bool {
        let computed_challenge = BASE64URL-ENCODE(SHA256(ASCII(code_verifier)));
        computed_challenge == stored_challenge
    }
}
```

---

## RFC 8707: Resource Indicators for OAuth 2.0

**Status**: Mandatory for TASK014 - Confused deputy attack prevention  
**Purpose**: Enables clients to explicitly signal the intended resource for access tokens.

### Key Implementation Requirements

#### Resource Parameter Usage
```http
# Authorization Request
GET /oauth/authorize?
  response_type=code&
  client_id=mcp-client&
  resource=https%3A%2F%2Fmcp.example.com%2F&
  code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&
  code_challenge_method=S256

# Token Request  
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=abc123&
resource=https%3A%2F%2Fmcp.example.com%2F&
code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
```

#### Resource Parameter Requirements
1. **Format**: MUST be absolute URI (RFC 3986 Section 4.3)
2. **Fragment**: MUST NOT include fragment component
3. **Query**: SHOULD NOT include query component (exceptions allowed)
4. **Specificity**: Use most specific URI for the complete API

### Security Benefits for TASK014
1. **Audience Restriction**: Tokens restricted to specific MCP resources
2. **Confused Deputy Prevention**: Prevents token misuse at unintended resources
3. **Multi-Tenant Safety**: Enables tenant-specific audience restrictions

### Implementation for TASK014
```rust
impl OAuth2Security {
    const MCP_RESOURCE_URI: &str = "https://mcp.example.com/";
    
    fn build_authorization_url(&self, state: &str) -> String {
        format!(
            "{}?response_type=code&client_id={}&resource={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.auth_endpoint,
            self.client_id,
            urlencoding::encode(Self::MCP_RESOURCE_URI),
            state,
            self.code_challenge
        )
    }
    
    fn validate_audience(&self, token_audience: &str) -> bool {
        token_audience == Self::MCP_RESOURCE_URI
    }
}
```

---

## TASK014 Implementation Summary

### Combined OAuth 2.1 Flow
1. **PKCE Generation**: Create S256 code challenge
2. **Resource Indication**: Specify MCP resource URI
3. **Authorization Request**: Include code_challenge, resource parameters
4. **Metadata Discovery**: Fetch `/.well-known/oauth-protected-resource`
5. **Token Exchange**: Verify code_verifier, validate audience
6. **Token Validation**: Confirm audience restriction to MCP resource

### Security Standards Achieved
- ✅ **RFC 9728**: Protected resource metadata with audience restriction
- ✅ **RFC 7636**: Universal S256 PKCE implementation
- ✅ **RFC 8707**: Resource indicators for confused deputy prevention
- ✅ **MCP 2025-03-26**: Latest MCP OAuth 2.1 specification compliance

This reference documentation provides the complete technical foundation for implementing enterprise-grade OAuth 2.1 authentication in TASK014 with full standards compliance.
