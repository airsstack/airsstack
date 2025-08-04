//! Task management and tracking domain
//!
//! This module contains comprehensive task management functionality including
//! task collections, indexing, statistics, and detailed task tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::review::CodeReviewInfo;
use super::testing::TestingInfo;
use super::types::{Priority, ProgressLogType, SubtaskStatus, TaskStatus};

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

/// Individual task definition with comprehensive tracking
///
/// Represents a complete task within the Multi-Project Memory Bank system,
/// including detailed progress tracking, subtasks, and extensive metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    /// Unique task identifier
    pub id: String,

    /// Human-readable task name
    pub name: String,

    /// Current task status
    pub status: TaskStatus,

    /// Task priority level
    pub priority: Priority,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Detailed task information
    pub details: TaskDetails,

    /// Progress tracking and metrics
    pub progress: TaskProgress,

    /// Subtask breakdown and tracking
    pub subtasks: Vec<Subtask>,

    /// Historical log of progress updates
    pub progress_log: Vec<ProgressLogEntry>,

    /// Task metadata and tags
    pub metadata: TaskMetadata,
}

/// Detailed task information and context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskDetails {
    /// Original user request or description
    pub original_request: String,

    /// Documented thought process and approach
    pub thought_process: String,

    /// Detailed implementation plan
    pub implementation_plan: Vec<String>,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,

    /// Dependencies on other tasks or components
    pub dependencies: Vec<String>,

    /// Estimated effort or complexity
    pub estimated_effort: Option<String>,

    /// Target completion date
    pub target_date: Option<DateTime<Utc>>,

    /// Actual completion date
    pub completed_date: Option<DateTime<Utc>>,

    /// Task category or domain
    pub category: Option<String>,

    /// Related files or components
    pub related_components: Vec<String>,
}

/// Task progress tracking and metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskProgress {
    /// Overall completion percentage
    pub completion_percentage: f64,

    /// Current phase or stage
    pub current_phase: String,

    /// Time spent on the task
    pub time_spent: Option<chrono::Duration>,

    /// Estimated time remaining
    pub estimated_time_remaining: Option<chrono::Duration>,

    /// Quality metrics
    pub quality_metrics: HashMap<String, f64>,

    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,

    /// Test coverage percentage
    pub test_coverage: Option<f64>,

    /// Number of subtasks completed vs total
    pub subtasks_completed: usize,

    /// Total number of subtasks
    pub subtasks_total: usize,

    /// Last progress update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Individual subtask within a larger task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtask {
    /// Subtask identifier (e.g., "1.1", "1.2")
    pub id: String,

    /// Subtask description
    pub description: String,

    /// Current status
    pub status: SubtaskStatus,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Any relevant notes or comments
    pub notes: Option<String>,

    /// Dependencies on other subtasks
    pub dependencies: Vec<String>,

    /// Estimated effort
    pub estimated_effort: Option<String>,

    /// Actual effort spent
    pub actual_effort: Option<chrono::Duration>,

    /// Associated files or components
    pub associated_files: Vec<String>,
}

/// Progress log entry for detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressLogEntry {
    /// Log entry timestamp
    pub timestamp: DateTime<Utc>,

    /// Entry type (update, milestone, issue, etc.)
    pub entry_type: ProgressLogType,

    /// Detailed description of progress
    pub description: String,

    /// Who made this update
    pub author: String,

    /// Associated subtasks affected
    pub affected_subtasks: Vec<String>,

    /// Changes in completion percentage
    pub completion_delta: Option<f64>,

    /// Any decisions made
    pub decisions: Vec<String>,

    /// Issues encountered
    pub issues_encountered: Vec<String>,

    /// Solutions implemented
    pub solutions_implemented: Vec<String>,

    /// Next planned actions
    pub next_actions: Vec<String>,
}

/// Task metadata and categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskMetadata {
    /// Tags for categorization
    pub tags: Vec<String>,

    /// Labels for quick identification
    pub labels: Vec<String>,

    /// Links to external resources
    pub external_links: Vec<String>,

    /// Related GitHub issues or PRs
    pub github_references: Vec<String>,

    /// Documentation references
    pub documentation: Vec<String>,

    /// Code review information
    pub code_review: Option<CodeReviewInfo>,

    /// Testing information
    pub testing_info: Option<TestingInfo>,

    /// Additional custom metadata
    pub custom_metadata: HashMap<String, String>,
}

/// Task progress summary for quick status overview
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskProgressSummary {
    /// Task identifier
    pub task_id: String,

    /// Task name
    pub task_name: String,

    /// Current status
    pub status: TaskStatus,

    /// Completion percentage
    pub completion_percentage: f64,

    /// Current phase
    pub current_phase: String,

    /// Completed subtasks
    pub subtasks_completed: usize,

    /// Total subtasks
    pub subtasks_total: usize,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,

    /// Whether task has blockers
    pub has_blockers: bool,

    /// Next pending subtask
    pub next_subtask: Option<String>,
}

impl Task {
    /// Create a new task with comprehensive default configuration
    pub fn new(id: String, name: String, original_request: String) -> Self {
        let now = Utc::now();

        Self {
            id: id.clone(),
            name: name.clone(),
            status: TaskStatus::Pending,
            priority: Priority::Medium,
            created_at: now,
            updated_at: now,
            details: TaskDetails {
                original_request,
                thought_process: String::new(),
                implementation_plan: Vec::new(),
                acceptance_criteria: Vec::new(),
                dependencies: Vec::new(),
                estimated_effort: None,
                target_date: None,
                completed_date: None,
                category: None,
                related_components: Vec::new(),
            },
            progress: TaskProgress {
                completion_percentage: 0.0,
                current_phase: "planning".to_string(),
                time_spent: None,
                estimated_time_remaining: None,
                quality_metrics: HashMap::new(),
                performance_metrics: HashMap::new(),
                test_coverage: None,
                subtasks_completed: 0,
                subtasks_total: 0,
                last_updated: now,
            },
            subtasks: Vec::new(),
            progress_log: vec![ProgressLogEntry {
                timestamp: now,
                entry_type: ProgressLogType::StatusChange,
                description: format!("Task '{}' created", name),
                author: "system".to_string(),
                affected_subtasks: Vec::new(),
                completion_delta: Some(0.0),
                decisions: Vec::new(),
                issues_encountered: Vec::new(),
                solutions_implemented: Vec::new(),
                next_actions: Vec::new(),
            }],
            metadata: TaskMetadata {
                tags: Vec::new(),
                labels: Vec::new(),
                external_links: Vec::new(),
                github_references: Vec::new(),
                documentation: Vec::new(),
                code_review: None,
                testing_info: None,
                custom_metadata: HashMap::new(),
            },
        }
    }

    /// Update task status with automatic logging
    pub fn update_status(&mut self, new_status: TaskStatus, author: String) {
        let old_status = self.status.clone();
        self.status = new_status.clone();
        self.updated_at = Utc::now();

        // Automatically set completion date if completed
        if new_status == TaskStatus::Completed {
            self.details.completed_date = Some(Utc::now());
            self.progress.completion_percentage = 100.0;
        }

        // Log the status change
        self.add_progress_log_entry(ProgressLogEntry {
            timestamp: Utc::now(),
            entry_type: ProgressLogType::StatusChange,
            description: format!("Status changed from {:?} to {:?}", old_status, new_status),
            author,
            affected_subtasks: Vec::new(),
            completion_delta: None,
            decisions: Vec::new(),
            issues_encountered: Vec::new(),
            solutions_implemented: Vec::new(),
            next_actions: Vec::new(),
        });

        self.update_progress_phase();
    }

    /// Add a subtask to the task
    pub fn add_subtask(&mut self, subtask: Subtask) {
        self.subtasks.push(subtask);
        self.progress.subtasks_total = self.subtasks.len();
        self.update_completion_from_subtasks();
        self.updated_at = Utc::now();
    }

    /// Update subtask status and recalculate progress
    pub fn update_subtask_status(
        &mut self,
        subtask_id: &str,
        new_status: SubtaskStatus,
        author: String,
    ) -> Result<(), String> {
        let subtask = self
            .subtasks
            .iter_mut()
            .find(|s| s.id == subtask_id)
            .ok_or_else(|| format!("Subtask '{}' not found", subtask_id))?;

        let old_status = subtask.status.clone();
        subtask.status = new_status.clone();
        subtask.updated_at = Utc::now();

        // Update subtasks completed count
        self.progress.subtasks_completed = self
            .subtasks
            .iter()
            .filter(|s| matches!(s.status, SubtaskStatus::Complete))
            .count();

        // Recalculate completion percentage
        self.update_completion_from_subtasks();

        // Log the subtask update
        self.add_progress_log_entry(ProgressLogEntry {
            timestamp: Utc::now(),
            entry_type: ProgressLogType::ProgressUpdate,
            description: format!(
                "Subtask {} status changed from {:?} to {:?}",
                subtask_id, old_status, new_status
            ),
            author,
            affected_subtasks: vec![subtask_id.to_string()],
            completion_delta: None,
            decisions: Vec::new(),
            issues_encountered: Vec::new(),
            solutions_implemented: Vec::new(),
            next_actions: Vec::new(),
        });

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add a comprehensive progress log entry
    pub fn add_progress_log_entry(&mut self, entry: ProgressLogEntry) {
        self.progress_log.push(entry);
        self.progress.last_updated = Utc::now();
        self.updated_at = Utc::now();

        // Keep progress log to a reasonable size
        if self.progress_log.len() > 100 {
            self.progress_log.remove(0);
        }
    }

    /// Update thought process documentation
    pub fn update_thought_process(&mut self, thought_process: String) {
        self.details.thought_process = thought_process;
        self.updated_at = Utc::now();
    }

    /// Set implementation plan
    pub fn set_implementation_plan(&mut self, plan: Vec<String>) {
        self.details.implementation_plan = plan;
        self.updated_at = Utc::now();
    }

    /// Add acceptance criteria
    pub fn add_acceptance_criteria(&mut self, criteria: Vec<String>) {
        self.details.acceptance_criteria.extend(criteria);
        self.updated_at = Utc::now();
    }

    /// Update completion percentage manually
    pub fn update_completion_percentage(&mut self, percentage: f64, author: String) {
        let old_percentage = self.progress.completion_percentage;
        self.progress.completion_percentage = percentage.clamp(0.0, 100.0);
        self.progress.last_updated = Utc::now();

        // Log the progress update
        self.add_progress_log_entry(ProgressLogEntry {
            timestamp: Utc::now(),
            entry_type: ProgressLogType::ProgressUpdate,
            description: format!(
                "Completion updated from {:.1}% to {:.1}%",
                old_percentage, percentage
            ),
            author,
            affected_subtasks: Vec::new(),
            completion_delta: Some(percentage - old_percentage),
            decisions: Vec::new(),
            issues_encountered: Vec::new(),
            solutions_implemented: Vec::new(),
            next_actions: Vec::new(),
        });

        self.update_progress_phase();
        self.updated_at = Utc::now();
    }

    /// Add tags for categorization
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.metadata.tags.extend(tags);
        self.updated_at = Utc::now();
    }

    /// Set testing information
    pub fn set_testing_info(&mut self, testing_info: TestingInfo) {
        self.metadata.testing_info = Some(testing_info);
        self.updated_at = Utc::now();
    }

    /// Set code review information
    pub fn set_code_review_info(&mut self, review_info: CodeReviewInfo) {
        self.metadata.code_review = Some(review_info);
        self.updated_at = Utc::now();
    }

    /// Get current progress summary
    pub fn get_progress_summary(&self) -> TaskProgressSummary {
        TaskProgressSummary {
            task_id: self.id.clone(),
            task_name: self.name.clone(),
            status: self.status.clone(),
            completion_percentage: self.progress.completion_percentage,
            current_phase: self.progress.current_phase.clone(),
            subtasks_completed: self.progress.subtasks_completed,
            subtasks_total: self.progress.subtasks_total,
            last_updated: self.progress.last_updated,
            has_blockers: self.has_blockers(),
            next_subtask: self.get_next_pending_subtask(),
        }
    }

    /// Check if task has any blockers
    pub fn has_blockers(&self) -> bool {
        self.subtasks
            .iter()
            .any(|s| matches!(s.status, SubtaskStatus::Blocked))
    }

    /// Get the next pending subtask
    pub fn get_next_pending_subtask(&self) -> Option<String> {
        self.subtasks
            .iter()
            .find(|s| {
                matches!(
                    s.status,
                    SubtaskStatus::NotStarted | SubtaskStatus::InProgress
                )
            })
            .map(|s| s.description.clone())
    }

    /// Calculate completion percentage from subtasks
    fn update_completion_from_subtasks(&mut self) {
        if self.subtasks.is_empty() {
            return;
        }

        let completed_count = self.progress.subtasks_completed as f64;
        let total_count = self.progress.subtasks_total as f64;
        self.progress.completion_percentage = (completed_count / total_count) * 100.0;
        self.progress.last_updated = Utc::now();
    }

    /// Update current phase based on progress
    fn update_progress_phase(&mut self) {
        self.progress.current_phase = match self.progress.completion_percentage {
            0.0 => "planning".to_string(),
            p if p < 25.0 => "early_development".to_string(),
            p if p < 50.0 => "development".to_string(),
            p if p < 75.0 => "integration".to_string(),
            p if p < 100.0 => "testing".to_string(),
            _ => "completed".to_string(),
        };
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new(
            "default".to_string(),
            "Default Task".to_string(),
            "Default task created".to_string(),
        )
    }
}

impl Subtask {
    /// Create a new subtask
    pub fn new(id: String, description: String) -> Self {
        Self {
            id,
            description,
            status: SubtaskStatus::NotStarted,
            updated_at: Utc::now(),
            notes: None,
            dependencies: Vec::new(),
            estimated_effort: None,
            actual_effort: None,
            associated_files: Vec::new(),
        }
    }

    /// Update subtask with notes
    pub fn add_notes(&mut self, notes: String) {
        self.notes = Some(notes);
        self.updated_at = Utc::now();
    }

    /// Add associated files
    pub fn add_associated_files(&mut self, files: Vec<String>) {
        self.associated_files.extend(files);
        self.updated_at = Utc::now();
    }
}
