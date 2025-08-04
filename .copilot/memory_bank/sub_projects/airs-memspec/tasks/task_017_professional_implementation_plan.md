# [task_017] - CLI Output Formatting Gap - PROFESSIONAL IMPLEMENTATION PLAN

**Status:** pending  
**Added:** 2025-08-04  
**Updated:** 2025-08-04
**Priority:** HIGH
**Type:** technical_debt
**Category:** user_experience
**Approach:** professional_engineering

## Architecture Philosophy

Instead of hardcoding layouts, build a **composable layout engine** that's extensible, testable, and maintainable. This solves the problem once and creates reusable infrastructure for future CLI formatting needs.

## Professional Implementation Plan (4-5 Days)

### Phase 1: Layout Engine Foundation (Day 1-2, ~12-16 hours)

#### Step 1: Core Layout Primitives (4 hours)
**File**: `src/utils/layout.rs` - New module

```rust
pub struct LayoutEngine {
    config: OutputConfig,
    width: usize,
}

pub enum LayoutElement {
    Header { icon: String, title: String, style: HeaderStyle },
    FieldRow { label: String, value: String, alignment: Alignment },
    TreeItem { level: usize, is_last: bool, icon: String, text: String },
    Section { title: String, children: Vec<LayoutElement> },
    Separator { style: SeparatorStyle, width: Option<usize> },
    IndentedList { items: Vec<IndentedItem> },
    EmptyLine,
}

pub enum HeaderStyle {
    Heavy,     // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    Light,     // ────────────────────────────────────────────────────────────────────────────────
    Minimal,   // Simple colored text
}

pub enum SeparatorStyle {
    Heavy,     // ━
    Light,     // ─
    Dotted,    // ·
    None,
}

pub enum Alignment {
    LeftAligned(usize),
    RightAligned(usize),
    Centered(usize),
    Tabbed(Vec<usize>),
}

pub struct TableBuilder {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    alignment: Vec<Alignment>,
}

pub struct IndentedItem {
    bullet: String,
    text: String,
    indent_level: usize,
}
```

#### Step 2: Rendering Engine (4 hours)
**Core rendering logic with composable elements**

```rust
impl LayoutEngine {
    pub fn new(config: OutputConfig) -> Self {
        Self {
            width: config.terminal_width.min(80),
            config,
        }
    }

    pub fn render(&self, elements: &[LayoutElement]) -> String {
        let mut output = String::new();
        for element in elements {
            match element {
                LayoutElement::Header { icon, title, style } => {
                    output.push_str(&self.render_header(icon, title, style));
                }
                LayoutElement::FieldRow { label, value, alignment } => {
                    output.push_str(&self.render_field_row(label, value, alignment));
                }
                LayoutElement::TreeItem { level, is_last, icon, text } => {
                    output.push_str(&self.render_tree_item(*level, *is_last, icon, text));
                }
                LayoutElement::Section { title, children } => {
                    output.push_str(&self.render_section(title, children));
                }
                LayoutElement::Separator { style, width } => {
                    output.push_str(&self.render_separator(style, *width));
                }
                LayoutElement::IndentedList { items } => {
                    output.push_str(&self.render_indented_list(items));
                }
                LayoutElement::EmptyLine => {
                    output.push('\n');
                }
            }
        }
        output
    }

    fn render_header(&self, icon: &str, title: &str, style: &HeaderStyle) -> String {
        match style {
            HeaderStyle::Heavy => {
                let separator = "━".repeat(self.width);
                format!("{} {}\n{}\n\n", icon, title, separator)
            }
            HeaderStyle::Light => {
                let separator = "─".repeat(self.width);
                format!("{}\n{}\n{}\n", separator, title, separator)
            }
            HeaderStyle::Minimal => {
                format!("{} {}\n", icon, title)
            }
        }
    }

    fn render_field_row(&self, label: &str, value: &str, alignment: &Alignment) -> String {
        match alignment {
            Alignment::LeftAligned(width) => {
                format!("{:<width$} {}\n", label, value, width = width)
            }
            Alignment::RightAligned(width) => {
                format!("{:>width$} {}\n", label, value, width = width)
            }
            Alignment::Centered(width) => {
                format!("{:^width$} {}\n", label, value, width = width)
            }
            Alignment::Tabbed(tab_stops) => {
                self.render_tabbed_row(label, value, tab_stops)
            }
        }
    }

    fn render_tree_item(&self, level: usize, is_last: bool, icon: &str, text: &str) -> String {
        let indent = "  ".repeat(level);
        let prefix = if is_last { "└─" } else { "├─" };
        format!("{}{} {} {}\n", indent, prefix, icon, text)
    }

    fn render_section(&self, title: &str, children: &[LayoutElement]) -> String {
        let mut output = format!("\n{}\n", title);
        for child in children {
            output.push_str(&self.render(&[child.clone()]));
        }
        output
    }

    fn render_separator(&self, style: &SeparatorStyle, width: Option<usize>) -> String {
        let actual_width = width.unwrap_or(self.width);
        let char = match style {
            SeparatorStyle::Heavy => "━",
            SeparatorStyle::Light => "─",
            SeparatorStyle::Dotted => "·",
            SeparatorStyle::None => " ",
        };
        format!("{}\n", char.repeat(actual_width))
    }

    fn render_indented_list(&self, items: &[IndentedItem]) -> String {
        let mut output = String::new();
        for item in items {
            let indent = "  ".repeat(item.indent_level);
            output.push_str(&format!("{}{} {}\n", indent, item.bullet, item.text));
        }
        output
    }

    fn render_tabbed_row(&self, label: &str, value: &str, tab_stops: &[usize]) -> String {
        // Advanced tabbed alignment for complex layouts
        let mut output = String::new();
        output.push_str(label);
        
        // Calculate spacing to next tab stop
        let label_len = label.len();
        for &tab_stop in tab_stops {
            if label_len < tab_stop {
                let spaces = tab_stop - label_len;
                output.push_str(&" ".repeat(spaces));
                break;
            }
        }
        
        output.push_str(value);
        output.push('\n');
        output
    }
}
```

#### Step 3: Template System (4 hours)
**File**: `src/utils/templates.rs` - Template definitions

```rust
use crate::utils::layout::{LayoutElement, LayoutEngine, HeaderStyle, Alignment, IndentedItem};

pub struct WorkspaceStatusTemplate {
    pub workspace_name: String,
    pub status: String,
    pub focus: String,
    pub updated: String,
    pub projects: Vec<ProjectSummary>,
    pub milestones: Vec<Milestone>,
    pub blockers: Vec<String>,
    pub analytics: Option<WorkspaceAnalytics>,
}

pub struct ProjectSummary {
    pub name: String,
    pub health_icon: String,
    pub completion: f32,
    pub phase: String,
    pub is_active: bool,
}

pub struct Milestone {
    pub name: String,
    pub eta: String,
    pub status: MilestoneStatus,
}

pub struct WorkspaceAnalytics {
    pub velocity: f32,
    pub eta_days: Option<f32>,
    pub trend: String,
    pub bottlenecks: Vec<String>,
}

impl WorkspaceStatusTemplate {
    pub fn to_layout(&self) -> Vec<LayoutElement> {
        let mut elements = vec![
            LayoutElement::Header {
                icon: "🏢".to_string(),
                title: format!("{} Workspace", self.workspace_name),
                style: HeaderStyle::Heavy,
            },
            LayoutElement::FieldRow {
                label: "Status".to_string(),
                value: self.status.clone(),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Focus".to_string(),
                value: self.focus.clone(),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::FieldRow {
                label: "Updated".to_string(),
                value: self.updated.clone(),
                alignment: Alignment::LeftAligned(15),
            },
            LayoutElement::EmptyLine,
        ];

        // Projects section
        elements.push(LayoutElement::FieldRow {
            label: "Projects".to_string(),
            value: format!("{} active, 0 paused", self.projects.len()),
            alignment: Alignment::LeftAligned(15),
        });

        // Project tree
        for (i, project) in self.projects.iter().enumerate() {
            let is_last = i == self.projects.len() - 1;
            let status_text = format!("{:.0}% - {}", project.completion, project.phase);
            elements.push(LayoutElement::TreeItem {
                level: 0,
                is_last,
                icon: project.health_icon.clone(),
                text: format!("{:<15} {}", project.name, status_text),
            });
        }

        elements.push(LayoutElement::EmptyLine);

        // Milestones
        if !self.milestones.is_empty() {
            elements.push(LayoutElement::FieldRow {
                label: "Next Milestone".to_string(),
                value: format!("{} ({})", self.milestones[0].name, self.milestones[0].eta),
                alignment: Alignment::LeftAligned(15),
            });
        }

        // Blockers
        let blockers_text = if self.blockers.is_empty() {
            "None".to_string()
        } else {
            self.blockers.join(", ")
        };
        elements.push(LayoutElement::FieldRow {
            label: "Blockers".to_string(),
            value: blockers_text,
            alignment: Alignment::LeftAligned(15),
        });

        // Analytics (if detailed)
        if let Some(analytics) = &self.analytics {
            elements.extend(self.render_analytics(analytics));
        }

        elements
    }

    fn render_analytics(&self, analytics: &WorkspaceAnalytics) -> Vec<LayoutElement> {
        vec![
            LayoutElement::EmptyLine,
            LayoutElement::Section {
                title: "Analytics".to_string(),
                children: vec![
                    LayoutElement::FieldRow {
                        label: "Velocity".to_string(),
                        value: format!("{:.1} tasks/week", analytics.velocity),
                        alignment: Alignment::LeftAligned(15),
                    },
                    LayoutElement::FieldRow {
                        label: "Trend".to_string(),
                        value: analytics.trend.clone(),
                        alignment: Alignment::LeftAligned(15),
                    },
                ],
            },
        ]
    }
}

pub struct ContextTemplate {
    pub project_name: String,
    pub current_focus: String,
    pub active_work: Vec<WorkItem>,
    pub integration_points: Vec<String>,
    pub constraints: Vec<String>,
}

pub struct WorkItem {
    pub icon: String,
    pub description: String,
    pub timing: Option<String>,
}

impl ContextTemplate {
    pub fn to_layout(&self) -> Vec<LayoutElement> {
        vec![
            LayoutElement::Header {
                icon: "🎯".to_string(),
                title: format!("{} Active Context", self.project_name),
                style: HeaderStyle::Heavy,
            },
            LayoutElement::Section {
                title: "Current Focus".to_string(),
                children: vec![
                    LayoutElement::IndentedList {
                        items: vec![IndentedItem {
                            bullet: "".to_string(),
                            text: self.current_focus.clone(),
                            indent_level: 1,
                        }],
                    },
                ],
            },
            LayoutElement::Section {
                title: "Active Work".to_string(),
                children: self.render_work_items(),
            },
            LayoutElement::Section {
                title: "Integration Points".to_string(),
                children: vec![
                    LayoutElement::IndentedList {
                        items: self.integration_points.iter().map(|point| IndentedItem {
                            bullet: "•".to_string(),
                            text: point.clone(),
                            indent_level: 1,
                        }).collect(),
                    },
                ],
            },
            LayoutElement::Section {
                title: "Constraints".to_string(),
                children: vec![
                    LayoutElement::IndentedList {
                        items: self.constraints.iter().map(|constraint| IndentedItem {
                            bullet: "•".to_string(),
                            text: constraint.clone(),
                            indent_level: 1,
                        }).collect(),
                    },
                ],
            },
        ]
    }

    fn render_work_items(&self) -> Vec<LayoutElement> {
        vec![
            LayoutElement::IndentedList {
                items: self.active_work.iter().map(|item| {
                    let text = if let Some(timing) = &item.timing {
                        format!("{} - {}", item.description, timing)
                    } else {
                        item.description.clone()
                    };
                    IndentedItem {
                        bullet: item.icon.clone(),
                        text,
                        indent_level: 1,
                    }
                }).collect(),
            },
        ]
    }
}
```

### Phase 2: Enhanced OutputFormatter Integration (Day 2-3, ~8-12 hours)

#### Step 1: OutputFormatter Redesign (4 hours)
**Enhanced `src/utils/output.rs`**

```rust
use crate::utils::layout::{LayoutEngine, LayoutElement};
use crate::utils::templates::{WorkspaceStatusTemplate, ContextTemplate};

impl OutputFormatter {
    pub fn render_layout(&self, elements: &[LayoutElement]) {
        if self.config.quiet {
            return;
        }

        let engine = LayoutEngine::new(self.config.clone());
        let output = engine.render(elements);
        
        if self.config.use_color {
            eprint!("{}", self.colorize_output(&output));
        } else {
            eprint!("{}", output);
        }
    }

    pub fn render_workspace_status(&self, template: &WorkspaceStatusTemplate) {
        let layout = template.to_layout();
        self.render_layout(&layout);
    }

    pub fn render_context(&self, template: &ContextTemplate) {
        let layout = template.to_layout();
        self.render_layout(&layout);
    }

    pub fn render_project_status(&self, template: &ProjectStatusTemplate) {
        let layout = template.to_layout();
        self.render_layout(&layout);
    }

    // Backward compatibility - existing methods still work
    pub fn header(&self, title: &str) {
        self.render_layout(&[LayoutElement::Header {
            icon: "".to_string(),
            title: title.to_string(),
            style: HeaderStyle::Light,
        }]);
    }

    pub fn separator(&self) {
        self.render_layout(&[LayoutElement::Separator {
            style: SeparatorStyle::Light,
            width: None,
        }]);
    }

    fn colorize_output(&self, output: &str) -> String {
        // Apply color formatting based on content patterns
        let mut colored = output.to_string();
        
        // Color headers
        colored = colored.replace("🏢", &"🏢".bright_blue().to_string());
        colored = colored.replace("🎯", &"🎯".bright_blue().to_string());
        
        // Color separators
        colored = colored.replace("━", &"━".bright_blue().to_string());
        colored = colored.replace("├─", &"├─".bright_black().to_string());
        colored = colored.replace("└─", &"└─".bright_black().to_string());
        
        // Color status indicators
        colored = colored.replace("🟢", &"🟢".green().to_string());
        colored = colored.replace("🟡", &"🟡".yellow().to_string());
        colored = colored.replace("🔴", &"🔴".red().to_string());
        
        colored
    }
}
```

#### Step 2: Data Model Integration (4 hours)
**Enhanced context integration**

```rust
// Enhanced src/parser/context.rs integration
impl WorkspaceContext {
    pub fn to_status_template(&self) -> WorkspaceStatusTemplate {
        WorkspaceStatusTemplate {
            workspace_name: "AIRS".to_string(),
            status: self.derive_workspace_status(),
            focus: self.derive_current_focus(),
            updated: self.get_last_updated(),
            projects: self.sub_project_contexts
                .iter()
                .map(|(name, ctx)| ctx.to_project_summary(name))
                .collect(),
            milestones: self.get_upcoming_milestones(),
            blockers: self.identify_blockers(),
            analytics: None, // Added when detailed=true
        }
    }

    fn derive_workspace_status(&self) -> String {
        // Analyze overall workspace health and phase
        if self.sub_project_contexts.values().any(|ctx| 
            matches!(ctx.derived_status.health, ProjectHealth::Critical)
        ) {
            "Critical Issues - Immediate Attention Required".to_string()
        } else if self.sub_project_contexts.values().any(|ctx| 
            matches!(ctx.derived_status.health, ProjectHealth::Warning)
        ) {
            "Active Development - Some Issues".to_string()
        } else {
            "Active Development - Foundation Phase".to_string()
        }
    }

    fn derive_current_focus(&self) -> String {
        // Extract focus from active project
        if let Some(active_context) = self.sub_project_contexts.get(&self.current_context.active_sub_project) {
            active_context.extract_workspace_focus()
        } else {
            "Multi-Project Development & Tooling".to_string()
        }
    }

    fn get_upcoming_milestones(&self) -> Vec<Milestone> {
        // Extract from progress files
        vec![
            Milestone {
                name: "CLI Enhancement Complete".to_string(),
                eta: "2 days".to_string(),
                status: MilestoneStatus::InProgress,
            }
        ]
    }

    fn identify_blockers(&self) -> Vec<String> {
        // Scan for critical issues across projects
        let mut blockers = Vec::new();
        
        for context in self.sub_project_contexts.values() {
            if matches!(context.derived_status.health, ProjectHealth::Critical) {
                blockers.push(format!("Critical issues in {}", context.project_name));
            }
        }
        
        blockers
    }
}

impl SubProjectContext {
    pub fn to_project_summary(&self, name: &str) -> ProjectSummary {
        ProjectSummary {
            name: name.to_string(),
            health_icon: match self.derived_status.health {
                ProjectHealth::Healthy => "🟢".to_string(),
                ProjectHealth::Warning => "🟡".to_string(),
                ProjectHealth::Critical => "🔴".to_string(),
                ProjectHealth::Unknown => "⚪".to_string(),
            },
            completion: self.task_summary.completion_percentage,
            phase: self.derived_status.current_phase.clone(),
            is_active: false, // Set by workspace context
        }
    }

    pub fn to_context_template(&self, project_name: &str) -> ContextTemplate {
        ContextTemplate {
            project_name: project_name.to_string(),
            current_focus: self.extract_current_focus(),
            active_work: self.get_active_work_items(),
            integration_points: self.get_integration_points(),
            constraints: self.get_constraints(),
        }
    }

    fn extract_current_focus(&self) -> String {
        // Parse active_context.md for current focus
        if let Some(active_content) = &self.active_context {
            // Extract from markdown content
            "CLI Output Formatting Enhancement & Professional Layout Engine".to_string()
        } else {
            "Development in progress".to_string()
        }
    }

    fn get_active_work_items(&self) -> Vec<WorkItem> {
        vec![
            WorkItem {
                icon: "🔧".to_string(),
                description: "Implementing composable layout engine".to_string(),
                timing: Some("Started 2 hours ago".to_string()),
            },
            WorkItem {
                icon: "📝".to_string(),
                description: "Template system for structured output".to_string(),
                timing: Some("In design phase".to_string()),
            },
            WorkItem {
                icon: "⏱️".to_string(),
                description: "README compliance verification".to_string(),
                timing: None,
            },
        ]
    }

    fn get_integration_points(&self) -> Vec<String> {
        vec![
            "OutputFormatter enhancement for structured layouts".to_string(),
            "Template system integration with context data".to_string(),
            "Command-specific formatting with backward compatibility".to_string(),
            "Terminal-adaptive visual hierarchy and color support".to_string(),
        ]
    }

    fn get_constraints(&self) -> Vec<String> {
        vec![
            "Must match README examples exactly".to_string(),
            "Maintain backward compatibility with existing output methods".to_string(),
            "Terminal width adaptation and color support required".to_string(),
            "Professional CLI tool standards compliance".to_string(),
        ]
    }
}
```

### Phase 3: Command Implementation (Day 3-4, ~8-12 hours)

#### Step 1: Status Command Enhancement (4 hours)
**Clean, maintainable status command implementation**

```rust
// Enhanced src/cli/commands/status.rs
fn show_workspace_status(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    detailed: bool,
    progress_analyzer: &ProgressAnalyzer,
) -> FsResult<()> {
    // Build template from context
    let mut template = workspace_context.to_status_template();
    
    // Add analytics if detailed mode
    if detailed {
        template.analytics = Some(WorkspaceAnalytics {
            velocity: progress_analyzer.calculate_velocity(workspace_context),
            eta_days: progress_analyzer.estimate_completion(workspace_context),
            trend: progress_analyzer.analyze_trend(workspace_context),
            bottlenecks: progress_analyzer.identify_bottlenecks(workspace_context),
        });
    }
    
    // Render with layout engine
    formatter.render_workspace_status(&template);
    
    Ok(())
}

fn show_sub_project_status(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    project_name: &str,
    detailed: bool,
    progress_analyzer: &ProgressAnalyzer,
) -> FsResult<()> {
    if let Some(context) = workspace_context.sub_project_contexts.get(project_name) {
        let template = context.to_detailed_status_template(project_name, detailed);
        formatter.render_project_status(&template);
    } else {
        formatter.error(&format!("Sub-project '{}' not found", project_name));
        
        // Show available projects
        formatter.info("Available sub-projects:");
        for name in workspace_context.sub_project_contexts.keys() {
            formatter.info(&format!("  - {}", name));
        }
    }
    
    Ok(())
}
```

#### Step 2: Context Command Enhancement (4 hours)
**Enhanced context command with rich formatting**

```rust
// Enhanced src/cli/commands/context.rs
fn show_sub_project_context(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    project_name: &str,
) -> FsResult<()> {
    if let Some(context) = workspace_context.sub_project_contexts.get(project_name) {
        let template = context.to_context_template(project_name);
        formatter.render_context(&template);
    } else {
        formatter.error(&format!("Sub-project '{}' not found", project_name));
        
        // Show available projects with brief descriptions
        formatter.info("Available sub-projects:");
        for (name, ctx) in &workspace_context.sub_project_contexts {
            let completion = ctx.task_summary.completion_percentage;
            formatter.info(&format!("  - {} ({:.0}% complete)", name, completion));
        }
    }
    
    Ok(())
}

fn show_workspace_context(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    let template = workspace_context.to_workspace_context_template();
    formatter.render_workspace_context(&template);
    
    Ok(())
}
```

### Phase 4: Testing & Quality Assurance (Day 4-5, ~8-12 hours)

#### Step 1: Unit Tests (4 hours)
**Comprehensive unit test suite**

```rust
#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::utils::output::OutputConfig;

    #[test]
    fn test_layout_engine_header_rendering() {
        let config = OutputConfig::new(false, false, false);
        let engine = LayoutEngine::new(config);
        
        let elements = vec![LayoutElement::Header {
            icon: "🏢".to_string(),
            title: "Test Workspace".to_string(),
            style: HeaderStyle::Heavy,
        }];
        
        let output = engine.render(&elements);
        assert!(output.contains("🏢 Test Workspace"));
        assert!(output.contains("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
    }

    #[test]
    fn test_field_row_alignment() {
        let config = OutputConfig::new(false, false, false);
        let engine = LayoutEngine::new(config);
        
        let elements = vec![LayoutElement::FieldRow {
            label: "Status".to_string(),
            value: "Active Development".to_string(),
            alignment: Alignment::LeftAligned(15),
        }];
        
        let output = engine.render(&elements);
        assert_eq!(output, "Status          Active Development\n");
    }

    #[test]
    fn test_tree_item_rendering() {
        let config = OutputConfig::new(false, false, false);
        let engine = LayoutEngine::new(config);
        
        let elements = vec![
            LayoutElement::TreeItem {
                level: 0,
                is_last: false,
                icon: "🟢".to_string(),
                text: "airs-mcp      85% - Implementation Phase".to_string(),
            },
            LayoutElement::TreeItem {
                level: 0,
                is_last: true,
                icon: "🟡".to_string(),
                text: "airs-memspec  72% - Enhancement Phase".to_string(),
            },
        ];
        
        let output = engine.render(&elements);
        assert!(output.contains("├─ 🟢 airs-mcp"));
        assert!(output.contains("└─ 🟡 airs-memspec"));
    }
}

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn test_workspace_status_template() {
        let template = WorkspaceStatusTemplate {
            workspace_name: "AIRS".to_string(),
            status: "Active Development - Foundation Phase".to_string(),
            focus: "MCP Protocol Implementation & Tooling".to_string(),
            updated: "2 hours ago".to_string(),
            projects: vec![
                ProjectSummary {
                    name: "airs-mcp".to_string(),
                    health_icon: "🟢".to_string(),
                    completion: 85.0,
                    phase: "Implementation Phase".to_string(),
                    is_active: true,
                },
                ProjectSummary {
                    name: "airs-memspec".to_string(),
                    health_icon: "🟡".to_string(),
                    completion: 72.0,
                    phase: "Enhancement Phase".to_string(),
                    is_active: false,
                },
            ],
            milestones: vec![
                Milestone {
                    name: "CLI Enhancement Complete".to_string(),
                    eta: "2 days".to_string(),
                    status: MilestoneStatus::InProgress,
                }
            ],
            blockers: vec![],
            analytics: None,
        };
        
        let layout = template.to_layout();
        assert!(!layout.is_empty());
        
        // Verify structure matches README format
        match &layout[0] {
            LayoutElement::Header { icon, title, style } => {
                assert_eq!(icon, "🏢");
                assert!(title.contains("AIRS Workspace"));
                assert!(matches!(style, HeaderStyle::Heavy));
            }
            _ => panic!("Expected header as first element"),
        }
        
        // Verify field rows
        let field_rows: Vec<_> = layout.iter()
            .filter_map(|element| match element {
                LayoutElement::FieldRow { label, value, .. } => Some((label, value)),
                _ => None,
            })
            .collect();
        
        assert!(field_rows.iter().any(|(label, _)| label == "Status"));
        assert!(field_rows.iter().any(|(label, _)| label == "Focus"));
        assert!(field_rows.iter().any(|(label, _)| label == "Updated"));
    }

    #[test]
    fn test_context_template() {
        let template = ContextTemplate {
            project_name: "airs-memspec".to_string(),
            current_focus: "CLI Output Formatting Enhancement".to_string(),
            active_work: vec![
                WorkItem {
                    icon: "🔧".to_string(),
                    description: "Implementing layout engine".to_string(),
                    timing: Some("Started 1 hour ago".to_string()),
                },
            ],
            integration_points: vec![
                "OutputFormatter enhancement".to_string(),
                "Template system integration".to_string(),
            ],
            constraints: vec![
                "Must match README examples exactly".to_string(),
            ],
        };
        
        let layout = template.to_layout();
        
        // Verify header
        match &layout[0] {
            LayoutElement::Header { icon, title, style } => {
                assert_eq!(icon, "🎯");
                assert!(title.contains("airs-memspec Active Context"));
                assert!(matches!(style, HeaderStyle::Heavy));
            }
            _ => panic!("Expected context header"),
        }
        
        // Verify sections exist
        let sections: Vec<_> = layout.iter()
            .filter_map(|element| match element {
                LayoutElement::Section { title, .. } => Some(title),
                _ => None,
            })
            .collect();
        
        assert!(sections.iter().any(|title| title == "Current Focus"));
        assert!(sections.iter().any(|title| title == "Active Work"));
        assert!(sections.iter().any(|title| title == "Integration Points"));
        assert!(sections.iter().any(|title| title == "Constraints"));
    }
}
```

#### Step 2: Integration Tests (4 hours)
**End-to-end integration testing**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::process::Command;

    fn create_test_memory_bank(dir: &std::path::Path) {
        // Create realistic memory bank structure for testing
        std::fs::create_dir_all(dir.join(".copilot/memory_bank/workspace")).unwrap();
        std::fs::create_dir_all(dir.join(".copilot/memory_bank/sub_projects/airs-mcp")).unwrap();
        std::fs::create_dir_all(dir.join(".copilot/memory_bank/sub_projects/airs-memspec")).unwrap();
        
        // Create test files with realistic content
        std::fs::write(
            dir.join(".copilot/memory_bank/current_context.md"),
            "**active_sub_project:** airs-memspec"
        ).unwrap();
        
        // Add more test memory bank files as needed
    }

    fn capture_command_output<F>(f: F) -> String 
    where F: FnOnce() -> Result<(), Box<dyn std::error::Error>> {
        // Capture stdout/stderr for testing
        use std::sync::{Arc, Mutex};
        
        let output = Arc::new(Mutex::new(String::new()));
        // Implementation depends on how we want to capture output
        // This is a placeholder for the actual capture mechanism
        String::new()
    }

    #[test]
    fn test_end_to_end_workspace_status() {
        let temp_dir = TempDir::new().unwrap();
        create_test_memory_bank(temp_dir.path());
        
        // Test workspace status command
        let args = crate::cli::Args {
            global: crate::cli::GlobalArgs {
                path: Some(temp_dir.path().to_path_buf()),
                verbose: false,
                quiet: false,
                no_color: true, // For consistent testing
            },
            command: crate::cli::Commands::Status {
                detailed: false,
                sub_project: None,
            },
        };
        
        // Run command and capture output
        let output = capture_command_output(|| {
            crate::cli::commands::status::run(&args.global, false, None)
        });
        
        // Verify output matches README format
        assert!(output.contains("🏢 AIRS Workspace"));
        assert!(output.contains("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
        assert!(output.contains("Status          "));
        assert!(output.contains("Focus           "));
        assert!(output.contains("Updated         "));
        assert!(output.contains("Projects        "));
        assert!(output.contains("├─"));
        assert!(output.contains("└─"));
        assert!(output.contains("Next Milestone  "));
        assert!(output.contains("Blockers        "));
    }

    #[test]
    fn test_end_to_end_context_display() {
        let temp_dir = TempDir::new().unwrap();
        create_test_memory_bank(temp_dir.path());
        
        // Test context command
        let args = crate::cli::Args {
            global: crate::cli::GlobalArgs {
                path: Some(temp_dir.path().to_path_buf()),
                verbose: false,
                quiet: false,
                no_color: true,
            },
            command: crate::cli::Commands::Context {
                workspace: false,
                project: Some("airs-memspec".to_string()),
            },
        };
        
        let output = capture_command_output(|| {
            crate::cli::commands::context::run(&args.global, false, Some("airs-memspec".to_string()))
        });
        
        // Verify context output format
        assert!(output.contains("🎯 airs-memspec Active Context"));
        assert!(output.contains("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
        assert!(output.contains("Current Focus"));
        assert!(output.contains("Active Work"));
        assert!(output.contains("Integration Points"));
        assert!(output.contains("Constraints"));
        assert!(output.contains("  🔧"));
        assert!(output.contains("  📝"));
        assert!(output.contains("  •"));
    }

    #[test]
    fn test_color_vs_no_color_output() {
        let temp_dir = TempDir::new().unwrap();
        create_test_memory_bank(temp_dir.path());
        
        // Test with colors
        let output_with_color = capture_command_output(|| {
            let args = crate::cli::GlobalArgs {
                path: Some(temp_dir.path().to_path_buf()),
                verbose: false,
                quiet: false,
                no_color: false,
            };
            crate::cli::commands::status::run(&args, false, None)
        });
        
        // Test without colors
        let output_no_color = capture_command_output(|| {
            let args = crate::cli::GlobalArgs {
                path: Some(temp_dir.path().to_path_buf()),
                verbose: false,
                quiet: false,
                no_color: true,
            };
            crate::cli::commands::status::run(&args, false, None)
        });
        
        // Both should have same structure, different styling
        assert_ne!(output_with_color, output_no_color);
        assert!(output_with_color.len() >= output_no_color.len()); // Color codes add length
        
        // But both should have core content
        for output in [&output_with_color, &output_no_color] {
            assert!(output.contains("🏢 AIRS Workspace"));
            assert!(output.contains("Status"));
            assert!(output.contains("Projects"));
        }
    }
}
```

## Professional Engineering Benefits

### Architecture Benefits:
- ✅ **Composable Layout System**: Reusable for any future CLI formatting needs
- ✅ **Separation of Concerns**: Data → Template → Layout → Rendering pipeline
- ✅ **Extensibility**: Easy to add new layout elements and output formats
- ✅ **Testability**: Each component unit testable independently
- ✅ **Maintainability**: Changes to layouts don't affect command logic
- ✅ **Performance**: Efficient string building with minimal allocations

### Quality Benefits:
- ✅ **README Compliance**: Exact visual match with documented examples
- ✅ **Backward Compatibility**: Existing output methods continue working
- ✅ **Terminal Adaptation**: Responsive to width, color support, TTY detection
- ✅ **Error Handling**: Graceful degradation with informative error messages
- ✅ **Documentation**: Comprehensive inline docs and usage examples

### Development Benefits:
- ✅ **Future-Proof**: Architecture supports any CLI output evolution
- ✅ **Team Collaboration**: Clear interfaces between data and presentation
- ✅ **Debugging**: Structured templates make output issues easy to trace
- ✅ **Professional Standards**: Enterprise-grade CLI tool quality

## Implementation Validation

### Success Criteria:
1. **Visual Match**: Output identical to README examples
2. **Architecture Quality**: Clean separation of concerns
3. **Test Coverage**: >95% unit test coverage
4. **Performance**: <5ms rendering time for complex layouts
5. **Maintainability**: New output formats added in <2 hours

### Risk Mitigation:
- **Complexity Management**: Phased approach with validation at each step
- **Backward Compatibility**: Existing output methods remain functional
- **Performance**: Efficient string operations and minimal allocations
- **Testing**: Comprehensive unit and integration test coverage

## Post-Implementation Notes

This implementation creates a **reusable asset** for the entire AIRS workspace. The layout engine can be extracted to a separate crate if other projects need similar capabilities.

**Total Effort**: 4-5 days of focused development
**Technical Debt Eliminated**: Complete gap between documentation and implementation
**Future Value**: Foundation for any CLI formatting needs across AIRS projects

---

*This represents professional-grade software engineering: solving the problem once, correctly, and creating reusable infrastructure for future needs.*
