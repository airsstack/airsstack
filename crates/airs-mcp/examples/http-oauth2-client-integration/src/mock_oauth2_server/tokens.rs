// JWT token generation utilities

// Third-party crate imports
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use uuid::Uuid;

// Internal module imports
use http_oauth2_client_integration::{OAuth2IntegrationError, TokenClaims};

/// Generate a JWT access token
pub fn generate_jwt_token(
    client_id: &str,
    scope: &str,
    issuer: &str,
    expires_in_seconds: i64,
    encoding_key: &EncodingKey,
) -> Result<String, OAuth2IntegrationError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(expires_in_seconds);

    let claims = TokenClaims {
        sub: client_id.to_string(),
        aud: vec!["mcp-server".to_string()],
        iss: issuer.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        scope: Some(scope.to_string()),
        client_id: Some(client_id.to_string()),
    };

    let header = Header {
        kid: Some("oauth2-mock-server-key".to_string()),
        alg: Algorithm::RS256,
        ..Default::default()
    };

    encode(&header, &claims, encoding_key)
        .map_err(|e| OAuth2IntegrationError::Jwt(e))
}

/// Generate a refresh token
#[allow(dead_code)]
pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, Validation};

    #[test]
    fn test_token_generation() {
        // This test would need a real private key
        // For now, just test that the function signature works
        let encoding_key = EncodingKey::from_secret("test-secret".as_bytes());
        
        let result = generate_jwt_token(
            "test-client",
            "mcp:read mcp:write",
            "http://localhost:3002",
            3600,
            &encoding_key,
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_refresh_token_generation() {
        let token1 = generate_refresh_token();
        let token2 = generate_refresh_token();
        
        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
    }
}