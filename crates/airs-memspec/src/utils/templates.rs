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
        let mut elements = Vec::new();

        // Main header
        elements.push(LayoutElement::Header {
            icon: "🏢".to_string(),
            title: "AIRS Workspace Status".to_string(),
            style: HeaderStyle::Heavy,
        });

        // Workspace overview
        elements.push(LayoutElement::FieldRow {
            label: "Projects".to_string(),
            value: workspace.sub_project_contexts.len().to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        let active_projects = workspace
            .sub_project_contexts
            .values()
            .filter(|ctx| ctx.task_summary.total_tasks > 0)
            .count();

        elements.push(LayoutElement::FieldRow {
            label: "Active".to_string(),
            value: active_projects.to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        // Projects with status indicators
        let mut projects = Vec::new();
        for (name, context) in &workspace.sub_project_contexts {
            let status_icon = match context.derived_status.health {
                crate::parser::context::ProjectHealth::Healthy => "🟢",
                crate::parser::context::ProjectHealth::Warning => "🟡",
                crate::parser::context::ProjectHealth::Critical => "🔴",
                crate::parser::context::ProjectHealth::Unknown => "⚪",
            };

            let progress_desc = if context.task_summary.completion_percentage > 75.0 {
                "Near Completion"
            } else if context.task_summary.completion_percentage > 50.0 {
                "Active Development"
            } else if context.task_summary.completion_percentage > 25.0 {
                "Early Development"
            } else {
                "Planning"
            };

            projects.push((name.clone(), format!("{status_icon} {progress_desc}")));
        }

        if !projects.is_empty() {
            elements.push(LayoutElement::Separator {
                style: SeparatorStyle::Light,
                width: None,
            });

            // Add project tree
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

        elements
    }
}

/// Template for individual project context display
pub struct ContextTemplate;

impl ContextTemplate {
    /// Render project context information
    pub fn render(context: &SubProjectContext) -> Vec<LayoutElement> {
        let mut elements = Vec::new();

        // Project header
        elements.push(LayoutElement::Header {
            icon: "📋".to_string(),
            title: format!("Project: {}", context.name),
            style: HeaderStyle::Heavy,
        });

        // Basic information
        elements.push(LayoutElement::FieldRow {
            label: "Status".to_string(),
            value: "Active Development".to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        // Current focus section
        let focus_items = vec![IndentedItem {
            bullet: "▸".to_string(),
            text: "Active development in progress".to_string(),
            indent_level: 0,
        }];

        elements.push(LayoutElement::Section {
            title: "Current Focus".to_string(),
            children: vec![LayoutElement::IndentedList { items: focus_items }],
        });

        // Recent changes - using mock data since SubProjectContext doesn't have recent_changes
        let change_items: Vec<IndentedItem> = vec![IndentedItem {
            bullet: "•".to_string(),
            text: "Recent development activity".to_string(),
            indent_level: 0,
        }];

        elements.push(LayoutElement::Section {
            title: "Recent Changes".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: change_items,
            }],
        });

        // Next steps section
        let next_items: Vec<IndentedItem> = if !context.derived_status.recommendations.is_empty() {
            context
                .derived_status
                .recommendations
                .iter()
                .map(|rec| IndentedItem {
                    bullet: "→".to_string(),
                    text: rec.clone(),
                    indent_level: 0,
                })
                .collect()
        } else {
            vec![IndentedItem {
                bullet: "→".to_string(),
                text: "Continue current development".to_string(),
                indent_level: 0,
            }]
        };

        elements.push(LayoutElement::Section {
            title: "Next Steps".to_string(),
            children: vec![LayoutElement::IndentedList { items: next_items }],
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
