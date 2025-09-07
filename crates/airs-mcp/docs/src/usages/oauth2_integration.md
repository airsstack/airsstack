# OAuth2 MCP Server Integration Guide

**Status:** Production-Ready ✅  
**Updated:** 2025-09-07  
**Validation:** MCP Inspector Compatible  

This guide demonstrates how to implement OAuth2 authentication for MCP servers using `airs-mcp`, validated with MCP Inspector and ready for production deployment.

## 🎯 Overview

OAuth2 integration with MCP enables enterprise-grade authentication while maintaining full compatibility with MCP protocol tools like MCP Inspector. This implementation uses:

- **Authorization Code Flow** with PKCE (Proof Key for Code Exchange)
- **JWT Token Authentication** with scope-based authorization  
- **Three-Server Architecture** for clean separation of concerns
- **MCP Inspector Compatibility** for testing and validation

## 🏗️ Architecture Overview

### Three-Server Architecture

The OAuth2 integration uses an innovative three-server architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Proxy Server  │────│ Custom Routes   │────│   MCP Server    │
│   Port 3002     │    │   Port 3003     │    │   Port 3004     │
│                 │    │                 │    │                 │
│ • Public API    │    │ • OAuth2 Flow   │    │ • MCP Protocol  │
│ • Smart Routing │    │ • Token Exchange│    │ • Providers     │
│ • MCP Inspector │    │ • JWT Generation│    │ • Business Logic│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Benefits:**
- Clean separation of authentication vs. protocol concerns
- MCP Inspector compatibility (expects OAuth2 discovery on MCP endpoint)
- Production-ready architecture with comprehensive logging
- Scalable design supporting multiple MCP servers

## 🚀 Quick Start

### 1. Run the OAuth2 Example

```bash
cd examples/mcp-remote-server-oauth2
cargo build --release
cargo run
```

The server starts with:
- **Proxy Server**: `http://127.0.0.1:3002` (public endpoint)
- **OAuth2 Endpoints**: `http://127.0.0.1:3003` (authentication)
- **MCP Server**: `http://127.0.0.1:3004` (protocol implementation)

### 2. Test with MCP Inspector

```bash
# Install MCP Inspector
npm install -g @modelcontextprotocol/inspector

# Configure MCP Inspector
npx @modelcontextprotocol/inspector
```

**MCP Inspector Configuration:**
```json
{
  "name": "OAuth2 MCP Server",
  "endpoint": "http://127.0.0.1:3002/mcp",
  "authentication": {
    "type": "oauth2",
    "discovery": "http://127.0.0.1:3002/.well-known/oauth-authorization-server",
    "client_id": "mcp-inspector-client"
  }
}
```

### 3. Verify OAuth2 Flow

1. **Discovery**: MCP Inspector fetches OAuth2 metadata
2. **Authorization**: Redirects to authorization endpoint with PKCE
3. **Token Exchange**: Exchanges authorization code for JWT token
4. **MCP Operations**: Uses Bearer token for all MCP requests

## 🔐 OAuth2 Implementation Details

### Authorization Code + PKCE Flow

```rust
// OAuth2 Discovery Metadata
#[derive(Serialize)]
pub struct OAuth2Metadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
}

// PKCE Challenge Generation
fn generate_pkce_challenge() -> (String, String) {
    let code_verifier = generate_random_string(43); // 43 chars, URL-safe
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(&digest);
    (code_verifier, code_challenge)
}
```

### JWT Token Structure

```json
{
  "sub": "dev_user_123",
  "aud": "mcp-server",
  "iss": "https://auth.example.com",
  "exp": 1757220537,
  "iat": 1757216937,
  "jti": "unique-token-id",
  "scope": "mcp:tools:execute mcp:resources:read mcp:prompts:read mcp:resources:list"
}
```

### Scope-Based Authorization

```rust
// MCP Method → OAuth2 Scope Mapping
const MCP_SCOPES: &[(&str, &str)] = &[
    ("resources/list", "mcp:resources:list"),
    ("resources/read", "mcp:resources:read"),
    ("tools/list", "mcp:tools:read"),
    ("tools/call", "mcp:tools:execute"),
    ("prompts/list", "mcp:prompts:list"),
    ("prompts/get", "mcp:prompts:read"),
];

// Validate MCP method requires specific scope
fn validate_mcp_scope(method: &str, token_scopes: &[String]) -> bool {
    if let Some((_, required_scope)) = MCP_SCOPES.iter()
        .find(|(m, _)| *m == method) {
        token_scopes.contains(&required_scope.to_string())
    } else {
        false // Unknown methods denied by default
    }
}
```

## 🏭 Production Implementation

### Complete OAuth2 Server Setup

```rust
use airs_mcp::{
    authentication::strategies::oauth2::OAuth2Strategy,
    oauth2::{config::OAuth2Config, validator::create_default_validator},
    transport::adapters::http::{
        auth::{oauth2::OAuth2StrategyAdapter, middleware::HttpAuthConfig},
        axum::AxumHttpServer,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OAuth2 Configuration
    let mut oauth2_config = OAuth2Config::default();
    oauth2_config.issuer = "https://auth.example.com".to_string();
    oauth2_config.audience = "mcp-server".to_string();
    
    // Create OAuth2 validator and strategy
    let validator = create_default_validator(oauth2_config)?;
    let oauth2_strategy = OAuth2Strategy::new(validator);
    let oauth2_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);
    
    // Configure authentication middleware
    let auth_config = HttpAuthConfig {
        skip_paths: vec![
            "/health".to_string(),
            "/.well-known/oauth-authorization-server".to_string(),
            "/authorize".to_string(),
            "/token".to_string(),
        ],
        include_error_details: false,
        auth_realm: "MCP Server".to_string(),
        request_timeout_secs: 30,
    };
    
    // Build server with OAuth2 authentication
    let server = AxumHttpServer::new(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        http_config,
    )
    .await?
    .with_authentication(oauth2_adapter, auth_config);
    
    server.serve().await?;
    Ok(())
}
```

### Resource Population for Immediate Testing

```rust
// Create sample files for demonstration
async fn create_sample_resources(temp_path: &Path) -> Result<(), Error> {
    // OAuth2 server welcome message
    tokio::fs::write(
        temp_path.join("welcome.txt"),
        "Welcome to the OAuth2 MCP Server!\n\n\
        This server provides:\n\
        - Filesystem resources\n\
        - Mathematical tools\n\
        - Code review prompts\n\n\
        Authenticate via OAuth2: Use Authorization: Bearer <token>.\n\
        Obtain tokens using the authorization code + PKCE flow."
    ).await?;
    
    // Server configuration
    tokio::fs::write(
        temp_path.join("config.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "server": {
                "name": "OAuth2 MCP Server",
                "version": "1.0.0",
                "authentication": "oauth2"
            },
            "capabilities": {
                "resources": true,
                "tools": true,
                "prompts": true
            },
            "endpoints": {
                "mcp": "http://127.0.0.1:3002/mcp",
                "oauth2_discovery": "http://127.0.0.1:3002/.well-known/oauth-authorization-server"
            }
        }))?
    ).await?;
    
    Ok(())
}
```

## 🧪 Testing and Validation

### Manual OAuth2 Flow Testing

```bash
# 1. Get authorization code
curl -G "http://127.0.0.1:3003/authorize" \
  --data-urlencode "response_type=code" \
  --data-urlencode "client_id=mcp-inspector-client" \
  --data-urlencode "redirect_uri=http://127.0.0.1:6274/oauth/callback/debug" \
  --data-urlencode "scope=mcp:tools:execute mcp:resources:read" \
  --data-urlencode "code_challenge=1OEicfz_Cio09rjbMf7Ot5F3GpAw6obak7CJrXsjtCg" \
  --data-urlencode "code_challenge_method=S256"

# 2. Exchange for token
curl -X POST "http://127.0.0.1:3003/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "grant_type=authorization_code" \
  --data-urlencode "code=AUTHORIZATION_CODE_HERE" \
  --data-urlencode "client_id=mcp-inspector-client" \
  --data-urlencode "code_verifier=ClW1UjPmI9pUt4J1yXV1ZwKGG4R7R8mrSdPGqUPjIjS"

# 3. Test MCP operations
curl -X POST "http://127.0.0.1:3002/mcp" \
  -H "Authorization: Bearer JWT_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": "1", "method": "resources/list"}'
```

### PKCE Challenge Generation

```python
# Generate proper PKCE challenge/verifier pair
import hashlib
import base64
import secrets

# Generate code verifier (43-128 characters, URL-safe)
code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode().rstrip('=')

# Generate code challenge (SHA256 hash of verifier)
digest = hashlib.sha256(code_verifier.encode('utf-8')).digest()
code_challenge = base64.urlsafe_b64encode(digest).decode().rstrip('=')

print(f"Code Verifier: {code_verifier}")
print(f"Code Challenge: {code_challenge}")
```

## 🔧 Advanced Configuration

### Production OAuth2 Provider Integration

```rust
// External OAuth2 provider configuration
let oauth2_config = OAuth2Config {
    issuer: "https://your-oauth-provider.com".to_string(),
    jwks_url: Some("https://your-oauth-provider.com/.well-known/jwks.json".parse()?),
    audience: "your-mcp-server".to_string(),
    // Additional production settings
    clock_skew_tolerance: Duration::from_secs(60),
    cache_ttl: Duration::from_secs(300),
    ..Default::default()
};
```

### Custom Scope Validation

```rust
// Custom scope validator for fine-grained permissions
pub struct CustomMcpScopeValidator {
    // Custom validation logic
}

impl ScopeValidator for CustomMcpScopeValidator {
    async fn validate_scope(&self, method: &str, scopes: &[String]) -> bool {
        match method {
            "resources/read" => {
                scopes.contains(&"mcp:resources:read".to_string())
                    || scopes.contains(&"admin".to_string())
            },
            "tools/call" => {
                let tool_name = extract_tool_name_from_context();
                scopes.contains(&format!("mcp:tools:{}", tool_name))
            },
            _ => false,
        }
    }
}
```

### Monitoring and Observability

```rust
// Production logging configuration
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("info,airs_mcp=debug"))
    .with(
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .json()
    )
    .init();

// Key metrics to monitor
// - OAuth2 authorization flow completion rate
// - JWT token validation latency
// - MCP operation success rates by scope
// - Authentication failure patterns
```

## 🚨 Common Issues and Solutions

### Issue 1: Empty Resources List

**Problem:** `resources/list` returns empty array
**Cause:** FileSystemResourceProvider serving empty directory
**Solution:** Create sample files during server startup (see Resource Population section)

### Issue 2: PKCE Verification Failed

**Problem:** Token exchange fails with "PKCE verification failed"
**Cause:** Using same string for code_challenge and code_verifier
**Solution:** Generate proper SHA256 hash for challenge (see PKCE section)

### Issue 3: OAuth2 Discovery Not Found

**Problem:** MCP Inspector can't find OAuth2 endpoints
**Cause:** Discovery endpoints not accessible from MCP server port
**Solution:** Use proxy architecture to route discovery requests

### Issue 4: Authorization Code Already Used

**Problem:** Token exchange fails with "code already used"
**Cause:** OAuth2 codes are single-use only
**Solution:** Generate fresh authorization code for each flow

## 📋 Best Practices

### Security
- ✅ Use PKCE S256 for all authorization flows
- ✅ Implement proper JWT audience validation
- ✅ Use reasonable token expiration times (10 min codes, 1 hour tokens)
- ✅ Validate all scopes against MCP method requirements
- ✅ Log all authentication events for audit trails

### Architecture
- ✅ Separate OAuth2 logic from MCP protocol implementation
- ✅ Use proxy patterns to bridge protocol incompatibilities
- ✅ Implement comprehensive request/response logging
- ✅ Design for horizontal scaling with shared OAuth2 provider

### Development
- ✅ Test with MCP Inspector for compatibility validation
- ✅ Create sample resources for immediate functionality testing
- ✅ Implement proper error handling with meaningful messages
- ✅ Use structured logging for debugging and monitoring

## 🔗 Additional Resources

- [OAuth2 MCP Inspector Integration Findings](../../../.copilot/memory_bank/sub_projects/airs-mcp/docs/knowledges/oauth2_mcp_inspector_integration_findings.md) - Comprehensive technical analysis
- [MCP Remote Server OAuth2 Example](../../../examples/mcp-remote-server-oauth2/) - Complete working implementation
- [MCP Inspector](https://github.com/modelcontextprotocol/inspector) - Official MCP testing tool
- [OAuth 2.1 Specification](https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/) - Complete OAuth2.1 reference

---

**This guide provides everything needed to implement production-ready OAuth2 authentication for MCP servers with full MCP Inspector compatibility and enterprise-grade security.**
