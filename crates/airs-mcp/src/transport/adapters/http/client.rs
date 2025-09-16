//! HTTP Transport Client Implementation
//!
//! This module provides a clean client-side HTTP transport implementation using the
//! TransportClient trait for direct request-response communication with MCP servers
//! over HTTP using JSON-RPC 2.0 protocol.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use reqwest;
use serde_json;
use url::Url;

// Layer 3: Internal module imports
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, TransportClient, TransportError};

/// HTTP Transport Client for communicating with MCP servers over HTTP
///
/// This client implementation sends JSON-RPC requests to MCP servers over HTTP
/// and receives responses. It supports various authentication methods and provides
/// a clean request-response interface.
///
/// # Architecture
///
/// ```text
/// HttpTransportClient -> HTTP POST -> MCP Server
///     |                                   |
///     |- JSON-RPC request in body ->      |- processes request
///     |                                   |- generates response
///     |<- JSON-RPC response in body <-    |- sends response
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::{HttpTransportClient, HttpTransportClientBuilder, AuthMethod};
/// use airs_mcp::protocol::{TransportClient, JsonRpcRequest, RequestId};
/// use serde_json::json;
/// use std::time::Duration;
/// use url::Url;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Build and configure the client
/// let mut client = HttpTransportClientBuilder::new()
///     .endpoint("https://api.example.com/mcp")?
///     .auth(AuthMethod::ApiKey {
///         key: "your-api-key".to_string(),
///         header: "X-API-Key".to_string()
///     })
///     .timeout(Duration::from_secs(30))
///     .build()
///     .await?;
///
/// // Send a request and get response directly
/// let request = JsonRpcRequest::new(
///     "initialize",
///     Some(json!({"capabilities": {}})),
///     RequestId::new_string("init-1")
/// );
///
/// let response = client.call(request).await?;
/// println!("Response: {:?}", response);
///
/// client.close().await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpTransportClient {
    /// HTTP client for making requests
    client: reqwest::Client,

    /// Endpoint URL for the MCP server
    endpoint: Url,

    /// Authentication method
    auth: Option<AuthMethod>,

    /// Client configuration
    config: HttpClientConfig,
}

/// Authentication methods supported by the HTTP transport client
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// API key authentication with custom header
    ApiKey {
        /// The API key value
        key: String,
        /// The header name to use (e.g., "X-API-Key", "Authorization")
        header: String,
    },

    /// Bearer token authentication
    Bearer {
        /// The bearer token value
        token: String,
    },

    /// Basic authentication with username and password
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },

    /// OAuth 2.0 token authentication
    OAuth2 {
        /// OAuth 2.0 access token
        access_token: String,
        /// Optional token type (defaults to "Bearer")
        token_type: Option<String>,
    },
}

/// Configuration for HttpTransportClient
///
/// Contains settings for HTTP communication, authentication,
/// timeouts, and retry logic.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Timeout for individual HTTP requests
    pub timeout: Duration,

    /// User agent string to send with requests
    pub user_agent: String,

    /// Additional headers to send with every request
    pub headers: HashMap<String, String>,

    /// Maximum number of redirect hops to follow
    pub max_redirects: usize,

    /// Whether to accept invalid TLS certificates (for development)
    pub accept_invalid_certs: bool,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Custom content type for requests (defaults to "application/json")
    pub content_type: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: format!("airs-mcp-client/{}", env!("CARGO_PKG_VERSION")),
            headers: HashMap::new(),
            max_redirects: 10,
            accept_invalid_certs: false,
            connect_timeout: Duration::from_secs(10),
            content_type: "application/json".to_string(),
        }
    }
}

#[async_trait]
impl TransportClient for HttpTransportClient {
    type Error = TransportError;

    async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
        // Serialize the request
        let request_body = serde_json::to_string(&request)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Build the HTTP request
        let mut http_request = self
            .client
            .post(self.endpoint.clone())
            .header("Content-Type", &self.config.content_type)
            .header("User-Agent", &self.config.user_agent)
            .body(request_body)
            .timeout(self.config.timeout);

        // Add custom headers
        for (key, value) in &self.config.headers {
            http_request = http_request.header(key, value);
        }

        // Add authentication
        if let Some(ref auth) = self.auth {
            http_request = match auth {
                AuthMethod::ApiKey { key, header } => http_request.header(header, key),
                AuthMethod::Bearer { token } => {
                    http_request.header("Authorization", format!("Bearer {token}"))
                }
                AuthMethod::Basic { username, password } => {
                    http_request.basic_auth(username, Some(password))
                }
                AuthMethod::OAuth2 {
                    access_token,
                    token_type,
                } => {
                    let token_type = token_type.as_deref().unwrap_or("Bearer");
                    http_request.header("Authorization", format!("{token_type} {access_token}"))
                }
            };
        }

        // Send the request
        let response = http_request.send().await.map_err(|e| {
            if e.is_timeout() {
                TransportError::request_timeout(self.config.timeout)
            } else if e.is_connect() {
                TransportError::Connection {
                    message: format!("Failed to connect to {}: {}", self.endpoint, e),
                }
            } else {
                TransportError::Connection {
                    message: format!("HTTP request failed: {e}"),
                }
            }
        })?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(TransportError::Protocol {
                message: format!(
                    "HTTP error {}: {}",
                    response.status().as_u16(),
                    response
                        .status()
                        .canonical_reason()
                        .unwrap_or("Unknown error")
                ),
            });
        }

        // Read response body
        let response_body = response.text().await.map_err(|e| TransportError::Io {
            source: std::io::Error::other(e),
        })?;

        // Parse JSON-RPC response
        let json_response: JsonRpcResponse = serde_json::from_str(&response_body).map_err(|e| {
            TransportError::invalid_response(format!("Failed to parse JSON-RPC response: {e}"))
        })?;

        Ok(json_response)
    }

    fn is_ready(&self) -> bool {
        // HTTP client is always ready if properly constructed
        true
    }

    fn transport_type(&self) -> &'static str {
        "http"
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        // HTTP client doesn't require explicit closing
        // The underlying reqwest client will be dropped when this struct is dropped
        Ok(())
    }
}

/// Builder for HttpTransportClient
///
/// Provides a fluent interface for configuring and creating HttpTransportClient instances.
/// The builder pattern ensures all required configuration is provided before creating the client.
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::{HttpTransportClientBuilder, AuthMethod};
/// use std::time::Duration;
/// use std::collections::HashMap;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut client = HttpTransportClientBuilder::new()
///     .endpoint("https://api.example.com/mcp")?
///     .auth(AuthMethod::Bearer {
///         token: "your-bearer-token".to_string()
///     })
///     .timeout(Duration::from_secs(60))
///     .user_agent("MyApp/1.0")
///     .header("X-Custom-Header", "custom-value")
///     .accept_invalid_certs(true)  // For development only
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpTransportClientBuilder {
    endpoint: Option<Url>,
    auth: Option<AuthMethod>,
    config: HttpClientConfig,
}

impl HttpTransportClientBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            endpoint: None,
            auth: None,
            config: HttpClientConfig::default(),
        }
    }

    /// Set the endpoint URL for the MCP server
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The URL of the MCP server endpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is malformed
    pub fn endpoint(mut self, endpoint: &str) -> Result<Self, TransportError> {
        self.endpoint = Some(
            Url::parse(endpoint).map_err(|e| TransportError::Connection {
                message: format!("Invalid endpoint URL '{endpoint}': {e}"),
            })?,
        );
        Ok(self)
    }

    /// Set the authentication method
    ///
    /// # Arguments
    ///
    /// * `auth` - The authentication method to use
    pub fn auth(mut self, auth: AuthMethod) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the request timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration to wait for each request to complete
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the connection timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration to wait for connection establishment
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set the user agent string
    ///
    /// # Arguments
    ///
    /// * `user_agent` - User agent string to send with requests
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Add a custom header to all requests
    ///
    /// # Arguments
    ///
    /// * `key` - Header name
    /// * `value` - Header value
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.headers.insert(key.into(), value.into());
        self
    }

    /// Set multiple custom headers
    ///
    /// # Arguments
    ///
    /// * `headers` - HashMap of headers to add to all requests
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.config.headers = headers;
        self
    }

    /// Set the maximum number of redirects to follow
    ///
    /// # Arguments
    ///
    /// * `max_redirects` - Maximum number of redirect hops
    pub fn max_redirects(mut self, max_redirects: usize) -> Self {
        self.config.max_redirects = max_redirects;
        self
    }

    /// Configure whether to accept invalid TLS certificates
    ///
    /// **Warning**: Only use this for development. Never use in production.
    ///
    /// # Arguments
    ///
    /// * `accept` - Whether to accept invalid certificates
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.config.accept_invalid_certs = accept;
        self
    }

    /// Set the content type for requests
    ///
    /// # Arguments
    ///
    /// * `content_type` - Content type header value
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.config.content_type = content_type.into();
        self
    }

    /// Build the HttpTransportClient
    ///
    /// This method creates the underlying HTTP client with the specified configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No endpoint was specified
    /// - Failed to create the HTTP client
    pub async fn build(self) -> Result<HttpTransportClient, TransportError> {
        let endpoint = self.endpoint.ok_or_else(|| {
            TransportError::not_ready("No endpoint specified for HTTP transport client")
        })?;

        // Create the reqwest client with configuration
        let mut client_builder = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .connect_timeout(self.config.connect_timeout)
            .redirect(reqwest::redirect::Policy::limited(
                self.config.max_redirects,
            ))
            .user_agent(&self.config.user_agent);

        if self.config.accept_invalid_certs {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        Ok(HttpTransportClient {
            client,
            endpoint,
            auth: self.auth,
            config: self.config,
        })
    }
}

impl Default for HttpTransportClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
