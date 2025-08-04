//! Progress Analysis Module
//!
//! Provides advanced progress tracking, milestone detection, and predictive analytics
//! for project status visualization. Analyzes task completion patterns, identifies
//! bottlenecks, and calculates project velocity metrics.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::parser::context::{ProjectHealth, SubProjectContext, WorkspaceContext};
use crate::parser::markdown::TaskStatus;

/// Comprehensive progress analytics for a project or workspace
#[derive(Debug, Clone)]
pub struct ProgressAnalytics {
    /// Current completion percentage
    pub completion_percentage: f64,
    /// Velocity (tasks completed per week)
    pub velocity: f64,
    /// Estimated time to completion (in days)
    pub eta_days: Option<f64>,
    /// Progress trend (Accelerating, Steady, Declining)
    pub trend: ProgressTrend,
    /// Identified milestones and their status
    pub milestones: Vec<Milestone>,
    /// Detected bottlenecks and blockers
    pub bottlenecks: Vec<Bottleneck>,
    /// Key performance indicators
    pub kpis: HashMap<String, f64>,
}

/// Progress trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressTrend {
    /// Progress is accelerating
    Accelerating,
    /// Progress is steady
    Steady,
    /// Progress is declining
    Declining,
    /// Not enough data to determine trend
    Unknown,
}

/// Project milestone representation
#[derive(Debug, Clone)]
pub struct Milestone {
    /// Milestone name/description
    pub name: String,
    /// Current completion percentage
    pub completion: f64,
    /// Whether this milestone is critical path
    pub is_critical: bool,
    /// Estimated completion date
    pub eta: Option<DateTime<Utc>>,
    /// Dependencies required for this milestone
    pub dependencies: Vec<String>,
    /// Tasks contributing to this milestone
    pub tasks: Vec<String>,
}

/// Identified bottleneck or blocker
#[derive(Debug, Clone)]
pub struct Bottleneck {
    /// Description of the bottleneck
    pub description: String,
    /// Severity level (Critical, High, Medium, Low)
    pub severity: BottleneckSeverity,
    /// Impact on overall progress (percentage points)
    pub impact: f64,
    /// Suggested resolution actions
    pub resolution_suggestions: Vec<String>,
    /// Tasks affected by this bottleneck
    pub affected_tasks: Vec<String>,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BottleneckSeverity {
    /// Blocking all progress
    Critical,
    /// Significantly slowing progress
    High,
    /// Moderately affecting progress
    Medium,
    /// Minor impact on progress
    Low,
}

/// Progress analyzer for calculating metrics and insights
pub struct ProgressAnalyzer {
    /// Historical data for trend analysis (simplified for now)
    historical_data: Vec<HistoricalDataPoint>,
}

/// Historical progress data point
#[derive(Debug, Clone)]
#[allow(dead_code)] // Framework for future historical data tracking
struct HistoricalDataPoint {
    /// When this measurement was taken
    timestamp: DateTime<Utc>,
    /// Completion percentage at this time
    completion: f64,
    /// Number of active tasks
    active_tasks: usize,
    /// Number of blocked tasks
    blocked_tasks: usize,
}

impl ProgressAnalyzer {
    /// Create a new progress analyzer
    pub fn new() -> Self {
        Self {
            historical_data: Vec::new(),
        }
    }

    /// Analyze progress for a workspace
    pub fn analyze_workspace(&self, workspace: &WorkspaceContext) -> ProgressAnalytics {
        let overall_completion = self.calculate_workspace_completion(workspace);
        let velocity = self.calculate_workspace_velocity(workspace);
        let trend = self.analyze_workspace_trend(workspace);
        let milestones = self.identify_workspace_milestones(workspace);
        let bottlenecks = self.detect_workspace_bottlenecks(workspace);
        let eta_days = self.calculate_workspace_eta(workspace, velocity);
        let kpis = self.calculate_workspace_kpis(workspace);

        ProgressAnalytics {
            completion_percentage: overall_completion,
            velocity,
            eta_days,
            trend,
            milestones,
            bottlenecks,
            kpis,
        }
    }

    /// Analyze progress for a specific sub-project
    pub fn analyze_sub_project(&self, context: &SubProjectContext) -> ProgressAnalytics {
        let completion = context.task_summary.completion_percentage;
        let velocity = self.calculate_project_velocity(context);
        let trend = self.analyze_project_trend(context);
        let milestones = self.identify_project_milestones(context);
        let bottlenecks = self.detect_project_bottlenecks(context);
        let eta_days = self.calculate_project_eta(context, velocity);
        let kpis = self.calculate_project_kpis(context);

        ProgressAnalytics {
            completion_percentage: completion,
            velocity,
            eta_days,
            trend,
            milestones,
            bottlenecks,
            kpis,
        }
    }

    /// Calculate workspace-level completion percentage
    fn calculate_workspace_completion(&self, workspace: &WorkspaceContext) -> f64 {
        if workspace.sub_project_contexts.is_empty() {
            return 0.0;
        }

        let total: f64 = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| ctx.task_summary.completion_percentage)
            .sum();

        total / workspace.sub_project_contexts.len() as f64
    }

    /// Calculate workspace velocity (tasks completed per week)
    fn calculate_workspace_velocity(&self, workspace: &WorkspaceContext) -> f64 {
        // Simplified velocity calculation based on completed tasks
        let total_completed: usize = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| {
                ctx.task_summary
                    .tasks_by_status
                    .get(&TaskStatus::Completed)
                    .map(|tasks| tasks.len())
                    .unwrap_or(0)
            })
            .sum();

        // Assume these tasks were completed over the last 4 weeks (simplified)
        total_completed as f64 / 4.0
    }

    /// Calculate project velocity
    fn calculate_project_velocity(&self, context: &SubProjectContext) -> f64 {
        let completed_tasks = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::Completed)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        // Simplified: assume completed over 4 weeks
        completed_tasks as f64 / 4.0
    }

    /// Analyze workspace progress trend
    fn analyze_workspace_trend(&self, _workspace: &WorkspaceContext) -> ProgressTrend {
        // Simplified trend analysis - in real implementation would use historical data
        if self.historical_data.len() < 3 {
            return ProgressTrend::Unknown;
        }

        // For demo purposes, return steady
        ProgressTrend::Steady
    }

    /// Analyze project progress trend
    fn analyze_project_trend(&self, _context: &SubProjectContext) -> ProgressTrend {
        // Simplified trend analysis
        ProgressTrend::Steady
    }

    /// Identify workspace milestones
    fn identify_workspace_milestones(&self, workspace: &WorkspaceContext) -> Vec<Milestone> {
        let mut milestones = Vec::new();

        // Create milestones for each sub-project
        for (name, context) in &workspace.sub_project_contexts {
            let completion = context.task_summary.completion_percentage;
            let is_critical = context.derived_status.health == ProjectHealth::Critical;

            milestones.push(Milestone {
                name: format!("{} Completion", name),
                completion,
                is_critical,
                eta: self.calculate_milestone_eta(completion),
                dependencies: vec![],
                tasks: self.get_project_task_names(context),
            });
        }

        // Add overall workspace milestone
        let overall_completion = self.calculate_workspace_completion(workspace);
        milestones.push(Milestone {
            name: "Workspace Completion".to_string(),
            completion: overall_completion,
            is_critical: overall_completion < 50.0,
            eta: self.calculate_milestone_eta(overall_completion),
            dependencies: workspace.sub_project_contexts.keys().cloned().collect(),
            tasks: vec![],
        });

        milestones
    }

    /// Identify project milestones
    fn identify_project_milestones(&self, context: &SubProjectContext) -> Vec<Milestone> {
        let mut milestones = Vec::new();

        // Create milestone for each 25% completion threshold
        let completion = context.task_summary.completion_percentage;
        let thresholds = vec![25.0, 50.0, 75.0, 100.0];

        for threshold in thresholds {
            let milestone_completion = if completion >= threshold {
                100.0
            } else {
                (completion / threshold) * 100.0
            };

            milestones.push(Milestone {
                name: format!("{}% Completion", threshold),
                completion: milestone_completion,
                is_critical: threshold == 100.0,
                eta: self.calculate_milestone_eta(milestone_completion),
                dependencies: vec![],
                tasks: self.get_project_task_names(context),
            });
        }

        milestones
    }

    /// Calculate milestone ETA
    fn calculate_milestone_eta(&self, completion: f64) -> Option<DateTime<Utc>> {
        if completion >= 100.0 {
            return None; // Already completed
        }

        // Simple ETA calculation - in practice would use velocity
        let remaining_percentage = 100.0 - completion;
        let estimated_days = remaining_percentage / 2.0; // Assume 2% per day

        Some(Utc::now() + Duration::days(estimated_days as i64))
    }

    /// Get task names for a project
    fn get_project_task_names(&self, context: &SubProjectContext) -> Vec<String> {
        let mut task_names = Vec::new();

        for tasks in context.task_summary.tasks_by_status.values() {
            for task in tasks {
                task_names.push(task.title.clone());
            }
        }

        task_names
    }

    /// Detect workspace bottlenecks
    fn detect_workspace_bottlenecks(&self, workspace: &WorkspaceContext) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Check for projects with critical health
        for (name, context) in &workspace.sub_project_contexts {
            if context.derived_status.health == ProjectHealth::Critical {
                bottlenecks.push(Bottleneck {
                    description: format!("Project {} has critical health status", name),
                    severity: BottleneckSeverity::Critical,
                    impact: 25.0, // Assume 25% impact
                    resolution_suggestions: vec![
                        "Review and resolve critical issues".to_string(),
                        "Allocate additional resources".to_string(),
                    ],
                    affected_tasks: self.get_project_task_names(context),
                });
            }
        }

        // Check for high blocked task ratio
        for (name, context) in &workspace.sub_project_contexts {
            let blocked_count = context.task_summary.blocked_tasks.len();
            let total_count = context.task_summary.total_tasks;

            if total_count > 0 && (blocked_count as f64 / total_count as f64) > 0.2 {
                bottlenecks.push(Bottleneck {
                    description: format!(
                        "High blocked task ratio in {}: {}/{}",
                        name, blocked_count, total_count
                    ),
                    severity: BottleneckSeverity::High,
                    impact: 15.0,
                    resolution_suggestions: vec![
                        "Review and unblock pending tasks".to_string(),
                        "Address dependency issues".to_string(),
                    ],
                    affected_tasks: context
                        .task_summary
                        .blocked_tasks
                        .iter()
                        .map(|t| t.title.clone())
                        .collect(),
                });
            }
        }

        bottlenecks
    }

    /// Detect project bottlenecks
    fn detect_project_bottlenecks(&self, context: &SubProjectContext) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Check blocked tasks
        if !context.task_summary.blocked_tasks.is_empty() {
            let blocked_count = context.task_summary.blocked_tasks.len();
            let severity = if blocked_count > 5 {
                BottleneckSeverity::Critical
            } else if blocked_count > 2 {
                BottleneckSeverity::High
            } else {
                BottleneckSeverity::Medium
            };

            bottlenecks.push(Bottleneck {
                description: format!("{} blocked tasks preventing progress", blocked_count),
                severity,
                impact: (blocked_count as f64 / context.task_summary.total_tasks as f64) * 100.0,
                resolution_suggestions: vec![
                    "Review blocked task dependencies".to_string(),
                    "Escalate blockers to appropriate teams".to_string(),
                    "Consider alternative implementation approaches".to_string(),
                ],
                affected_tasks: context
                    .task_summary
                    .blocked_tasks
                    .iter()
                    .map(|t| t.title.clone())
                    .collect(),
            });
        }

        // Check for critical health
        if context.derived_status.health == ProjectHealth::Critical {
            bottlenecks.push(Bottleneck {
                description: "Project has critical health status".to_string(),
                severity: BottleneckSeverity::Critical,
                impact: 30.0,
                resolution_suggestions: vec![
                    "Address critical issues immediately".to_string(),
                    "Allocate additional resources".to_string(),
                ],
                affected_tasks: vec![],
            });
        }

        bottlenecks
    }

    /// Calculate workspace ETA
    fn calculate_workspace_eta(&self, workspace: &WorkspaceContext, velocity: f64) -> Option<f64> {
        if velocity <= 0.0 {
            return None;
        }

        let remaining_tasks: usize = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| {
                ctx.task_summary.total_tasks
                    - ctx
                        .task_summary
                        .tasks_by_status
                        .get(&TaskStatus::Completed)
                        .map(|t| t.len())
                        .unwrap_or(0)
            })
            .sum();

        Some((remaining_tasks as f64 / velocity) * 7.0) // Convert weeks to days
    }

    /// Calculate project ETA
    fn calculate_project_eta(&self, context: &SubProjectContext, velocity: f64) -> Option<f64> {
        if velocity <= 0.0 {
            return None;
        }

        let completed_tasks = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::Completed)
            .map(|t| t.len())
            .unwrap_or(0);

        let remaining_tasks = context.task_summary.total_tasks - completed_tasks;

        Some((remaining_tasks as f64 / velocity) * 7.0) // Convert weeks to days
    }

    /// Calculate workspace KPIs
    fn calculate_workspace_kpis(&self, workspace: &WorkspaceContext) -> HashMap<String, f64> {
        let mut kpis = HashMap::new();

        let total_projects = workspace.sub_project_contexts.len() as f64;
        let healthy_projects = workspace
            .sub_project_contexts
            .values()
            .filter(|ctx| ctx.derived_status.health == ProjectHealth::Healthy)
            .count() as f64;

        kpis.insert(
            "Health Score".to_string(),
            (healthy_projects / total_projects) * 100.0,
        );
        kpis.insert(
            "Overall Progress".to_string(),
            self.calculate_workspace_completion(workspace),
        );
        kpis.insert(
            "Velocity".to_string(),
            self.calculate_workspace_velocity(workspace),
        );

        let total_tasks: usize = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| ctx.task_summary.total_tasks)
            .sum();

        let blocked_tasks: usize = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| ctx.task_summary.blocked_tasks.len())
            .sum();

        if total_tasks > 0 {
            kpis.insert(
                "Blocked Task Ratio".to_string(),
                (blocked_tasks as f64 / total_tasks as f64) * 100.0,
            );
        }

        kpis
    }

    /// Calculate project KPIs
    fn calculate_project_kpis(&self, context: &SubProjectContext) -> HashMap<String, f64> {
        let mut kpis = HashMap::new();

        kpis.insert(
            "Completion".to_string(),
            context.task_summary.completion_percentage,
        );
        kpis.insert(
            "Velocity".to_string(),
            self.calculate_project_velocity(context),
        );

        let total_tasks = context.task_summary.total_tasks as f64;
        let blocked_tasks = context.task_summary.blocked_tasks.len() as f64;

        if total_tasks > 0.0 {
            kpis.insert(
                "Blocked Ratio".to_string(),
                (blocked_tasks / total_tasks) * 100.0,
            );
        }

        let in_progress_tasks = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::InProgress)
            .map(|t| t.len())
            .unwrap_or(0) as f64;

        if total_tasks > 0.0 {
            kpis.insert(
                "Active Ratio".to_string(),
                (in_progress_tasks / total_tasks) * 100.0,
            );
        }

        kpis
    }
}

impl Default for ProgressAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
