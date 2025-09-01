# KNOWLEDGE-005: Multi-Method Authentication Strategy

**Category**: Security  
**Created**: 2025-09-01  
**Updated**: 2025-09-01  
**Status**: Active  
**Priority**: High

## Overview

Comprehensive multi-method authentication strategy for MCP transport implementations, extending our existing OAuth2 `AuthContext` to support OAuth, API keys, and username/password combinations as specified in the official MCP documentation, while maintaining backward compatibility and framework flexibility.

## Multi-Method Authentication Architecture

### Integration Points

```text
┌─────────────────────────────────────────────────────────────┐
│                    Client Application                      │
│       (OAuth Token, API Key, Basic Auth Headers)          │
└─────────────────────┬───────────────────────────────────────┘
                      │ Multiple authentication methods
┌─────────────────────▼───────────────────────────────────────┐
│                 HTTP Framework                             │
│           (Axum, Rocket, Warp Middleware)                  │
└─────────────────────┬───────────────────────────────────────┘
                      │ Authentication middleware routing
┌─────────────────────▼───────────────────────────────────────┐
│           AuthenticationManager                            │
│        (Strategy pattern for multiple auth methods)        │
└─────────────────────┬───────────────────────────────────────┘
                      │ Unified AuthContext (extended existing)
┌─────────────────────▼───────────────────────────────────────┐
│                MCP Transport Layer                         │
│            (Enhanced MessageContext)                       │
└─────────────────────┬───────────────────────────────────────┘
                      │ Authorized MCP requests
┌─────────────────────▼───────────────────────────────────────┐
│                MCP Protocol Layer                          │
│          (McpServer with authorization checks)             │
└─────────────────────────────────────────────────────────────┘
```

## Extended AuthContext Design

### Current Implementation Analysis

Our existing `AuthContext` in `src/oauth2/context.rs` is well-designed for OAuth2 but needs evolution to support multiple authentication methods as required by the MCP specification:

**Current Structure (OAuth2-specific)**:
```rust
pub struct AuthContext {
    pub claims: JwtClaims,           // OAuth2/JWT specific
    pub scopes: Vec<String>,         // OAuth2 scopes  
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub request_id: Option<String>,
    pub metadata: AuthMetadata,      // Some extensibility
}
```

**MCP Specification Requirements**:
- OAuth (✅ already supported)
- API Keys (❌ not supported)
- Username/Password combinations (❌ not supported)
- Custom authentication schemes (❌ not supported)

```rust
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// JWKS endpoint for token validation
    pub jwks_url: Url,
    /// Expected audience in JWT tokens
    pub audience: String,
    /// Expected issuer in JWT tokens  
    pub issuer: String,
    /// Accepted algorithms for JWT verification
    pub algorithms: Vec<Algorithm>,
    /// Token validation settings
    pub validation: ValidationConfig,
    /// Cache settings for JWKS keys
    pub cache_config: JwksCacheConfig,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to validate token expiration
    pub validate_exp: bool,
    /// Whether to validate not-before time
    pub validate_nbf: bool,
    /// Clock skew tolerance for time validation
    pub leeway: Duration,
    /// Required claims that must be present
    pub required_claims: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct JwksCacheConfig {
    /// How long to cache JWKS keys
    pub cache_duration: Duration,
    /// How often to refresh keys proactively
    pub refresh_interval: Duration,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl OAuth2Config {
    pub fn builder() -> OAuth2ConfigBuilder {
        OAuth2ConfigBuilder::new()
    }
    
    /// Create config for common providers
    pub fn for_auth0(domain: &str, audience: &str) -> Result<Self, OAuth2Error> {
        let jwks_url = Url::parse(&format!("https://{}/.well-known/jwks.json", domain))?;
        Ok(Self {
            jwks_url,
            audience: audience.to_string(),
            issuer: format!("https://{}/", domain),
            algorithms: vec![Algorithm::RS256],
            validation: ValidationConfig::default(),
            cache_config: JwksCacheConfig::default(),
        })
    }
    
    pub fn for_keycloak(base_url: &str, realm: &str, audience: &str) -> Result<Self, OAuth2Error> {
        let jwks_url = Url::parse(&format!("{}/realms/{}/protocol/openid-connect/certs", base_url, realm))?;
        Ok(Self {
            jwks_url,
            audience: audience.to_string(),
            issuer: format!("{}/realms/{}", base_url, realm),
            algorithms: vec![Algorithm::RS256],
            validation: ValidationConfig::default(),
            cache_config: JwksCacheConfig::default(),
        })
    }
    
    pub fn for_google(audience: &str) -> Result<Self, OAuth2Error> {
        let jwks_url = Url::parse("https://www.googleapis.com/oauth2/v3/certs")?;
        Ok(Self {
            jwks_url,
            audience: audience.to_string(),
            issuer: "https://accounts.google.com".to_string(),
            algorithms: vec![Algorithm::RS256],
            validation: ValidationConfig::default(),
            cache_config: JwksCacheConfig::default(),
        })
    }
}
```

### Evolution Strategy: Extending Existing AuthContext

**Approach**: Evolve our existing `AuthContext` to support multiple authentication methods while maintaining 100% backward compatibility.

```rust
// src/oauth2/context.rs - Extended AuthContext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// User identifier (consistent across auth methods)
    pub user_id: String,
    
    /// Authentication method used
    pub auth_method: AuthMethod,
    
    /// User permissions/scopes (normalized across auth methods)
    pub scopes: Vec<String>,
    
    /// Authentication expiration (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Timestamp when this context was created
    pub created_at: DateTime<Utc>,
    
    /// Request ID for audit logging
    pub request_id: Option<String>,
    
    /// Additional context metadata (existing)
    pub metadata: AuthMetadata,
    
    /// Method-specific authentication data
    pub auth_data: AuthData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth2 { provider: String, token_type: String },
    ApiKey { key_type: String, scope: Option<String> },
    BasicAuth { realm: Option<String> },
    Custom { scheme: String, version: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthData {
    OAuth2(JwtClaims),                                    // Existing OAuth2 claims
    ApiKey { key_id: String, permissions: Vec<String> }, // API key authentication data
    BasicAuth { username: String, groups: Vec<String> }, // Username/password groups
    Custom(HashMap<String, serde_json::Value>),          // Extensible custom data
}
```

### Backward Compatibility Implementation

```rust
impl AuthContext {
    /// Create OAuth2 context (existing behavior - BACKWARD COMPATIBLE)
    pub fn from_oauth2(claims: JwtClaims, scopes: Vec<String>) -> Self {
        let user_id = claims.sub.clone();
        let expires_at = claims.exp.map(|exp| DateTime::from_timestamp(exp, 0).unwrap_or_else(Utc::now));
        
        Self {
            user_id,
            auth_method: AuthMethod::OAuth2 { 
                provider: claims.iss.clone().unwrap_or_default(),
                token_type: "Bearer".to_string() 
            },
            scopes,
            expires_at,
            created_at: Utc::now(),
            request_id: None,
            metadata: AuthMetadata::default(),
            auth_data: AuthData::OAuth2(claims),
        }
    }
    
    /// Create API key context (NEW)
    pub fn from_api_key(key_id: String, permissions: Vec<String>, key_type: Option<String>) -> Self {
        Self {
            user_id: format!("apikey:{}", key_id),
            auth_method: AuthMethod::ApiKey { 
                key_type: key_type.unwrap_or_else(|| "Bearer".to_string()), 
                scope: None 
            },
            scopes: permissions.clone(),
            expires_at: None, // API keys typically don't expire
            created_at: Utc::now(),
            request_id: None,
            metadata: AuthMetadata::default(),
            auth_data: AuthData::ApiKey { key_id, permissions },
        }
    }
    
    /// Create basic auth context (NEW)
    pub fn from_basic_auth(username: String, groups: Vec<String>, realm: Option<String>) -> Self {
        Self {
            user_id: username.clone(),
            auth_method: AuthMethod::BasicAuth { realm },
            scopes: groups.clone(),
            expires_at: None, // Basic auth sessions managed by session timeout
            created_at: Utc::now(),
            request_id: None,
            metadata: AuthMetadata::default(),
            auth_data: AuthData::BasicAuth { username, groups },
        }
    }
    
    /// Create custom auth context (NEW)
    pub fn from_custom(user_id: String, scheme: String, scopes: Vec<String>, 
                       custom_data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            user_id,
            auth_method: AuthMethod::Custom { scheme, version: None },
            scopes,
            expires_at: None,
            created_at: Utc::now(),
            request_id: None,
            metadata: AuthMetadata::default(),
            auth_data: AuthData::Custom(custom_data),
        }
    }
    
    // BACKWARD COMPATIBILITY: Keep existing OAuth2-specific methods
    pub fn jwt_claims(&self) -> Option<&JwtClaims> {
        match &self.auth_data {
            AuthData::OAuth2(claims) => Some(claims),
            _ => None,
        }
    }
    
    // Existing methods continue to work (user_id, has_scope, etc.)
    // All existing OAuth2-specific functionality preserved
}

## Authentication Strategy Pattern

### Strategy Interface

```rust
#[async_trait]
pub trait AuthenticationStrategy: Send + Sync {
    /// Authenticate a request and return AuthContext
    async fn authenticate(&self, request: &AuthenticationRequest) -> Result<AuthContext, AuthError>;
    
    /// Extract authentication data from HTTP request
    fn extract_from_request(&self, headers: &HeaderMap, body: Option<&[u8]>) -> Result<AuthenticationRequest, AuthError>;
    
    /// Strategy identifier
    fn strategy_name(&self) -> &'static str;
    
    /// Check if strategy supports token renewal
    fn supports_renewal(&self) -> bool { false }
    
    /// Renew authentication if supported
    async fn renew(&self, context: &AuthContext) -> Result<AuthContext, AuthError> {
        Err(AuthError::RenewalNotSupported)
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticationRequest {
    pub method: String,           // "Bearer", "Basic", "ApiKey", etc.
    pub credentials: String,      // Token, encoded credentials, API key
    pub metadata: HashMap<String, String>, // Additional request metadata
}
```

### Strategy Implementations

#### OAuth2 Strategy (Existing - Enhanced)
```rust
pub struct OAuth2AuthStrategy {
    config: OAuth2Config,
    jwks_client: Arc<JwksClient>,
    validation: Validation,
}

#[async_trait]
impl AuthenticationStrategy for OAuth2AuthStrategy {
    async fn authenticate(&self, request: &AuthenticationRequest) -> Result<AuthContext, AuthError> {
        // Extract JWT token from credentials
        let token = &request.credentials;
        
        // Existing OAuth2 validation logic (reuse current implementation)
        let header = decode_header(token).map_err(AuthError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthError::MissingKeyId)?;
        
        let jwk = self.jwks_client.get(&kid).await.map_err(AuthError::JwksError)?;
        let key = DecodingKey::from_jwk(&jwk).map_err(AuthError::InvalidKey)?;
        
        let token_data = decode::<JwtClaims>(token, &key, &self.validation)
            .map_err(AuthError::InvalidToken)?;
        
        // Convert to extended AuthContext
        let scopes = self.extract_scopes(&token_data.claims)?;
        Ok(AuthContext::from_oauth2(token_data.claims, scopes))
    }
    
    fn extract_from_request(&self, headers: &HeaderMap, _body: Option<&[u8]>) -> Result<AuthenticationRequest, AuthError> {
        let auth_header = headers.get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingAuthorizationHeader)?;
        
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidAuthorizationHeader);
        }
        
        Ok(AuthenticationRequest {
            method: "Bearer".to_string(),
            credentials: auth_header[7..].to_string(),
            metadata: HashMap::new(),
        })
    }
    
    fn strategy_name(&self) -> &'static str { "oauth2" }
    fn supports_renewal(&self) -> bool { true } // If refresh tokens supported
}
```

#### API Key Strategy (New)
```rust
pub struct ApiKeyAuthStrategy {
    key_store: Arc<dyn ApiKeyStore>,
    header_name: String, // "X-API-Key", "Authorization", etc.
    key_prefix: Option<String>, // "Bearer ", "ApiKey ", etc.
}

#[async_trait]
impl AuthenticationStrategy for ApiKeyAuthStrategy {
    async fn authenticate(&self, request: &AuthenticationRequest) -> Result<AuthContext, AuthError> {
        let api_key = &request.credentials;
        
        // Validate API key against key store
        let key_info = self.key_store.validate_key(api_key).await
            .map_err(|_| AuthError::InvalidApiKey)?;
        
        // Convert to AuthContext
        Ok(AuthContext::from_api_key(
            key_info.key_id,
            key_info.permissions,
            Some(request.method.clone())
        ))
    }
    
    fn extract_from_request(&self, headers: &HeaderMap, _body: Option<&[u8]>) -> Result<AuthenticationRequest, AuthError> {
        let header_value = headers.get(&self.header_name)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingApiKey)?;
        
        let credentials = if let Some(prefix) = &self.key_prefix {
            if !header_value.starts_with(prefix) {
                return Err(AuthError::InvalidApiKeyFormat);
            }
            header_value[prefix.len()..].to_string()
        } else {
            header_value.to_string()
        };
        
        Ok(AuthenticationRequest {
            method: self.key_prefix.clone().unwrap_or_else(|| "ApiKey".to_string()),
            credentials,
            metadata: HashMap::new(),
        })
    }
    
    fn strategy_name(&self) -> &'static str { "api_key" }
}

#[async_trait]
pub trait ApiKeyStore: Send + Sync {
    async fn validate_key(&self, key: &str) -> Result<ApiKeyInfo, ApiKeyError>;
}

pub struct ApiKeyInfo {
    pub key_id: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<RateLimit>,
}
```

#### Basic Auth Strategy (New)
```rust
pub struct BasicAuthStrategy {
    user_store: Arc<dyn UserStore>,
    realm: Option<String>,
    password_hasher: Arc<dyn PasswordHasher>,
}

#[async_trait]
impl AuthenticationStrategy for BasicAuthStrategy {
    async fn authenticate(&self, request: &AuthenticationRequest) -> Result<AuthContext, AuthError> {
        // Decode base64 credentials
        let decoded = base64::decode(&request.credentials)
            .map_err(|_| AuthError::InvalidBasicAuth)?;
        let credentials = String::from_utf8(decoded)
            .map_err(|_| AuthError::InvalidBasicAuth)?;
        
        let mut parts = credentials.splitn(2, ':');
        let username = parts.next().ok_or(AuthError::InvalidBasicAuth)?;
        let password = parts.next().ok_or(AuthError::InvalidBasicAuth)?;
        
        // Validate against user store
        let user_info = self.user_store.validate_user(username, password).await
            .map_err(|_| AuthError::InvalidCredentials)?;
        
        // Convert to AuthContext
        Ok(AuthContext::from_basic_auth(
            username.to_string(),
            user_info.groups,
            self.realm.clone()
        ))
    }
    
    fn extract_from_request(&self, headers: &HeaderMap, _body: Option<&[u8]>) -> Result<AuthenticationRequest, AuthError> {
        let auth_header = headers.get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingAuthorizationHeader)?;
        
        if !auth_header.starts_with("Basic ") {
            return Err(AuthError::InvalidAuthorizationHeader);
        }
        
        Ok(AuthenticationRequest {
            method: "Basic".to_string(),
            credentials: auth_header[6..].to_string(),
            metadata: HashMap::new(),
        })
    }
    
    fn strategy_name(&self) -> &'static str { "basic_auth" }
}

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn validate_user(&self, username: &str, password: &str) -> Result<UserInfo, UserStoreError>;
}

pub struct UserInfo {
    pub groups: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

### Authentication Manager

```rust
pub struct AuthenticationManager {
    strategies: HashMap<String, Box<dyn AuthenticationStrategy>>,
    strategy_order: Vec<String>, // Try strategies in this order
    default_strategy: Option<String>,
}

impl AuthenticationManager {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            strategy_order: Vec::new(),
            default_strategy: None,
        }
    }
    
    pub fn add_strategy<S>(&mut self, name: String, strategy: S) 
    where S: AuthenticationStrategy + 'static 
    {
        self.strategies.insert(name.clone(), Box::new(strategy));
        self.strategy_order.push(name);
    }
    
    pub fn set_default_strategy(&mut self, name: String) {
        self.default_strategy = Some(name);
    }
    
    pub fn set_strategy_order(&mut self, order: Vec<String>) {
        self.strategy_order = order;
    }
    
    pub async fn authenticate(&self, headers: &HeaderMap, body: Option<&[u8]>) -> Result<AuthContext, AuthError> {
        // Try strategies in order until one succeeds
        for strategy_name in &self.strategy_order {
            if let Some(strategy) = self.strategies.get(strategy_name) {
                if let Ok(request) = strategy.extract_from_request(headers, body) {
                    if let Ok(context) = strategy.authenticate(&request).await {
                        return Ok(context);
                    }
                }
                // Continue to next strategy if this one fails
            }
        }
        
        Err(AuthError::NoValidAuthentication)
    }
}

impl AuthContext {
    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
    
    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }
    
    /// Check if token is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
    
    /// Get custom claim value
    pub fn get_claim<T>(&self, claim_name: &str) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.claims.get(claim_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
    
    /// Extract from JWT claims
    pub fn from_jwt_claims(claims: &JwtClaims) -> Result<Self, OAuth2Error> {
        let user_id = claims.sub.clone()
            .ok_or(OAuth2Error::MissingClaim("sub".to_string()))?;
        
        let email = claims.email.clone();
        
        // Extract roles from various common claim formats
        let roles = Self::extract_roles(claims)?;
        
        // Convert all custom claims
        let mut custom_claims = HashMap::new();
        for (key, value) in &claims.custom {
            custom_claims.insert(key.clone(), value.clone());
        }
        
        Ok(Self {
            user_id,
            email,
            roles,
            claims: custom_claims,
            expires_at: DateTime::from_timestamp(claims.exp, 0)
                .unwrap_or_else(|| Utc::now()),
            issued_at: DateTime::from_timestamp(claims.iat.unwrap_or(0), 0)
                .unwrap_or_else(|| Utc::now()),
            metadata: HashMap::new(),
        })
    }
    
    fn extract_roles(claims: &JwtClaims) -> Result<Vec<String>, OAuth2Error> {
        // Try common role claim formats
        if let Some(roles) = claims.custom.get("roles") {
            return Ok(serde_json::from_value(roles.clone()).unwrap_or_default());
        }
        
        if let Some(groups) = claims.custom.get("groups") {
            return Ok(serde_json::from_value(groups.clone()).unwrap_or_default());
        }
        
        if let Some(permissions) = claims.custom.get("permissions") {
            return Ok(serde_json::from_value(permissions.clone()).unwrap_or_default());
        }
        
        // Check realm access for Keycloak
        if let Some(realm_access) = claims.custom.get("realm_access") {
            if let Some(roles) = realm_access.get("roles") {
                return Ok(serde_json::from_value(roles.clone()).unwrap_or_default());
            }
        }
        
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: Option<String>,
    /// Audience
    pub aud: Option<String>,
    /// Issuer
    pub iss: Option<String>,
    /// Expiration time
    pub exp: i64,
    /// Issued at time
    pub iat: Option<i64>,
    /// Not before time
    pub nbf: Option<i64>,
    /// Email
    pub email: Option<String>,
    /// Email verified
    pub email_verified: Option<bool>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}
```

### Framework-Agnostic OAuth2 Core

```rust
pub struct OAuth2MiddlewareCore {
    config: OAuth2Config,
    jwks_client: Arc<JwksClient>,
    validation: Validation,
}

impl OAuth2MiddlewareCore {
    pub fn new(config: OAuth2Config) -> Self {
        let mut validation = Validation::new(config.algorithms[0]);
        validation.set_audience(&[config.audience.clone()]);
        validation.set_issuer(&[config.issuer.clone()]);
        validation.validate_exp = config.validation.validate_exp;
        validation.validate_nbf = config.validation.validate_nbf;
        validation.leeway = config.validation.leeway.as_secs();
        
        let jwks_client = Arc::new(JwksClient::builder()
            .cache_duration(config.cache_config.cache_duration)
            .build(config.jwks_url.clone())
            .expect("Failed to create JWKS client"));
        
        Self {
            config,
            jwks_client,
            validation,
        }
    }
    
    /// Validate JWT token and extract AuthContext
    pub async fn validate_token(&self, token: &str) -> Result<AuthContext, OAuth2Error> {
        // Decode header to get key ID
        let header = decode_header(token)
            .map_err(OAuth2Error::InvalidToken)?;
        
        let kid = header.kid.ok_or(OAuth2Error::MissingKeyId)?;
        
        // Get public key from JWKS
        let jwk = self.jwks_client.get(&kid).await
            .map_err(OAuth2Error::JwksError)?;
        
        let key = DecodingKey::from_jwk(&jwk)
            .map_err(OAuth2Error::InvalidKey)?;
        
        // Validate and decode token
        let token_data = decode::<JwtClaims>(token, &key, &self.validation)
            .map_err(OAuth2Error::InvalidToken)?;
        
        // Convert to AuthContext
        AuthContext::from_jwt_claims(&token_data.claims)
    }
    
    /// Extract token from Authorization header
    pub fn extract_bearer_token(auth_header: &str) -> Result<&str, OAuth2Error> {
        if !auth_header.starts_with("Bearer ") {
            return Err(OAuth2Error::InvalidAuthorizationHeader);
        }
        
        Ok(&auth_header[7..])
    }
}

// Framework-specific middleware implementations
impl OAuth2MiddlewareCore {
    /// Convert to Axum middleware layer
    pub fn into_axum_layer(self) -> axum::middleware::AxumMiddleware {
        axum::middleware::from_fn_with_state(
            Arc::new(self),
            |State(oauth): State<Arc<OAuth2MiddlewareCore>>, 
             mut req: Request<Body>, 
             next: Next<Body>| async move {
                // Extract authorization header
                let auth_header = req.headers()
                    .get(AUTHORIZATION)
                    .and_then(|h| h.to_str().ok())
                    .ok_or_else(|| {
                        (StatusCode::UNAUTHORIZED, "Missing authorization header").into_response()
                    })?;
                
                // Extract and validate token
                let token = OAuth2MiddlewareCore::extract_bearer_token(auth_header)
                    .map_err(|_| {
                        (StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response()
                    })?;
                
                let auth_context = oauth.validate_token(token).await
                    .map_err(|_| {
                        (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
                    })?;
                
                // Add AuthContext to request extensions
                req.extensions_mut().insert(auth_context);
                
                Ok(next.run(req).await)
            }
        )
    }
    
    /// Convert to Rocket fairing
    pub fn into_rocket_fairing(self) -> impl rocket::fairing::Fairing {
        OAuth2RocketFairing::new(self)
    }
    
    /// Convert to Warp filter
    pub fn into_warp_filter(self) -> impl warp::Filter<Extract = (AuthContext,), Error = warp::Rejection> + Clone {
        warp::header::<String>("authorization")
            .and_then(move |auth_header: String| {
                let oauth = self.clone();
                async move {
                    let token = OAuth2MiddlewareCore::extract_bearer_token(&auth_header)
                        .map_err(|_| warp::reject::custom(OAuth2Error::InvalidAuthorizationHeader))?;
                    
                    let auth_context = oauth.validate_token(token).await
                        .map_err(|e| warp::reject::custom(e))?;
                    
                    Ok::<AuthContext, warp::Rejection>(auth_context)
                }
            })
    }
}

#[derive(Debug)]
pub struct OAuth2RocketFairing {
    oauth: OAuth2MiddlewareCore,
}

impl OAuth2RocketFairing {
    pub fn new(oauth: OAuth2MiddlewareCore) -> Self {
        Self { oauth }
    }
}

#[rocket::async_trait]
impl rocket::fairing::Fairing for OAuth2RocketFairing {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "OAuth2 Authentication",
            kind: rocket::fairing::Kind::Request,
        }
    }
    
    async fn on_request(&self, req: &mut rocket::Request<'_>, _: &mut rocket::Data<'_>) {
        if let Some(auth_header) = req.headers().get_one("authorization") {
            if let Ok(token) = OAuth2MiddlewareCore::extract_bearer_token(auth_header) {
                if let Ok(auth_context) = self.oauth.validate_token(token).await {
                    req.local_cache(|| Some(auth_context));
                    return;
                }
            }
        }
        
        req.local_cache(|| Option::<AuthContext>::None);
    }
}

// Rocket request guard for AuthContext
#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthContext {
    type Error = OAuth2Error;
    
    async fn from_request(req: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        if let Some(auth_context) = req.local_cache(|| Option::<AuthContext>::None) {
            rocket::request::Outcome::Success(auth_context.clone())
        } else {
            rocket::request::Outcome::Failure((
                rocket::http::Status::Unauthorized, 
                OAuth2Error::MissingAuthContext
            ))
        }
    }
}
```

## Enhanced MessageContext Integration

### Protocol-Level Authorization

```rust
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Session identifier
    pub session_id: String,
    /// Request identifier for correlation
    pub request_id: String,
    /// Timestamp when message was received
    pub timestamp: DateTime<Utc>,
    /// Authentication context from OAuth2
    pub auth_context: Option<AuthContext>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

impl MessageContext {
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_context.as_ref()
            .map(|auth| auth.is_valid())
            .unwrap_or(false)
    }
    
    /// Get user ID if authenticated
    pub fn user_id(&self) -> Option<&str> {
        self.auth_context.as_ref().map(|auth| auth.user_id.as_str())
    }
    
    /// Check if user has required role
    pub fn has_role(&self, role: &str) -> bool {
        self.auth_context.as_ref()
            .map(|auth| auth.has_role(role))
            .unwrap_or(false)
    }
    
    /// Check if user has any of the required roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.auth_context.as_ref()
            .map(|auth| auth.has_any_role(roles))
            .unwrap_or(false)
    }
    
    /// Require authentication (returns error if not authenticated)
    pub fn require_auth(&self) -> Result<&AuthContext, McpError> {
        self.auth_context.as_ref()
            .filter(|auth| auth.is_valid())
            .ok_or(McpError::AuthenticationRequired)
    }
    
    /// Require specific role (returns error if role not present)
    pub fn require_role(&self, role: &str) -> Result<&AuthContext, McpError> {
        let auth = self.require_auth()?;
        if auth.has_role(role) {
            Ok(auth)
        } else {
            Err(McpError::InsufficientPermissions)
        }
    }
    
    /// Require any of the specified roles
    pub fn require_any_role(&self, roles: &[&str]) -> Result<&AuthContext, McpError> {
        let auth = self.require_auth()?;
        if auth.has_any_role(roles) {
            Ok(auth)
        } else {
            Err(McpError::InsufficientPermissions)
        }
    }
}
```

### Authorization-Aware Tool Implementations

```rust
#[async_trait]
impl Tool for FileReadTool {
    async fn execute(&self, params: Value, context: MessageContext) -> Result<Value, McpError> {
        // Require authentication for file operations
        let auth = context.require_auth()?;
        
        // Require 'file:read' permission
        context.require_role("file:read")?;
        
        // Extract file path from params
        let file_path: String = serde_json::from_value(
            params.get("path").cloned().unwrap_or_default()
        )?;
        
        // Additional authorization checks based on file path
        if file_path.starts_with("/etc") || file_path.starts_with("/root") {
            context.require_role("file:read:system")?;
        }
        
        // Log access attempt
        tracing::info!(
            user_id = auth.user_id,
            file_path = file_path,
            "File read access requested"
        );
        
        // Perform file read operation
        let content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| McpError::ToolExecutionError(format!("Failed to read file: {}", e)))?;
        
        Ok(serde_json::json!({
            "content": content,
            "path": file_path,
            "accessed_by": auth.user_id,
            "accessed_at": Utc::now().to_rfc3339()
        }))
    }
}

#[async_trait]
impl Tool for DatabaseQueryTool {
    async fn execute(&self, params: Value, context: MessageContext) -> Result<Value, McpError> {
        // Require authentication
        let auth = context.require_auth()?;
        
        // Extract query from params
        let query: String = serde_json::from_value(
            params.get("query").cloned().unwrap_or_default()
        )?;
        
        // Role-based query restrictions
        if query.to_lowercase().contains("delete") || query.to_lowercase().contains("drop") {
            context.require_role("database:write")?;
        } else if query.to_lowercase().contains("insert") || query.to_lowercase().contains("update") {
            context.require_role("database:modify")?;
        } else {
            context.require_role("database:read")?;
        }
        
        // User-specific data filtering
        let filtered_query = if auth.has_role("admin") {
            query // Admins can access all data
        } else {
            // Non-admins can only access their own data
            format!("{} WHERE user_id = '{}'", query, auth.user_id)
        };
        
        // Execute query with appropriate restrictions
        let results = self.execute_query(&filtered_query).await?;
        
        Ok(serde_json::json!({
            "results": results,
            "query": filtered_query,
            "executed_by": auth.user_id,
            "executed_at": Utc::now().to_rfc3339()
        }))
    }
}
```

## Configuration Examples

### Provider-Specific Configurations

```rust
// Auth0 configuration
let auth0_config = OAuth2Config::for_auth0(
    "your-domain.auth0.com",
    "your-mcp-server-audience"
)?;

// Keycloak configuration
let keycloak_config = OAuth2Config::for_keycloak(
    "https://auth.example.com",
    "mcp-realm", 
    "mcp-server"
)?;

// Google OAuth2 configuration
let google_config = OAuth2Config::for_google(
    "your-google-client-id.googleusercontent.com"
)?;

// Custom provider configuration
let custom_config = OAuth2Config::builder()
    .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json")?)
    .audience("mcp-server".to_string())
    .issuer("https://auth.example.com".to_string())
    .algorithms(vec![Algorithm::RS256, Algorithm::ES256])
    .validation(ValidationConfig {
        validate_exp: true,
        validate_nbf: true,
        leeway: Duration::from_secs(30),
        required_claims: vec!["email".to_string(), "roles".to_string()],
    })
    .cache_config(JwksCacheConfig {
        cache_duration: Duration::from_secs(3600),
        refresh_interval: Duration::from_secs(300),
        max_cache_size: 100,
    })
    .build()?;
```

### Framework Integration Examples

```rust
// Axum with OAuth2
let axum_config = AxumEngineConfig::default();
let oauth_config = OAuth2Config::for_auth0("domain.auth0.com", "audience")?;

let transport = HttpServerTransport::with_axum_and_oauth(
    axum_config,
    oauth_config,
    session_manager,
)?;

// Rocket with OAuth2
let rocket_config = RocketEngineConfig::default();
let oauth_config = OAuth2Config::for_keycloak(
    "https://keycloak.example.com",
    "mcp-realm",
    "mcp-server"
)?;

let transport = HttpServerTransport::with_rocket_and_oauth(
    rocket_config, 
    oauth_config,
    session_manager,
)?;

// Warp with OAuth2
let warp_config = WarpEngineConfig::default();
let oauth_config = OAuth2Config::for_google("client-id.googleusercontent.com")?;

let transport = HttpServerTransport::with_warp_and_oauth(
    warp_config,
    oauth_config, 
    session_manager,
)?;
```

## Security Considerations

### Token Validation
- **Algorithm Whitelisting**: Only accept specified JWT algorithms (RS256, ES256, etc.)
- **JWKS Caching**: Cache public keys with appropriate TTL and refresh logic
- **Clock Skew**: Handle reasonable clock differences between issuer and validator
- **Revocation**: Consider token revocation mechanisms for enhanced security

### Authorization Patterns
- **Role-Based Access Control (RBAC)**: Use roles for coarse-grained permissions
- **Attribute-Based Access Control (ABAC)**: Use claims for fine-grained permissions
- **Resource-Level Security**: Filter data based on user context and ownership
- **Audit Logging**: Log all authorization decisions for security monitoring

### Error Handling
- **Token Errors**: Return generic unauthorized responses to avoid information leakage
- **Authorization Failures**: Log detailed errors for debugging while returning generic responses
- **Rate Limiting**: Implement rate limiting on authentication endpoints
- **Attack Protection**: Protect against common JWT attacks (algorithm confusion, etc.)

## Performance Optimizations

### JWKS Caching Strategy
- **Proactive Refresh**: Refresh keys before expiration to avoid cache misses
- **Multiple Key Support**: Cache multiple keys for key rotation scenarios
- **Error Fallback**: Fallback to fresh key fetch if cached key validation fails
- **Memory Management**: Limit cache size and implement LRU eviction

### Request Processing
- **Token Parsing**: Parse tokens once and pass AuthContext through request pipeline
- **Connection Reuse**: Reuse HTTP connections for JWKS fetches
- **Async Validation**: Validate tokens asynchronously to avoid blocking request processing
- **Batch Operations**: Batch authorization checks where possible

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_valid_token_validation() {
        let config = OAuth2Config::for_testing();
        let middleware = OAuth2MiddlewareCore::new(config);
        
        let token = create_test_jwt_token();
        let auth_context = middleware.validate_token(&token).await.unwrap();
        
        assert_eq!(auth_context.user_id, "test-user");
        assert!(auth_context.has_role("user"));
    }
    
    #[tokio::test]
    async fn test_expired_token_rejection() {
        let config = OAuth2Config::for_testing();
        let middleware = OAuth2MiddlewareCore::new(config);
        
        let expired_token = create_expired_jwt_token();
        let result = middleware.validate_token(&expired_token).await;
        
        assert!(matches!(result, Err(OAuth2Error::InvalidToken(_))));
    }
    
    #[tokio::test]
    async fn test_role_based_authorization() {
        let auth_context = AuthContext {
            user_id: "test-user".to_string(),
            roles: vec!["user".to_string(), "file:read".to_string()],
            // ... other fields
        };
        
        let context = MessageContext {
            auth_context: Some(auth_context),
            // ... other fields
        };
        
        assert!(context.has_role("user"));
        assert!(context.has_role("file:read"));
        assert!(!context.has_role("admin"));
        
        assert!(context.require_role("user").is_ok());
        assert!(context.require_role("admin").is_err());
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_end_to_end_oauth_flow() {
    // Setup test OAuth2 provider (e.g., with wiremock)
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/.well-known/jwks.json"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(test_jwks()))
        .mount(&mock_server)
        .await;
    
    // Configure OAuth2 with mock server
    let config = OAuth2Config::builder()
        .jwks_url(Url::parse(&format!("{}{}", mock_server.uri(), "/.well-known/jwks.json")).unwrap())
        .audience("test-audience".to_string())
        .issuer(mock_server.uri())
        .build()
        .unwrap();
    
    // Create HTTP transport with OAuth2
    let transport = HttpServerTransport::with_axum_and_oauth(
        AxumEngineConfig::default(),
        config,
        session_manager,
    ).unwrap();
    
    // Test authenticated request
    let token = create_test_jwt_token();
    let response = reqwest::Client::new()
        .post("http://localhost:3000/mcp")
        .header("Authorization", format!("Bearer {}", token))
        .json(&test_mcp_request())
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Test unauthenticated request
    let response = reqwest::Client::new()
        .post("http://localhost:3000/mcp")
        .json(&test_mcp_request())
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
}
```

## Migration Strategy

### Existing System Integration
1. **Gradual Rollout**: Enable OAuth2 on non-critical endpoints first
2. **Fallback Authentication**: Support both OAuth2 and existing auth during transition
3. **User Migration**: Provide tools for users to link existing accounts with OAuth2 providers
4. **Monitoring**: Monitor authentication success rates and performance impact

### Configuration Management
1. **Environment-Based Config**: Use different OAuth2 configs for dev/staging/prod
2. **Runtime Configuration**: Support OAuth2 config changes without service restart
3. **Provider Failover**: Support multiple OAuth2 providers for high availability
4. **Graceful Degradation**: Handle OAuth2 provider outages gracefully

## Related Documentation
- [ADR-001: MCP-Compliant Transport Redesign](../adr/ADR-001-mcp-compliant-transport-redesign.md)
- [KNOWLEDGE-004: HTTP Engine Abstraction Architecture](./KNOWLEDGE-004-http-engine-abstraction-architecture.md)
- [OAuth2 Implementation Guide](../../oauth2/README.md)
