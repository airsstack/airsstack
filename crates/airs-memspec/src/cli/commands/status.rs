//! Status command implementation
//!
//! Displays comprehensive memory bank status and project overview with support for
//! workspace-level and sub-project-specific views. Provides progress tracking,
//! health assessment, and actionable insights.

use crate::cli::commands::progress_analyzer::{
    BottleneckSeverity, ProgressAnalyzer, ProgressTrend,
};
use crate::cli::GlobalArgs;
use crate::parser::context::{ContextCorrelator, WorkspaceContext};
use crate::utils::fs::FsResult;
use crate::utils::output::{OutputConfig, OutputFormatter};
use crate::utils::templates::{ProjectStatusTemplate, WorkspaceStatusTemplate};

/// Run the status command with comprehensive workspace and project analysis
///
/// This function orchestrates the complete status analysis workflow:
/// 1. Initialize output formatting based on user preferences
/// 2. Discover and analyze workspace context using ContextCorrelator
/// 3. Generate appropriate status display (workspace or project specific)
/// 4. Present formatted results with visual enhancements
///
/// # Arguments
///
/// * `global` - Global CLI arguments including path and verbosity settings
/// * `detailed` - Whether to show detailed status information
/// * `project` - Optional project name for focused analysis
///
/// # Output Modes
///
/// - **Workspace Mode** (default): Overview of all projects and workspace health
/// - **Project Mode**: Detailed view of specific project when name provided
/// - **Detailed Mode**: Enhanced information when --detailed flag is used
pub fn run(
    global: &GlobalArgs,
    detailed: bool,
    project: Option<String>,
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
    match project {
        Some(project_name) => {
            show_project_status(
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
    // Use professional template system for workspace status display
    let elements = WorkspaceStatusTemplate::render(workspace_context);
    formatter.render_layout(&elements);

    // If detailed mode is requested, add additional analytics
    if detailed {
        // Perform advanced progress analysis
        let analytics = progress_analyzer.analyze_workspace(workspace_context);

        formatter.verbose(&format!(
            "⚡ Velocity: {:.1} tasks/week",
            analytics.velocity
        ));

        if let Some(eta_days) = analytics.eta_days {
            formatter.verbose(&format!("🏁 Estimated Completion: {eta_days:.0} days"));
        }

        let trend_icon = match analytics.trend {
            ProgressTrend::Accelerating => "📈",
            ProgressTrend::Steady => "➡️",
            ProgressTrend::Declining => "📉",
            ProgressTrend::Unknown => "❓",
        };
        formatter.verbose(&format!(
            "{} Progress Trend: {:?}",
            trend_icon, analytics.trend
        ));
    }

    Ok(())
}

/// Display detailed project status
///
/// Provides comprehensive analysis of a specific project including task breakdown,
/// progress tracking, current status, and actionable insights.
fn show_project_status(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    project_name: &str,
    detailed: bool,
    progress_analyzer: &ProgressAnalyzer,
) -> FsResult<()> {
    // Find the requested project
    let project_context = workspace_context
        .sub_project_contexts
        .get(project_name)
        .ok_or_else(|| {
            crate::utils::fs::FsError::ParseError {
                message: format!(
                    "Sub-project '{project_name}' not found. Available projects: {}",
                    workspace_context
                        .sub_project_contexts
                        .keys()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                suggestion: "Use 'airs-memspec status' to see all available projects, or check the project name spelling.".to_string(),
            }
        })?;

    // Use proper ProjectStatusTemplate instead of ContextTemplate
    // This shows status information (progress, health, tasks) instead of context information
    let elements = ProjectStatusTemplate::render(project_context);
    formatter.render_layout(&elements);

    // If detailed mode is requested, add additional analytics
    if detailed {
        // Perform advanced progress analysis
        let analytics = progress_analyzer.analyze_sub_project(project_context);

        formatter.verbose(&format!(
            "⚡ Velocity: {:.1} tasks/week",
            analytics.velocity
        ));

        if let Some(eta_days) = analytics.eta_days {
            formatter.verbose(&format!("🏁 Estimated Completion: {eta_days:.0} days"));
        }

        // Show any specific analytics for this project
        for bottleneck in analytics.bottlenecks.iter().take(3) {
            let severity_icon = match bottleneck.severity {
                BottleneckSeverity::High => "🔴",
                BottleneckSeverity::Medium => "🟡",
                BottleneckSeverity::Low => "🟢",
                BottleneckSeverity::Critical => "💥",
            };
            formatter.verbose(&format!(
                "{} Bottleneck: {}",
                severity_icon, bottleneck.description
            ));
        }
    }

    Ok(())
}
