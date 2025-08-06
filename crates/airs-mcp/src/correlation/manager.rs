//! Correlation Manager implementation
//!
//! This module provides the main CorrelationManager that handles bidirectional
//! JSON-RPC request/response correlation with timeout management and background cleanup.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use chrono::TimeDelta;
use dashmap::DashMap;
use serde_json::Value;
use tokio::{
    sync::{oneshot, RwLock},
    task::JoinHandle,
    time::{interval, Duration},
};
use tracing::{debug, trace};

use crate::correlation::{
    error::{CorrelationError, CorrelationResult, RequestId},
    types::{PendingRequest, RequestIdGenerator},
};

/// Configuration for the correlation manager
///
/// Controls behavior like cleanup intervals, default timeouts, and capacity limits.
#[derive(Debug, Clone)]
pub struct CorrelationConfig {
    /// Default timeout for requests if not specified
    pub default_timeout: TimeDelta,

    /// How often to run cleanup of expired requests  
    pub cleanup_interval: Duration,

    /// Maximum number of pending requests (0 = unlimited)
    pub max_pending_requests: usize,

    /// Whether to enable detailed tracing
    pub enable_tracing: bool,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            default_timeout: TimeDelta::seconds(30),
            cleanup_interval: Duration::from_secs(5),
            max_pending_requests: 1000,
            enable_tracing: true,
        }
    }
}

/// Main correlation manager for JSON-RPC request/response correlation
///
/// Provides thread-safe management of pending requests with automatic timeout
/// handling and background cleanup. Supports both numeric and string request IDs.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
/// use chrono::TimeDelta;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CorrelationConfig {
///     default_timeout: TimeDelta::seconds(60),
///     max_pending_requests: 500,
///     ..Default::default()
/// };
///
/// let manager = CorrelationManager::new(config).await?;
///
/// // Register a request
/// let (id, receiver) = manager.register_request(
///     Some(TimeDelta::seconds(30)),
///     json!({"method": "test", "params": {}})
/// ).await?;
///
/// // Later, correlate the response
/// manager.correlate_response(&id, Ok(json!({"result": "success"}))).await?;
///
/// manager.shutdown().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CorrelationManager {
    /// Stores pending requests by their ID
    requests: Arc<DashMap<RequestId, PendingRequest>>,

    /// Generates unique request IDs
    id_generator: Arc<RequestIdGenerator>,

    /// Configuration settings
    config: CorrelationConfig,

    /// Background cleanup task handle
    cleanup_task: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// Shutdown signal for background tasks
    shutdown_signal: Arc<AtomicBool>,
}

impl CorrelationManager {
    /// Create a new correlation manager with the given configuration
    ///
    /// This starts the background cleanup task immediately.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the manager
    ///
    /// # Returns
    ///
    /// A new `CorrelationManager` instance ready for use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: CorrelationConfig) -> CorrelationResult<Self> {
        let requests = Arc::new(DashMap::new());
        let id_generator = Arc::new(RequestIdGenerator::new());
        let shutdown_signal = Arc::new(AtomicBool::new(false));

        let manager = Self {
            requests: Arc::clone(&requests),
            id_generator,
            config: config.clone(),
            cleanup_task: Arc::new(RwLock::new(None)),
            shutdown_signal: Arc::clone(&shutdown_signal),
        };

        // Start background cleanup task
        let cleanup_handle = manager.start_cleanup_task().await;
        *manager.cleanup_task.write().await = Some(cleanup_handle);

        if config.enable_tracing {
            debug!("CorrelationManager initialized with config: {:?}", config);
        }

        Ok(manager)
    }

    /// Create a new correlation manager without starting the background cleanup task
    ///
    /// This is useful for testing and benchmarking where you want to control
    /// cleanup timing manually.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the manager
    ///
    /// # Returns
    ///
    /// A new `CorrelationManager` instance without background cleanup
    ///
    /// This method is intended for testing and benchmarking scenarios where
    /// the background cleanup task would interfere with measurements.
    #[doc(hidden)]
    pub async fn new_without_cleanup(config: CorrelationConfig) -> CorrelationResult<Self> {
        let requests = Arc::new(DashMap::new());
        let id_generator = Arc::new(RequestIdGenerator::new());
        let shutdown_signal = Arc::new(AtomicBool::new(false));

        let manager = Self {
            requests: Arc::clone(&requests),
            id_generator,
            config: config.clone(),
            cleanup_task: Arc::new(RwLock::new(None)),
            shutdown_signal: Arc::clone(&shutdown_signal),
        };

        if config.enable_tracing {
            debug!(
                "CorrelationManager initialized (no cleanup task) with config: {:?}",
                config
            );
        }

        Ok(manager)
    }

    /// Register a new request for correlation
    ///
    /// Creates a new request ID, stores the request details, and returns both the ID
    /// and a receiver channel for getting the correlated response.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Optional timeout override (uses config default if None)
    /// * `request_data` - The original request data for debugging context
    ///
    /// # Returns
    ///
    /// A tuple of `(RequestId, oneshot::Receiver<CorrelationResult<Value>>)`
    ///
    /// # Errors
    ///
    /// Returns `CorrelationError::Internal` if unable to register the request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    /// use chrono::TimeDelta;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// let (id, receiver) = manager.register_request(
    ///     Some(TimeDelta::seconds(30)),
    ///     json!({"method": "initialize", "params": {}})
    /// ).await?;
    ///
    /// // Use the receiver to wait for the response
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_request(
        &self,
        timeout: Option<TimeDelta>,
        request_data: Value,
    ) -> CorrelationResult<(RequestId, oneshot::Receiver<CorrelationResult<Value>>)> {
        // Check capacity limit
        if self.config.max_pending_requests > 0
            && self.requests.len() >= self.config.max_pending_requests
        {
            return Err(CorrelationError::Internal {
                message: "Maximum pending requests exceeded".to_string(),
            });
        }

        let (sender, receiver) = oneshot::channel();
        let request_id = self.id_generator.next_id();
        let timeout = timeout.unwrap_or(self.config.default_timeout);

        let pending_request = PendingRequest::new(sender, timeout, request_data.clone());

        if self.config.enable_tracing {
            debug!(
                "Registering request {} with timeout {:?}",
                request_id, timeout
            );
            trace!("Request data: {}", request_data);
        }

        self.requests.insert(request_id.clone(), pending_request);

        Ok((request_id, receiver))
    }

    /// Correlate a response with a pending request
    ///
    /// Finds the pending request by ID and sends the response through its channel.
    /// The request is automatically removed from pending requests.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the request to correlate
    /// * `response` - The response data or error to send
    ///
    /// # Errors
    ///
    /// * `CorrelationError::RequestNotFound` - No pending request with this ID
    /// * `CorrelationError::AlreadyCompleted` - Request was already correlated
    /// * `CorrelationError::ChannelClosed` - Response channel was closed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// # let (id, _receiver) = manager.register_request(None, json!({})).await?;
    /// // Correlate with success response
    /// manager.correlate_response(
    ///     &id,
    ///     Ok(json!({"result": "success"}))
    /// ).await?;
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn correlate_response(
        &self,
        request_id: &RequestId,
        response: CorrelationResult<Value>,
    ) -> CorrelationResult<()> {
        let (_, pending_request) =
            self.requests
                .remove(request_id)
                .ok_or_else(|| CorrelationError::RequestNotFound {
                    id: request_id.clone(),
                })?;

        if self.config.enable_tracing {
            debug!("Correlating response for request {}", request_id);
            if let Ok(ref value) = response {
                trace!("Response data: {}", value);
            }
        }

        pending_request
            .sender
            .send(response)
            .map_err(|_| CorrelationError::ChannelClosed {
                id: request_id.clone(),
                details: "Response channel was closed".to_string(),
            })?;

        Ok(())
    }

    /// Get the current number of pending requests
    ///
    /// # Returns
    ///
    /// The count of requests waiting for correlation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// assert_eq!(manager.pending_count().await, 0);
    ///
    /// let (_id, _receiver) = manager.register_request(None, json!({})).await?;
    /// assert_eq!(manager.pending_count().await, 1);
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pending_count(&self) -> usize {
        self.requests.len()
    }

    /// Cancel a pending request
    ///
    /// Removes the request and sends a cancellation error through its channel.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the request to cancel
    ///
    /// # Errors
    ///
    /// * `CorrelationError::RequestNotFound` - No pending request with this ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// let (id, receiver) = manager.register_request(None, json!({})).await?;
    ///
    /// manager.cancel_request(&id).await?;
    ///
    /// // The receiver will get a cancellation error
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_request(&self, request_id: &RequestId) -> CorrelationResult<()> {
        let (_, pending_request) =
            self.requests
                .remove(request_id)
                .ok_or_else(|| CorrelationError::RequestNotFound {
                    id: request_id.clone(),
                })?;

        if self.config.enable_tracing {
            debug!("Cancelling request {}", request_id);
        }

        // Send cancellation error through the channel
        let _ = pending_request
            .sender
            .send(Err(CorrelationError::Cancelled {
                id: request_id.clone(),
            }));

        Ok(())
    }

    /// Check if a request is currently pending
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID to check
    ///
    /// # Returns
    ///
    /// `true` if the request is pending, `false` otherwise
    pub async fn is_pending(&self, request_id: &RequestId) -> bool {
        self.requests.contains_key(request_id)
    }

    /// Get all currently pending request IDs
    ///
    /// # Returns
    ///
    /// A vector of all pending request IDs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// let (_id1, _receiver1) = manager.register_request(None, json!({})).await?;
    /// let (_id2, _receiver2) = manager.register_request(None, json!({})).await?;
    ///
    /// let pending_ids = manager.get_pending_request_ids().await;
    /// assert_eq!(pending_ids.len(), 2);
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_pending_request_ids(&self) -> Vec<RequestId> {
        self.requests
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Manually trigger cleanup of expired requests
    ///
    /// This is called automatically by the background cleanup task, but can also
    /// be called manually for immediate cleanup.
    ///
    /// # Returns
    ///
    /// The number of requests that were cleaned up
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    /// let cleaned_count = manager.cleanup_expired_requests().await;
    /// println!("Cleaned up {} expired requests", cleaned_count);
    /// # manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cleanup_expired_requests(&self) -> usize {
        // Cache timestamp to avoid repeated system calls
        let now = chrono::Utc::now();

        // True single-pass: collect expired IDs while iterating, then batch remove
        // This avoids cloning IDs during iteration
        let expired_ids: Vec<RequestId> = self
            .requests
            .iter()
            .filter_map(|entry| {
                if entry.value().is_expired_at(&now) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        let cleaned_count = expired_ids.len();

        if cleaned_count > 0 {
            if self.config.enable_tracing {
                debug!("Cleaning up {} expired requests", cleaned_count);
            }

            // Batch remove and notify in single pass
            for request_id in expired_ids {
                if let Some((_, pending_request)) = self.requests.remove(&request_id) {
                    let _ = pending_request.sender.send(Err(CorrelationError::Timeout {
                        id: request_id.clone(),
                        duration: pending_request.timeout,
                    }));

                    if self.config.enable_tracing {
                        trace!("Request {} timed out", request_id);
                    }
                }
            }
        }

        cleaned_count
    }

    /// Start the background cleanup task
    ///
    /// This task runs periodically to clean up expired requests automatically.
    async fn start_cleanup_task(&self) -> JoinHandle<()> {
        let requests = Arc::clone(&self.requests);
        let cleanup_interval = self.config.cleanup_interval;
        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        let enable_tracing = self.config.enable_tracing;

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

            while !shutdown_signal.load(Ordering::Relaxed) {
                interval.tick().await;

                let mut expired_requests = Vec::new();

                // Cache timestamp for consistent, efficient expiration checking
                let now = chrono::Utc::now();

                // Single pass: find expired requests with cached timestamp
                for entry in requests.iter() {
                    let pending_request = entry.value();
                    if pending_request.is_expired_at(&now) {
                        expired_requests.push(entry.key().clone());
                    }
                }

                let cleanup_count = expired_requests.len();

                if cleanup_count > 0 {
                    if enable_tracing {
                        debug!(
                            "Background cleanup: processing {} expired requests",
                            cleanup_count
                        );
                    }

                    // Remove expired requests and send timeout errors
                    for request_id in expired_requests {
                        if let Some((_, pending_request)) = requests.remove(&request_id) {
                            let _ = pending_request.sender.send(Err(CorrelationError::Timeout {
                                id: request_id.clone(),
                                duration: pending_request.timeout,
                            }));

                            if enable_tracing {
                                trace!("Background cleanup: request {} timed out", request_id);
                            }
                        }
                    }
                }
            }

            if enable_tracing {
                debug!("Background cleanup task shutting down");
            }
        })
    }

    /// Shutdown the correlation manager
    ///
    /// Stops the background cleanup task and cancels all pending requests.
    /// This should be called when the manager is no longer needed.
    ///
    /// # Errors
    ///
    /// Returns `CorrelationError::Internal` if unable to join the cleanup task
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::manager::{CorrelationManager, CorrelationConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = CorrelationManager::new(CorrelationConfig::default()).await?;
    ///
    /// // ... use the manager ...
    ///
    /// manager.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(self) -> CorrelationResult<()> {
        if self.config.enable_tracing {
            debug!("Shutting down CorrelationManager");
        }

        // Signal shutdown to background task
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Wait for cleanup task to finish
        if let Some(cleanup_handle) = self.cleanup_task.write().await.take() {
            cleanup_handle
                .await
                .map_err(|e| CorrelationError::Internal {
                    message: format!("Failed to join cleanup task: {e}"),
                })?;
        }

        // Cancel all remaining requests
        let pending_ids: Vec<_> = self
            .requests
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for request_id in pending_ids {
            if let Some((_, pending_request)) = self.requests.remove(&request_id) {
                let _ = pending_request
                    .sender
                    .send(Err(CorrelationError::Cancelled { id: request_id }));
            }
        }

        if self.config.enable_tracing {
            debug!("CorrelationManager shutdown complete");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test basic manager lifecycle
    #[tokio::test]
    async fn test_manager_lifecycle() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;
        assert_eq!(manager.pending_count().await, 0);

        manager.shutdown().await?;
        Ok(())
    }

    /// Test request registration and correlation
    #[tokio::test]
    async fn test_request_registration_and_correlation() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;

        // Register a request
        let request_data = json!({"method": "test", "params": {}});
        let (request_id, receiver) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data)
            .await?;

        assert_eq!(manager.pending_count().await, 1);
        assert!(manager.is_pending(&request_id).await);

        // Correlate with a response
        let response_data = json!({"result": "success"});
        manager
            .correlate_response(&request_id, Ok(response_data.clone()))
            .await?;

        // Verify the response was received
        let received_response = receiver.await.unwrap()?;
        assert_eq!(received_response, response_data);

        assert_eq!(manager.pending_count().await, 0);
        assert!(!manager.is_pending(&request_id).await);

        manager.shutdown().await?;
        Ok(())
    }

    /// Test request timeout functionality
    #[tokio::test]
    async fn test_request_timeout() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            cleanup_interval: Duration::from_millis(100),
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;

        // Register a request with a very short timeout
        let request_data = json!({"method": "test"});
        let (request_id, receiver) = manager
            .register_request(Some(TimeDelta::milliseconds(50)), request_data)
            .await?;

        assert_eq!(manager.pending_count().await, 1);

        // Wait for timeout to occur
        sleep(Duration::from_millis(200)).await;

        // Verify the request was cleaned up
        assert_eq!(manager.pending_count().await, 0);

        // Verify we get a timeout error
        let result = receiver.await.unwrap();
        match result {
            Err(CorrelationError::Timeout { id, .. }) => {
                assert_eq!(id, request_id);
            }
            _ => panic!("Expected timeout error, got: {result:?}"),
        }

        manager.shutdown().await?;
        Ok(())
    }

    /// Test request cancellation
    #[tokio::test]
    async fn test_request_cancellation() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;

        // Register a request
        let request_data = json!({"method": "test"});
        let (request_id, receiver) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data)
            .await?;

        assert_eq!(manager.pending_count().await, 1);

        // Cancel the request
        manager.cancel_request(&request_id).await?;

        assert_eq!(manager.pending_count().await, 0);

        // Verify we get a cancellation error
        let result = receiver.await.unwrap();
        match result {
            Err(CorrelationError::Cancelled { id }) => {
                assert_eq!(id, request_id);
            }
            _ => panic!("Expected cancellation error, got: {result:?}"),
        }

        manager.shutdown().await?;
        Ok(())
    }

    /// Test concurrent request handling
    #[tokio::test]
    async fn test_concurrent_requests() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            enable_tracing: false,
            ..Default::default()
        };

        let manager = std::sync::Arc::new(CorrelationManager::new(config).await?);

        // Spawn multiple concurrent requests
        let mut handles = Vec::new();
        for i in 0..5 {
            let manager_clone = std::sync::Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let request_data = json!({"method": "test", "id": i});
                let (request_id, receiver) = manager_clone
                    .register_request(Some(TimeDelta::seconds(30)), request_data)
                    .await?;

                // Simulate some processing time
                sleep(Duration::from_millis(10)).await;

                // Correlate response
                let response_data = json!({"result": format!("response_{}", i)});
                manager_clone
                    .correlate_response(&request_id, Ok(response_data.clone()))
                    .await?;

                // Verify response
                let received = receiver.await.unwrap()?;
                assert_eq!(received, response_data);

                CorrelationResult::Ok(())
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            handle.await.unwrap()?;
        }

        assert_eq!(manager.pending_count().await, 0);

        // Use Arc to access the manager for shutdown
        let manager = std::sync::Arc::try_unwrap(manager).unwrap();
        manager.shutdown().await?;
        Ok(())
    }

    /// Test maximum pending requests limit
    #[tokio::test]
    async fn test_max_pending_requests() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            max_pending_requests: 2,
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;

        // Register two requests (should succeed)
        let request_data = json!({"method": "test"});
        let (_id1, _receiver1) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data.clone())
            .await?;
        let (_id2, _receiver2) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data.clone())
            .await?;

        assert_eq!(manager.pending_count().await, 2);

        // Third request should fail
        let result = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data)
            .await;

        match result {
            Err(CorrelationError::Internal { message }) => {
                assert!(message.contains("Maximum pending requests exceeded"));
            }
            _ => panic!(
                "Expected Internal error for max requests, got: {result:?}"
            ),
        }

        manager.shutdown().await?;
        Ok(())
    }

    /// Test shutdown with pending requests
    #[tokio::test]
    async fn test_shutdown_with_pending_requests() -> CorrelationResult<()> {
        let config = CorrelationConfig {
            enable_tracing: false,
            ..Default::default()
        };

        let manager = CorrelationManager::new(config).await?;

        // Register some requests
        let request_data = json!({"method": "test"});
        let (_id1, receiver1) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data.clone())
            .await?;
        let (_id2, receiver2) = manager
            .register_request(Some(TimeDelta::seconds(30)), request_data)
            .await?;

        assert_eq!(manager.pending_count().await, 2);

        // Shutdown should cancel all pending requests
        manager.shutdown().await?;

        // Verify cancellation errors were sent
        let result1 = receiver1.await.unwrap();
        let result2 = receiver2.await.unwrap();

        assert!(matches!(result1, Err(CorrelationError::Cancelled { .. })));
        assert!(matches!(result2, Err(CorrelationError::Cancelled { .. })));

        Ok(())
    }
}
