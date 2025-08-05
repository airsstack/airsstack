//! Progress tracking and status management domain
//!
//! This module contains data structures for tracking project progress,
//! managing milestones, documenting issues, and measuring progress metrics.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{IssueSeverity, Priority, ProgressStatus};

/// Progress tracking and status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    /// What currently works
    pub working_components: Vec<WorkingComponent>,

    /// What's left to build
    pub remaining_work: Vec<WorkItem>,

    /// Current overall status
    pub current_status: ProgressStatus,

    /// Known issues and bugs
    pub known_issues: Vec<Issue>,

    /// Completed milestones
    pub completed_milestones: Vec<Milestone>,

    /// Upcoming milestones
    pub upcoming_milestones: Vec<Milestone>,

    /// Progress metrics
    pub metrics: ProgressMetrics,
}

/// Working component status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkingComponent {
    /// Component name
    pub name: String,

    /// Current functionality
    pub functionality: String,

    /// Test coverage status
    pub test_coverage: String,

    /// Performance status
    pub performance: String,

    /// Last validated
    pub last_validated: DateTime<Utc>,

    /// Known limitations
    pub limitations: Vec<String>,
}

/// Work item definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkItem {
    /// Work item name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Estimated effort
    pub estimated_effort: String,

    /// Priority level
    pub priority: Priority,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
}

/// Issue documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    /// Issue title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Issue severity
    pub severity: IssueSeverity,

    /// Steps to reproduce
    pub reproduction_steps: Vec<String>,

    /// Expected behavior
    pub expected_behavior: String,

    /// Actual behavior
    pub actual_behavior: String,

    /// Workarounds
    pub workarounds: Vec<String>,

    /// When issue was discovered
    pub discovered_at: DateTime<Utc>,

    /// Who discovered the issue
    pub discovered_by: String,
}

/// Milestone definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Milestone {
    /// Milestone name
    pub name: String,

    /// Milestone description
    pub description: String,

    /// Target completion date
    pub target_date: Option<DateTime<Utc>>,

    /// Actual completion date
    pub completed_date: Option<DateTime<Utc>>,

    /// Success criteria
    pub success_criteria: Vec<String>,

    /// Associated work items
    pub work_items: Vec<String>,

    /// Progress percentage
    pub progress_percentage: f64,
}

/// Progress metrics and tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressMetrics {
    /// Overall completion percentage
    pub completion_percentage: f64,

    /// Code coverage percentage
    pub code_coverage: Option<f64>,

    /// Test success rate
    pub test_success_rate: Option<f64>,

    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,

    /// Quality metrics
    pub quality_metrics: HashMap<String, f64>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}
