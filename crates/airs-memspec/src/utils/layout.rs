//! Layout engine for structured CLI output formatting
//!
//! This module provides a composable layout system for creating sophisticated CLI output
//! that matches the professional formatting shown in README examples. The layout engine
//! supports structured layouts with proper alignment, visual hierarchy, and terminal adaptation.
//!
//! # Design Philosophy
//!
//! The layout engine follows a composable architecture where complex layouts are built
//! from simple, reusable elements. Each element encapsulates its own rendering logic
//! while the engine coordinates the overall composition and formatting.
//!
//! # Core Components
//!
//! - **LayoutEngine**: Main coordinator that renders layout elements to strings
//! - **LayoutElement**: Composable building blocks for different content types
//! - **Alignment**: Flexible alignment system for tabular and structured data
//! - **Visual Elements**: Professional separators, headers, and tree structures
//!
//! # Usage Example
//!
//! ```rust
//! use airs_memspec::utils::layout::{LayoutEngine, LayoutElement, HeaderStyle, Alignment};
//! use airs_memspec::utils::output::OutputConfig;
//!
//! let config = OutputConfig::new(false, false, false);
//! let engine = LayoutEngine::new(config);
//!
//! let elements = vec![
//!     LayoutElement::Header {
//!         icon: "🏢".to_string(),
//!         title: "AIRS Workspace".to_string(),
//!         style: HeaderStyle::Heavy,
//!     },
//!     LayoutElement::FieldRow {
//!         label: "Status".to_string(),
//!         value: "Active Development".to_string(),
//!         alignment: Alignment::LeftAligned(15),
//!     },
//! ];
//!
//! let output = engine.render(&elements);
//! println!("{}", output);
//! ```

use crate::utils::output::OutputConfig;
use colored::*;

/// Main layout engine responsible for rendering structured CLI output
///
/// The LayoutEngine takes a collection of LayoutElements and renders them into
/// a formatted string suitable for terminal display. It handles terminal width
/// adaptation, color management, and consistent spacing.
///
/// # Terminal Adaptation
///
/// The engine automatically adapts to terminal characteristics:
/// - Respects terminal width for responsive layouts
/// - Honors color preferences from OutputConfig
/// - Handles fallback formatting for limited terminals
#[derive(Debug, Clone)]
pub struct LayoutEngine {
    /// Output configuration controlling colors, verbosity, and terminal characteristics
    config: OutputConfig,
    /// Maximum width for layout rendering (adapted from terminal width)
    width: usize,
}

/// Composable layout elements that can be combined to create complex layouts
///
/// Each variant represents a different type of content with its own rendering
/// characteristics. Elements can be nested within sections to create hierarchical
/// layouts with proper indentation and visual organization.
#[derive(Debug, Clone)]
pub enum LayoutElement {
    /// Header with icon, title, and optional separator line
    Header {
        icon: String,
        title: String,
        style: HeaderStyle,
    },
    /// Labeled field with value, supporting various alignment options
    FieldRow {
        label: String,
        value: String,
        alignment: Alignment,
    },
    /// Tree structure item with proper indentation and connectors
    TreeItem {
        level: usize,
        is_last: bool,
        icon: String,
        text: String,
    },
    /// Section container with title and child elements
    Section {
        title: String,
        children: Vec<LayoutElement>,
    },
    /// Visual separator line with configurable style and width
    Separator {
        style: SeparatorStyle,
        width: Option<usize>,
    },
    /// List of indented items with custom bullets and hierarchy
    IndentedList { items: Vec<IndentedItem> },
    /// Empty line for spacing control
    EmptyLine,
}

/// Header styling options for different visual emphasis levels
#[derive(Debug, Clone)]
pub enum HeaderStyle {
    /// Heavy separator with thick Unicode line characters (━)
    Heavy,
    /// Light separator with thin Unicode line characters (─)
    Light,
    /// Simple colored text without separator lines
    Minimal,
}

/// Visual separator line styles for section division
#[derive(Debug, Clone, PartialEq)]
pub enum SeparatorStyle {
    /// Heavy Unicode line characters (━)
    Heavy,
    /// Light Unicode line characters (─)
    Light,
    /// Dotted separator (·)
    Dotted,
    /// No visible separator (just spacing)
    None,
}

/// Alignment options for field rows and tabular data
///
/// Supports various alignment strategies to create clean, readable layouts
/// with consistent spacing and professional appearance.
#[derive(Debug, Clone)]
pub enum Alignment {
    /// Left-align label within specified width
    LeftAligned(usize),
    /// Right-align label within specified width
    RightAligned(usize),
    /// Center-align label within specified width
    Centered(usize),
    /// Use custom tab stops for complex alignment patterns
    Tabbed(Vec<usize>),
}

/// Individual item within an indented list structure
///
/// Supports hierarchical lists with custom bullets and indentation levels
/// for creating nested information displays.
#[derive(Debug, Clone)]
pub struct IndentedItem {
    /// Bullet character or prefix (•, -, >, etc.)
    pub bullet: String,
    /// Main text content
    pub text: String,
    /// Indentation level (0 = no indent, 1 = first level, etc.)
    pub indent_level: usize,
}

impl LayoutEngine {
    /// Create a new layout engine with the specified output configuration
    ///
    /// The engine adapts its rendering behavior based on the configuration:
    /// - Color support affects separator and text styling
    /// - Terminal width influences layout responsiveness
    /// - Verbosity settings may affect element visibility
    ///
    /// # Arguments
    ///
    /// * `config` - Output configuration controlling rendering behavior
    pub fn new(config: OutputConfig) -> Self {
        Self {
            // Cap width at 120 characters for readability, minimum 60 for basic layouts
            width: config.terminal_width.clamp(60, 120),
            config,
        }
    }

    /// Render a collection of layout elements into a formatted string
    ///
    /// This is the main entry point for the layout engine. It processes each
    /// element in sequence, applying appropriate formatting and spacing to
    /// create a cohesive, professional output.
    ///
    /// # Arguments
    ///
    /// * `elements` - Vector of layout elements to render
    ///
    /// # Returns
    ///
    /// Formatted string ready for terminal display
    pub fn render(&self, elements: &[LayoutElement]) -> String {
        let mut output = String::new();

        for element in elements {
            match element {
                LayoutElement::Header { icon, title, style } => {
                    output.push_str(&self.render_header(icon, title, style));
                }
                LayoutElement::FieldRow {
                    label,
                    value,
                    alignment,
                } => {
                    output.push_str(&self.render_field_row(label, value, alignment));
                }
                LayoutElement::TreeItem {
                    level,
                    is_last,
                    icon,
                    text,
                } => {
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

    /// Render a header with icon, title, and optional separator line
    ///
    /// Headers create strong visual anchors for major sections. The style
    /// determines the visual weight and separator treatment.
    fn render_header(&self, icon: &str, title: &str, style: &HeaderStyle) -> String {
        let header_text = format!("{icon} {title}");

        match style {
            HeaderStyle::Heavy => {
                let separator = if self.config.use_color {
                    "━".repeat(self.width).bright_blue().to_string()
                } else {
                    "━".repeat(self.width)
                };

                let colored_header = if self.config.use_color {
                    header_text.bright_white().bold().to_string()
                } else {
                    header_text
                };

                format!("{colored_header}\n{separator}\n\n")
            }
            HeaderStyle::Light => {
                let separator = if self.config.use_color {
                    "─".repeat(self.width).blue().to_string()
                } else {
                    "─".repeat(self.width)
                };

                let colored_header = if self.config.use_color {
                    header_text.white().bold().to_string()
                } else {
                    header_text
                };

                format!("{colored_header}\n{separator}\n\n")
            }
            HeaderStyle::Minimal => {
                let colored_header = if self.config.use_color {
                    header_text.bright_cyan().bold().to_string()
                } else {
                    header_text
                };

                format!("{colored_header}\n\n")
            }
        }
    }

    /// Render a field row with label and value using specified alignment
    ///
    /// Field rows are the building blocks of structured data display,
    /// providing consistent alignment and spacing for key-value pairs.
    fn render_field_row(&self, label: &str, value: &str, alignment: &Alignment) -> String {
        match alignment {
            Alignment::LeftAligned(width) => {
                let colored_label = if self.config.use_color {
                    format!("{label:<width$}").bright_white().to_string()
                } else {
                    format!("{label:<width$}")
                };

                let colored_value = if self.config.use_color {
                    value.white().to_string()
                } else {
                    value.to_string()
                };

                format!("{colored_label} {colored_value}\n")
            }
            Alignment::RightAligned(width) => {
                let colored_label = if self.config.use_color {
                    format!("{label:>width$}").bright_white().to_string()
                } else {
                    format!("{label:>width$}")
                };

                let colored_value = if self.config.use_color {
                    value.white().to_string()
                } else {
                    value.to_string()
                };

                format!("{colored_label} {colored_value}\n")
            }
            Alignment::Centered(width) => {
                let colored_label = if self.config.use_color {
                    format!("{label:^width$}").bright_white().to_string()
                } else {
                    format!("{label:^width$}")
                };

                let colored_value = if self.config.use_color {
                    value.white().to_string()
                } else {
                    value.to_string()
                };

                format!("{colored_label} {colored_value}\n")
            }
            Alignment::Tabbed(tab_stops) => self.render_tabbed_row(label, value, tab_stops),
        }
    }

    /// Render a tree structure item with proper indentation and connectors
    ///
    /// Tree items create hierarchical displays with visual connectors that
    /// clearly show the relationship between parent and child elements.
    fn render_tree_item(&self, level: usize, is_last: bool, icon: &str, text: &str) -> String {
        let indent = "  ".repeat(level);
        let connector = if is_last { "└─" } else { "├─" };

        let colored_connector = if self.config.use_color {
            connector.bright_blue().to_string()
        } else {
            connector.to_string()
        };

        let colored_text = if self.config.use_color {
            format!("{icon} {text}").white().to_string()
        } else {
            format!("{icon} {text}")
        };

        format!("{indent}{colored_connector} {colored_text}\n")
    }

    /// Render a section with title and nested child elements
    ///
    /// Sections provide logical grouping with proper spacing and hierarchy.
    /// Child elements are rendered with appropriate indentation.
    fn render_section(&self, title: &str, children: &[LayoutElement]) -> String {
        let colored_title = if self.config.use_color {
            title.bright_yellow().bold().to_string()
        } else {
            title.to_string()
        };

        let mut output = format!("{colored_title}\n");

        for child in children {
            // Render children with slight indentation for visual hierarchy
            let child_output = self.render(&[child.clone()]);
            // Add indentation to each line of child output
            for line in child_output.lines() {
                if !line.is_empty() {
                    output.push_str(&format!("  {line}\n"));
                } else {
                    output.push('\n');
                }
            }
        }

        output
    }

    /// Render a visual separator line with specified style and width
    ///
    /// Separators provide visual breaks between sections and help organize
    /// complex layouts into digestible chunks.
    fn render_separator(&self, style: &SeparatorStyle, width: Option<usize>) -> String {
        let actual_width = width.unwrap_or(self.width);

        let separator = match style {
            SeparatorStyle::Heavy => "━".repeat(actual_width),
            SeparatorStyle::Light => "─".repeat(actual_width),
            SeparatorStyle::Dotted => "·".repeat(actual_width),
            SeparatorStyle::None => " ".repeat(actual_width),
        };

        let colored_separator = if self.config.use_color && style != &SeparatorStyle::None {
            separator.blue().to_string()
        } else {
            separator
        };

        format!("{colored_separator}\n")
    }

    /// Render an indented list with custom bullets and hierarchy levels
    ///
    /// Supports complex nested lists with different bullet styles and
    /// indentation levels for organizing detailed information.
    fn render_indented_list(&self, items: &[IndentedItem]) -> String {
        let mut output = String::new();

        for item in items {
            let indent = "  ".repeat(item.indent_level);

            let colored_bullet = if self.config.use_color {
                item.bullet.bright_blue().to_string()
            } else {
                item.bullet.clone()
            };

            let colored_text = if self.config.use_color {
                item.text.white().to_string()
            } else {
                item.text.clone()
            };

            output.push_str(&format!("{indent}{colored_bullet} {colored_text}\n"));
        }

        output
    }

    /// Render a field row with advanced tabbed alignment
    ///
    /// Uses custom tab stops for complex layouts requiring precise alignment
    /// across multiple columns or irregular spacing patterns.
    fn render_tabbed_row(&self, label: &str, value: &str, tab_stops: &[usize]) -> String {
        let mut output = String::new();
        output.push_str(label);

        // Calculate spacing to next appropriate tab stop
        let label_len = label.len();
        for &tab_stop in tab_stops {
            if label_len < tab_stop {
                let spaces = tab_stop - label_len;
                output.push_str(&" ".repeat(spaces));
                break;
            }
        }

        // If no tab stop found, use default spacing
        if !tab_stops.iter().any(|&stop| label_len < stop) {
            output.push_str("  ");
        }

        let colored_value = if self.config.use_color {
            value.white().to_string()
        } else {
            value.to_string()
        };

        output.push_str(&colored_value);
        output.push('\n');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> OutputConfig {
        OutputConfig::new(true, false, false) // no_color=true for predictable tests
    }

    #[test]
    fn test_layout_engine_creation() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        assert!(!engine.config.use_color);
        assert!(engine.width >= 60);
        assert!(engine.width <= 120);
    }

    #[test]
    fn test_header_rendering_heavy() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let elements = vec![LayoutElement::Header {
            icon: "🏢".to_string(),
            title: "Test Workspace".to_string(),
            style: HeaderStyle::Heavy,
        }];

        let output = engine.render(&elements);

        assert!(output.contains("🏢 Test Workspace"));
        assert!(output.contains("━"));
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_field_row_left_aligned() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let elements = vec![LayoutElement::FieldRow {
            label: "Status".to_string(),
            value: "Active".to_string(),
            alignment: Alignment::LeftAligned(15),
        }];

        let output = engine.render(&elements);

        assert!(output.contains("Status"));
        assert!(output.contains("Active"));
        assert!(output.ends_with("\n"));
    }

    #[test]
    fn test_tree_item_rendering() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let elements = vec![
            LayoutElement::TreeItem {
                level: 0,
                is_last: false,
                icon: "🟢".to_string(),
                text: "First Project".to_string(),
            },
            LayoutElement::TreeItem {
                level: 0,
                is_last: true,
                icon: "🟡".to_string(),
                text: "Second Project".to_string(),
            },
        ];

        let output = engine.render(&elements);

        assert!(output.contains("├─ 🟢 First Project"));
        assert!(output.contains("└─ 🟡 Second Project"));
    }

    #[test]
    fn test_indented_list_rendering() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let items = vec![
            IndentedItem {
                bullet: "•".to_string(),
                text: "First item".to_string(),
                indent_level: 0,
            },
            IndentedItem {
                bullet: "•".to_string(),
                text: "Nested item".to_string(),
                indent_level: 1,
            },
        ];

        let elements = vec![LayoutElement::IndentedList { items }];

        let output = engine.render(&elements);

        assert!(output.contains("• First item"));
        assert!(output.contains("  • Nested item"));
    }

    #[test]
    fn test_separator_rendering() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let elements = vec![
            LayoutElement::Separator {
                style: SeparatorStyle::Heavy,
                width: Some(20),
            },
            LayoutElement::Separator {
                style: SeparatorStyle::Light,
                width: Some(20),
            },
        ];

        let output = engine.render(&elements);

        assert!(output.contains("━".repeat(20).as_str()));
        assert!(output.contains("─".repeat(20).as_str()));
    }

    #[test]
    fn test_section_rendering() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let children = vec![LayoutElement::FieldRow {
            label: "Item".to_string(),
            value: "Value".to_string(),
            alignment: Alignment::LeftAligned(10),
        }];

        let elements = vec![LayoutElement::Section {
            title: "Test Section".to_string(),
            children,
        }];

        let output = engine.render(&elements);

        assert!(output.contains("Test Section"));
        assert!(output.contains("  Item"));
        assert!(output.contains("Value"));
    }

    #[test]
    fn test_empty_line_rendering() {
        let config = create_test_config();
        let engine = LayoutEngine::new(config);

        let elements = vec![LayoutElement::EmptyLine, LayoutElement::EmptyLine];

        let output = engine.render(&elements);

        assert_eq!(output, "\n\n");
    }
}
