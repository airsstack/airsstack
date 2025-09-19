// OAuth2 endpoint handlers

// Standard library imports
use std::sync::Arc;

// Third-party crate imports
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Form, Router,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

// Internal module imports
use http_oauth2_client_integration::{
    AuthorizationCode, OAuth2Error, TokenClaims, TokenResponse,
};
use super::{server::OAuth2ServerState, tokens::generate_jwt_token};

/// Create OAuth2 endpoints router
pub fn create_oauth2_router() -> Router<Arc<OAuth2ServerState>> {
    Router::new()
        .route("/authorize", get(authorization_endpoint))
        .route("/token", post(token_endpoint))
        .route("/.well-known/openid-configuration", get(openid_configuration))
        .route("/health", get(health_check))
}

/// OAuth2 authorization request parameters
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// OAuth2 authorization endpoint
async fn authorization_endpoint(
    Query(params): Query<AuthorizeRequest>,
    State(state): State<Arc<OAuth2ServerState>>,
) -> Result<Html<String>, (StatusCode, Json<OAuth2Error>)> {
    info!("🔐 OAuth2 authorization request received");
    debug!("📋 Authorization parameters: {:?}", params);

    // Validate response_type
    if params.response_type != "code" {
        warn!("❌ Invalid response_type: {}", params.response_type);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OAuth2Error {
                error: "unsupported_response_type".to_string(),
                error_description: Some("Only 'code' response type is supported".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Validate client and redirect URI
    let client = match state.validate_client(&params.client_id, Some(&params.redirect_uri)) {
        Ok(client) => client,
        Err(e) => {
            warn!("❌ Client validation failed: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "invalid_client".to_string(),
                    error_description: Some(e.to_string()),
                    error_uri: None,
                }),
            ));
        }
    };

    // Validate PKCE challenge method
    if let Some(ref method) = params.code_challenge_method {
        if method != "S256" {
            warn!("❌ Unsupported code_challenge_method: {}", method);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "invalid_request".to_string(),
                    error_description: Some("Only S256 code_challenge_method is supported".to_string()),
                    error_uri: None,
                }),
            ));
        }
    }

    // Validate and process scopes
    let scope = params.scope.unwrap_or_else(|| "openid".to_string());
    let validated_scopes = match state.validate_scopes(&params.client_id, &scope) {
        Ok(scopes) => scopes,
        Err(e) => {
            warn!("❌ Scope validation failed: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "invalid_scope".to_string(),
                    error_description: Some(e.to_string()),
                    error_uri: None,
                }),
            ));
        }
    };

    // Generate authorization code
    let authorization_code = Uuid::new_v4().to_string();
    let auth_data = AuthorizationCode {
        code: authorization_code.clone(),
        client_id: params.client_id.clone(),
        redirect_uri: params.redirect_uri.clone(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: params.code_challenge_method.clone(),
        scope: validated_scopes.join(" "),
        state: params.state.clone(),
        expires_at: Utc::now() + Duration::minutes(10), // 10 minute expiration
    };

    // Store authorization code
    state.store_authorization_code(authorization_code.clone(), auth_data).await;

    info!("✅ Authorization code generated for client: {}", params.client_id);
    debug!("🎫 Authorization code: {}", authorization_code);

    // In a real OAuth2 server, this would render a consent page
    // For this mock server, we'll auto-approve and return a simple HTML page
    let html_response = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>OAuth2 Mock Authorization Server</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        .success {{ background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 15px; border-radius: 4px; }}
        .code {{ background: #f8f9fa; padding: 10px; font-family: monospace; border-radius: 4px; margin: 10px 0; }}
        .redirect {{ margin-top: 20px; }}
        .redirect a {{ background: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 4px; }}
    </style>
</head>
<body>
    <h1>🔐 OAuth2 Authorization</h1>
    <div class="success">
        <h3>✅ Authorization Successful</h3>
        <p><strong>Client:</strong> {} ({})</p>
        <p><strong>Scopes:</strong> {}</p>
        <p><strong>Authorization Code:</strong></p>
        <div class="code">{}</div>
    </div>
    
    <div class="redirect">
        <p>In a real OAuth2 flow, you would be redirected automatically.</p>
        <p>For testing, use this authorization code with your OAuth2 client:</p>
        <a href="{}?code={}&state={}" target="_blank">
            Continue to Client
        </a>
    </div>
    
    <hr>
    <p><em>This is a mock OAuth2 authorization server for development and testing purposes only.</em></p>
</body>
</html>
        "#,
        client.name,
        params.client_id,
        validated_scopes.join(", "),
        authorization_code,
        params.redirect_uri,
        authorization_code,
        params.state.unwrap_or_else(|| "no-state".to_string())
    );

    Ok(Html(html_response))
}

/// OAuth2 token request parameters
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// Grant type - must be "authorization_code" for this implementation
    pub grant_type: String,
    /// Authorization code from /authorize endpoint
    pub code: String,
    /// Redirection URI (must match the one from /authorize)
    pub redirect_uri: String,
    /// Client identifier (must match the one from /authorize)
    pub client_id: String,
    /// PKCE code verifier (RFC 7636)
    pub code_verifier: Option<String>,
    /// For refresh token grant (not implemented in this demo)
    #[allow(dead_code)]
    pub refresh_token: Option<String>,
    /// Scope for refresh token grant (not implemented in this demo)
    #[allow(dead_code)]
    pub scope: Option<String>,
}

/// OAuth2 token endpoint
async fn token_endpoint(
    State(state): State<Arc<OAuth2ServerState>>,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OAuth2Error>)> {
    info!("🎫 OAuth2 token request received");
    debug!("📋 Token request: {:?}", request);

    match request.grant_type.as_str() {
        "authorization_code" => handle_authorization_code_grant(state, request).await,
        "refresh_token" => handle_refresh_token_grant(state, request).await,
        _ => {
            warn!("❌ Unsupported grant_type: {}", request.grant_type);
            Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "unsupported_grant_type".to_string(),
                    error_description: Some(format!("Grant type '{}' is not supported", request.grant_type)),
                    error_uri: None,
                }),
            ))
        }
    }
}

/// Handle authorization code grant
async fn handle_authorization_code_grant(
    state: Arc<OAuth2ServerState>,
    request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OAuth2Error>)> {
    let code = request.code;
    let redirect_uri = request.redirect_uri;
    let client_id = request.client_id;
    let code_verifier = request.code_verifier;

    // Get and consume authorization code
    let auth_data = match state.consume_authorization_code(&code).await {
        Some(data) => data,
        None => {
            warn!("❌ Invalid or expired authorization code: {}", code);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "invalid_grant".to_string(),
                    error_description: Some("Invalid or expired authorization code".to_string()),
                    error_uri: None,
                }),
            ));
        }
    };

    // Validate client_id and redirect_uri match
    if auth_data.client_id != client_id || auth_data.redirect_uri != redirect_uri {
        warn!("❌ Client ID or redirect URI mismatch");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(OAuth2Error {
                error: "invalid_grant".to_string(),
                error_description: Some("Client ID or redirect URI mismatch".to_string()),
                error_uri: None,
            }),
        ));
    }

    // Validate PKCE code verifier if present
    if let (Some(challenge), Some(verifier)) = (&auth_data.code_challenge, &code_verifier) {
        use http_oauth2_client_integration::create_code_challenge;
        let expected_challenge = create_code_challenge(verifier);
        if expected_challenge != *challenge {
            warn!("❌ PKCE verification failed");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(OAuth2Error {
                    error: "invalid_grant".to_string(),
                    error_description: Some("PKCE verification failed".to_string()),
                    error_uri: None,
                }),
            ));
        }
    }

    // Generate JWT access token
    let access_token = generate_jwt_token(
        &client_id,
        &auth_data.scope,
        &state.config.issuer,
        3600, // 1 hour
        &state.encoding_key,
    ).map_err(|e| {
        warn!("❌ Failed to generate access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(OAuth2Error {
                error: "server_error".to_string(),
                error_description: Some("Failed to generate access token".to_string()),
                error_uri: None,
            }),
        )
    })?;

    // Generate refresh token
    let refresh_token = Uuid::new_v4().to_string();

    // Store token claims for validation
    let claims = TokenClaims {
        sub: client_id.clone(),
        aud: vec!["mcp-server".to_string()],
        iss: state.config.issuer.clone(),
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        scope: Some(auth_data.scope.clone()),
        client_id: Some(client_id.clone()),
    };

    state.store_active_token(access_token.clone(), claims).await;

    info!("✅ Access token generated for client: {}", client_id);

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: Some(refresh_token),
        scope: Some(auth_data.scope),
    }))
}

/// Handle refresh token grant
async fn handle_refresh_token_grant(
    _state: Arc<OAuth2ServerState>,
    _request: TokenRequest,
) -> Result<Json<TokenResponse>, (StatusCode, Json<OAuth2Error>)> {
    // For simplicity, refresh token flow is not fully implemented in this mock server
    // In a real implementation, you would validate the refresh token and issue new tokens
    Err((
        StatusCode::BAD_REQUEST,
        Json(OAuth2Error {
            error: "unsupported_grant_type".to_string(),
            error_description: Some("Refresh token grant not implemented in mock server".to_string()),
            error_uri: None,
        }),
    ))
}

/// OpenID Connect discovery endpoint
async fn openid_configuration(
    State(state): State<Arc<OAuth2ServerState>>,
) -> Json<serde_json::Value> {
    let base_url = &state.config.issuer;
    
    Json(serde_json::json!({
        "issuer": base_url,
        "authorization_endpoint": format!("{}/authorize", base_url),
        "token_endpoint": format!("{}/token", base_url),
        "jwks_uri": format!("{}/jwks", base_url),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "mcp:read", "mcp:write"],
        "token_endpoint_auth_methods_supported": ["none"],
        "subject_types_supported": ["public"]
    }))
}

/// Health check endpoint
async fn health_check(State(state): State<Arc<OAuth2ServerState>>) -> Json<serde_json::Value> {
    let stats = state.get_stats().await;
    
    Json(serde_json::json!({
        "status": "healthy",
        "server": "oauth2-mock-authorization-server",
        "timestamp": Utc::now().to_rfc3339(),
        "stats": {
            "active_authorization_codes": stats.active_authorization_codes,
            "active_tokens": stats.active_tokens,
            "registered_clients": stats.registered_clients
        }
    }))
}