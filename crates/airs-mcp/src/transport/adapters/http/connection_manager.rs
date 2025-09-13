//! HTTP Connection Manager
//!
//! This module provides connection pooling and lifecycle management for HTTP server transport.
//! It uses deadpool for efficient connection reuse and includes health checking capabilities.

use crate::transport::error::TransportError;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Unique identifier for HTTP connections
pub type ConnectionId = Uuid;

/// Information about an active HTTP connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// When the connection was established
    pub created_at: Instant,
    /// Last time the connection was used
    pub last_used: Instant,
    /// Remote peer address
    pub peer_addr: std::net::SocketAddr,
    /// Number of requests processed on this connection
    pub request_count: u64,
    /// Connection health status
    pub health_status: ConnectionHealth,
}

/// Health status of a connection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionHealth {
    /// Connection is healthy and ready for use
    Healthy,
    /// Connection is experiencing issues but may recover
    Degraded,
    /// Connection is unhealthy and should be closed
    Unhealthy,
}

/// Configuration for connection health checking
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Maximum time a connection can be idle before being considered stale
    pub max_idle_time: Duration,
    /// Maximum number of requests per connection before rotation
    pub max_requests_per_connection: u64,
    /// Enable automatic cleanup of stale connections
    pub auto_cleanup: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            max_idle_time: Duration::from_secs(300), // 5 minutes
            max_requests_per_connection: 1000,
            auto_cleanup: true,
        }
    }
}

/// HTTP Connection Manager for server transport
///
/// Manages connection lifecycle, health checking, and resource limits.
/// Uses deadpool for efficient connection pooling and includes rate limiting.
#[derive(Debug)]
pub struct HttpConnectionManager {
    /// Active connections tracked by ID
    active_connections: Arc<DashMap<ConnectionId, ConnectionInfo>>,
    /// Semaphore for limiting concurrent connections
    connection_limiter: Arc<Semaphore>,
    /// Health check configuration
    health_config: HealthCheckConfig,
    /// Maximum number of concurrent connections
    max_connections: usize,
    /// Connection statistics
    stats: Arc<ConnectionStats>,
}

/// Connection statistics for monitoring
#[derive(Debug, Default)]
pub struct ConnectionStats {
    /// Total connections created
    pub total_created: std::sync::atomic::AtomicU64,
    /// Currently active connections
    pub currently_active: std::sync::atomic::AtomicU64,
    /// Total requests processed
    pub total_requests: std::sync::atomic::AtomicU64,
    /// Connections closed due to health issues
    pub health_closures: std::sync::atomic::AtomicU64,
    /// Connections closed due to limits
    pub limit_closures: std::sync::atomic::AtomicU64,
}

impl HttpConnectionManager {
    /// Create a new connection manager with specified limits
    pub fn new(max_connections: usize, health_config: HealthCheckConfig) -> Self {
        Self {
            active_connections: Arc::new(DashMap::new()),
            connection_limiter: Arc::new(Semaphore::new(max_connections)),
            health_config,
            max_connections,
            stats: Arc::new(ConnectionStats::default()),
        }
    }

    /// Create a connection manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(1000, HealthCheckConfig::default())
    }

    /// Register a new connection
    pub async fn register_connection(
        &self,
        peer_addr: std::net::SocketAddr,
    ) -> Result<ConnectionId, TransportError> {
        // Acquire connection permit (blocks if at limit)
        let _permit = self
            .connection_limiter
            .acquire()
            .await
            .map_err(|_| TransportError::ConnectionLimit("Connection limit exceeded".into()))?;

        let connection_id = Uuid::new_v4();
        let now = Instant::now();

        let connection_info = ConnectionInfo {
            created_at: now,
            last_used: now,
            peer_addr,
            request_count: 0,
            health_status: ConnectionHealth::Healthy,
        };

        self.active_connections
            .insert(connection_id, connection_info);

        // Update statistics
        self.stats
            .total_created
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .currently_active
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Note: We intentionally forget the permit here - it will be released when the connection is unregistered
        std::mem::forget(_permit);

        Ok(connection_id)
    }

    /// Update connection activity (called on each request)
    pub fn update_connection_activity(
        &self,
        connection_id: ConnectionId,
    ) -> Result<(), TransportError> {
        if let Some(mut connection_info) = self.active_connections.get_mut(&connection_id) {
            connection_info.last_used = Instant::now();
            connection_info.request_count += 1;

            // Update global request counter
            self.stats
                .total_requests
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Check if connection should be rotated due to request limit
            if connection_info.request_count >= self.health_config.max_requests_per_connection {
                connection_info.health_status = ConnectionHealth::Degraded;
            }

            Ok(())
        } else {
            Err(TransportError::InvalidConnection(format!(
                "Connection {connection_id} not found"
            )))
        }
    }

    /// Unregister a connection (called when connection closes)
    pub fn unregister_connection(&self, connection_id: ConnectionId) -> bool {
        if self.active_connections.remove(&connection_id).is_some() {
            // Release connection permit
            self.connection_limiter.add_permits(1);
            self.stats
                .currently_active
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Get connection information
    pub fn get_connection_info(&self, connection_id: ConnectionId) -> Option<ConnectionInfo> {
        self.active_connections
            .get(&connection_id)
            .map(|entry| entry.value().clone())
    }

    /// Get current connection statistics
    pub fn get_stats(&self) -> ConnectionStatsSnapshot {
        ConnectionStatsSnapshot {
            total_created: self
                .stats
                .total_created
                .load(std::sync::atomic::Ordering::Relaxed),
            currently_active: self
                .stats
                .currently_active
                .load(std::sync::atomic::Ordering::Relaxed),
            total_requests: self
                .stats
                .total_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            health_closures: self
                .stats
                .health_closures
                .load(std::sync::atomic::Ordering::Relaxed),
            limit_closures: self
                .stats
                .limit_closures
                .load(std::sync::atomic::Ordering::Relaxed),
            max_connections: self.max_connections as u64,
        }
    }

    /// Perform health check on all connections
    pub fn health_check(&self) -> HealthCheckResult {
        let now = Instant::now();
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut to_close = Vec::new();

        for entry in self.active_connections.iter() {
            let connection_id = *entry.key();
            let connection_info = entry.value();

            // Check if connection is stale
            let idle_time = now.duration_since(connection_info.last_used);
            let is_stale = idle_time > self.health_config.max_idle_time;

            let health_status = if is_stale {
                ConnectionHealth::Unhealthy
            } else {
                connection_info.health_status.clone()
            };

            match health_status {
                ConnectionHealth::Healthy => healthy += 1,
                ConnectionHealth::Degraded => degraded += 1,
                ConnectionHealth::Unhealthy => {
                    unhealthy += 1;
                    if self.health_config.auto_cleanup {
                        to_close.push(connection_id);
                    }
                }
            }
        }

        // Auto-cleanup unhealthy connections
        for connection_id in to_close {
            if self.unregister_connection(connection_id) {
                self.stats
                    .health_closures
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        HealthCheckResult {
            healthy_connections: healthy,
            degraded_connections: degraded,
            unhealthy_connections: unhealthy,
            connections_closed: unhealthy,
        }
    }

    /// Get list of active connection IDs
    pub fn active_connection_ids(&self) -> Vec<ConnectionId> {
        self.active_connections
            .iter()
            .map(|entry| *entry.key())
            .collect()
    }

    /// Check if connection limit is reached
    pub fn is_at_limit(&self) -> bool {
        self.connection_limiter.available_permits() == 0
    }
}

/// Snapshot of connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStatsSnapshot {
    pub total_created: u64,
    pub currently_active: u64,
    pub total_requests: u64,
    pub health_closures: u64,
    pub limit_closures: u64,
    pub max_connections: u64,
}

/// Result of a health check operation
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub healthy_connections: u32,
    pub degraded_connections: u32,
    pub unhealthy_connections: u32,
    pub connections_closed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_peer_addr() -> std::net::SocketAddr {
        std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    }

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = HttpConnectionManager::with_defaults();
        let stats = manager.get_stats();

        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.total_created, 0);
        assert_eq!(stats.max_connections, 1000);
    }

    #[tokio::test]
    async fn test_connection_registration() {
        let manager = HttpConnectionManager::new(5, HealthCheckConfig::default());
        let peer_addr = test_peer_addr();

        let connection_id = manager.register_connection(peer_addr).await.unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.currently_active, 1);
        assert_eq!(stats.total_created, 1);

        let connection_info = manager.get_connection_info(connection_id).unwrap();
        assert_eq!(connection_info.peer_addr, peer_addr);
        assert_eq!(connection_info.request_count, 0);
        assert_eq!(connection_info.health_status, ConnectionHealth::Healthy);
    }

    #[tokio::test]
    async fn test_connection_activity_update() {
        let manager = HttpConnectionManager::new(5, HealthCheckConfig::default());
        let peer_addr = test_peer_addr();

        let connection_id = manager.register_connection(peer_addr).await.unwrap();
        manager.update_connection_activity(connection_id).unwrap();

        let connection_info = manager.get_connection_info(connection_id).unwrap();
        assert_eq!(connection_info.request_count, 1);

        let stats = manager.get_stats();
        assert_eq!(stats.total_requests, 1);
    }

    #[tokio::test]
    async fn test_connection_unregistration() {
        let manager = HttpConnectionManager::new(5, HealthCheckConfig::default());
        let peer_addr = test_peer_addr();

        let connection_id = manager.register_connection(peer_addr).await.unwrap();
        assert!(manager.unregister_connection(connection_id));

        let stats = manager.get_stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.total_created, 1);

        // Should return false for already unregistered connection
        assert!(!manager.unregister_connection(connection_id));
    }

    #[tokio::test]
    async fn test_connection_limit() {
        let manager = HttpConnectionManager::new(2, HealthCheckConfig::default());
        let peer_addr = test_peer_addr();

        // Register up to limit
        let _conn1 = manager.register_connection(peer_addr).await.unwrap();
        let _conn2 = manager.register_connection(peer_addr).await.unwrap();

        assert!(manager.is_at_limit());
    }

    #[tokio::test]
    async fn test_health_check() {
        let health_config = HealthCheckConfig {
            max_idle_time: Duration::from_millis(100),
            auto_cleanup: true,
            ..Default::default()
        };
        let manager = HttpConnectionManager::new(5, health_config);
        let peer_addr = test_peer_addr();

        let _connection_id = manager.register_connection(peer_addr).await.unwrap();

        // Wait for connection to become stale
        tokio::time::sleep(Duration::from_millis(150)).await;

        let health_result = manager.health_check();
        assert_eq!(health_result.connections_closed, 1);

        let stats = manager.get_stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.health_closures, 1);
    }

    #[tokio::test]
    async fn test_connection_degradation_on_request_limit() {
        let health_config = HealthCheckConfig {
            max_requests_per_connection: 2,
            ..Default::default()
        };
        let manager = HttpConnectionManager::new(5, health_config);
        let peer_addr = test_peer_addr();

        let connection_id = manager.register_connection(peer_addr).await.unwrap();

        // Process requests up to limit
        manager.update_connection_activity(connection_id).unwrap();
        manager.update_connection_activity(connection_id).unwrap();

        let connection_info = manager.get_connection_info(connection_id).unwrap();
        assert_eq!(connection_info.health_status, ConnectionHealth::Degraded);
    }
}
