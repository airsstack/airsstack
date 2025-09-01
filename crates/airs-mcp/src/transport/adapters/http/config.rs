//! HTTP Transport Configuration
//!
//! This module provides configuration structures for the HTTP Streamable Transport
//! with a builder pattern for progressive optimization.

use std::net::SocketAddr;
use std::time::Duration;

/// Core configuration for HTTP Streamable Transport
///
/// This configuration follows the validated architectural decisions:
/// - Simple defaults with progressive optimization
/// - Builder pattern for ease of use
/// - No environment-specific presets (anti-pattern avoided)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::HttpTransportConfig;
/// use std::time::Duration;
///
/// // Simple default configuration
/// let config = HttpTransportConfig::new();
///
/// // Custom configuration with builder pattern
/// let config = HttpTransportConfig::new()
///     .bind_address("0.0.0.0:8080".parse().unwrap())
///     .max_connections(5000)
///     .session_timeout(Duration::from_secs(300))
///     .enable_buffer_pool();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct HttpTransportConfig {
    /// Address to bind the HTTP server
    pub bind_address: SocketAddr,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Maximum concurrent requests per connection
    pub max_concurrent_requests: usize,
    
    /// Session timeout duration
    pub session_timeout: Duration,
    
    /// Keep-alive timeout for HTTP connections
    pub keep_alive_timeout: Duration,
    
    /// Request processing timeout
    pub request_timeout: Duration,
    
    /// Parser configuration for optimization
    pub parser: ParserConfig,
}

impl HttpTransportConfig {
    /// Create a new configuration with sensible defaults
    ///
    /// Default values are optimized for typical MCP workloads:
    /// - Bind to localhost:3000
    /// - Support 1000 concurrent connections
    /// - 10 concurrent requests per connection
    /// - 5-minute session timeout
    /// - 30-second keep-alive timeout
    /// - 30-second request timeout
    /// - No buffer pooling (simple per-request allocation)
    pub fn new() -> Self {
        Self {
            bind_address: "127.0.0.1:3000".parse().expect("Valid default address"),
            max_connections: 1000,
            max_concurrent_requests: 10,
            session_timeout: Duration::from_secs(300), // 5 minutes
            keep_alive_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(30),
            parser: ParserConfig::new(),
        }
    }
    
    /// Set the bind address
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.bind_address = addr;
        self
    }
    
    /// Set maximum concurrent connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }
    
    /// Set maximum concurrent requests per connection
    pub fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent_requests = max;
        self
    }
    
    /// Set session timeout
    pub fn session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }
    
    /// Set keep-alive timeout
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = timeout;
        self
    }
    
    /// Set request timeout
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
    
    /// Enable buffer pooling with default configuration
    pub fn enable_buffer_pool(mut self) -> Self {
        self.parser.optimization_strategy = OptimizationStrategy::BufferPool(BufferPoolConfig::default());
        self
    }
    
    /// Set custom buffer pool configuration
    pub fn buffer_pool(mut self, config: BufferPoolConfig) -> Self {
        self.parser.optimization_strategy = OptimizationStrategy::BufferPool(config);
        self
    }
    
    /// Set buffer pool size (convenience method)
    pub fn buffer_pool_size(mut self, max_buffers: usize) -> Self {
        if let OptimizationStrategy::BufferPool(ref mut config) = self.parser.optimization_strategy {
            config.max_buffers = max_buffers;
        } else {
            self.parser.optimization_strategy = OptimizationStrategy::BufferPool(
                BufferPoolConfig {
                    max_buffers,
                    ..BufferPoolConfig::default()
                }
            );
        }
        self
    }
    
    /// Set maximum message size
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.parser.max_message_size = size;
        self
    }
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser configuration for optimization strategies
///
/// This configuration allows progressive optimization from simple
/// per-request allocation to advanced buffer pooling.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserConfig {
    /// Optimization strategy for buffer management
    pub optimization_strategy: OptimizationStrategy,
    
    /// Maximum message size in bytes (16 MB default)
    pub max_message_size: usize,
}

impl ParserConfig {
    /// Create new parser config with no optimization
    pub fn new() -> Self {
        Self {
            optimization_strategy: OptimizationStrategy::None,
            max_message_size: 16 * 1024 * 1024, // 16 MB
        }
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization strategy for buffer management
///
/// Based on principal engineer review findings:
/// - `None`: Simple per-request allocation (default, no contention)
/// - `BufferPool`: Reuse memory buffers for high-throughput scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Simple per-request allocation
    /// 
    /// - No shared state or contention
    /// - 800ns-3.5μs allocation overhead per request
    /// - Recommended for most use cases
    None,
    
    /// Buffer pooling for memory reuse
    /// 
    /// - 80% faster for small messages
    /// - Configurable pool size and buffer size
    /// - Recommended for high-throughput scenarios
    BufferPool(BufferPoolConfig),
}

/// Buffer pool configuration
///
/// Controls memory buffer reuse strategy for optimization.
/// Buffer pooling reuses memory allocations (Vec<u8>) rather than
/// entire parser objects for better performance and lower overhead.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPoolConfig {
    /// Maximum number of buffers to keep in pool
    pub max_buffers: usize,
    
    /// Size of each buffer in bytes
    pub buffer_size: usize,
    
    /// Enable adaptive buffer sizing based on usage patterns
    pub adaptive_sizing: bool,
}

impl BufferPoolConfig {
    /// Create new buffer pool config with sensible defaults
    pub fn new() -> Self {
        Self {
            max_buffers: 100,           // Support 100 concurrent requests
            buffer_size: 8 * 1024,      // 8 KB buffers (typical JSON-RPC message size)
            adaptive_sizing: false,     // Start with fixed sizing
        }
    }
    
    /// Set maximum number of buffers
    pub fn max_buffers(mut self, max: usize) -> Self {
        self.max_buffers = max;
        self
    }
    
    /// Set buffer size in bytes
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    /// Enable adaptive buffer sizing
    pub fn adaptive_sizing(mut self, enabled: bool) -> Self {
        self.adaptive_sizing = enabled;
        self
    }
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_default_config() {
        let config = HttpTransportConfig::new();
        
        assert_eq!(config.bind_address.to_string(), "127.0.0.1:3000");
        assert_eq!(config.max_connections, 1000);
        assert_eq!(config.max_concurrent_requests, 10);
        assert_eq!(config.session_timeout, Duration::from_secs(300));
        assert!(matches!(config.parser.optimization_strategy, OptimizationStrategy::None));
    }

    #[test]
    fn test_builder_pattern() {
        let config = HttpTransportConfig::new()
            .bind_address("0.0.0.0:8080".parse().unwrap())
            .max_connections(5000)
            .session_timeout(Duration::from_secs(600))
            .enable_buffer_pool();

        assert_eq!(config.bind_address.to_string(), "0.0.0.0:8080");
        assert_eq!(config.max_connections, 5000);
        assert_eq!(config.session_timeout, Duration::from_secs(600));
        assert!(matches!(config.parser.optimization_strategy, OptimizationStrategy::BufferPool(_)));
    }

    #[test]
    fn test_buffer_pool_config() {
        let config = BufferPoolConfig::new()
            .max_buffers(200)
            .buffer_size(16 * 1024)
            .adaptive_sizing(true);

        assert_eq!(config.max_buffers, 200);
        assert_eq!(config.buffer_size, 16 * 1024);
        assert!(config.adaptive_sizing);
    }

    #[test]
    fn test_buffer_pool_size_convenience() {
        let config = HttpTransportConfig::new()
            .buffer_pool_size(500);

        if let OptimizationStrategy::BufferPool(pool_config) = config.parser.optimization_strategy {
            assert_eq!(pool_config.max_buffers, 500);
        } else {
            panic!("Expected BufferPool optimization strategy");
        }
    }
}
