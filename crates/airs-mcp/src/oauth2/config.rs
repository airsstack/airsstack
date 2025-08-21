//! OAuth 2.1 Configuration Structures
//!
//! This module provides configuration structures for OAuth 2.1 authentication
//! with reasonable defaults and validation.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// OAuth 2.1 configuration for MCP authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Authorization server's JWKS endpoint URL
    pub jwks_url: Url,

    /// Expected JWT audience (typically the MCP server's identifier)
    pub audience: String,

    /// Expected JWT issuer (authorization server identifier)
    pub issuer: String,

    /// Optional documentation URL for the protected resource
    pub documentation_url: Option<String>,

    /// Cache configuration for JWKS keys and tokens
    pub cache: CacheConfig,

    /// Token validation configuration
    pub validation: ValidationConfig,

    /// MCP scope mappings (method -> required scope)
    pub scope_mappings: Vec<ScopeMapping>,
}

/// Cache configuration for OAuth components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// JWKS key cache TTL (time-to-live)
    pub jwks_cache_ttl: Duration,

    /// Maximum number of cached JWKS keys
    pub jwks_cache_max_size: usize,

    /// Token validation cache TTL (for performance optimization)
    pub token_cache_ttl: Duration,

    /// Maximum number of cached token validations
    pub token_cache_max_size: usize,
}

/// Token validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Clock skew tolerance for token expiration (in seconds)
    pub leeway: Duration,

    /// Require 'exp' (expiration) claim
    pub require_exp: bool,

    /// Require 'aud' (audience) claim
    pub require_aud: bool,

    /// Require 'iss' (issuer) claim
    pub require_iss: bool,

    /// Validate 'nbf' (not before) claim
    pub validate_nbf: bool,

    /// Required algorithms (e.g., ["RS256"])
    pub algorithms: Vec<String>,
}

/// MCP method to OAuth scope mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeMapping {
    /// MCP method name (e.g., "tools/call")
    pub method: String,

    /// Required OAuth scope (e.g., "mcp:tools:execute")
    pub scope: String,

    /// Whether this mapping is optional (defaults to false)
    #[serde(default)]
    pub optional: bool,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            // These would typically come from environment or config file
            jwks_url: Url::parse("https://example.com/.well-known/jwks.json")
                .expect("Default JWKS URL should be valid"),
            audience: "mcp-server".to_string(),
            issuer: "https://example.com".to_string(),
            documentation_url: None,
            cache: CacheConfig::default(),
            validation: ValidationConfig::default(),
            scope_mappings: Self::default_scope_mappings(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            jwks_cache_ttl: Duration::from_secs(3600), // 1 hour
            jwks_cache_max_size: 100,
            token_cache_ttl: Duration::from_secs(300), // 5 minutes
            token_cache_max_size: 1000,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            leeway: Duration::from_secs(60), // 1 minute clock skew tolerance
            require_exp: true,
            require_aud: true,
            require_iss: true,
            validate_nbf: true,
            algorithms: vec!["RS256".to_string()], // Only RS256 by default
        }
    }
}

impl OAuth2Config {
    /// Create a new OAuth2Config with builder pattern
    pub fn builder() -> OAuth2ConfigBuilder {
        OAuth2ConfigBuilder::default()
    }

    /// Default MCP method to OAuth scope mappings
    pub fn default_scope_mappings() -> Vec<ScopeMapping> {
        vec![
            // Tool operations
            ScopeMapping {
                method: "tools/call".to_string(),
                scope: "mcp:tools:execute".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "tools/list".to_string(),
                scope: "mcp:tools:read".to_string(),
                optional: false,
            },
            // Resource operations
            ScopeMapping {
                method: "resources/read".to_string(),
                scope: "mcp:resources:read".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "resources/list".to_string(),
                scope: "mcp:resources:list".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "resources/subscribe".to_string(),
                scope: "mcp:resources:subscribe".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "resources/unsubscribe".to_string(),
                scope: "mcp:resources:subscribe".to_string(),
                optional: false,
            },
            // Prompt operations
            ScopeMapping {
                method: "prompts/get".to_string(),
                scope: "mcp:prompts:read".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "prompts/list".to_string(),
                scope: "mcp:prompts:list".to_string(),
                optional: false,
            },
            // Logging operations
            ScopeMapping {
                method: "logging/setLevel".to_string(),
                scope: "mcp:logging:configure".to_string(),
                optional: false,
            },
            // Completion operations
            ScopeMapping {
                method: "completion/complete".to_string(),
                scope: "mcp:completion:read".to_string(),
                optional: false,
            },
        ]
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.audience.is_empty() {
            return Err("Audience cannot be empty".to_string());
        }

        if self.issuer.is_empty() {
            return Err("Issuer cannot be empty".to_string());
        }

        if self.validation.algorithms.is_empty() {
            return Err("At least one algorithm must be specified".to_string());
        }

        // Validate that all algorithms are supported
        for algorithm in &self.validation.algorithms {
            if !["RS256", "RS384", "RS512", "ES256", "ES384"].contains(&algorithm.as_str()) {
                return Err(format!("Unsupported algorithm: {}", algorithm));
            }
        }

        Ok(())
    }
}

/// Builder for OAuth2Config
#[derive(Debug, Default)]
pub struct OAuth2ConfigBuilder {
    jwks_url: Option<Url>,
    audience: Option<String>,
    issuer: Option<String>,
    documentation_url: Option<String>,
    cache: Option<CacheConfig>,
    validation: Option<ValidationConfig>,
    scope_mappings: Option<Vec<ScopeMapping>>,
}

impl OAuth2ConfigBuilder {
    pub fn jwks_url(mut self, url: Url) -> Self {
        self.jwks_url = Some(url);
        self
    }

    pub fn audience(mut self, audience: String) -> Self {
        self.audience = Some(audience);
        self
    }

    pub fn issuer(mut self, issuer: String) -> Self {
        self.issuer = Some(issuer);
        self
    }

    pub fn documentation_url(mut self, url: String) -> Self {
        self.documentation_url = Some(url);
        self
    }

    pub fn cache_config(mut self, cache: CacheConfig) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn validation_config(mut self, validation: ValidationConfig) -> Self {
        self.validation = Some(validation);
        self
    }

    pub fn scope_mappings(mut self, mappings: Vec<ScopeMapping>) -> Self {
        self.scope_mappings = Some(mappings);
        self
    }

    pub fn build(self) -> Result<OAuth2Config, String> {
        let config = OAuth2Config {
            jwks_url: self.jwks_url.ok_or("JWKS URL is required")?,
            audience: self.audience.ok_or("Audience is required")?,
            issuer: self.issuer.ok_or("Issuer is required")?,
            documentation_url: self.documentation_url,
            cache: self.cache.unwrap_or_default(),
            validation: self.validation.unwrap_or_default(),
            scope_mappings: self
                .scope_mappings
                .unwrap_or_else(|| OAuth2Config::default_scope_mappings()),
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OAuth2Config::default();
        assert_eq!(config.audience, "mcp-server");
        assert_eq!(config.issuer, "https://example.com");
        assert!(!config.scope_mappings.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = OAuth2Config::builder()
            .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json").unwrap())
            .audience("my-mcp-server".to_string())
            .issuer("https://auth.example.com".to_string())
            .build()
            .expect("Config should build successfully");

        assert_eq!(config.audience, "my-mcp-server");
        assert_eq!(config.issuer, "https://auth.example.com");
    }

    #[test]
    fn test_config_validation() {
        let mut config = OAuth2Config::default();
        config.audience = "".to_string();

        assert!(config.validate().is_err());

        config.audience = "valid-audience".to_string();
        config.validation.algorithms = vec!["INVALID_ALG".to_string()];

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_default_scope_mappings() {
        let mappings = OAuth2Config::default_scope_mappings();

        // Check that tools/call maps to mcp:tools:execute
        let tools_call = mappings
            .iter()
            .find(|m| m.method == "tools/call")
            .expect("tools/call mapping should exist");
        assert_eq!(tools_call.scope, "mcp:tools:execute");
        assert!(!tools_call.optional);

        // Check that we have mappings for all major MCP operations
        let methods: Vec<&str> = mappings.iter().map(|m| m.method.as_str()).collect();
        assert!(methods.contains(&"tools/call"));
        assert!(methods.contains(&"resources/read"));
        assert!(methods.contains(&"prompts/get"));
        assert!(methods.contains(&"logging/setLevel"));
    }

    #[test]
    fn test_scope_mapping_equality() {
        let mapping1 = ScopeMapping {
            method: "tools/call".to_string(),
            scope: "mcp:tools:execute".to_string(),
            optional: false,
        };

        let mapping2 = ScopeMapping {
            method: "tools/call".to_string(),
            scope: "mcp:tools:execute".to_string(),
            optional: false,
        };

        assert_eq!(mapping1, mapping2);
    }
}
