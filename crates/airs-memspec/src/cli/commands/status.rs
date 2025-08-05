//! Status command implementation
//!
//! Displays comprehensive memory bank status and project overview with support for
//! workspace-level and sub-project-specific views. Provides progress tracking,
//! health assessment, and actionable insights.

use crate::cli::commands::progress_analyzer::{
    BottleneckSeverity, ProgressAnalyzer, ProgressTrend,
};
use crate::cli::GlobalArgs;
use crate::parser::context::{ContextCorrelator, ProjectHealth, TaskSummary, WorkspaceContext};
use crate::parser::markdown::TaskStatus;
use crate::utils::fs::FsResult;
use crate::utils::output::{OutputConfig, OutputFormatter};

/// Run the status command with comprehensive workspace and project analysis
///
/// This function orchestrates the complete status analysis workflow:
/// 1. Initialize output formatting based on user preferences
/// 2. Discover and analyze workspace context using ContextCorrelator
/// 3. Generate appropriate status display (workspace or sub-project specific)
/// 4. Present formatted results with visual enhancements
///
/// # Arguments
///
/// * `global` - Global CLI arguments including path and verbosity settings
/// * `detailed` - Whether to show detailed status information
/// * `sub_project` - Optional sub-project name for focused analysis
///
/// # Output Modes
///
/// - **Workspace Mode** (default): Overview of all sub-projects and workspace health
/// - **Sub-Project Mode**: Detailed view of specific sub-project when name provided
/// - **Detailed Mode**: Enhanced information when --detailed flag is used
pub fn run(
    global: &GlobalArgs,
    detailed: bool,
    sub_project: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize output formatter with user preferences
    let output_config = OutputConfig::new(global.no_color, global.verbose, global.quiet);
    let formatter = OutputFormatter::new(output_config);

    // Initialize context correlator and discover workspace structure
    let mut correlator = ContextCorrelator::new();

    // Determine workspace path from global args or current directory
    let current_dir = std::env::current_dir()?;
    let workspace_path = global.path.as_deref().unwrap_or(&current_dir);

    // Discover and correlate workspace context
    formatter.verbose(&format!(
        "Analyzing workspace: {}",
        workspace_path.display()
    ));
    let workspace_context = correlator.discover_and_correlate(workspace_path)?;

    // Initialize progress analyzer for enhanced metrics
    let progress_analyzer = ProgressAnalyzer::new();

    // Generate status display based on requested mode
    match sub_project {
        Some(project_name) => {
            show_sub_project_status(
                &formatter,
                workspace_context,
                &project_name,
                detailed,
                &progress_analyzer,
            )?;
        }
        None => {
            show_workspace_status(&formatter, workspace_context, detailed, &progress_analyzer)?;
        }
    }

    Ok(())
}

/// Display comprehensive workspace status overview
///
/// Shows overall workspace health, sub-project summaries, progress metrics,
/// and high-level actionable insights across all projects.
fn show_workspace_status(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    detailed: bool,
    progress_analyzer: &ProgressAnalyzer,
) -> FsResult<()> {
    // Calculate overall workspace health and metrics
    let workspace_health = calculate_workspace_health(workspace_context);
    let overall_progress = calculate_overall_progress(workspace_context);
    let active_project = &workspace_context.current_context.active_sub_project;

    // Perform advanced progress analysis
    let analytics = progress_analyzer.analyze_workspace(workspace_context);

    // Display workspace header
    formatter.header("Workspace Status");

    // Overall health indicator
    let health_icon = match workspace_health {
        ProjectHealth::Healthy => "✅",
        ProjectHealth::Warning => "⚠️",
        ProjectHealth::Critical => "❌",
        ProjectHealth::Unknown => "❓",
    };

    formatter.info(&format!(
        "{health_icon} Workspace Health: {workspace_health:?}"
    ));
    formatter.info(&format!("🎯 Active Context: {active_project}"));
    formatter.info(&format!("📈 Overall Progress: {overall_progress:.1}%"));

    // Enhanced progress metrics
    formatter.info(&format!(
        "⚡ Velocity: {:.1} tasks/week",
        analytics.velocity
    ));

    if let Some(eta_days) = analytics.eta_days {
        formatter.info(&format!("🏁 Estimated Completion: {eta_days:.0} days"));
    }

    let trend_icon = match analytics.trend {
        ProgressTrend::Accelerating => "📈",
        ProgressTrend::Steady => "➡️",
        ProgressTrend::Declining => "📉",
        ProgressTrend::Unknown => "❓",
    };
    formatter.info(&format!(
        "{} Progress Trend: {:?}",
        trend_icon, analytics.trend
    ));

    formatter.separator();

    // Sub-projects summary
    formatter.header("Sub-Projects");

    let mut project_count = 0;
    let mut healthy_count = 0;

    for (name, context) in &workspace_context.sub_project_contexts {
        project_count += 1;

        let health_icon = match context.derived_status.health {
            ProjectHealth::Healthy => {
                healthy_count += 1;
                "✅"
            }
            ProjectHealth::Warning => "⚠️",
            ProjectHealth::Critical => "❌",
            ProjectHealth::Unknown => "❓",
        };

        let completion = context.task_summary.completion_percentage;
        let phase = &context.derived_status.current_phase;
        let active_indicator = if name == active_project {
            " (ACTIVE)"
        } else {
            ""
        };

        formatter.info(&format!(
            "  {health_icon} {name}: {completion:.1}% - {phase}{active_indicator}"
        ));

        // Show additional details in detailed mode
        if detailed {
            show_project_summary_details(formatter, context);
        }
    }

    formatter.separator();

    // Summary statistics
    formatter.header("Workspace Summary");
    formatter.info(&format!("📊 Total Projects: {project_count}"));
    formatter.info(&format!(
        "✅ Healthy Projects: {healthy_count}/{project_count}"
    ));

    // Recent activity
    show_recent_activity(formatter, workspace_context);

    // Enhanced analytics sections
    if detailed {
        show_workspace_milestones(formatter, &analytics);
        show_workspace_bottlenecks(formatter, &analytics);
        show_workspace_kpis(formatter, &analytics);
    }

    // Issues and recommendations
    show_workspace_issues_and_recommendations(formatter, workspace_context);

    Ok(())
}

/// Display detailed sub-project status
///
/// Provides comprehensive analysis of a specific sub-project including task breakdown,
/// progress tracking, current context, and actionable insights.
fn show_sub_project_status(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    project_name: &str,
    detailed: bool,
    progress_analyzer: &ProgressAnalyzer,
) -> FsResult<()> {
    // Find the requested sub-project
    let project_context = workspace_context
        .sub_project_contexts
        .get(project_name)
        .ok_or_else(|| {
            crate::utils::fs::FsError::ParseError(format!(
                "Sub-project '{project_name}' not found in workspace"
            ))
        })?;

    // Display project header
    formatter.header(&format!("Sub-Project Status: {project_name}"));

    // Perform advanced progress analysis
    let analytics = progress_analyzer.analyze_sub_project(project_context);

    // Project health and overview
    let health_icon = match project_context.derived_status.health {
        ProjectHealth::Healthy => "✅",
        ProjectHealth::Warning => "⚠️",
        ProjectHealth::Critical => "❌",
        ProjectHealth::Unknown => "❓",
    };

    formatter.info(&format!(
        "{} Health: {:?}",
        health_icon, project_context.derived_status.health
    ));
    formatter.info(&format!(
        "📈 Progress: {:.1}%",
        project_context.task_summary.completion_percentage
    ));
    formatter.info(&format!(
        "🎯 Current Phase: {}",
        project_context.derived_status.current_phase
    ));

    // Enhanced progress metrics
    formatter.info(&format!(
        "⚡ Velocity: {:.1} tasks/week",
        analytics.velocity
    ));

    if let Some(eta_days) = analytics.eta_days {
        formatter.info(&format!("🏁 Estimated Completion: {eta_days:.0} days"));
    }

    let trend_icon = match analytics.trend {
        ProgressTrend::Accelerating => "📈",
        ProgressTrend::Steady => "➡️",
        ProgressTrend::Declining => "📉",
        ProgressTrend::Unknown => "❓",
    };
    formatter.info(&format!(
        "{} Progress Trend: {:?}",
        trend_icon, analytics.trend
    ));

    formatter.separator();

    // Task breakdown
    show_task_breakdown(formatter, &project_context.task_summary);

    // Current context and recent activity
    show_project_context_details(formatter, project_context, detailed);

    // Enhanced analytics sections
    if detailed {
        show_project_milestones(formatter, &analytics);
        show_project_bottlenecks(formatter, &analytics);
        show_project_kpis(formatter, &analytics);
    }

    // Issues and recommendations
    show_project_issues_and_recommendations(formatter, project_context);

    Ok(())
}

/// Calculate overall workspace health based on all sub-projects
fn calculate_workspace_health(workspace_context: &WorkspaceContext) -> ProjectHealth {
    let healths: Vec<&ProjectHealth> = workspace_context
        .sub_project_contexts
        .values()
        .map(|ctx| &ctx.derived_status.health)
        .collect();

    if healths.is_empty() {
        return ProjectHealth::Unknown;
    }

    // If any project is critical, workspace is critical
    if healths.contains(&(&ProjectHealth::Critical)) {
        return ProjectHealth::Critical;
    }

    // If any project has warnings, workspace has warnings
    if healths.contains(&(&ProjectHealth::Warning)) {
        return ProjectHealth::Warning;
    }

    // If all projects are healthy, workspace is healthy
    if healths.iter().all(|&h| h == &ProjectHealth::Healthy) {
        return ProjectHealth::Healthy;
    }

    ProjectHealth::Unknown
}

/// Calculate overall progress across all sub-projects
fn calculate_overall_progress(workspace_context: &WorkspaceContext) -> f64 {
    if workspace_context.sub_project_contexts.is_empty() {
        return 0.0;
    }

    let total_progress: f64 = workspace_context
        .sub_project_contexts
        .values()
        .map(|ctx| ctx.task_summary.completion_percentage)
        .sum();

    total_progress / workspace_context.sub_project_contexts.len() as f64
}

/// Show summary details for a project in workspace mode
fn show_project_summary_details(
    formatter: &OutputFormatter,
    context: &crate::parser::context::SubProjectContext,
) {
    formatter.verbose(&format!(
        "    📝 Total Tasks: {}",
        context.task_summary.total_tasks
    ));

    if !context.task_summary.blocked_tasks.is_empty() {
        formatter.verbose(&format!(
            "    🚧 Blocked Tasks: {}",
            context.task_summary.blocked_tasks.len()
        ));
    }

    if !context.derived_status.issues.is_empty() {
        formatter.verbose(&format!(
            "    ⚠️  Issues: {}",
            context.derived_status.issues.len()
        ));
    }
}

/// Display task breakdown with status distribution
fn show_task_breakdown(formatter: &OutputFormatter, task_summary: &TaskSummary) {
    formatter.header("Task Breakdown");

    formatter.info(&format!("📝 Total Tasks: {}", task_summary.total_tasks));
    formatter.info(&format!(
        "📈 Completion: {:.1}%",
        task_summary.completion_percentage
    ));

    // Task status distribution
    for (status, tasks) in &task_summary.tasks_by_status {
        let status_icon = match status {
            TaskStatus::Completed => "✅",
            TaskStatus::InProgress => "🔄",
            TaskStatus::NotStarted => "⏳",
            TaskStatus::Blocked => "🚧",
            TaskStatus::Abandoned => "❌",
            TaskStatus::Unknown(_) => "❓",
        };

        formatter.info(&format!("  {} {:?}: {}", status_icon, status, tasks.len()));
    }

    // Blocked tasks details
    if !task_summary.blocked_tasks.is_empty() {
        formatter.separator();
        formatter.warning(&format!(
            "🚧 Blocked Tasks ({})",
            task_summary.blocked_tasks.len()
        ));
        for task in &task_summary.blocked_tasks {
            formatter.info(&format!("  • {}", task.title));
        }
    }

    // Next priority tasks
    if !task_summary.next_tasks.is_empty() {
        formatter.separator();
        formatter.info(&format!(
            "🎯 Next Priority Tasks ({})",
            task_summary.next_tasks.len()
        ));
        for task in task_summary.next_tasks.iter().take(3) {
            formatter.info(&format!("  • {}", task.title));
        }
    }
}

/// Display project context and recent activity details
fn show_project_context_details(
    formatter: &OutputFormatter,
    context: &crate::parser::context::SubProjectContext,
    detailed: bool,
) {
    formatter.separator();
    formatter.header("Recent Activity");

    // Show recent tasks
    if !context.task_summary.recent_tasks.is_empty() {
        for task in context.task_summary.recent_tasks.iter().take(3) {
            let status_icon = match task.status {
                TaskStatus::Completed => "✅",
                TaskStatus::InProgress => "🔄",
                TaskStatus::NotStarted => "⏳",
                TaskStatus::Blocked => "🚧",
                TaskStatus::Abandoned => "❌",
                TaskStatus::Unknown(_) => "❓",
            };
            formatter.info(&format!("  {} {}", status_icon, task.title));
        }
    } else {
        formatter.info("  No recent task activity");
    }

    if detailed {
        formatter.separator();
        formatter.header("Progress Indicators");

        for indicator in &context.derived_status.progress_indicators {
            formatter.info(&format!(
                "  📊 {}: {:.1}%",
                indicator.description, indicator.current_value
            ));
            if let Some(target) = indicator.target_value {
                formatter.verbose(&format!("    Target: {target:.1}%"));
            }
        }
    }
}

/// Display recent activity across the workspace
fn show_recent_activity(formatter: &OutputFormatter, workspace_context: &WorkspaceContext) {
    formatter.header("Recent Activity");

    // Collect recent tasks from all projects
    let mut all_recent_tasks = Vec::new();
    for (project_name, context) in &workspace_context.sub_project_contexts {
        for task in &context.task_summary.recent_tasks {
            all_recent_tasks.push((project_name, task));
        }
    }

    // Sort by most recent and take top 3
    all_recent_tasks.sort_by(|a, b| b.1.title.cmp(&a.1.title)); // Simple sort by title for now

    if all_recent_tasks.is_empty() {
        formatter.info("  No recent activity");
    } else {
        for (project_name, task) in all_recent_tasks.iter().take(3) {
            let status_icon = match task.status {
                TaskStatus::Completed => "✅",
                TaskStatus::InProgress => "🔄",
                TaskStatus::NotStarted => "⏳",
                TaskStatus::Blocked => "🚧",
                TaskStatus::Abandoned => "❌",
                TaskStatus::Unknown(_) => "❓",
            };
            formatter.info(&format!(
                "  {} [{}] {}",
                status_icon, project_name, task.title
            ));
        }
    }
}

/// Display workspace-level issues and recommendations
fn show_workspace_issues_and_recommendations(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) {
    // Collect all issues across projects
    let mut all_issues = Vec::new();
    let mut all_recommendations = Vec::new();

    for context in workspace_context.sub_project_contexts.values() {
        all_issues.extend(&context.derived_status.issues);
        all_recommendations.extend(&context.derived_status.recommendations);
    }

    if !all_issues.is_empty() {
        formatter.separator();
        formatter.header("Issues");

        for issue in all_issues.iter().take(3) {
            let severity_icon = match issue.severity {
                crate::parser::context::IssueSeverity::Critical => "🔴",
                crate::parser::context::IssueSeverity::High => "🟡",
                crate::parser::context::IssueSeverity::Medium => "🟡",
                crate::parser::context::IssueSeverity::Low => "🔵",
            };
            formatter.warning(&format!("  {} {}", severity_icon, issue.description));
        }
    }

    if !all_recommendations.is_empty() {
        formatter.separator();
        formatter.header("Recommendations");

        for recommendation in all_recommendations.iter().take(3) {
            formatter.info(&format!("  💡 {recommendation}"));
        }
    }
}

/// Display project-specific issues and recommendations
fn show_project_issues_and_recommendations(
    formatter: &OutputFormatter,
    context: &crate::parser::context::SubProjectContext,
) {
    if !context.derived_status.issues.is_empty() {
        formatter.separator();
        formatter.header("Issues");

        for issue in &context.derived_status.issues {
            let severity_icon = match issue.severity {
                crate::parser::context::IssueSeverity::Critical => "🔴",
                crate::parser::context::IssueSeverity::High => "🟡",
                crate::parser::context::IssueSeverity::Medium => "🟡",
                crate::parser::context::IssueSeverity::Low => "🔵",
            };
            formatter.warning(&format!("  {} {}", severity_icon, issue.description));
        }
    }

    if !context.derived_status.recommendations.is_empty() {
        formatter.separator();
        formatter.header("Recommendations");

        for recommendation in &context.derived_status.recommendations {
            formatter.info(&format!("  💡 {recommendation}"));
        }
    }
}

/// Display workspace milestones
fn show_workspace_milestones(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.milestones.is_empty() {
        formatter.separator();
        formatter.header("Milestones");

        for milestone in &analytics.milestones {
            let critical_icon = if milestone.is_critical {
                "🔥"
            } else {
                "🎯"
            };
            formatter.info(&format!(
                "  {} {}: {:.1}%",
                critical_icon, milestone.name, milestone.completion
            ));

            if let Some(eta) = milestone.eta {
                formatter.verbose(&format!("    ETA: {}", eta.format("%Y-%m-%d")));
            }

            if !milestone.dependencies.is_empty() {
                formatter.verbose(&format!(
                    "    Dependencies: {}",
                    milestone.dependencies.join(", ")
                ));
            }
        }
    }
}

/// Display workspace bottlenecks
fn show_workspace_bottlenecks(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.bottlenecks.is_empty() {
        formatter.separator();
        formatter.header("Bottlenecks");

        for bottleneck in &analytics.bottlenecks {
            let severity_icon = match bottleneck.severity {
                BottleneckSeverity::Critical => "🔴",
                BottleneckSeverity::High => "🟡",
                BottleneckSeverity::Medium => "🟠",
                BottleneckSeverity::Low => "🔵",
            };

            formatter.warning(&format!(
                "  {} {} (Impact: {:.1}%)",
                severity_icon, bottleneck.description, bottleneck.impact
            ));

            for suggestion in &bottleneck.resolution_suggestions {
                formatter.verbose(&format!("    💡 {suggestion}"));
            }
        }
    }
}

/// Display workspace KPIs
fn show_workspace_kpis(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.kpis.is_empty() {
        formatter.separator();
        formatter.header("Key Performance Indicators");

        for (name, value) in &analytics.kpis {
            formatter.info(&format!("  📊 {name}: {value:.1}"));
        }
    }
}

/// Display project milestones
fn show_project_milestones(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.milestones.is_empty() {
        formatter.separator();
        formatter.header("Milestones");

        for milestone in &analytics.milestones {
            let critical_icon = if milestone.is_critical {
                "🔥"
            } else {
                "🎯"
            };
            let progress_bar = create_progress_bar(milestone.completion);

            formatter.info(&format!(
                "  {} {}: {:.1}% {}",
                critical_icon, milestone.name, milestone.completion, progress_bar
            ));

            if let Some(eta) = milestone.eta {
                formatter.verbose(&format!("    ETA: {}", eta.format("%Y-%m-%d")));
            }
        }
    }
}

/// Display project bottlenecks
fn show_project_bottlenecks(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.bottlenecks.is_empty() {
        formatter.separator();
        formatter.header("Bottlenecks");

        for bottleneck in &analytics.bottlenecks {
            let severity_icon = match bottleneck.severity {
                BottleneckSeverity::Critical => "🔴",
                BottleneckSeverity::High => "🟡",
                BottleneckSeverity::Medium => "🟠",
                BottleneckSeverity::Low => "🔵",
            };

            formatter.warning(&format!(
                "  {} {} (Impact: {:.1}%)",
                severity_icon, bottleneck.description, bottleneck.impact
            ));

            if !bottleneck.affected_tasks.is_empty() {
                formatter.verbose(&format!(
                    "    Affected tasks: {}",
                    bottleneck.affected_tasks.len()
                ));
            }

            for suggestion in bottleneck.resolution_suggestions.iter().take(2) {
                formatter.verbose(&format!("    💡 {suggestion}"));
            }
        }
    }
}

/// Display project KPIs
fn show_project_kpis(
    formatter: &OutputFormatter,
    analytics: &crate::cli::commands::progress_analyzer::ProgressAnalytics,
) {
    if !analytics.kpis.is_empty() {
        formatter.separator();
        formatter.header("Key Performance Indicators");

        for (name, value) in &analytics.kpis {
            let kpi_icon = match name.as_str() {
                "Completion" => "📈",
                "Velocity" => "⚡",
                "Blocked Ratio" => "🚧",
                "Active Ratio" => "🔄",
                _ => "📊",
            };

            formatter.info(&format!("  {kpi_icon} {name}: {value:.1}"));
        }
    }
}

/// Create a simple text-based progress bar
fn create_progress_bar(percentage: f64) -> String {
    let filled = (percentage / 10.0) as usize;
    let empty = 10 - filled;

    format!(
        "[{}{}] {:.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}
