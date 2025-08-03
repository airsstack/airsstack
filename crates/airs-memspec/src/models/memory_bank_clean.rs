//! Memory bank data structures for workspace and sub-project organization
//!
//! This module provides a compatibility layer for the refactored memory bank system.
//! All types have been moved to domain-specific modules for better organization.
//!
//! # Migration Guide
//!
//! The memory bank module has been refactored into domain-specific modules:
//! - `workspace` - Workspace-level configuration and context
//! - `sub_project` - Individual project management
//! - `system` - System architecture and technical decisions
//! - `tech` - Technology context and infrastructure
//! - `monitoring` - Observability and monitoring setup
//! - `progress` - Progress tracking and metrics
//! - `testing` - Testing and quality assurance
//! - `review` - Code review management
//! - `task_management` - Comprehensive task tracking
//! - `types` - Shared enumerations and types
//!
//! # Memory Bank Hierarchy
//!
//! ```text
//! Workspace
//! ├── workspace/ (shared configuration)
//! ├── current_context.md (active project tracking)
//! ├── context_snapshots/ (historical states)
//! └── sub_projects/
//!     └── project_name/
//!         ├── project_brief.md
//!         ├── active_context.md
//!         ├── progress.md
//!         └── tasks/
//! ```
//!
//! # Design Principles
//!
//! - **Type Safety**: All data structures use Rust's type system for validation
//! - **Serde Integration**: Full serialization support for JSON and YAML
//! - **Extensibility**: Designed to accommodate future memory bank features
//! - **Validation**: Built-in validation for data consistency and integrity
//! - **Domain Separation**: Clear separation of concerns across logical domains

// Re-export all types from the new domain modules for backward compatibility

// Workspace domain
pub use super::workspace::{
    Workspace, WorkspaceMetadata, SharedPatterns, Pattern, SharedUtility,
    CurrentContext, ContextSnapshot
};

// Sub-project domain
pub use super::sub_project::{
    SubProject, SubProjectMetadata, ProductContext, ActiveContext, Change, Blocker
};

// System architecture domain
pub use super::system::{
    SystemPatterns, ArchitectureDescription, Component, Integration,
    TechnicalDecision, ComponentRelationship
};

// Technology domain
pub use super::tech::{
    TechContext, Technology, DevelopmentSetup, TechnicalConstraint,
    DependencyManagement, DeploymentContext, Environment, InfrastructureRequirement
};

// Monitoring domain
pub use super::monitoring::{
    MonitoringSetup, LoggingConfig, MetricsConfig, AlertingConfig,
    AlertRule, TracingConfig
};

// Progress tracking domain
pub use super::progress::{
    Progress, WorkingComponent, WorkItem, Issue, Milestone, ProgressMetrics
};

// Testing domain
pub use super::testing::{
    TestingInfo, TestType, TestResults, TestFailure, PerformanceResults,
    BenchmarkResult, PerformanceStatus, ManualTestItem, ManualTestStatus
};

// Code review domain
pub use super::review::{CodeReviewInfo, ReviewStatus};

// Task management domain
pub use super::task_management::{
    TaskCollection, TaskIndex, TaskStatistics, Task, TaskDetails,
    TaskProgress, Subtask, ProgressLogEntry, TaskMetadata, TaskProgressSummary
};

// Shared types
pub use super::types::{
    Priority, ProgressStatus, IssueSeverity, TaskStatus, SubtaskStatus,
    ProgressLogType, ProjectStatus
};
