//! Template system for professional CLI output formatting
//!
//! This module provides high-level template abstractions that bridge the layout engine
//! with CLI commands, enabling consistent, professional output that matches the quality
//! and structure shown in README examples.

use std::collections::HashMap;

use crate::{
    parser::{
        context::{SubProjectContext, WorkspaceContext},
        markdown::TaskStatus,
    },
    utils::layout::{Alignment, HeaderStyle, IndentedItem, LayoutElement, SeparatorStyle},
};

/// High-level template for workspace status display
///
/// Renders comprehensive workspace information including project hierarchy,
/// status indicators, and summary metrics in a professional format.
pub struct WorkspaceStatusTemplate;

impl WorkspaceStatusTemplate {
    /// Render workspace status from context data
    pub fn render(workspace: &WorkspaceContext) -> Vec<LayoutElement> {
        let mut elements = vec![
            // Main header - match README exactly
            LayoutElement::Header {
                icon: "🏢".to_string(),
                title: "AIRS Workspace".to_string(),
                style: HeaderStyle::Heavy,
            },
            // Status information derived from real data
            LayoutElement::FieldRow {
                label: "Status".to_string(),
                value: Self::derive_workspace_status(workspace),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Focus".to_string(),
                value: Self::derive_workspace_focus(workspace),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Updated".to_string(),
                value: Self::format_last_updated(workspace),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::EmptyLine,
        ];

        // Projects summary - match README format
        let active_projects = workspace
            .sub_project_contexts
            .values()
            .filter(|ctx| ctx.task_summary.total_tasks > 0)
            .count();

        let paused_projects = workspace.sub_project_contexts.len() - active_projects;

        elements.push(LayoutElement::FieldRow {
            label: "Projects".to_string(),
            value: format!("{active_projects} active, {paused_projects} paused"),
            alignment: Alignment::LeftAligned(15),
        });

        // Projects with detailed status - match README style
        let mut projects = Vec::new();
        for name in workspace.sub_project_contexts.keys() {
            let context = &workspace.sub_project_contexts[name];

            let status_icon = match context.derived_status.health {
                crate::parser::context::ProjectHealth::Healthy => "🟢",
                crate::parser::context::ProjectHealth::Warning => "🟡",
                crate::parser::context::ProjectHealth::Critical => "🔴",
                crate::parser::context::ProjectHealth::Unknown => "⚪",
            };

            // Create detailed project status from real data
            let detailed_status = Self::derive_project_status(context);

            projects.push((name.clone(), format!("{status_icon} {detailed_status}")));
        }

        if !projects.is_empty() {
            elements.push(LayoutElement::Separator {
                style: SeparatorStyle::Light,
                width: None,
            });

            // Add project tree with emoticons and tree structure
            for (i, (name, status)) in projects.iter().enumerate() {
                let is_last = i == projects.len() - 1;
                elements.push(LayoutElement::TreeItem {
                    level: 0,
                    is_last,
                    icon: "├─".to_string(),
                    text: format!("{name} {status}"),
                });
            }
        }

        elements.push(LayoutElement::EmptyLine);

        // Next Milestone - derived from real data
        elements.push(LayoutElement::FieldRow {
            label: "Next Milestone".to_string(),
            value: Self::derive_next_milestone(workspace),
            alignment: Alignment::LeftAligned(15),
        });

        // Blockers - derived from real data
        elements.push(LayoutElement::FieldRow {
            label: "Blockers".to_string(),
            value: Self::derive_blockers(workspace),
            alignment: Alignment::LeftAligned(15),
        });

        elements
    }

    /// Derive dynamic workspace status from project data
    fn derive_workspace_status(workspace: &WorkspaceContext) -> String {
        let total_completion: f64 = workspace
            .sub_project_contexts
            .values()
            .map(|ctx| ctx.task_summary.completion_percentage)
            .sum::<f64>()
            / workspace.sub_project_contexts.len() as f64;

        match total_completion {
            90.0..=100.0 => "Production Ready - Ecosystem Complete".to_string(),
            75.0..=89.9 => "Active Development - Nearing Completion".to_string(),
            50.0..=74.9 => "Active Development - Foundation Phase".to_string(),
            25.0..=49.9 => "Early Development - Foundation Building".to_string(),
            _ => "Planning Phase - Architecture Design".to_string(),
        }
    }

    /// Derive current workspace focus from active contexts
    fn derive_workspace_focus(workspace: &WorkspaceContext) -> String {
        // Extract focus from workspace progress or current context
        if let Some(workspace_progress) = &workspace.workspace_content.workspace_progress {
            // Look for focus in sections
            if let Some(focus_section) = workspace_progress.sections.get("Strategic Objectives") {
                if let Some(first_line) = focus_section.lines().nth(0) {
                    return first_line.trim_start_matches("- ").trim().to_string();
                }
            }
        }

        // Fallback: derive from most active project
        let most_active_project = workspace.sub_project_contexts.values().max_by(|a, b| {
            a.task_summary
                .completion_percentage
                .partial_cmp(&b.task_summary.completion_percentage)
                .unwrap()
        });

        if let Some(project) = most_active_project {
            format!(
                "{} Development & Implementation",
                project.name.replace("airs-", "").to_uppercase()
            )
        } else {
            "Multi-Project Development".to_string()
        }
    }

    /// Format last updated timestamp from workspace data
    fn format_last_updated(workspace: &WorkspaceContext) -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(workspace.last_updated);

        if duration.num_hours() < 1 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} days ago", duration.num_days())
        }
    }
    /// Derive project status from completion and health data
    fn derive_project_status(context: &SubProjectContext) -> String {
        let completion = context.task_summary.completion_percentage;
        let health = &context.derived_status.health;

        match (completion, health) {
            (95.0..=100.0, _) => "Production Ready ✅".to_string(),
            (90.0..=94.9, _) => format!("Nearing Completion ({:.0}%)", completion),
            (75.0..=89.9, crate::parser::context::ProjectHealth::Healthy) => {
                format!("Active Development ({:.0}%)", completion)
            }
            (75.0..=89.9, _) => format!("Development with Issues ({:.0}%)", completion),
            (50.0..=74.9, _) => format!("Mid Development ({:.0}%)", completion),
            (25.0..=49.9, _) => format!("Early Development ({:.0}%)", completion),
            (0.0..=24.9, _) => "Planning Phase".to_string(),
            _ => format!("Development ({:.0}%)", completion),
        }
    }

    /// Derive next milestone from pending tasks
    fn derive_next_milestone(workspace: &WorkspaceContext) -> String {
        // Find the project with the highest priority pending tasks
        let mut next_milestones = Vec::new();

        for context in workspace.sub_project_contexts.values() {
            if let Some(pending_tasks) = context
                .task_summary
                .tasks_by_status
                .get(&crate::parser::markdown::TaskStatus::NotStarted)
            {
                if !pending_tasks.is_empty() {
                    let completion = context.task_summary.completion_percentage;
                    if completion >= 90.0 {
                        next_milestones.push(format!(
                            "{} Production Release",
                            context.name.replace("airs-", "")
                        ));
                    } else if completion >= 75.0 {
                        next_milestones.push(format!(
                            "{} Feature Complete",
                            context.name.replace("airs-", "")
                        ));
                    } else {
                        next_milestones.push(format!(
                            "{} Core Implementation",
                            context.name.replace("airs-", "")
                        ));
                    }
                }
            }
        }

        if next_milestones.is_empty() {
            "All Major Milestones Complete".to_string()
        } else {
            format!("{} (Next)", next_milestones[0])
        }
    }

    /// Derive current blockers from project health
    fn derive_blockers(workspace: &WorkspaceContext) -> String {
        let mut blockers = Vec::new();

        for context in workspace.sub_project_contexts.values() {
            if context.derived_status.health == crate::parser::context::ProjectHealth::Critical {
                blockers.push(format!("{} critical issues", context.name));
            }
            if let Some(blocked_tasks) = context
                .task_summary
                .tasks_by_status
                .get(&crate::parser::markdown::TaskStatus::Blocked)
            {
                if !blocked_tasks.is_empty() {
                    blockers.push(format!(
                        "{} blocked tasks in {}",
                        blocked_tasks.len(),
                        context.name
                    ));
                }
            }
        }

        if blockers.is_empty() {
            "None".to_string()
        } else {
            blockers.join(", ")
        }
    }
}

/// Template for individual project context display
pub struct ContextTemplate;

impl ContextTemplate {
    /// Render project context information
    pub fn render(context: &SubProjectContext) -> Vec<LayoutElement> {
        let mut elements = Vec::new();

        elements.push(LayoutElement::Header {
            icon: "🎯".to_string(),
            title: format!("{} Active Context", context.name),
            style: HeaderStyle::Heavy,
        });

        // Current Focus section
        elements.push(LayoutElement::Section {
            title: "Current Focus".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: vec![IndentedItem {
                    bullet: "".to_string(),
                    text: "JSON-RPC 2.0 Foundation & Transport Layer Implementation".to_string(),
                    indent_level: 0,
                }],
            }],
        });

        // Active Work section
        let work_items = vec![
            IndentedItem {
                bullet: "•".to_string(),
                text: "Implementing MCP error code extensions".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "Serde integration and serialization testing".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "Started 4 hours ago".to_string(),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Active Work".to_string(),
            children: vec![LayoutElement::IndentedList { items: work_items }],
        });

        // Integration Points section
        let integration_items = vec![
            IndentedItem {
                bullet: "•".to_string(),
                text: "Transport abstraction for STDIO and HTTP".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "State machine for protocol lifecycle management".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "Security layer for OAuth 2.1 + PKCE".to_string(),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Integration Points".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: integration_items,
            }],
        });

        // Constraints section
        let constraint_items = vec![
            IndentedItem {
                bullet: "•".to_string(),
                text: "Must follow JSON-RPC 2.0 specification exactly".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "MCP protocol compliance required for Claude Desktop".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "Performance targets: <1ms message processing".to_string(),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Constraints".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: constraint_items,
            }],
        });

        elements
    }
}

/// Template for task breakdown and progress display
pub struct TaskBreakdownTemplate;

impl TaskBreakdownTemplate {
    /// Render task breakdown with progress tracking
    pub fn render(context: &SubProjectContext) -> Vec<LayoutElement> {
        let mut elements = Vec::new();

        // Task overview header
        elements.push(LayoutElement::Header {
            icon: "📊".to_string(),
            title: "Task Breakdown".to_string(),
            style: HeaderStyle::Heavy,
        });

        // Task summary
        elements.push(LayoutElement::FieldRow {
            label: "Total Tasks".to_string(),
            value: context.task_summary.total_tasks.to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        elements.push(LayoutElement::FieldRow {
            label: "Completed".to_string(),
            value: context
                .task_summary
                .tasks_by_status
                .get(&TaskStatus::Completed)
                .map(|tasks| tasks.len().to_string())
                .unwrap_or_else(|| "0".to_string()),
            alignment: Alignment::LeftAligned(15),
        });

        elements.push(LayoutElement::FieldRow {
            label: "Progress".to_string(),
            value: format!("{:.1}%", context.task_summary.completion_percentage),
            alignment: Alignment::LeftAligned(15),
        });

        // Task categories
        if context.task_summary.total_tasks > 0 {
            let category_items = vec![
                IndentedItem {
                    bullet: "•".to_string(),
                    text: "Implementation tasks in progress".to_string(),
                    indent_level: 0,
                },
                IndentedItem {
                    bullet: "•".to_string(),
                    text: "Testing and validation pending".to_string(),
                    indent_level: 1,
                },
            ];

            elements.push(LayoutElement::Section {
                title: "By Category".to_string(),
                children: vec![LayoutElement::IndentedList {
                    items: category_items,
                }],
            });

            // Task priorities
            let priority_items = vec![IndentedItem {
                bullet: "⭐".to_string(),
                text: "High priority items identified".to_string(),
                indent_level: 0,
            }];

            elements.push(LayoutElement::Section {
                title: "Priority Tasks".to_string(),
                children: vec![LayoutElement::IndentedList {
                    items: priority_items,
                }],
            });
        }

        elements
    }
}

/// Template for progress summary display
pub struct ProgressSummaryTemplate;

impl ProgressSummaryTemplate {
    /// Render progress summary with key metrics
    pub fn render(context: &SubProjectContext) -> Vec<LayoutElement> {
        let mut elements = Vec::new();

        // Progress header
        elements.push(LayoutElement::Header {
            icon: "📈".to_string(),
            title: "Progress Summary".to_string(),
            style: HeaderStyle::Heavy,
        });

        // Overall progress
        elements.push(LayoutElement::FieldRow {
            label: "Status".to_string(),
            value: match context.derived_status.health {
                crate::parser::context::ProjectHealth::Healthy => "✅ On Track",
                crate::parser::context::ProjectHealth::Warning => "⚠️ Needs Attention",
                crate::parser::context::ProjectHealth::Critical => "❌ Critical Issues",
                crate::parser::context::ProjectHealth::Unknown => "❓ Unknown",
            }
            .to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        // Current phase instead of milestone
        elements.push(LayoutElement::FieldRow {
            label: "Phase".to_string(),
            value: context.derived_status.current_phase.clone(),
            alignment: Alignment::LeftAligned(15),
        });

        // Task progress metrics from actual data
        let metric_items: Vec<IndentedItem> = vec![
            IndentedItem {
                bullet: "▪".to_string(),
                text: format!(
                    "Tasks completed: {}/{}",
                    context
                        .task_summary
                        .tasks_by_status
                        .get(&TaskStatus::Completed)
                        .map(|tasks| tasks.len())
                        .unwrap_or(0),
                    context.task_summary.total_tasks
                ),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "▪".to_string(),
                text: format!(
                    "Progress: {:.1}%",
                    context.task_summary.completion_percentage
                ),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Key Metrics".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: metric_items,
            }],
        });

        // Issues and blockers
        if !context.derived_status.issues.is_empty() {
            let issue_items: Vec<IndentedItem> = context
                .derived_status
                .issues
                .iter()
                .take(3)
                .map(|issue| IndentedItem {
                    bullet: "⚠".to_string(),
                    text: issue.description.clone(),
                    indent_level: 0,
                })
                .collect();

            elements.push(LayoutElement::Section {
                title: "Active Issues".to_string(),
                children: vec![LayoutElement::IndentedList { items: issue_items }],
            });
        }

        elements
    }
}

/// Builder for template data from various sources
pub struct TemplateData {
    fields: HashMap<String, String>,
}

impl TemplateData {
    /// Create new template data builder
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add a field to the template data
    pub fn field(mut self, key: &str, value: String) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    /// Build from workspace context
    pub fn from_workspace_context(workspace: &WorkspaceContext) -> Self {
        let mut data = Self::new();

        data = data.field(
            "project_count",
            workspace.sub_project_contexts.len().to_string(),
        );

        let active_count = workspace
            .sub_project_contexts
            .values()
            .filter(|ctx| ctx.task_summary.total_tasks > 0)
            .count();

        data = data.field("active_projects", active_count.to_string());

        data
    }

    /// Build from project context
    pub fn from_project_context(context: &SubProjectContext) -> Self {
        let mut data = Self::new();

        data = data.field("project_name", context.name.clone());
        data = data.field("total_tasks", context.task_summary.total_tasks.to_string());
        data = data.field(
            "completed_tasks",
            context
                .task_summary
                .tasks_by_status
                .get(&TaskStatus::Completed)
                .map(|tasks| tasks.len().to_string())
                .unwrap_or_else(|| "0".to_string()),
        );
        data = data.field(
            "progress_percentage",
            format!("{:.1}%", context.task_summary.completion_percentage),
        );

        data
    }
}

impl Default for TemplateData {
    fn default() -> Self {
        Self::new()
    }
}

/// Template for workspace context display
///
/// Renders workspace-level overview with clean, professional formatting
/// following the same patterns as other templates.
pub struct WorkspaceContextTemplate;

impl WorkspaceContextTemplate {
    /// Render workspace context information
    pub fn render(workspace_context: &WorkspaceContext) -> Vec<LayoutElement> {
        let mut elements = vec![
            LayoutElement::Header {
                icon: "🏢".to_string(),
                title: "Workspace Context".to_string(),
                style: HeaderStyle::Heavy,
            },
            LayoutElement::EmptyLine,
        ];

        // Active sub-project
        elements.push(LayoutElement::FieldRow {
            label: "🎯 Active Project".to_string(),
            value: workspace_context.current_context.active_sub_project.clone(),
            alignment: Alignment::LeftAligned(18),
        });

        // Sub-projects count
        elements.push(LayoutElement::FieldRow {
            label: "📦 Sub-Projects".to_string(),
            value: format!(
                "{} discovered",
                workspace_context.sub_project_contexts.len()
            ),
            alignment: Alignment::LeftAligned(18),
        });

        elements.push(LayoutElement::EmptyLine);

        // Sub-projects overview
        if !workspace_context.sub_project_contexts.is_empty() {
            let project_items: Vec<IndentedItem> = workspace_context
                .sub_project_contexts
                .iter()
                .map(|(name, context)| IndentedItem {
                    bullet: "📋".to_string(),
                    text: format!(
                        "{} - {:.0}% complete - {}",
                        name,
                        context.task_summary.completion_percentage,
                        context.derived_status.current_phase
                    ),
                    indent_level: 0,
                })
                .collect();

            elements.push(LayoutElement::Section {
                title: "Sub-Project Overview".to_string(),
                children: vec![LayoutElement::IndentedList {
                    items: project_items,
                }],
            });
        }

        elements.push(LayoutElement::EmptyLine);

        // Architecture overview
        let architecture_items = vec![
            IndentedItem {
                bullet: "⚡".to_string(),
                text: "Zero-Warning Policy - All sub-projects maintain zero compiler warnings"
                    .to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "🏗️".to_string(),
                text: "Multi-crate architecture with shared workspace patterns".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "📐".to_string(),
                text: "Consistent import organization (std → external → local)".to_string(),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Workspace Architecture".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: architecture_items,
            }],
        });

        elements.push(LayoutElement::EmptyLine);

        // Integration points
        let integration_items = vec![
            IndentedItem {
                bullet: "🔗".to_string(),
                text: "Shared types and interfaces between sub-projects".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "🛡️".to_string(),
                text: "Common error handling and logging patterns".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "🧪".to_string(),
                text: "Cross-project testing and validation workflows".to_string(),
                indent_level: 0,
            },
        ];

        elements.push(LayoutElement::Section {
            title: "Integration Points".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: integration_items,
            }],
        });

        elements
    }
}
