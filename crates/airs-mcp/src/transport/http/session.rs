//! HTTP Session Management
//!
//! This module provides session management for HTTP server transport,
//! including session correlation, state management, and integration with
//! the existing correlation system.

use crate::correlation::CorrelationManager;
use crate::transport::error::TransportError;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Unique identifier for HTTP sessions
pub type SessionId = Uuid;

/// HTTP Session context containing correlation and state information
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: SessionId,
    /// When the session was created
    pub created_at: Instant,
    /// Last time the session was accessed
    pub last_accessed: Instant,
    /// Client information
    pub client_info: ClientInfo,
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// Client information for session tracking
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client's remote address
    pub remote_addr: std::net::SocketAddr,
    /// User-Agent header if provided
    pub user_agent: Option<String>,
    /// MCP client capabilities (if negotiated)
    pub client_capabilities: Option<String>,
}

/// Session metadata for additional context
#[derive(Debug, Clone, Default)]
pub struct SessionMetadata {
    /// Number of requests processed in this session
    pub request_count: u64,
    /// Custom metadata fields
    pub custom_fields: std::collections::HashMap<String, String>,
}

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum session idle time before cleanup
    pub max_idle_time: Duration,
    /// Interval for session cleanup task
    pub cleanup_interval: Duration,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Enable automatic session cleanup
    pub auto_cleanup: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_idle_time: Duration::from_secs(1800),  // 30 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
            max_sessions: 10000,
            auto_cleanup: true,
        }
    }
}

/// HTTP Session Manager
///
/// Manages HTTP sessions for the server transport, providing correlation
/// with the existing correlation system and automatic cleanup of stale sessions.
pub struct SessionManager {
    /// Active sessions indexed by session ID
    sessions: Arc<DashMap<SessionId, SessionContext>>,
    /// Correlation manager for request/response matching
    correlation_manager: Arc<CorrelationManager>,
    /// Session configuration
    config: SessionConfig,
    /// Background cleanup task handle
    cleanup_task: Option<JoinHandle<()>>,
    /// Session statistics
    stats: Arc<SessionStats>,
}

/// Session statistics for monitoring
#[derive(Debug, Default)]
pub struct SessionStats {
    /// Total sessions created
    pub total_created: std::sync::atomic::AtomicU64,
    /// Currently active sessions
    pub currently_active: std::sync::atomic::AtomicU64,
    /// Total requests processed across all sessions
    pub total_requests: std::sync::atomic::AtomicU64,
    /// Sessions cleaned up due to timeout
    pub timeout_cleanups: std::sync::atomic::AtomicU64,
    /// Sessions manually closed
    pub manual_closures: std::sync::atomic::AtomicU64,
}

impl SessionManager {
    /// Create a new session manager with the given configuration
    pub fn new(correlation_manager: Arc<CorrelationManager>, config: SessionConfig) -> Self {
        let sessions = Arc::new(DashMap::new());
        let stats = Arc::new(SessionStats::default());

        let mut manager = Self {
            sessions,
            correlation_manager,
            config,
            cleanup_task: None,
            stats,
        };

        // Start cleanup task if auto-cleanup is enabled
        if manager.config.auto_cleanup {
            manager.start_cleanup_task();
        }

        manager
    }

    /// Create a session manager with default configuration
    pub fn with_defaults(correlation_manager: Arc<CorrelationManager>) -> Self {
        Self::new(correlation_manager, SessionConfig::default())
    }

    /// Create a new session for a client
    pub fn create_session(&self, client_info: ClientInfo) -> Result<SessionId, TransportError> {
        // Check session limit
        if self.sessions.len() >= self.config.max_sessions {
            return Err(TransportError::session_error(format!(
                "Session limit exceeded: {}",
                self.config.max_sessions
            )));
        }

        let session_id = Uuid::new_v4();
        let now = Instant::now();

        let session_context = SessionContext {
            session_id,
            created_at: now,
            last_accessed: now,
            client_info,
            metadata: SessionMetadata::default(),
        };

        self.sessions.insert(session_id, session_context);

        // Update statistics
        self.stats
            .total_created
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .currently_active
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(session_id)
    }

    /// Get session context by ID
    pub fn get_session(&self, session_id: SessionId) -> Option<SessionContext> {
        self.sessions.get(&session_id).map(|entry| {
            let mut context = entry.value().clone();
            context.last_accessed = Instant::now();
            drop(entry);

            // Update the last accessed time in storage
            if let Some(mut stored_context) = self.sessions.get_mut(&session_id) {
                stored_context.last_accessed = context.last_accessed;
            }

            context
        })
    }

    /// Update session activity (called on each request)
    pub fn update_session_activity(&self, session_id: SessionId) -> Result<(), TransportError> {
        if let Some(mut session_context) = self.sessions.get_mut(&session_id) {
            session_context.last_accessed = Instant::now();
            session_context.metadata.request_count += 1;

            // Update global request counter
            self.stats
                .total_requests
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            Ok(())
        } else {
            Err(TransportError::session_error(format!(
                "Session {session_id} not found"
            )))
        }
    }

    /// Close a session manually
    pub fn close_session(&self, session_id: SessionId) -> bool {
        if self.sessions.remove(&session_id).is_some() {
            self.stats
                .currently_active
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            self.stats
                .manual_closures
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Get list of active session IDs
    pub fn active_session_ids(&self) -> Vec<SessionId> {
        self.sessions.iter().map(|entry| *entry.key()).collect()
    }

    /// Get current session statistics
    pub fn get_stats(&self) -> SessionStatsSnapshot {
        SessionStatsSnapshot {
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
            timeout_cleanups: self
                .stats
                .timeout_cleanups
                .load(std::sync::atomic::Ordering::Relaxed),
            manual_closures: self
                .stats
                .manual_closures
                .load(std::sync::atomic::Ordering::Relaxed),
            max_sessions: self.config.max_sessions as u64,
        }
    }

    /// Perform cleanup of stale sessions
    pub fn cleanup_stale_sessions(&self) -> SessionCleanupResult {
        let now = Instant::now();
        let mut cleaned_up = 0;
        let mut to_remove = Vec::new();

        // Identify stale sessions
        for entry in self.sessions.iter() {
            let session_id = *entry.key();
            let session_context = entry.value();

            let idle_time = now.duration_since(session_context.last_accessed);
            if idle_time > self.config.max_idle_time {
                to_remove.push(session_id);
            }
        }

        // Remove stale sessions
        for session_id in to_remove {
            if self.sessions.remove(&session_id).is_some() {
                cleaned_up += 1;
                self.stats
                    .currently_active
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                self.stats
                    .timeout_cleanups
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        SessionCleanupResult {
            sessions_cleaned: cleaned_up,
            sessions_remaining: self.sessions.len() as u32,
        }
    }

    /// Start the background cleanup task
    fn start_cleanup_task(&mut self) {
        let sessions = Arc::clone(&self.sessions);
        let config = self.config.clone();
        let stats = Arc::clone(&self.stats);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);

            loop {
                interval.tick().await;

                let now = Instant::now();
                let mut cleaned_up = 0;
                let mut to_remove = Vec::new();

                // Identify stale sessions
                for entry in sessions.iter() {
                    let session_id = *entry.key();
                    let session_context = entry.value();

                    let idle_time = now.duration_since(session_context.last_accessed);
                    if idle_time > config.max_idle_time {
                        to_remove.push(session_id);
                    }
                }

                // Remove stale sessions
                for session_id in to_remove {
                    if sessions.remove(&session_id).is_some() {
                        cleaned_up += 1;
                        stats
                            .currently_active
                            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        stats
                            .timeout_cleanups
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }

                if cleaned_up > 0 {
                    tracing::debug!("Cleaned up {} stale sessions", cleaned_up);
                }
            }
        });

        self.cleanup_task = Some(task);
    }

    /// Stop the background cleanup task
    pub async fn stop_cleanup_task(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
            let _ = task.await;
        }
    }

    /// Get access to the correlation manager
    pub fn correlation_manager(&self) -> &Arc<CorrelationManager> {
        &self.correlation_manager
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Abort cleanup task if it exists
        if let Some(task) = &self.cleanup_task {
            task.abort();
        }
    }
}

/// Snapshot of session statistics
#[derive(Debug, Clone)]
pub struct SessionStatsSnapshot {
    pub total_created: u64,
    pub currently_active: u64,
    pub total_requests: u64,
    pub timeout_cleanups: u64,
    pub manual_closures: u64,
    pub max_sessions: u64,
}

/// Result of a session cleanup operation
#[derive(Debug, Clone)]
pub struct SessionCleanupResult {
    pub sessions_cleaned: u32,
    pub sessions_remaining: u32,
}

/// Extract session ID from HTTP headers
pub fn extract_session_id(headers: &axum::http::HeaderMap) -> Result<SessionId, TransportError> {
    let session_header = headers
        .get("Mcp-Session-Id")
        .or_else(|| headers.get("mcp-session-id"))
        .ok_or_else(|| TransportError::session_error("Missing Mcp-Session-Id header"))?;

    let session_str = session_header
        .to_str()
        .map_err(|_| TransportError::session_error("Invalid Mcp-Session-Id header format"))?;

    Uuid::parse_str(session_str)
        .map_err(|_| TransportError::session_error("Invalid Mcp-Session-Id format"))
}

/// Extract optional Last-Event-ID from headers (for SSE reconnection)
pub fn extract_last_event_id(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("Last-Event-ID")
        .or_else(|| headers.get("last-event-id"))
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::CorrelationConfig;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_client_info() -> ClientInfo {
        ClientInfo {
            remote_addr: std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            user_agent: Some("test-client/1.0".to_string()),
            client_capabilities: None,
        }
    }

    async fn test_correlation_manager() -> Arc<CorrelationManager> {
        Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let correlation_manager = test_correlation_manager().await;
        let session_manager = SessionManager::with_defaults(correlation_manager);

        let stats = session_manager.get_stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.total_created, 0);
    }

    #[tokio::test]
    async fn test_session_creation() {
        let correlation_manager = test_correlation_manager().await;
        let session_manager = SessionManager::with_defaults(correlation_manager);
        let client_info = test_client_info();

        let session_id = session_manager.create_session(client_info.clone()).unwrap();

        let stats = session_manager.get_stats();
        assert_eq!(stats.currently_active, 1);
        assert_eq!(stats.total_created, 1);

        let session_context = session_manager.get_session(session_id).unwrap();
        assert_eq!(session_context.session_id, session_id);
        assert_eq!(
            session_context.client_info.remote_addr,
            client_info.remote_addr
        );
        assert_eq!(session_context.metadata.request_count, 0);
    }

    #[tokio::test]
    async fn test_session_activity_update() {
        let correlation_manager = test_correlation_manager().await;
        let session_manager = SessionManager::with_defaults(correlation_manager);
        let client_info = test_client_info();

        let session_id = session_manager.create_session(client_info).unwrap();
        session_manager.update_session_activity(session_id).unwrap();

        let session_context = session_manager.get_session(session_id).unwrap();
        assert_eq!(session_context.metadata.request_count, 1);

        let stats = session_manager.get_stats();
        assert_eq!(stats.total_requests, 1);
    }

    #[tokio::test]
    async fn test_session_closure() {
        let correlation_manager = test_correlation_manager().await;
        let session_manager = SessionManager::with_defaults(correlation_manager);
        let client_info = test_client_info();

        let session_id = session_manager.create_session(client_info).unwrap();
        assert!(session_manager.close_session(session_id));

        let stats = session_manager.get_stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.manual_closures, 1);

        // Should return false for already closed session
        assert!(!session_manager.close_session(session_id));
    }

    #[tokio::test]
    async fn test_session_limit() {
        let correlation_manager = test_correlation_manager().await;
        let config = SessionConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let session_manager = SessionManager::new(correlation_manager, config);
        let client_info = test_client_info();

        // Create sessions up to limit
        let _session1 = session_manager.create_session(client_info.clone()).unwrap();
        let _session2 = session_manager.create_session(client_info.clone()).unwrap();

        // Third session should fail
        let result = session_manager.create_session(client_info);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Session limit exceeded"));
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let correlation_manager = test_correlation_manager().await;
        let config = SessionConfig {
            max_idle_time: Duration::from_millis(100),
            auto_cleanup: false, // Manual cleanup for testing
            ..Default::default()
        };
        let session_manager = SessionManager::new(correlation_manager, config);
        let client_info = test_client_info();

        let _session_id = session_manager.create_session(client_info).unwrap();

        // Wait for session to become stale
        tokio::time::sleep(Duration::from_millis(150)).await;

        let cleanup_result = session_manager.cleanup_stale_sessions();
        assert_eq!(cleanup_result.sessions_cleaned, 1);
        assert_eq!(cleanup_result.sessions_remaining, 0);

        let stats = session_manager.get_stats();
        assert_eq!(stats.currently_active, 0);
        assert_eq!(stats.timeout_cleanups, 1);
    }

    #[test]
    fn test_extract_session_id() {
        let mut headers = axum::http::HeaderMap::new();
        let session_id = Uuid::new_v4();
        headers.insert("Mcp-Session-Id", session_id.to_string().parse().unwrap());

        let extracted = extract_session_id(&headers).unwrap();
        assert_eq!(extracted, session_id);
    }

    #[test]
    fn test_extract_session_id_missing() {
        let headers = axum::http::HeaderMap::new();
        let result = extract_session_id(&headers);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing Mcp-Session-Id header"));
    }

    #[test]
    fn test_extract_last_event_id() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("Last-Event-ID", "12345".parse().unwrap());

        let extracted = extract_last_event_id(&headers);
        assert_eq!(extracted, Some("12345".to_string()));
    }

    #[test]
    fn test_extract_last_event_id_missing() {
        let headers = axum::http::HeaderMap::new();
        let extracted = extract_last_event_id(&headers);
        assert_eq!(extracted, None);
    }
}
