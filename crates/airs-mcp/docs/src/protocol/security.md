# Security Model & Trust Boundaries

## Authentication Architecture

Transport-Specific Authentication:

```rust,ignore
// STDIO Transport: Environment-based credentials
pub struct StdioAuth {
    environment_vars: HashMap<String, String>,
    process_isolation: bool,
}

// HTTP Transport: OAuth 2.1 + PKCE
pub struct HttpAuth {
    oauth_config: OAuth21Config,
    pkce_verifier: PkceCodeVerifier,
    token_storage: SecureTokenStorage,
}
```

OAuth 2.1 + PKCE Implementation:

```rust,ignore
#[derive(Debug, Clone)]
pub struct OAuth21Config {
    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    pub client_id: String,
    pub redirect_uri: Url,
    pub scopes: Vec<String>,
    pub code_challenge_method: CodeChallengeMethod, // S256 only
}

#[async_trait]
pub trait OAuth21Authenticator {
    async fn authenticate(&self, config: OAuth21Config) -> Result<AccessToken, AuthError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AccessToken, AuthError>;
    async fn revoke_token(&self, token: &str) -> Result<(), AuthError>;
}
```

## Authorization Framework

Capability-Based Access Control:

```rust,ignore
#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub can_list_resources: bool,
    pub can_read_resources: Vec<String>, // URI patterns
    pub can_execute_tools: Vec<String>,  // Tool names
    pub can_access_prompts: Vec<String>, // Prompt names
    pub can_request_sampling: bool,
}

impl Connection {
    pub fn check_permission(&self, operation: &Operation) -> Result<(), AuthError> {
        match operation {
            Operation::ReadResource(uri) => {
                if self.permissions.can_read_resource(uri) {
                    Ok(())
                } else {
                    Err(AuthError::InsufficientPermissions)
                }
            }
            // ... other permission checks
        }
    }
}
```

## Audit & Compliance Framework

```rust,ignore
#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub connection_id: String,
    pub operation: String,
    pub user_id: Option<String>,
    pub resource_accessed: Option<String>,
    pub tool_executed: Option<String>,
    pub approval_required: bool,
    pub approval_granted: Option<bool>,
    pub result: AuditResult,
}

#[async_trait]
pub trait AuditLogger {
    async fn log_event(&self, event: AuditEvent) -> Result<(), AuditError>;
    async fn query_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError>;
}
```
