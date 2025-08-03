//! Monitoring and observability domain
//!
//! This module contains data structures for configuring monitoring, logging,
//! metrics collection, alerting, and distributed tracing systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Monitoring and observability setup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringSetup {
    /// Logging configuration
    pub logging: LoggingConfig,

    /// Metrics collection
    pub metrics: MetricsConfig,

    /// Alerting rules
    pub alerting: AlertingConfig,

    /// Distributed tracing
    pub tracing: Option<TracingConfig>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,

    /// Log format
    pub format: String,

    /// Log destinations
    pub destinations: Vec<String>,

    /// Retention policy
    pub retention: String,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsConfig {
    /// Metrics system (Prometheus, etc.)
    pub system: String,

    /// Collection interval
    pub interval: String,

    /// Custom metrics
    pub custom_metrics: Vec<String>,

    /// Dashboards
    pub dashboards: Vec<String>,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertingConfig {
    /// Alerting system
    pub system: String,

    /// Alert rules
    pub rules: Vec<AlertRule>,

    /// Notification channels
    pub channels: Vec<String>,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertRule {
    /// Rule name
    pub name: String,

    /// Condition that triggers the alert
    pub condition: String,

    /// Alert severity
    pub severity: String,

    /// Alert description
    pub description: String,

    /// Runbook or resolution steps
    pub runbook: Option<String>,
}

/// Distributed tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TracingConfig {
    /// Tracing system (Jaeger, Zipkin, etc.)
    pub system: String,

    /// Sampling rate
    pub sampling_rate: f64,

    /// Service name
    pub service_name: String,

    /// Additional configuration
    pub configuration: HashMap<String, String>,
}
