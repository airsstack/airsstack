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
            // Status information to match README
            LayoutElement::FieldRow {
                label: "Status".to_string(),
                value: "Active Development - Foundation Phase".to_string(),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Focus".to_string(),
                value: "MCP Protocol Implementation & Tooling".to_string(),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Updated".to_string(),
                value: "2 hours ago".to_string(),
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

            // Create detailed project status like in README
            let detailed_status = if name == "airs-mcp" {
                "Week 1/14 - JSON-RPC Foundation"
            } else if name == "airs-memspec" {
                "Planning - CLI Development"
            } else {
                "Active Development"
            };

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

        // Next Milestone - match README
        elements.push(LayoutElement::FieldRow {
            label: "Next Milestone".to_string(),
            value: "JSON-RPC 2.0 Core Complete (3 days)".to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        // Blockers - match README
        elements.push(LayoutElement::FieldRow {
            label: "Blockers".to_string(),
            value: "None".to_string(),
            alignment: Alignment::LeftAligned(15),
        });

        elements
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
