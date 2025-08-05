//! Sub-project domain models and functionality
//!
//! This module contains data structures and operations for individual sub-projects
//! within a workspace, including metadata, context management, and task integration.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::progress::Progress;
use super::system::SystemPatterns;
use super::task_management::{Task, TaskCollection, TaskIndex, TaskStatistics};
use super::tech::TechContext;
use super::types::{ProjectStatus, TaskStatus};

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
                architecture: super::system::ArchitectureDescription {
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
                development_setup: super::tech::DevelopmentSetup {
                    required_tools: Vec::new(),
                    environment_variables: HashMap::new(),
                    setup_instructions: Vec::new(),
                    ide_configuration: Vec::new(),
                    development_scripts: HashMap::new(),
                },
                constraints: Vec::new(),
                dependencies: super::tech::DependencyManagement {
                    package_manager: String::new(),
                    lock_file: String::new(),
                    update_policy: String::new(),
                    security_scanning: None,
                    known_issues: Vec::new(),
                },
                deployment: super::tech::DeploymentContext {
                    environments: Vec::new(),
                    deployment_strategy: String::new(),
                    infrastructure: Vec::new(),
                    monitoring: super::monitoring::MonitoringSetup {
                        logging: super::monitoring::LoggingConfig {
                            level: "info".to_string(),
                            format: "json".to_string(),
                            destinations: Vec::new(),
                            retention: "30d".to_string(),
                        },
                        metrics: super::monitoring::MetricsConfig {
                            system: "prometheus".to_string(),
                            interval: "1m".to_string(),
                            custom_metrics: Vec::new(),
                            dashboards: Vec::new(),
                        },
                        alerting: super::monitoring::AlertingConfig {
                            system: "alertmanager".to_string(),
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
                current_status: super::types::ProgressStatus::NotStarted,
                known_issues: Vec::new(),
                completed_milestones: Vec::new(),
                upcoming_milestones: Vec::new(),
                metrics: super::progress::ProgressMetrics {
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

        // Add task to collection
        self.tasks.tasks.insert(task_id.clone(), task);

        // Update index
        self.tasks
            .task_index
            .by_status
            .entry(task_status)
            .or_default()
            .push(task_id.clone());

        self.tasks
            .task_index
            .by_priority
            .entry(task_priority)
            .or_default()
            .push(task_id.clone());

        self.tasks.task_index.recently_updated.push(task_id);

        // Update statistics
        self.reindex_tasks();
        self.metadata.updated_at = Utc::now();
    }

    /// Update task status and reindex
    pub fn update_task_status(
        &mut self,
        task_id: &str,
        new_status: TaskStatus,
        author: String,
    ) -> Result<(), String> {
        let task = self
            .tasks
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{task_id}' not found"))?;

        task.update_status(new_status, author);
        self.reindex_tasks();
        self.metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Get tasks by status
    pub fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<&Task> {
        self.tasks
            .task_index
            .by_status
            .get(status)
            .map(|task_ids| {
                task_ids
                    .iter()
                    .filter_map(|id| self.tasks.tasks.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Reindex all tasks and update statistics
    fn reindex_tasks(&mut self) {
        // Clear existing indices
        self.tasks.task_index.by_status.clear();
        self.tasks.task_index.by_priority.clear();
        self.tasks.task_index.by_tag.clear();

        // Rebuild indices
        for (task_id, task) in &self.tasks.tasks {
            // Index by status
            self.tasks
                .task_index
                .by_status
                .entry(task.status.clone())
                .or_default()
                .push(task_id.clone());

            // Index by priority
            self.tasks
                .task_index
                .by_priority
                .entry(task.priority.clone())
                .or_default()
                .push(task_id.clone());

            // Index by tags
            for tag in &task.metadata.tags {
                self.tasks
                    .task_index
                    .by_tag
                    .entry(tag.clone())
                    .or_default()
                    .push(task_id.clone());
            }
        }

        // Update statistics
        self.tasks.statistics.total_tasks = self.tasks.tasks.len();
        self.tasks.statistics.status_counts.clear();

        for (status, task_ids) in &self.tasks.task_index.by_status {
            self.tasks
                .statistics
                .status_counts
                .insert(status.clone(), task_ids.len());
        }

        self.tasks.statistics.updated_at = Utc::now();
    }
}
