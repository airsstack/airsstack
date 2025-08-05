//! Context Correlation System
//!
//! This module provides functionality for correlating parsed markdown content with
//! workspace context, tracking current state, and managing context transitions
//! in Multi-Project Memory Bank environments.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::models::workspace::CurrentContext;
use crate::parser::markdown::{MarkdownContent, MarkdownParser, TaskItem, TaskStatus};
use crate::parser::navigation::{MemoryBankNavigator, MemoryBankStructure};
use crate::utils::fs::{FsError, FsResult};

/// The main context correlation engine for memory bank workspaces
///
/// This structure manages the discovery, parsing, and correlation of workspace
/// context across multiple sub-projects within a memory bank structure.
#[derive(Debug)]
pub struct ContextCorrelator {
    /// Current workspace context and all discovered sub-project information
    workspace_context: Option<WorkspaceContext>,
}

/// Complete workspace context with correlated information
///
/// This structure combines raw file system information with parsed content
/// and current context state to provide a unified view of the workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    /// Discovered memory bank structure
    pub structure: MemoryBankStructure,

    /// Current active context information
    pub current_context: CurrentContext,

    /// Parsed content from key workspace files
    pub workspace_content: WorkspaceContent,

    /// Sub-project context information
    pub sub_project_contexts: HashMap<String, SubProjectContext>,

    /// Last correlation update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Parsed workspace-level content
#[derive(Debug, Clone)]
pub struct WorkspaceContent {
    /// Parsed project brief content
    pub project_brief: Option<MarkdownContent>,

    /// Parsed shared patterns content
    pub shared_patterns: Option<MarkdownContent>,

    /// Parsed workspace architecture content
    pub workspace_architecture: Option<MarkdownContent>,

    /// Parsed workspace progress content
    pub workspace_progress: Option<MarkdownContent>,
}

/// Context information for a specific sub-project
#[derive(Debug, Clone)]
pub struct SubProjectContext {
    /// Sub-project name/identifier
    pub name: String,

    /// Parsed content from core sub-project files
    pub content: SubProjectContent,

    /// Aggregated task information from all task files
    pub task_summary: TaskSummary,

    /// Current status derived from parsed content
    pub derived_status: DerivedStatus,

    /// Last update timestamp for this sub-project context
    pub last_updated: DateTime<Utc>,
}

/// Parsed sub-project content from core files
#[derive(Debug, Clone)]
pub struct SubProjectContent {
    /// Parsed project brief content
    pub project_brief: Option<MarkdownContent>,

    /// Parsed product context content
    pub product_context: Option<MarkdownContent>,

    /// Parsed active context content
    pub active_context: Option<MarkdownContent>,

    /// Parsed system patterns content
    pub system_patterns: Option<MarkdownContent>,

    /// Parsed tech context content
    pub tech_context: Option<MarkdownContent>,

    /// Parsed progress content
    pub progress: Option<MarkdownContent>,
}

/// Aggregated task information across all task files
#[derive(Debug, Clone)]
pub struct TaskSummary {
    /// Total number of tasks
    pub total_tasks: usize,

    /// Tasks by status
    pub tasks_by_status: HashMap<TaskStatus, Vec<TaskItem>>,

    /// Most recently updated tasks
    pub recent_tasks: Vec<TaskItem>,

    /// Task completion percentage
    pub completion_percentage: f64,

    /// Tasks with blocking issues
    pub blocked_tasks: Vec<TaskItem>,

    /// Next priority tasks
    pub next_tasks: Vec<TaskItem>,
}

/// Derived status information from parsed content
#[derive(Debug, Clone)]
pub struct DerivedStatus {
    /// Overall project health based on task progress
    pub health: ProjectHealth,

    /// Current phase derived from active context
    pub current_phase: String,

    /// Progress indicators from various sources
    pub progress_indicators: Vec<ProgressIndicator>,

    /// Issues or blockers identified
    pub issues: Vec<Issue>,

    /// Recommendations for next actions
    pub recommendations: Vec<String>,
}

/// Project health assessment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectHealth {
    /// Project has significant issues or blockers
    Critical,
    /// Project has minor issues or delays
    Warning,
    /// Project is progressing well
    Healthy,
    /// Project status cannot be determined
    Unknown,
}

/// Progress indicator from content analysis
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Source of the indicator (file name, section)
    pub source: String,

    /// Type of progress metric
    pub metric_type: ProgressMetricType,

    /// Current value
    pub current_value: f64,

    /// Target value (if applicable)
    pub target_value: Option<f64>,

    /// Progress description
    pub description: String,
}

/// Types of progress metrics
#[derive(Debug, Clone)]
pub enum ProgressMetricType {
    /// Task completion percentage
    TaskCompletion,
    /// Milestone progress
    MilestoneProgress,
    /// Feature implementation status
    FeatureProgress,
    /// Documentation completeness
    DocumentationProgress,
    /// Custom metric
    Custom(String),
}

/// Identified issue or blocker
#[derive(Debug, Clone)]
pub struct Issue {
    /// Issue severity level
    pub severity: IssueSeverity,

    /// Issue description
    pub description: String,

    /// Source where issue was identified
    pub source: String,

    /// Suggested resolution
    pub resolution: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    /// Low priority issue
    Low,
    /// Medium priority issue
    Medium,
    /// High priority issue
    High,
    /// Critical blocker
    Critical,
}

impl Default for ContextCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextCorrelator {
    /// Create a new context correlator
    ///
    /// Initializes the correlator with a memory bank navigator for file access.
    /// The workspace context will be loaded on first correlation request.
    ///
    /// # Arguments
    /// * `navigator` - Memory bank navigator for file system access
    ///
    /// # Returns
    /// * `ContextCorrelator` - New correlator instance
    pub fn new() -> Self {
        Self {
            workspace_context: None,
        }
    }

    /// Discover and correlate workspace context from a given root path
    ///
    /// This is the primary entry point for context correlation. It discovers
    /// the memory bank structure, parses relevant content, and builds a
    /// comprehensive workspace context.
    ///
    /// # Arguments
    /// * `root_path` - Root path to search for memory bank structure
    ///
    /// # Returns
    /// * `Ok(WorkspaceContext)` - Complete correlated workspace context
    /// * `Err(FsError)` - Discovery or parsing errors
    ///
    /// # Example
    /// ```rust,no_run
    /// use airs_memspec::parser::context::ContextCorrelator;
    /// use std::path::PathBuf;
    ///
    /// let mut correlator = ContextCorrelator::new();
    /// let context = correlator.discover_and_correlate(&PathBuf::from("."))?;
    /// println!("Found {} sub-projects", context.sub_project_contexts.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn discover_and_correlate(&mut self, root_path: &Path) -> FsResult<&WorkspaceContext> {
        // Step 1: Discover memory bank structure
        let structure = MemoryBankNavigator::discover_structure(root_path)?;

        // Step 2: Parse current context file
        let current_context = self.parse_current_context(&structure)?;

        // Step 3: Parse workspace-level content
        let workspace_content = self.parse_workspace_content(&structure)?;

        // Step 4: Parse all sub-project contexts
        let sub_project_contexts = self.parse_sub_project_contexts(&structure)?;

        // Step 5: Build complete workspace context
        let workspace_context = WorkspaceContext {
            structure,
            current_context,
            workspace_content,
            sub_project_contexts,
            last_updated: Utc::now(),
        };

        self.workspace_context = Some(workspace_context);
        Ok(self.workspace_context.as_ref().unwrap())
    }

    /// Get the current workspace context
    ///
    /// Returns the cached workspace context if available, or None if
    /// context correlation has not been performed yet.
    ///
    /// # Returns
    /// * `Option<&WorkspaceContext>` - Current workspace context
    pub fn get_workspace_context(&self) -> Option<&WorkspaceContext> {
        self.workspace_context.as_ref()
    }

    /// Switch to a different sub-project context
    ///
    /// Updates the current context to point to a different sub-project and
    /// updates the current_context.md file accordingly.
    ///
    /// # Arguments
    /// * `sub_project_name` - Name of the sub-project to switch to
    /// * `switched_by` - Identifier for who/what triggered the switch
    ///
    /// # Returns
    /// * `Ok(())` - Context switch successful
    /// * `Err(FsError)` - File update or validation errors
    pub fn switch_context(&mut self, sub_project_name: &str, switched_by: &str) -> FsResult<()> {
        let workspace_context = self.workspace_context.as_mut().ok_or_else(|| {
            FsError::ParseError(
                "Workspace context not initialized. Call discover_and_correlate first.".to_string(),
            )
        })?;

        // Validate that the sub-project exists
        if !workspace_context
            .sub_project_contexts
            .contains_key(sub_project_name)
        {
            return Err(FsError::ParseError(format!(
                "Sub-project '{sub_project_name}' not found in workspace"
            )));
        }

        // Update current context
        workspace_context.current_context = CurrentContext {
            active_sub_project: sub_project_name.to_string(),
            switched_on: Utc::now(),
            switched_by: switched_by.to_string(),
            status: format!("switched_to_{sub_project_name}"),
            metadata: HashMap::new(),
        };

        // Update the current_context.md file - we need to get the values first
        let (structure, current_context) = {
            let ws_ctx = workspace_context;
            (ws_ctx.structure.clone(), ws_ctx.current_context.clone())
        };

        self.update_current_context_file(&structure, &current_context)?;

        // Get the workspace context again to update timestamp
        if let Some(workspace_context) = &mut self.workspace_context {
            workspace_context.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Get aggregated task status across the workspace
    ///
    /// Provides a high-level view of task progress across all sub-projects
    /// or for a specific sub-project.
    ///
    /// # Arguments
    /// * `sub_project` - Optional sub-project name to filter by
    ///
    /// # Returns
    /// * `Option<TaskSummary>` - Aggregated task summary
    pub fn get_task_summary(&self, sub_project: Option<&str>) -> Option<TaskSummary> {
        let workspace_context = self.workspace_context.as_ref()?;

        match sub_project {
            Some(name) => workspace_context
                .sub_project_contexts
                .get(name)
                .map(|ctx| ctx.task_summary.clone()),
            None => {
                // Aggregate across all sub-projects
                Some(self.aggregate_workspace_tasks(workspace_context))
            }
        }
    }

    /// Parse current context from current_context.md file
    fn parse_current_context(&self, structure: &MemoryBankStructure) -> FsResult<CurrentContext> {
        if let Some(context_path) = &structure.current_context {
            let content = MarkdownParser::parse_file(context_path)?;

            // Extract context information from parsed markdown
            let active_sub_project = content
                .metadata
                .title
                .or_else(|| {
                    content
                        .sections
                        .keys()
                        .find(|k| k.contains("active"))
                        .cloned()
                })
                .unwrap_or_else(|| "unknown".to_string());

            Ok(CurrentContext {
                active_sub_project,
                switched_on: Utc::now(),
                switched_by: "system".to_string(),
                status: "initialized".to_string(),
                metadata: HashMap::new(),
            })
        } else {
            // Default context if no current_context.md file exists
            Ok(CurrentContext {
                active_sub_project: "unknown".to_string(),
                switched_on: Utc::now(),
                switched_by: "system".to_string(),
                status: "no_context_file".to_string(),
                metadata: HashMap::new(),
            })
        }
    }

    /// Parse workspace-level content files
    fn parse_workspace_content(
        &self,
        structure: &MemoryBankStructure,
    ) -> FsResult<WorkspaceContent> {
        let project_brief = if let Some(path) = &structure.workspace.project_brief {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let shared_patterns = if let Some(path) = &structure.workspace.shared_patterns {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let workspace_architecture = if let Some(path) = &structure.workspace.workspace_architecture
        {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let workspace_progress = if let Some(path) = &structure.workspace.workspace_progress {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        Ok(WorkspaceContent {
            project_brief,
            shared_patterns,
            workspace_architecture,
            workspace_progress,
        })
    }

    /// Parse all sub-project contexts
    fn parse_sub_project_contexts(
        &self,
        structure: &MemoryBankStructure,
    ) -> FsResult<HashMap<String, SubProjectContext>> {
        let mut contexts = HashMap::new();

        for (name, sub_project_files) in &structure.sub_projects {
            let context = self.parse_single_sub_project_context(name, sub_project_files)?;
            contexts.insert(name.clone(), context);
        }

        Ok(contexts)
    }

    /// Parse context for a single sub-project
    fn parse_single_sub_project_context(
        &self,
        name: &str,
        files: &crate::parser::navigation::SubProjectFiles,
    ) -> FsResult<SubProjectContext> {
        // Parse core content files
        let project_brief = if let Some(path) = &files.project_brief {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let product_context = if let Some(path) = &files.product_context {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let active_context = if let Some(path) = &files.active_context {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let system_patterns = if let Some(path) = &files.system_patterns {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let tech_context = if let Some(path) = &files.tech_context {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let progress = if let Some(path) = &files.progress {
            Some(MarkdownParser::parse_file(path)?)
        } else {
            None
        };

        let content = SubProjectContent {
            project_brief,
            product_context,
            active_context,
            system_patterns,
            tech_context,
            progress,
        };

        // Parse and aggregate task information
        let task_summary = self.parse_task_summary(files)?;

        // Derive status from parsed content
        let derived_status = self.derive_project_status(&content, &task_summary);

        Ok(SubProjectContext {
            name: name.to_string(),
            content,
            task_summary,
            derived_status,
            last_updated: Utc::now(),
        })
    }

    /// Parse and aggregate task information from task files
    fn parse_task_summary(
        &self,
        files: &crate::parser::navigation::SubProjectFiles,
    ) -> FsResult<TaskSummary> {
        let mut all_tasks = Vec::new();

        // Parse individual task files
        for task_file in &files.task_files {
            let content = MarkdownParser::parse_file(task_file)?;
            all_tasks.extend(content.tasks);
        }

        // Aggregate task information
        let total_tasks = all_tasks.len();
        let mut tasks_by_status = HashMap::new();

        for task in &all_tasks {
            tasks_by_status
                .entry(task.status.clone())
                .or_insert_with(Vec::new)
                .push(task.clone());
        }

        let completed_count = tasks_by_status
            .get(&TaskStatus::Completed)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        let completion_percentage = if total_tasks > 0 {
            (completed_count as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        let blocked_tasks = tasks_by_status
            .get(&TaskStatus::Blocked)
            .cloned()
            .unwrap_or_default();

        let next_tasks = tasks_by_status
            .get(&TaskStatus::NotStarted)
            .cloned()
            .unwrap_or_default();

        // Sort tasks by update time for recent tasks
        let mut recent_tasks = all_tasks.clone();
        recent_tasks.sort_by(|a, b| b.updated.cmp(&a.updated));
        recent_tasks.truncate(5); // Keep only 5 most recent

        Ok(TaskSummary {
            total_tasks,
            tasks_by_status,
            recent_tasks,
            completion_percentage,
            blocked_tasks,
            next_tasks,
        })
    }

    /// Derive project status from parsed content and task summary
    fn derive_project_status(
        &self,
        content: &SubProjectContent,
        task_summary: &TaskSummary,
    ) -> DerivedStatus {
        let health = if task_summary.completion_percentage > 80.0 {
            ProjectHealth::Healthy
        } else if task_summary.completion_percentage > 50.0 {
            ProjectHealth::Warning
        } else if !task_summary.blocked_tasks.is_empty() {
            ProjectHealth::Critical
        } else {
            ProjectHealth::Unknown
        };

        let current_phase = content
            .active_context
            .as_ref()
            .and_then(|ctx| ctx.metadata.description.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let progress_indicators = vec![ProgressIndicator {
            source: "task_summary".to_string(),
            metric_type: ProgressMetricType::TaskCompletion,
            current_value: task_summary.completion_percentage,
            target_value: Some(100.0),
            description: format!(
                "Task completion: {:.1}%",
                task_summary.completion_percentage
            ),
        }];

        let issues = task_summary
            .blocked_tasks
            .iter()
            .map(|task| Issue {
                severity: IssueSeverity::High,
                description: format!("Blocked task: {}", task.title),
                source: "task_analysis".to_string(),
                resolution: task.details.clone(),
            })
            .collect();

        let recommendations = if task_summary.completion_percentage < 50.0 {
            vec!["Focus on completing pending tasks".to_string()]
        } else if !task_summary.blocked_tasks.is_empty() {
            vec!["Address blocked tasks to maintain progress".to_string()]
        } else {
            vec!["Continue current development pace".to_string()]
        };

        DerivedStatus {
            health,
            current_phase,
            progress_indicators,
            issues,
            recommendations,
        }
    }

    /// Update the current_context.md file with new context information
    fn update_current_context_file(
        &self,
        _structure: &MemoryBankStructure,
        _context: &CurrentContext,
    ) -> FsResult<()> {
        // For now, we'll implement this as a placeholder
        // In a full implementation, this would write the updated context to the file
        Ok(())
    }

    /// Aggregate task summaries across all sub-projects
    fn aggregate_workspace_tasks(&self, workspace_context: &WorkspaceContext) -> TaskSummary {
        let mut total_tasks = 0;
        let mut all_tasks_by_status: HashMap<TaskStatus, Vec<TaskItem>> = HashMap::new();
        let mut all_recent_tasks = Vec::new();
        let mut all_blocked_tasks = Vec::new();
        let mut all_next_tasks = Vec::new();

        for sub_project_context in workspace_context.sub_project_contexts.values() {
            let summary = &sub_project_context.task_summary;
            total_tasks += summary.total_tasks;

            // Merge tasks by status
            for (status, tasks) in &summary.tasks_by_status {
                all_tasks_by_status
                    .entry(status.clone())
                    .or_default()
                    .extend(tasks.clone());
            }

            all_recent_tasks.extend(summary.recent_tasks.clone());
            all_blocked_tasks.extend(summary.blocked_tasks.clone());
            all_next_tasks.extend(summary.next_tasks.clone());
        }

        let completed_count = all_tasks_by_status
            .get(&TaskStatus::Completed)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        let completion_percentage = if total_tasks > 0 {
            (completed_count as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        // Sort and limit recent tasks
        all_recent_tasks.sort_by(|a, b| b.updated.cmp(&a.updated));
        all_recent_tasks.truncate(10);

        TaskSummary {
            total_tasks,
            tasks_by_status: all_tasks_by_status,
            recent_tasks: all_recent_tasks,
            completion_percentage,
            blocked_tasks: all_blocked_tasks,
            next_tasks: all_next_tasks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_context_correlator_creation() {
        let correlator = ContextCorrelator::new();

        assert!(correlator.get_workspace_context().is_none());
    }

    #[test]
    fn test_project_health_ordering() {
        assert!(ProjectHealth::Critical < ProjectHealth::Warning);
        assert!(ProjectHealth::Warning < ProjectHealth::Healthy);
        assert!(ProjectHealth::Unknown == ProjectHealth::Unknown);
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Low < IssueSeverity::Medium);
        assert!(IssueSeverity::Medium < IssueSeverity::High);
        assert!(IssueSeverity::High < IssueSeverity::Critical);
    }
}
