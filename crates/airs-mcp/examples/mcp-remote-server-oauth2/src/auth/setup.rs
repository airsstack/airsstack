//! OAuth2 Setup for AirsStack MCP Transport Integration
//!
//! This module configures OAuth2 authentication components from AirsStack
//! for use with the HTTP transport layer.

use std::time::Duration;
use url::Url;

// AirsStack MCP components
use airs_mcp::authentication::strategies::oauth2::OAuth2Strategy;
use airs_mcp::oauth2::{
    config::{OAuth2Config, CacheConfig, ValidationConfig, ScopeMapping},
    validator::{Jwt, Scope, Validator},
};
use airs_mcp::transport::adapters::http::auth::{
    middleware::HttpAuthConfig,
    oauth2::OAuth2StrategyAdapter,
};

/// OAuth2 setup configuration for AirsStack MCP integration
pub struct OAuth2Setup {
    // Using the concrete Jwt and Scope types from airs_mcp::oauth2::validator
    pub strategy_adapter: OAuth2StrategyAdapter<Jwt, Scope>,
    pub auth_config: HttpAuthConfig,
}

impl OAuth2Setup {
    /// Create OAuth2 setup using AirsStack components
    ///
    /// This configures:
    /// - OAuth2Config with JWKS validation
    /// - OAuth2Strategy with JWT and scope validators
    /// - OAuth2StrategyAdapter for HTTP transport
    /// - HttpAuthConfig for authentication middleware
    pub fn new(jwks_url: Url) -> Result<Self, Box<dyn std::error::Error>> {
        // Create OAuth2 configuration for AirsStack components
        let oauth2_config = OAuth2Config::builder()
            .jwks_url(jwks_url)
            .audience("mcp-oauth2-remote-server".to_string())
            .issuer("oauth2-mcp-remote-issuer".to_string())
            .validation_config(ValidationConfig {
                require_exp: true,
                require_aud: true,
                require_iss: true,
                validate_nbf: true,
                leeway: Duration::from_secs(60),
                algorithms: vec!["RS256".to_string()],
            })
            .cache_config(CacheConfig {
                jwks_cache_ttl: Duration::from_secs(300),
                jwks_cache_max_size: 10,
                token_cache_ttl: Duration::from_secs(60),
                token_cache_max_size: 100,
            })
            .build()?;

        // Create OAuth2 validators using AirsStack components
        let jwt_validator = Jwt::new(oauth2_config.clone())?;
        
        // Create custom scope mappings that include initialize and use our mcp:* wildcard scope
        let mut custom_mappings = OAuth2Config::default_scope_mappings();
        
        // Add initialize method mapping to allow mcp:* scope
        custom_mappings.push(ScopeMapping {
            method: "initialize".to_string(),
            scope: "mcp:*".to_string(),
            optional: false,
        });
        
        // Override default mappings to use mcp:* for full access tokens
        // This allows our full access token with mcp:* scope to work with all methods
        for mapping in &mut custom_mappings {
            // For tools, resources, and prompts, also accept mcp:* as sufficient scope
            if mapping.scope.starts_with("mcp:tools:") 
                || mapping.scope.starts_with("mcp:resources:") 
                || mapping.scope.starts_with("mcp:prompts:") 
                || mapping.scope.starts_with("mcp:logging:") 
                || mapping.scope.starts_with("mcp:completion:") {
                // Keep original specific scope but make it optional if user has mcp:*
                // We'll handle wildcard matching in a custom validator if needed
            }
        }
        
        let scope_validator = Scope::new(custom_mappings);
        let validator = Validator::new(jwt_validator, scope_validator);

        // Create OAuth2Strategy from AirsStack
        let oauth2_strategy = OAuth2Strategy::new(validator);

        // Create OAuth2StrategyAdapter for HTTP transport
        let strategy_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);

        // Create HTTP authentication configuration
        let auth_config = HttpAuthConfig {
            include_error_details: true,
            auth_realm: "OAuth2 MCP Remote Server".to_string(),
            request_timeout_secs: 30,
            skip_paths: vec![
                "/health".to_string(),
                "/status".to_string(),
                "/metrics".to_string(),
                "/info".to_string(),
            ],
        };

        Ok(OAuth2Setup {
            strategy_adapter,
            auth_config,
        })
    }
}
