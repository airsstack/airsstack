//! Progress Analysis Module
//!
//! Provides advanced progress tracking, milestone detection, and predictive analytics
//! for project status visualization. Analyzes task completion patterns, identifies
//! bottlenecks, and calculates project velocity metrics.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};

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
                name: format!("{name} Completion"),
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
                name: format!("{threshold}% Completion"),
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

    /// Detect workspace bottlenecks with enhanced analysis
    fn detect_workspace_bottlenecks(&self, workspace: &WorkspaceContext) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Enhanced critical project analysis
        for (name, context) in &workspace.sub_project_contexts {
            if context.derived_status.health == ProjectHealth::Critical {
                let issue_count = context.derived_status.issues.len();
                let blocked_count = context.task_summary.blocked_tasks.len();

                bottlenecks.push(Bottleneck {
                    description: format!(
                        "Project {name} has critical health ({issue_count} issues, {blocked_count} blocked tasks)"
                    ),
                    severity: BottleneckSeverity::Critical,
                    impact: 30.0 + (issue_count as f64 * 5.0),
                    resolution_suggestions: vec![
                        format!("URGENT: Focus leadership attention on {}", name),
                        "Conduct immediate project health review".to_string(),
                        "Consider reallocating resources".to_string(),
                        "Implement daily standups for critical items".to_string(),
                    ],
                    affected_tasks: self.get_project_task_names(context),
                });
            }
        }

        // Enhanced high blocked task ratio analysis
        for (name, context) in &workspace.sub_project_contexts {
            let blocked_count = context.task_summary.blocked_tasks.len();
            let total_count = context.task_summary.total_tasks;
            let blocked_ratio = if total_count > 0 {
                blocked_count as f64 / total_count as f64
            } else {
                0.0
            };

            if total_count > 0 && blocked_ratio > 0.2 {
                let severity = if blocked_ratio > 0.4 {
                    BottleneckSeverity::Critical
                } else if blocked_ratio > 0.3 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                };

                let mut suggestions = vec![
                    format!("Prioritize unblocking tasks in {}", name),
                    "Analyze dependency chains".to_string(),
                    "Consider parallel work streams".to_string(),
                ];

                if blocked_ratio > 0.35 {
                    suggestions.insert(0, "ESCALATION REQUIRED: High blocker density".to_string());
                }

                bottlenecks.push(Bottleneck {
                    description: format!(
                        "High blocked task ratio in {}: {}/{} ({:.1}%)",
                        name,
                        blocked_count,
                        total_count,
                        blocked_ratio * 100.0
                    ),
                    severity,
                    impact: 20.0 + (blocked_ratio * 30.0),
                    resolution_suggestions: suggestions,
                    affected_tasks: context
                        .task_summary
                        .blocked_tasks
                        .iter()
                        .map(|t| t.title.clone())
                        .collect(),
                });
            }
        }

        // Detect cross-project dependency bottlenecks
        let projects_with_issues: Vec<_> = workspace
            .sub_project_contexts
            .iter()
            .filter(|(_, ctx)| !ctx.derived_status.issues.is_empty())
            .collect();

        if projects_with_issues.len() > workspace.sub_project_contexts.len() / 2 {
            bottlenecks.push(Bottleneck {
                description: format!(
                    "Widespread issues across {} of {} projects indicate systemic problems",
                    projects_with_issues.len(),
                    workspace.sub_project_contexts.len()
                ),
                severity: BottleneckSeverity::High,
                impact: 25.0,
                resolution_suggestions: vec![
                    "Conduct workspace-wide architecture review".to_string(),
                    "Identify common patterns in issues".to_string(),
                    "Review cross-project dependencies".to_string(),
                    "Consider standardizing tools and processes".to_string(),
                ],
                affected_tasks: vec!["Multiple projects".to_string()],
            });
        }

        // Detect velocity distribution issues
        let velocities: Vec<f64> = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| self.calculate_project_velocity(ctx))
            .collect();

        if velocities.len() > 1 {
            let avg_velocity = velocities.iter().sum::<f64>() / velocities.len() as f64;
            let low_velocity_projects: Vec<_> = workspace
                .sub_project_contexts
                .iter()
                .filter(|(_, ctx)| {
                    let velocity = self.calculate_project_velocity(ctx);
                    velocity < avg_velocity * 0.5 && velocity < 1.0
                })
                .collect();

            if low_velocity_projects.len() > 1 {
                let project_names: Vec<String> = low_velocity_projects
                    .iter()
                    .map(|(name, _)| (*name).clone())
                    .collect();

                bottlenecks.push(Bottleneck {
                    description: format!(
                        "Low velocity in projects: {} (avg: {:.1} tasks/week)",
                        project_names.join(", "),
                        avg_velocity
                    ),
                    severity: BottleneckSeverity::Medium,
                    impact: 15.0,
                    resolution_suggestions: vec![
                        "Analyze resource allocation across projects".to_string(),
                        "Review task complexity and breakdown".to_string(),
                        "Consider cross-project knowledge sharing".to_string(),
                        "Identify skill gaps and training needs".to_string(),
                    ],
                    affected_tasks: project_names,
                });
            }
        }

        bottlenecks
    }

    /// Detect project bottlenecks with enhanced analysis
    fn detect_project_bottlenecks(&self, context: &SubProjectContext) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Enhanced blocked tasks analysis
        if !context.task_summary.blocked_tasks.is_empty() {
            let blocked_count = context.task_summary.blocked_tasks.len();
            let total_tasks = context.task_summary.total_tasks;
            let blocked_ratio = blocked_count as f64 / total_tasks as f64;

            // Calculate severity based on multiple factors
            let severity = if blocked_ratio > 0.4 || blocked_count > 8 {
                BottleneckSeverity::Critical
            } else if blocked_ratio > 0.25 || blocked_count > 4 {
                BottleneckSeverity::High
            } else if blocked_ratio > 0.15 || blocked_count > 2 {
                BottleneckSeverity::Medium
            } else {
                BottleneckSeverity::Low
            };

            // Enhanced impact calculation
            let base_impact = blocked_ratio * 100.0;
            let velocity_impact = if blocked_count > 3 { 15.0 } else { 5.0 };
            let timeline_impact = if severity >= BottleneckSeverity::High {
                20.0
            } else {
                10.0
            };
            let total_impact = base_impact + velocity_impact + timeline_impact;

            // Enhanced resolution suggestions based on context
            let mut suggestions = vec![
                "Analyze root causes of blocked tasks".to_string(),
                "Prioritize unblocking critical path items".to_string(),
            ];

            if blocked_count > 5 {
                suggestions.extend_from_slice(&[
                    "Consider parallel work streams to reduce dependencies".to_string(),
                    "Schedule dedicated blocker resolution session".to_string(),
                ]);
            }

            if blocked_ratio > 0.3 {
                suggestions.push("Review project architecture for dependency issues".to_string());
            }

            // Add escalation suggestions based on severity
            match severity {
                BottleneckSeverity::Critical => {
                    suggestions.insert(
                        0,
                        "IMMEDIATE ACTION REQUIRED: Escalate to project leadership".to_string(),
                    );
                    suggestions.push("Consider bringing in additional expertise".to_string());
                }
                BottleneckSeverity::High => {
                    suggestions.push("Escalate to technical leads within 24 hours".to_string());
                }
                _ => {}
            }

            bottlenecks.push(Bottleneck {
                description: format!(
                    "{} blocked tasks ({:.1}% of total) creating significant bottleneck",
                    blocked_count,
                    blocked_ratio * 100.0
                ),
                severity,
                impact: total_impact.min(100.0),
                resolution_suggestions: suggestions,
                affected_tasks: context
                    .task_summary
                    .blocked_tasks
                    .iter()
                    .map(|t| t.title.clone())
                    .collect(),
            });
        }

        // Enhanced critical health analysis
        if context.derived_status.health == ProjectHealth::Critical {
            let critical_issues = context.derived_status.issues.len();
            let severity_multiplier = if critical_issues > 3 { 1.5 } else { 1.0 };

            bottlenecks.push(Bottleneck {
                description: format!(
                    "Critical project health with {critical_issues} unresolved issues"
                ),
                severity: BottleneckSeverity::Critical,
                impact: (35.0_f64 * severity_multiplier).min(100.0),
                resolution_suggestions: vec![
                    "URGENT: Address all critical issues within 48 hours".to_string(),
                    "Conduct immediate project health assessment".to_string(),
                    "Allocate senior resources to critical path".to_string(),
                    "Consider project scope adjustment if necessary".to_string(),
                ],
                affected_tasks: vec!["All project tasks".to_string()],
            });
        }

        // Detect dependency chain bottlenecks
        let in_progress_count = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::InProgress)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        let not_started_count = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::NotStarted)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        // If too many tasks are not started vs in progress, might indicate dependency issues
        if not_started_count > 0 && in_progress_count > 0 {
            let dependency_ratio =
                not_started_count as f64 / (in_progress_count + not_started_count) as f64;

            if dependency_ratio > 0.7 && not_started_count > 3 {
                bottlenecks.push(Bottleneck {
                    description: format!(
                        "Potential dependency bottleneck: {not_started_count} tasks waiting to start vs {in_progress_count} in progress"
                    ),
                    severity: BottleneckSeverity::Medium,
                    impact: 15.0,
                    resolution_suggestions: vec![
                        "Review task dependencies and sequencing".to_string(),
                        "Identify tasks that can be parallelized".to_string(),
                        "Consider breaking down large tasks".to_string(),
                    ],
                    affected_tasks: vec![format!("{} pending tasks", not_started_count)],
                });
            }
        }

        // Detect velocity bottlenecks (low completion rate)
        let completed_count = context
            .task_summary
            .tasks_by_status
            .get(&TaskStatus::Completed)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        let completion_ratio = completed_count as f64 / context.task_summary.total_tasks as f64;

        if context.task_summary.total_tasks > 5 && completion_ratio < 0.3 {
            bottlenecks.push(Bottleneck {
                description: format!(
                    "Low completion velocity: only {:.1}% tasks completed",
                    completion_ratio * 100.0
                ),
                severity: if completion_ratio < 0.15 {
                    BottleneckSeverity::High
                } else {
                    BottleneckSeverity::Medium
                },
                impact: (30.0 - completion_ratio * 30.0).max(10.0),
                resolution_suggestions: vec![
                    "Analyze task complexity and breakdown".to_string(),
                    "Review resource allocation and availability".to_string(),
                    "Consider if scope needs adjustment".to_string(),
                    "Identify and address skill gaps".to_string(),
                ],
                affected_tasks: vec!["All incomplete tasks".to_string()],
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
