//! OAuth2 Authorization Flow Implementation
//!
//! This module implements the complete OAuth2 authorization code flow with PKCE support
//! for MCP Inspector compatibility. It provides authorization code management,
//! /authorize and /token endpoints, and OAuth2 discovery metadata.

// Layer 1: Standard library imports
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

// Layer 2: Third-party crate imports
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect, Response},
    Form,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::tokens::{generate_test_token, TestKeys, TokenConfig};

/// OAuth2 authorization request parameters (RFC 6749)
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    /// Response type - must be "code" for authorization code flow
    pub response_type: String,
    /// Client identifier
    pub client_id: String,
    /// Redirection URI where authorization code will be sent
    pub redirect_uri: String,
    /// Space-delimited scope values (optional)
    pub scope: Option<String>,
    /// Opaque value used to maintain state between request and callback (recommended)
    pub state: Option<String>,
    /// PKCE code challenge (RFC 7636)
    pub code_challenge: String,
    /// PKCE code challenge method - "S256" or "plain"
    pub code_challenge_method: Option<String>,
}

/// OAuth2 token request parameters (RFC 6749)
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// Grant type - must be "authorization_code"
    pub grant_type: String,
    /// Authorization code from /authorize endpoint
    pub code: String,
    /// Redirection URI (must match the one from /authorize)
    pub redirect_uri: String,
    /// Client identifier (must match the one from /authorize)
    pub client_id: String,
    /// PKCE code verifier (RFC 7636)
    pub code_verifier: String,
}

/// OAuth2 token response (RFC 6749)
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Access token for API access
    pub access_token: String,
    /// Token type - always "Bearer"
    pub token_type: String,
    /// Token lifetime in seconds
    pub expires_in: u64,
    /// Space-delimited list of granted scopes
    pub scope: String,
    /// JWT algorithm used for token signing
    pub algorithm: String,
    /// Key ID used for token verification
    pub key_id: String,
}

/// OAuth2 error response (RFC 6749)
#[derive(Debug, Serialize)]
pub struct OAuth2Error {
    /// Error code
    pub error: String,
    /// Human-readable error description
    pub error_description: String,
    /// URI for error documentation (optional)
    pub error_uri: Option<String>,
}

/// Authorization code with metadata and expiration
#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    /// Unique authorization code
    pub code: String,
    /// Client ID that requested the code
    pub client_id: String,
    /// Redirect URI for callback
    pub redirect_uri: String,
    /// Requested scope
    pub scope: String,
    /// PKCE code challenge
    pub code_challenge: String,
    /// PKCE code challenge method
    pub code_challenge_method: String,
    /// Expiration timestamp
    pub expires_at: SystemTime,
    /// Whether the code has been used (single-use only)
    pub used: bool,
    /// State parameter for CSRF protection
    pub state: Option<String>,
}

impl AuthorizationCode {
    /// Create a new authorization code with 10-minute expiration
    pub fn new(
        client_id: String,
        redirect_uri: String,
        scope: String,
        code_challenge: String,
        code_challenge_method: String,
        state: Option<String>,
    ) -> Self {
        let code = format!("auth_{}", Uuid::new_v4().to_string().replace('-', ""));
        let expires_at = SystemTime::now() + Duration::from_secs(600); // 10 minutes

        Self {
            code,
            client_id,
            redirect_uri,
            scope,
            code_challenge,
            code_challenge_method,
            expires_at,
            used: false,
            state,
        }
    }

    /// Check if the authorization code is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Mark the authorization code as used
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

/// Thread-safe authorization code storage
pub type AuthCodeStorage = Arc<Mutex<HashMap<String, AuthorizationCode>>>;

/// OAuth2 authorization flow state
#[derive(Clone)]
pub struct OAuth2FlowState {
    /// Authorization code storage
    pub auth_codes: AuthCodeStorage,
    /// Test keys for JWT signing
    pub test_keys: TestKeys,
    /// Available token configurations
    pub token_configs: HashMap<String, TokenConfig>,
}

impl OAuth2FlowState {
    /// Create new OAuth2 flow state
    pub fn new(test_keys: TestKeys) -> Self {
        Self {
            auth_codes: Arc::new(Mutex::new(HashMap::new())),
            test_keys,
            token_configs: TokenConfig::all_configs(),
        }
    }

    /// Store authorization code
    pub fn store_auth_code(&self, auth_code: AuthorizationCode) -> String {
        let code = auth_code.code.clone();
        let mut codes = self.auth_codes.lock().unwrap();
        codes.insert(code.clone(), auth_code);

        // Clean up expired codes while we have the lock
        let now = SystemTime::now();
        codes.retain(|_, code| now <= code.expires_at);

        code
    }

    /// Retrieve and consume authorization code
    pub fn consume_auth_code(&self, code: &str) -> Option<AuthorizationCode> {
        let mut codes = self.auth_codes.lock().unwrap();

        // Clean up expired codes first
        let now = SystemTime::now();
        codes.retain(|_, code| now <= code.expires_at);

        // Get the authorization code
        if let Some(mut auth_code) = codes.remove(code) {
            if auth_code.is_expired() || auth_code.used {
                None
            } else {
                auth_code.mark_used();
                Some(auth_code)
            }
        } else {
            None
        }
    }
}

/// Verify PKCE challenge according to RFC 7636
pub fn verify_pkce_challenge(code_verifier: &str, code_challenge: &str, method: &str) -> bool {
    match method {
        "S256" => {
            // Create SHA256 hash of code_verifier
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let digest = hasher.finalize();

            // Base64url encode the hash
            let calculated_challenge = general_purpose::URL_SAFE_NO_PAD.encode(digest);

            // Compare with provided challenge
            calculated_challenge == code_challenge
        }
        "plain" => {
            // Plain method: code_verifier should equal code_challenge
            code_verifier == code_challenge
        }
        _ => {
            error!(method = %method, "Unsupported PKCE challenge method");
            false
        }
    }
}

/// OAuth2 authorization endpoint handler
/// Implements authorization code flow with PKCE (RFC 6749 + RFC 7636)
pub async fn authorize_handler(
    Query(params): Query<AuthorizeRequest>,
    State(state): State<OAuth2FlowState>,
) -> Result<Response, StatusCode> {
    info!(
        client_id = %params.client_id,
        redirect_uri = %params.redirect_uri,
        scope = ?params.scope,
        code_challenge_method = ?params.code_challenge_method,
        "OAuth2 authorization request received"
    );

    // Validate response_type
    if params.response_type != "code" {
        error!(response_type = %params.response_type, "Invalid response_type, must be 'code'");
        return create_error_redirect(
            &params.redirect_uri,
            "unsupported_response_type",
            "Response type must be 'code'",
            params.state.as_deref(),
        );
    }

    // Validate PKCE parameters
    if params.code_challenge.is_empty() {
        error!("Missing required PKCE code_challenge parameter");
        return create_error_redirect(
            &params.redirect_uri,
            "invalid_request",
            "Missing code_challenge parameter",
            params.state.as_deref(),
        );
    }

    let code_challenge_method = params.code_challenge_method.as_deref().unwrap_or("plain");

    if code_challenge_method != "S256" && code_challenge_method != "plain" {
        error!(method = %code_challenge_method, "Invalid code_challenge_method");
        return create_error_redirect(
            &params.redirect_uri,
            "invalid_request",
            "Invalid code_challenge_method",
            params.state.as_deref(),
        );
    }

    // Default scope if not provided
    let scope = params.scope.unwrap_or_else(|| "mcp:*".to_string());

    // Create authorization code
    let auth_code = AuthorizationCode::new(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        scope,
        params.code_challenge.clone(),
        code_challenge_method.to_string(),
        params.state.clone(),
    );

    // Store the authorization code
    let code = state.store_auth_code(auth_code);

    info!(
        code = %code,
        client_id = %params.client_id,
        "Authorization code generated and stored"
    );

    // Create redirect URI with authorization code
    let mut redirect_url = format!("{}?code={}", params.redirect_uri, code);
    if let Some(state_param) = &params.state {
        redirect_url.push_str(&format!("&state={}", state_param));
    }

    debug!(redirect_url = %redirect_url, "Redirecting with authorization code");

    Ok(Redirect::to(&redirect_url).into_response())
}

/// OAuth2 token endpoint handler
/// Exchanges authorization code for JWT access token (RFC 6749)
pub async fn oauth_token_handler(
    State(state): State<OAuth2FlowState>,
    Form(params): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OAuth2Error>)> {
    info!(
        grant_type = %params.grant_type,
        client_id = %params.client_id,
        code = %params.code,
        "OAuth2 token request received"
    );

    // Validate grant_type
    if params.grant_type != "authorization_code" {
        error!(grant_type = %params.grant_type, "Invalid grant_type");
        return Err(create_oauth2_error(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            "Grant type must be 'authorization_code'",
        ));
    }

    // Retrieve and consume authorization code
    let auth_code = state.consume_auth_code(&params.code).ok_or_else(|| {
        error!(code = %params.code, "Invalid or expired authorization code");
        create_oauth2_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "Invalid or expired authorization code",
        )
    })?;

    // Validate client_id matches
    if auth_code.client_id != params.client_id {
        error!(
            auth_client_id = %auth_code.client_id,
            request_client_id = %params.client_id,
            "Client ID mismatch"
        );
        return Err(create_oauth2_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "Client ID mismatch",
        ));
    }

    // Validate redirect_uri matches
    if auth_code.redirect_uri != params.redirect_uri {
        error!(
            auth_redirect_uri = %auth_code.redirect_uri,
            request_redirect_uri = %params.redirect_uri,
            "Redirect URI mismatch"
        );
        return Err(create_oauth2_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "Redirect URI mismatch",
        ));
    }

    // Verify PKCE code_verifier
    if !verify_pkce_challenge(
        &params.code_verifier,
        &auth_code.code_challenge,
        &auth_code.code_challenge_method,
    ) {
        error!(
            code_challenge = %auth_code.code_challenge,
            code_challenge_method = %auth_code.code_challenge_method,
            "PKCE verification failed"
        );
        return Err(create_oauth2_error(
            StatusCode::BAD_REQUEST,
            "invalid_grant",
            "PKCE verification failed",
        ));
    }

    // Determine token configuration based on scope
    let token_config = if auth_code.scope.contains("mcp:*") {
        &state.token_configs["full"]
    } else if auth_code.scope.contains("mcp:tools") {
        &state.token_configs["tools"]
    } else if auth_code.scope.contains("mcp:resources") {
        &state.token_configs["resources"]
    } else {
        &state.token_configs["readonly"]
    };

    // Generate JWT token using the existing function
    let scope_strs: Vec<&str> = token_config.scopes.iter().map(|s| s.as_str()).collect();
    let jwt_token = generate_test_token(
        &token_config.subject,
        &scope_strs,
        "mcp-server",
        "https://auth.example.com",
        token_config.expires_minutes,
        &state.test_keys.encoding_key,
    )
    .map_err(|e| {
        error!(error = %e, "Failed to generate JWT token");
        create_oauth2_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Failed to generate access token",
        )
    })?;

    info!(
        client_id = %params.client_id,
        scope = %auth_code.scope,
        token_type = %token_config.name,
        "JWT access token generated successfully"
    );

    // Create token response
    let token_response = TokenResponse {
        access_token: jwt_token,
        token_type: "Bearer".to_string(),
        expires_in: (token_config.expires_minutes * 60) as u64, // Convert minutes to seconds
        scope: auth_code.scope,
        algorithm: "RS256".to_string(),
        key_id: "test-key-oauth2-mcp".to_string(), // Match the kid from JWKS
    };

    Ok(Json(token_response))
}

/// Create OAuth2 error redirect response
fn create_error_redirect(
    redirect_uri: &str,
    error: &str,
    description: &str,
    state: Option<&str>,
) -> Result<Response, StatusCode> {
    let mut redirect_url = format!(
        "{}?error={}&error_description={}",
        redirect_uri,
        urlencoding::encode(error),
        urlencoding::encode(description)
    );

    if let Some(state_param) = state {
        redirect_url.push_str(&format!("&state={}", urlencoding::encode(state_param)));
    }

    warn!(
        error = %error,
        description = %description,
        redirect_url = %redirect_url,
        "OAuth2 error redirect"
    );

    Ok(Redirect::to(&redirect_url).into_response())
}

/// Create OAuth2 error response
fn create_oauth2_error(
    status: StatusCode,
    error: &str,
    description: &str,
) -> (StatusCode, Json<OAuth2Error>) {
    let error_response = OAuth2Error {
        error: error.to_string(),
        error_description: description.to_string(),
        error_uri: None,
    };

    (status, Json(error_response))
}

/// OAuth2 Authorization Server Metadata endpoint (RFC 8414)
/// Returns OAuth2 server configuration for client discovery
pub async fn oauth2_metadata_handler() -> Result<Json<Value>, StatusCode> {
    info!("OAuth2 metadata discovery endpoint accessed");

    let metadata = json!({
        "issuer": "https://auth.example.com",
        "authorization_endpoint": "http://127.0.0.1:3003/authorize",
        "token_endpoint": "http://127.0.0.1:3003/token",
        "jwks_uri": "http://127.0.0.1:3003/.well-known/jwks.json",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256", "plain"],
        "scopes_supported": [
            "mcp:tools:execute",
            "mcp:resources:read",
            "mcp:resources:write",
            "mcp:resources:list",
            "mcp:tools:read",
            "mcp:prompts:read",
            "mcp:prompts:list",
            "mcp:*"
        ],
        "token_endpoint_auth_methods_supported": ["none"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "claims_supported": ["sub", "aud", "iss", "exp", "iat", "scope", "jti"]
    });

    debug!(
        "OAuth2 metadata: {}",
        serde_json::to_string_pretty(&metadata)
            .unwrap_or_else(|_| "Failed to serialize".to_string())
    );

    Ok(Json(metadata))
}
