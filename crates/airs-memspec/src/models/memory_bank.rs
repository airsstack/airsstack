//! Memory bank data structures for workspace and sub-project organization
//!
//! This module defines the core data models that represent the Multi-Project Memory Bank
//! structure, including workspace-level configuration, sub-project contexts, and the
//! relationships between different components.
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root workspace configuration and metadata
///
/// Represents the top-level workspace containing multiple sub-projects,
/// shared patterns, and workspace-wide configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    /// Workspace metadata and configuration
    pub metadata: WorkspaceMetadata,

    /// Shared patterns and architectural decisions
    pub shared_patterns: SharedPatterns,

    /// Current active context tracking
    pub current_context: CurrentContext,

    /// Map of sub-project name to sub-project data
    pub sub_projects: HashMap<String, SubProject>,

    /// Historical context snapshots
    pub snapshots: Vec<ContextSnapshot>,
}

/// Workspace-level metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceMetadata {
    /// Workspace name/identifier
    pub name: String,

    /// Brief description of the workspace purpose
    pub description: Option<String>,

    /// Workspace creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// Workspace version for compatibility tracking
    pub version: String,

    /// Root directory path
    pub root_path: PathBuf,

    /// Additional metadata fields
    pub metadata: HashMap<String, String>,
}

/// Shared patterns and architectural decisions across the workspace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedPatterns {
    /// Core implementation patterns
    pub implementation_patterns: Vec<Pattern>,

    /// Architecture and design patterns  
    pub architecture_patterns: Vec<Pattern>,

    /// Methodology and workflow patterns
    pub methodology_patterns: Vec<Pattern>,

    /// Cross-cutting concerns and shared utilities
    pub shared_utilities: Vec<SharedUtility>,
}

/// A documented pattern or practice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    /// Pattern name/identifier
    pub name: String,

    /// Detailed pattern description
    pub description: String,

    /// When to apply this pattern
    pub usage_context: String,

    /// Code examples or templates
    pub examples: Vec<String>,

    /// Related patterns or references
    pub references: Vec<String>,

    /// Pattern category/tags
    pub tags: Vec<String>,
}

/// Shared utility or cross-cutting concern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedUtility {
    /// Utility name
    pub name: String,

    /// Purpose and functionality
    pub description: String,

    /// Location/path to the utility
    pub location: String,

    /// API or usage documentation
    pub usage: String,

    /// Dependencies and requirements
    pub dependencies: Vec<String>,
}

/// Current context tracking for active sub-project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrentContext {
    /// Currently active sub-project
    pub active_sub_project: String,

    /// When context was last switched
    pub switched_on: DateTime<Utc>,

    /// Who/what triggered the context switch
    pub switched_by: String,

    /// Current status/phase description
    pub status: String,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

impl Workspace {
    /// Create a new workspace with default configuration
    pub fn new(name: String, root_path: PathBuf) -> Self {
        let now = Utc::now();

        Self {
            metadata: WorkspaceMetadata {
                name: name.clone(),
                description: None,
                created_at: now,
                updated_at: now,
                version: "1.0.0".to_string(),
                root_path,
                metadata: HashMap::new(),
            },
            shared_patterns: SharedPatterns {
                implementation_patterns: Vec::new(),
                architecture_patterns: Vec::new(),
                methodology_patterns: Vec::new(),
                shared_utilities: Vec::new(),
            },
            current_context: CurrentContext {
                active_sub_project: String::new(),
                switched_on: now,
                switched_by: "system".to_string(),
                status: "initialized".to_string(),
                metadata: HashMap::new(),
            },
            sub_projects: HashMap::new(),
            snapshots: Vec::new(),
        }
    }

    /// Add a new sub-project to the workspace
    pub fn add_sub_project(&mut self, name: String, sub_project: SubProject) {
        self.sub_projects.insert(name, sub_project);
        self.metadata.updated_at = Utc::now();
    }

    /// Switch active context to a different sub-project
    pub fn switch_context(
        &mut self,
        sub_project: String,
        switched_by: String,
    ) -> Result<(), String> {
        if !self.sub_projects.contains_key(&sub_project) {
            return Err(format!("Sub-project '{}' not found", sub_project));
        }

        self.current_context = CurrentContext {
            active_sub_project: sub_project,
            switched_on: Utc::now(),
            switched_by,
            status: "active".to_string(),
            metadata: HashMap::new(),
        };

        self.metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Get the currently active sub-project
    pub fn get_active_sub_project(&self) -> Option<&SubProject> {
        if self.current_context.active_sub_project.is_empty() {
            return None;
        }
        self.sub_projects
            .get(&self.current_context.active_sub_project)
    }

    /// Create a context snapshot for historical tracking
    pub fn create_snapshot(&mut self, description: String) {
        let snapshot = ContextSnapshot {
            timestamp: Utc::now(),
            description,
            active_sub_project: self.current_context.active_sub_project.clone(),
            workspace_state: self.metadata.clone(),
            sub_project_states: self.sub_projects.clone(),
        };

        self.snapshots.push(snapshot);
        self.metadata.updated_at = Utc::now();
    }
}

/// Historical context snapshot for restoration and analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSnapshot {
    /// Snapshot creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Human-readable description
    pub description: String,

    /// Active sub-project at snapshot time
    pub active_sub_project: String,

    /// Workspace state at snapshot time
    pub workspace_state: WorkspaceMetadata,

    /// Sub-project states at snapshot time
    pub sub_project_states: HashMap<String, SubProject>,
}

/// Sub-project configuration and state management
///
/// Represents an individual project within the workspace, containing all the
/// context, progress tracking, and task management for that specific project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubProject {
    /// Sub-project metadata
    pub metadata: SubProjectMetadata,

    /// Product context and requirements
    pub product_context: ProductContext,

    /// System architecture and patterns
    pub system_patterns: SystemPatterns,

    /// Technology context and constraints
    pub tech_context: TechContext,

    /// Current active work context
    pub active_context: ActiveContext,

    /// Progress tracking and status
    pub progress: Progress,

    /// Task management
    pub tasks: TaskCollection,
}

/// Sub-project metadata and identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubProjectMetadata {
    /// Sub-project name/identifier
    pub name: String,

    /// Brief description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// Project version
    pub version: String,

    /// Project status (active, paused, completed, archived)
    pub status: ProjectStatus,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Sub-project status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    /// Project is actively being worked on
    Active,
    /// Project is temporarily paused
    Paused,
    /// Project has been completed
    Completed,
    /// Project has been archived
    Archived,
    /// Project is in planning phase
    Planning,
}

/// Product context defining why the sub-project exists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductContext {
    /// Problems this project solves
    pub problems: Vec<String>,

    /// How the project should work
    pub functionality: Vec<String>,

    /// User experience goals
    pub user_experience: Vec<String>,

    /// Success criteria
    pub success_criteria: Vec<String>,

    /// Target users or stakeholders
    pub target_users: Vec<String>,

    /// Business or technical value
    pub value_proposition: String,
}

/// System architecture and design patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemPatterns {
    /// System architecture description
    pub architecture: ArchitectureDescription,

    /// Key technical decisions
    pub technical_decisions: Vec<TechnicalDecision>,

    /// Design patterns in use
    pub design_patterns: Vec<Pattern>,

    /// Component relationships
    pub component_relationships: Vec<ComponentRelationship>,
}

/// Architecture description and documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectureDescription {
    /// High-level architecture overview
    pub overview: String,

    /// System components
    pub components: Vec<Component>,

    /// Data flow description
    pub data_flow: String,

    /// Integration points
    pub integrations: Vec<Integration>,

    /// Architecture diagrams or references
    pub diagrams: Vec<String>,
}

/// System component definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    /// Component name
    pub name: String,

    /// Component purpose and responsibility
    pub purpose: String,

    /// Interface definition
    pub interface: String,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Location or path
    pub location: String,
}

/// Integration point with external systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Integration {
    /// Integration name
    pub name: String,

    /// External system or service
    pub external_system: String,

    /// Integration type (API, database, file, etc.)
    pub integration_type: String,

    /// Protocol or method
    pub protocol: String,

    /// Configuration requirements
    pub configuration: HashMap<String, String>,
}

/// Technical decision documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalDecision {
    /// Decision title
    pub title: String,

    /// What was decided
    pub decision: String,

    /// Context that led to the decision
    pub context: String,

    /// Alternatives considered
    pub alternatives: Vec<String>,

    /// Rationale for the chosen approach
    pub rationale: String,

    /// Expected impact
    pub impact: String,

    /// Decision timestamp
    pub decided_at: DateTime<Utc>,

    /// Who made the decision
    pub decided_by: String,

    /// Review conditions or schedule
    pub review_criteria: Option<String>,
}

/// Component relationship definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentRelationship {
    /// Source component
    pub from_component: String,

    /// Target component
    pub to_component: String,

    /// Relationship type (depends_on, calls, inherits_from, etc.)
    pub relationship_type: String,

    /// Relationship description
    pub description: String,

    /// Any constraints or conditions
    pub constraints: Vec<String>,
}

/// Technology context and constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechContext {
    /// Technologies used in the project
    pub technologies: Vec<Technology>,

    /// Development setup requirements
    pub development_setup: DevelopmentSetup,

    /// Technical constraints
    pub constraints: Vec<TechnicalConstraint>,

    /// Dependencies and their management
    pub dependencies: DependencyManagement,

    /// Deployment and infrastructure
    pub deployment: DeploymentContext,
}

/// Technology definition and usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technology {
    /// Technology name
    pub name: String,

    /// Version or version range
    pub version: String,

    /// Purpose in the project
    pub purpose: String,

    /// Configuration or setup notes
    pub configuration: String,

    /// Documentation links
    pub documentation: Vec<String>,
}

/// Development environment setup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevelopmentSetup {
    /// Required tools and their versions
    pub required_tools: Vec<Technology>,

    /// Environment variables needed
    pub environment_variables: HashMap<String, String>,

    /// Setup instructions
    pub setup_instructions: Vec<String>,

    /// IDE or editor configuration
    pub ide_configuration: Vec<String>,

    /// Local development scripts
    pub development_scripts: HashMap<String, String>,
}

/// Technical constraint documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalConstraint {
    /// Constraint name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Why this constraint exists
    pub rationale: String,

    /// Impact on implementation
    pub impact: String,

    /// Workarounds or mitigations
    pub mitigations: Vec<String>,
}

/// Dependency management configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyManagement {
    /// Package manager used
    pub package_manager: String,

    /// Lock file location
    pub lock_file: String,

    /// Update policy
    pub update_policy: String,

    /// Security scanning configuration
    pub security_scanning: Option<String>,

    /// Known dependency issues
    pub known_issues: Vec<String>,
}

/// Deployment and infrastructure context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploymentContext {
    /// Target environments
    pub environments: Vec<Environment>,

    /// Deployment strategy
    pub deployment_strategy: String,

    /// Infrastructure requirements
    pub infrastructure: Vec<InfrastructureRequirement>,

    /// Monitoring and observability
    pub monitoring: MonitoringSetup,
}

/// Environment definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    /// Environment name (dev, staging, prod, etc.)
    pub name: String,

    /// Environment purpose
    pub purpose: String,

    /// Configuration differences
    pub configuration: HashMap<String, String>,

    /// Access requirements
    pub access: Vec<String>,

    /// Health check endpoints
    pub health_checks: Vec<String>,
}

/// Infrastructure requirement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InfrastructureRequirement {
    /// Resource type (compute, storage, network, etc.)
    pub resource_type: String,

    /// Specification requirements
    pub specifications: HashMap<String, String>,

    /// Scaling requirements
    pub scaling: Option<String>,

    /// High availability requirements
    pub high_availability: bool,

    /// Security requirements
    pub security_requirements: Vec<String>,
}

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

/// Current active work context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveContext {
    /// Current work focus
    pub current_focus: String,

    /// Recent changes made
    pub recent_changes: Vec<Change>,

    /// Next planned steps
    pub next_steps: Vec<String>,

    /// Active decisions and considerations
    pub active_considerations: Vec<String>,

    /// Blockers or impediments
    pub blockers: Vec<Blocker>,

    /// Context last updated
    pub updated_at: DateTime<Utc>,
}

/// Change documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    /// Change description
    pub description: String,

    /// When the change was made
    pub timestamp: DateTime<Utc>,

    /// Who made the change
    pub author: String,

    /// Files or components affected
    pub affected_components: Vec<String>,

    /// Reason for the change
    pub rationale: String,

    /// Impact assessment
    pub impact: String,
}

/// Blocker or impediment documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blocker {
    /// Blocker description
    pub description: String,

    /// Blocker type (technical, resource, external, etc.)
    pub blocker_type: String,

    /// Impact on progress
    pub impact: String,

    /// Potential solutions or workarounds
    pub potential_solutions: Vec<String>,

    /// Who can help resolve this
    pub escalation_path: Vec<String>,

    /// When this blocker was identified
    pub identified_at: DateTime<Utc>,
}

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

/// Priority enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Progress status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    OnTrack,
    AtRisk,
    Blocked,
    Completed,
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

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Enhancement,
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

/// Task collection and management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCollection {
    /// All tasks in the project
    pub tasks: HashMap<String, Task>,

    /// Task index for quick lookups
    pub task_index: TaskIndex,

    /// Task statistics
    pub statistics: TaskStatistics,
}

/// Task index for organizing and finding tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskIndex {
    /// Tasks by status
    pub by_status: HashMap<TaskStatus, Vec<String>>,

    /// Tasks by priority
    pub by_priority: HashMap<Priority, Vec<String>>,

    /// Tasks by tag
    pub by_tag: HashMap<String, Vec<String>>,

    /// Recently updated tasks
    pub recently_updated: Vec<String>,
}

/// Task statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatistics {
    /// Total number of tasks
    pub total_tasks: usize,

    /// Tasks by status count
    pub status_counts: HashMap<TaskStatus, usize>,

    /// Average completion time
    pub average_completion_time: Option<chrono::Duration>,

    /// Productivity metrics
    pub productivity_metrics: HashMap<String, f64>,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Individual task definition (placeholder - will reference task.rs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    /// Task ID
    pub id: String,

    /// Task name
    pub name: String,

    /// Current status
    pub status: TaskStatus,

    /// Task priority
    pub priority: Priority,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Abandoned,
    Blocked,
}

impl SubProject {
    /// Create a new sub-project with default configuration
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();

        Self {
            metadata: SubProjectMetadata {
                name: name.clone(),
                description,
                created_at: now,
                updated_at: now,
                version: "0.1.0".to_string(),
                status: ProjectStatus::Planning,
                metadata: HashMap::new(),
            },
            product_context: ProductContext {
                problems: Vec::new(),
                functionality: Vec::new(),
                user_experience: Vec::new(),
                success_criteria: Vec::new(),
                target_users: Vec::new(),
                value_proposition: String::new(),
            },
            system_patterns: SystemPatterns {
                architecture: ArchitectureDescription {
                    overview: String::new(),
                    components: Vec::new(),
                    data_flow: String::new(),
                    integrations: Vec::new(),
                    diagrams: Vec::new(),
                },
                technical_decisions: Vec::new(),
                design_patterns: Vec::new(),
                component_relationships: Vec::new(),
            },
            tech_context: TechContext {
                technologies: Vec::new(),
                development_setup: DevelopmentSetup {
                    required_tools: Vec::new(),
                    environment_variables: HashMap::new(),
                    setup_instructions: Vec::new(),
                    ide_configuration: Vec::new(),
                    development_scripts: HashMap::new(),
                },
                constraints: Vec::new(),
                dependencies: DependencyManagement {
                    package_manager: String::new(),
                    lock_file: String::new(),
                    update_policy: String::new(),
                    security_scanning: None,
                    known_issues: Vec::new(),
                },
                deployment: DeploymentContext {
                    environments: Vec::new(),
                    deployment_strategy: String::new(),
                    infrastructure: Vec::new(),
                    monitoring: MonitoringSetup {
                        logging: LoggingConfig {
                            level: "info".to_string(),
                            format: "json".to_string(),
                            destinations: Vec::new(),
                            retention: "30d".to_string(),
                        },
                        metrics: MetricsConfig {
                            system: String::new(),
                            interval: "30s".to_string(),
                            custom_metrics: Vec::new(),
                            dashboards: Vec::new(),
                        },
                        alerting: AlertingConfig {
                            system: String::new(),
                            rules: Vec::new(),
                            channels: Vec::new(),
                        },
                        tracing: None,
                    },
                },
            },
            active_context: ActiveContext {
                current_focus: String::new(),
                recent_changes: Vec::new(),
                next_steps: Vec::new(),
                active_considerations: Vec::new(),
                blockers: Vec::new(),
                updated_at: now,
            },
            progress: Progress {
                working_components: Vec::new(),
                remaining_work: Vec::new(),
                current_status: ProgressStatus::NotStarted,
                known_issues: Vec::new(),
                completed_milestones: Vec::new(),
                upcoming_milestones: Vec::new(),
                metrics: ProgressMetrics {
                    completion_percentage: 0.0,
                    code_coverage: None,
                    test_success_rate: None,
                    performance_metrics: HashMap::new(),
                    quality_metrics: HashMap::new(),
                    updated_at: now,
                },
            },
            tasks: TaskCollection {
                tasks: HashMap::new(),
                task_index: TaskIndex {
                    by_status: HashMap::new(),
                    by_priority: HashMap::new(),
                    by_tag: HashMap::new(),
                    recently_updated: Vec::new(),
                },
                statistics: TaskStatistics {
                    total_tasks: 0,
                    status_counts: HashMap::new(),
                    average_completion_time: None,
                    productivity_metrics: HashMap::new(),
                    updated_at: now,
                },
            },
        }
    }

    /// Update the sub-project status
    pub fn update_status(&mut self, status: ProjectStatus) {
        self.metadata.status = status;
        self.metadata.updated_at = Utc::now();
    }

    /// Add a task to the sub-project
    pub fn add_task(&mut self, task: Task) {
        let task_id = task.id.clone();
        let task_status = task.status.clone();
        let task_priority = task.priority.clone();

        // Update task collection
        self.tasks.tasks.insert(task_id.clone(), task);

        // Update task index
        self.tasks
            .task_index
            .by_status
            .entry(task_status)
            .or_insert_with(Vec::new)
            .push(task_id.clone());

        self.tasks
            .task_index
            .by_priority
            .entry(task_priority)
            .or_insert_with(Vec::new)
            .push(task_id.clone());

        self.tasks.task_index.recently_updated.push(task_id);

        // Update statistics
        self.tasks.statistics.total_tasks += 1;
        *self
            .tasks
            .statistics
            .status_counts
            .entry(TaskStatus::Pending)
            .or_insert(0) += 1;

        self.tasks.statistics.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Update task status and reindex
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        new_status: TaskStatus,
    ) -> Result<(), String> {
        let task = self
            .tasks
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        let old_status = task.status.clone();
        task.status = new_status.clone();
        task.updated_at = Utc::now();

        // Update task index
        self.reindex_task(task_id, &old_status, &new_status);

        // Update statistics
        if let Some(count) = self.tasks.statistics.status_counts.get_mut(&old_status) {
            *count = count.saturating_sub(1);
        }
        *self
            .tasks
            .statistics
            .status_counts
            .entry(new_status)
            .or_insert(0) += 1;

        self.tasks.statistics.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();

        Ok(())
    }

    /// Get tasks by status
    pub fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<&Task> {
        self.tasks
            .task_index
            .by_status
            .get(status)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.tasks.tasks.get(id))
            .collect()
    }

    /// Get task statistics
    pub fn get_task_statistics(&self) -> &TaskStatistics {
        &self.tasks.statistics
    }

    /// Update progress metrics
    pub fn update_progress(&mut self, completion_percentage: f64) {
        self.progress.metrics.completion_percentage = completion_percentage;
        self.progress.metrics.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Add a blocker to the active context
    pub fn add_blocker(&mut self, blocker: Blocker) {
        self.active_context.blockers.push(blocker);
        self.active_context.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Record a change in the active context
    pub fn record_change(&mut self, change: Change) {
        self.active_context.recent_changes.push(change);
        self.active_context.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Update current focus
    pub fn update_focus(&mut self, focus: String) {
        self.active_context.current_focus = focus;
        self.active_context.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Add next steps
    pub fn add_next_steps(&mut self, steps: Vec<String>) {
        self.active_context.next_steps.extend(steps);
        self.active_context.updated_at = Utc::now();
        self.metadata.updated_at = Utc::now();
    }

    /// Helper method to reindex a task after status change
    fn reindex_task(&mut self, task_id: &str, old_status: &TaskStatus, new_status: &TaskStatus) {
        // Remove from old status index
        if let Some(status_vec) = self.tasks.task_index.by_status.get_mut(old_status) {
            status_vec.retain(|id| id != task_id);
        }

        // Add to new status index
        self.tasks
            .task_index
            .by_status
            .entry(new_status.clone())
            .or_insert_with(Vec::new)
            .push(task_id.to_string());

        // Update recently updated list
        self.tasks
            .task_index
            .recently_updated
            .retain(|id| id != task_id);
        self.tasks
            .task_index
            .recently_updated
            .push(task_id.to_string());

        // Keep recently updated list to a reasonable size
        if self.tasks.task_index.recently_updated.len() > 50 {
            self.tasks.task_index.recently_updated.remove(0);
        }
    }
}

impl Default for SubProject {
    fn default() -> Self {
        Self::new("unnamed".to_string(), None)
    }
}
