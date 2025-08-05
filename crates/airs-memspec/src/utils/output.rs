//! Output formatting utilities for the airs-memspec CLI
//!
//! This module provides comprehensive terminal output formatting with support for:
//! - Cross-platform color detection and adaptive output
//! - Terminal width detection and responsive formatting
//! - Multiple output modes (verbose, quiet, no-color)
//! - Consistent message formatting across all commands
//! - Progress indicators and visual separators
//!
//! # Design Philosophy
//!
//! The output framework follows these principles:
//! - **Adaptive**: Automatically detects terminal capabilities and adjusts output accordingly
//! - **Consistent**: All CLI commands use the same formatting patterns and visual hierarchy
//! - **Accessible**: Respects user preferences for color output and verbosity levels
//! - **Professional**: Clean, readable output that enhances the developer experience
//!
//! # Usage
//!
//! ```rust
//! use airs_memspec::utils::output::{OutputConfig, OutputFormatter};
//!
//! // Create configuration based on CLI flags
//! let no_color_flag = false;
//! let verbose_flag = true;
//! let quiet_flag = false;
//! let config = OutputConfig::new(no_color_flag, verbose_flag, quiet_flag);
//! let formatter = OutputFormatter::new(config);
//!
//! // Use formatted output
//! formatter.success("Operation completed successfully");
//! formatter.error("Something went wrong");
//! formatter.verbose("Detailed debugging information");
//! ```

use colored::*;
use std::io::{self, IsTerminal, Write};

/// Configuration for terminal output formatting and behavior
///
/// This struct encapsulates all settings that control how output is displayed,
/// including color support detection, verbosity levels, and terminal characteristics.
///
/// # Terminal Detection
///
/// The configuration automatically detects:
/// - Whether the current terminal supports color output
/// - Terminal width for responsive formatting
/// - TTY status for appropriate output routing
///
/// # Output Modes
///
/// - **Normal**: Standard output with colors and formatting
/// - **Verbose**: Additional debugging and process information
/// - **Quiet**: Only essential output (errors and critical messages)
/// - **No-color**: Plain text output without ANSI escape sequences
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Whether color output is enabled (combines terminal support detection with user preference)
    pub use_color: bool,
    /// Whether verbose output is enabled (shows additional debugging information)
    pub verbose: bool,
    /// Whether quiet mode is enabled (suppresses non-essential output)
    pub quiet: bool,
    /// Detected terminal width in columns (defaults to 80 if detection fails)
    pub terminal_width: usize,
}

impl OutputConfig {
    /// Create a new output configuration with automatic terminal detection
    ///
    /// This constructor performs comprehensive terminal environment analysis:
    /// 1. Detects color support capability
    /// 2. Measures terminal width for responsive formatting
    /// 3. Configures global color settings for the colored crate
    ///
    /// # Arguments
    ///
    /// * `force_no_color` - Override color detection and disable colors
    /// * `verbose` - Enable verbose output mode
    /// * `quiet` - Enable quiet mode (takes precedence over verbose)
    ///
    /// # Color Detection Logic
    ///
    /// Color support is enabled when:
    /// - Terminal is a TTY (not redirected)
    /// - colored crate detects terminal color capability
    /// - User hasn't explicitly disabled colors via `force_no_color`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_memspec::utils::output::OutputConfig;
    ///
    /// // Standard configuration
    /// let config = OutputConfig::new(false, false, false);
    ///
    /// // Force monochrome output
    /// let config = OutputConfig::new(true, false, false);
    ///
    /// // Verbose mode with colors
    /// let config = OutputConfig::new(false, true, false);
    /// ```
    pub fn new(force_no_color: bool, verbose: bool, quiet: bool) -> Self {
        let use_color = Self::detect_color_support() && !force_no_color;
        let terminal_width = Self::detect_terminal_width();

        // Configure global color settings for the colored crate
        // This ensures consistent behavior across all colored text operations
        if !use_color {
            colored::control::set_override(false);
        }

        Self {
            use_color,
            verbose,
            quiet,
            terminal_width,
        }
    }

    /// Detect if the current terminal supports color output
    ///
    /// This method uses a two-stage detection process:
    /// 1. Verify we're running in a TTY (not redirected to file/pipe)
    /// 2. Use the colored crate's built-in terminal capability detection
    ///
    /// # Terminal Detection Details
    ///
    /// The colored crate checks for:
    /// - TERM environment variable settings
    /// - NO_COLOR environment variable (disables colors)
    /// - FORCE_COLOR environment variable (forces colors)
    /// - Platform-specific terminal capabilities
    ///
    /// # Returns
    ///
    /// `true` if colors should be used, `false` for plain text output
    fn detect_color_support() -> bool {
        // Check if stderr is connected to a terminal (not redirected)
        if !io::stderr().is_terminal() {
            return false;
        }

        // Delegate to colored crate's sophisticated detection logic
        colored::control::SHOULD_COLORIZE.should_colorize()
    }

    /// Detect the current terminal width in columns
    ///
    /// This method attempts to query the terminal for its current dimensions.
    /// Terminal width detection is used for:
    /// - Responsive formatting of separators and headers
    /// - Progress bar sizing
    /// - Table column width calculations
    ///
    /// # Fallback Behavior
    ///
    /// If terminal size detection fails (e.g., in non-interactive environments),
    /// defaults to 80 columns for consistent formatting.
    ///
    /// # Platform Support
    ///
    /// Uses the `terminal_size` crate which supports:
    /// - Unix-like systems (Linux, macOS, BSD)
    /// - Windows (both cmd.exe and PowerShell)
    /// - Various terminal emulators
    ///
    /// # Returns
    ///
    /// Terminal width in columns, minimum 80 for readability
    fn detect_terminal_width() -> usize {
        // Attempt to query terminal dimensions
        if let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() {
            width as usize
        } else {
            // Fallback to standard 80-column width for consistency
            80
        }
    }
}

/// High-level output formatter providing consistent CLI messaging
///
/// The OutputFormatter provides a unified interface for all CLI output,
/// ensuring consistent visual hierarchy, color usage, and message formatting
/// across all commands in the application.
///
/// # Design Principles
///
/// - **Hierarchy**: Different message types have distinct visual styling
/// - **Consistency**: Same message types look identical across commands
/// - **Accessibility**: Respects color preferences and screen readers
/// - **Performance**: Minimal overhead for high-frequency operations
///
/// # Message Types
///
/// The formatter supports these semantic message categories:
/// - **Success**: ✅ Positive outcomes and completions
/// - **Error**: ❌ Failures and critical issues (always shown)
/// - **Warning**: ⚠️ Potential issues and advisories
/// - **Info**: ℹ️ General information and status updates
/// - **Verbose**: 🔍 Detailed debugging and process information
/// - **Essential**: Plain text that bypasses quiet mode
///
/// # Quiet Mode Behavior
///
/// When quiet mode is enabled:
/// - Errors are always displayed (critical for user awareness)
/// - Success, warning, info, and verbose messages are suppressed
/// - Essential messages bypass quiet mode for critical output
/// - Headers and separators are hidden to minimize noise
///
/// # Examples
///
/// ```rust
/// use airs_memspec::utils::output::{OutputConfig, OutputFormatter};
///
/// let config = OutputConfig::new(false, false, false);
/// let formatter = OutputFormatter::new(config);
///
/// // Standard messages
/// formatter.success("Installation completed");
/// formatter.error("Configuration file not found");
/// formatter.warning("Using default settings");
///
/// // Conditional output
/// formatter.verbose("Processing file: config.toml");
/// formatter.essential("Required action: run setup");
///
/// // Visual elements
/// formatter.header("Memory Bank Status");
/// formatter.separator();
/// ```
pub struct OutputFormatter {
    config: OutputConfig,
}

impl OutputFormatter {
    /// Create a new output formatter with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `config` - OutputConfig containing terminal detection results and user preferences
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Display a success message with green checkmark
    ///
    /// Success messages indicate positive outcomes, completed operations,
    /// or successful validations. They use a green checkmark emoji and
    /// are suppressed in quiet mode.
    ///
    /// # Output Format
    /// ```text
    /// ✅ Operation completed successfully
    /// ```
    ///
    /// # Arguments
    ///
    /// * `message` - The success message to display
    pub fn success(&self, message: &str) {
        if self.config.quiet {
            return;
        }

        eprintln!("{} {}", "✅".green(), message);
    }

    /// Display an error message with red X mark
    ///
    /// Error messages indicate failures, critical issues, or blocking problems.
    /// They use a red X emoji and are ALWAYS displayed regardless of quiet mode
    /// to ensure users are aware of critical issues.
    ///
    /// # Output Format
    /// ```text
    /// ❌ Error: Configuration file not found
    /// ```
    ///
    /// # Design Note
    ///
    /// Error messages bypass quiet mode because users must be informed of
    /// failures even when they've requested minimal output.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to display
    pub fn error(&self, message: &str) {
        eprintln!("{} {}: {}", "❌".red(), "Error".red().bold(), message);
    }

    /// Display a warning message with yellow warning triangle
    ///
    /// Warning messages indicate potential issues, suboptimal conditions,
    /// or advisory information that doesn't prevent operation but may
    /// require user attention.
    ///
    /// # Output Format
    /// ```text
    /// ⚠️ Warning: Using default configuration
    /// ```
    ///
    /// # Arguments
    ///
    /// * `message` - The warning message to display
    pub fn warning(&self, message: &str) {
        if self.config.quiet {
            return;
        }

        eprintln!(
            "{} {}: {}",
            "⚠️".yellow(),
            "Warning".yellow().bold(),
            message
        );
    }

    /// Display an informational message with blue info symbol
    ///
    /// Info messages provide general status updates, progress information,
    /// or contextual details that help users understand what's happening.
    ///
    /// # Output Format
    /// ```text
    /// ℹ️ Scanning project files...
    /// ```
    ///
    /// # Arguments
    ///
    /// * `message` - The informational message to display
    pub fn info(&self, message: &str) {
        if self.config.quiet {
            return;
        }

        eprintln!("{} {}", "ℹ️".cyan(), message);
    }

    /// Display a verbose debug message with magnifying glass
    ///
    /// Verbose messages provide detailed debugging information, internal
    /// process details, and diagnostic data. They are only shown when
    /// verbose mode is explicitly enabled and are always suppressed in quiet mode.
    ///
    /// # Visibility Conditions
    ///
    /// Verbose messages are shown only when:
    /// - Verbose mode is enabled (`--verbose` flag)
    /// - Quiet mode is NOT enabled (quiet takes precedence)
    ///
    /// # Output Format
    /// ```text
    /// 🔍 Processing file: /path/to/file.md
    /// ```
    ///
    /// # Style Note
    ///
    /// Verbose messages use bright_black (dark gray) coloring to visually
    /// distinguish them as supplementary information.
    ///
    /// # Arguments
    ///
    /// * `message` - The verbose debug message to display
    pub fn verbose(&self, message: &str) {
        if !self.config.verbose || self.config.quiet {
            return;
        }

        eprintln!("{} {}", "🔍".bright_black(), message.bright_black());
    }

    /// Display a section header with surrounding separators
    ///
    /// Headers create visual separation between major sections of output,
    /// improving readability and helping users navigate complex information.
    ///
    /// # Output Format
    /// ```text
    /// ────────────────────────────────────────
    /// Memory Bank Status
    /// ────────────────────────────────────────
    /// ```
    ///
    /// # Terminal Width Adaptation
    ///
    /// The separator length adapts to terminal width but is capped at 80
    /// characters to maintain readability on very wide terminals.
    ///
    /// # Arguments
    ///
    /// * `title` - The header text to display
    pub fn header(&self, title: &str) {
        if self.config.quiet {
            return;
        }

        let separator = "─".repeat(self.config.terminal_width.min(80));

        eprintln!("{}", separator.bright_blue());
        eprintln!("{}", title.bright_blue().bold());
        eprintln!("{}", separator.bright_blue());
    }

    /// Display a visual section separator
    ///
    /// Separators provide visual breaks between related content sections
    /// without the emphasis of a full header. Useful for organizing
    /// command output into logical groups.
    ///
    /// # Output Format
    /// ```text
    /// ────────────────────────────────────────
    /// ```
    ///
    /// # Style Difference
    ///
    /// Uses bright_black (dark gray) instead of blue to be less prominent
    /// than headers while still providing visual structure.
    pub fn separator(&self) {
        if self.config.quiet {
            return;
        }

        let separator = "─".repeat(self.config.terminal_width.min(80));
        eprintln!("{}", separator.bright_black());
    }

    /// Display a plain message that respects quiet mode
    ///
    /// Simple message output without special formatting or icons.
    /// Respects quiet mode for non-essential information.
    ///
    /// # Arguments
    ///
    /// * `message` - The plain message to display
    pub fn message(&self, message: &str) {
        if self.config.quiet {
            return;
        }
        eprintln!("{message}");
    }

    /// Display a progress bar for percentage completion
    ///
    /// Creates a visual progress bar showing completion percentage with
    /// customizable width and styling. The bar adapts to terminal width
    /// and includes percentage text.
    ///
    /// # Output Format
    /// ```text
    /// ████████████░░░░░░░░ 60% (3/5 tasks)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `percentage` - Completion percentage (0.0 to 100.0)
    /// * `label` - Optional descriptive label (e.g., "3/5 tasks")
    /// * `width` - Progress bar width in characters (default: 20)
    pub fn progress_bar(&self, percentage: f32, label: Option<&str>, width: Option<usize>) {
        if self.config.quiet {
            return;
        }

        let bar_width = width.unwrap_or(20);
        let filled = ((percentage / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        let filled_bar = "█".repeat(filled);
        let empty_bar = "░".repeat(empty);

        let percentage_text = format!("{percentage:.1}%");
        let label_text = label.map(|l| format!(" ({l})")).unwrap_or_default();

        if self.config.use_color {
            let color = if percentage >= 80.0 {
                filled_bar.green()
            } else if percentage >= 50.0 {
                filled_bar.yellow()
            } else {
                filled_bar.red()
            };
            eprintln!(
                "{}{} {} {}",
                color,
                empty_bar.bright_black(),
                percentage_text.bold(),
                label_text.dimmed()
            );
        } else {
            eprintln!("{filled_bar}{empty_bar} {percentage_text}{label_text}");
        }
    }

    /// Display a compact task summary with visual indicators
    ///
    /// Shows task counts and completion statistics in a concise,
    /// visually appealing format with progress indicators.
    ///
    /// # Output Format
    /// ```text
    /// 📊 Project Summary: 73 tasks  █████████░░ 72.6% complete
    ///    🔄 1 active  ⏳ 20 pending  ✅ 52 done
    /// ```
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of tasks
    /// * `completed` - Number of completed tasks
    /// * `in_progress` - Number of tasks in progress
    /// * `pending` - Number of pending tasks
    /// * `project_name` - Optional project name for context
    pub fn task_summary(
        &self,
        total: usize,
        completed: usize,
        in_progress: usize,
        pending: usize,
        project_name: Option<&str>,
    ) {
        if self.config.quiet {
            return;
        }

        let completion_percentage = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let project_text = project_name
            .map(|name| format!(" for {name}"))
            .unwrap_or_default();

        // Summary line with progress bar
        let summary_line = format!("📊 Task Summary{project_text}: {total} tasks");
        eprint!("{summary_line} ");

        // Inline progress bar
        let bar_width: usize = 12;
        let filled = ((completion_percentage / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        let filled_bar = "█".repeat(filled);
        let empty_bar = "░".repeat(empty);

        if self.config.use_color {
            let color = if completion_percentage >= 80.0 {
                filled_bar.green()
            } else if completion_percentage >= 50.0 {
                filled_bar.yellow()
            } else {
                filled_bar.red()
            };
            eprintln!(
                "{}{} {:.1}% complete",
                color,
                empty_bar.bright_black(),
                completion_percentage
            );
        } else {
            eprintln!("{filled_bar}{empty_bar} {completion_percentage:.1}% complete");
        }

        // Status breakdown
        let status_line =
            format!("   🔄 {in_progress} active  ⏳ {pending} pending  ✅ {completed} completed");

        if self.config.use_color {
            eprintln!("{}", status_line.dimmed());
        } else {
            eprintln!("{status_line}");
        }
    }

    /// Display a section divider with optional title
    ///
    /// Creates a visual section break that's less prominent than headers
    /// but more structured than simple separators. Useful for organizing
    /// content into logical groups.
    ///
    /// # Output Format
    /// ```text
    /// ┌─ Section Title ─────────────────────┐
    /// ```
    ///
    /// # Arguments
    ///
    /// * `title` - Optional section title
    pub fn section_divider(&self, title: Option<&str>) {
        if self.config.quiet {
            return;
        }

        let max_width = self.config.terminal_width.min(80);

        match title {
            Some(title_text) => {
                let title_with_spaces = format!(" {title_text} ");
                let remaining_width = max_width.saturating_sub(title_with_spaces.len() + 4); // 4 for "┌─" and "─┐"
                let left_pad = remaining_width / 2;
                let right_pad = remaining_width - left_pad;

                let line = format!(
                    "┌─{}{}{}─┐",
                    "─".repeat(left_pad),
                    title_with_spaces,
                    "─".repeat(right_pad)
                );

                if self.config.use_color {
                    eprintln!("{}", line.bright_blue().dimmed());
                } else {
                    eprintln!("{line}");
                }
            }
            None => {
                let line = format!("┌{}┐", "─".repeat(max_width.saturating_sub(2)));
                if self.config.use_color {
                    eprintln!("{}", line.bright_blue().dimmed());
                } else {
                    eprintln!("{line}");
                }
            }
        }
    }

    /// Display an indented bullet point with custom icon
    ///
    /// Creates consistent indented content with customizable icons
    /// for hierarchical information display.
    ///
    /// # Arguments
    ///
    /// * `icon` - Icon/emoji to display (e.g., "•", "▶", "📝")
    /// * `text` - The text content to display
    /// * `indent_level` - Indentation level (0 = no indent, 1 = 2 spaces, etc.)
    pub fn bullet_point(&self, icon: &str, text: &str, indent_level: usize) {
        if self.config.quiet {
            return;
        }

        let indent = "  ".repeat(indent_level);
        eprintln!("{indent}{icon} {text}");
    }

    /// Display a status badge with colored background
    ///
    /// Creates a visually distinct status indicator with background
    /// color coding for different states.
    ///
    /// # Output Format
    /// ```text
    /// [ACTIVE] [PENDING] [COMPLETED] [BLOCKED]
    /// ```
    ///
    /// # Arguments
    ///
    /// * `status` - Status text to display
    /// * `color` - Background color for the badge
    pub fn status_badge(&self, status: &str, color: Color) {
        if self.config.quiet {
            return;
        }

        let badge_text = format!(" {} ", status.to_uppercase());

        if self.config.use_color {
            let colored_badge = match color {
                Color::Green => badge_text.black().on_green(),
                Color::Yellow => badge_text.black().on_yellow(),
                Color::Red => badge_text.white().on_red(),
                Color::Blue => badge_text.white().on_blue(),
                Color::Cyan => badge_text.black().on_cyan(),
                Color::Magenta => badge_text.white().on_magenta(),
                Color::White => badge_text.black().on_white(),
                Color::BrightBlack => badge_text.white().on_bright_black(),
            };
            eprint!("{colored_badge}");
        } else {
            eprint!("[{}]", status.to_uppercase());
        }
    }

    /// Display an essential message that bypasses quiet mode
    ///
    /// Essential messages are always displayed regardless of quiet mode settings.
    /// Use this for critical information that users must see, such as:
    /// - Required user actions
    /// - Critical configuration issues
    /// - Legal notices or license information
    ///
    /// # Design Philosophy
    ///
    /// Use sparingly - if everything is "essential", nothing is. Reserve for
    /// information that could cause user confusion or workflow breakdown if missed.
    ///
    /// # Arguments
    ///
    /// * `message` - The essential message to display
    pub fn essential(&self, message: &str) {
        eprintln!("{message}");
    }

    /// Create a colored text string that respects color preferences
    ///
    /// This method provides a unified way to apply colors that automatically
    /// respects the user's color preferences and terminal capabilities.
    /// When colors are disabled, returns the original text unchanged.
    ///
    /// # Color Consistency
    ///
    /// Use this method instead of directly calling colored methods to ensure
    /// consistent behavior with the `--no-color` flag and terminal detection.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to colorize
    /// * `color` - The color to apply (see Color enum)
    ///
    /// # Returns
    ///
    /// Colored string if colors are enabled, plain string otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_memspec::utils::output::{OutputConfig, OutputFormatter, Color};
    ///
    /// let config = OutputConfig::new(false, false, false);
    /// let formatter = OutputFormatter::new(config);
    /// let red_text = formatter.colored_text("Error", Color::Red);
    /// let green_text = formatter.colored_text("Success", Color::Green);
    /// ```
    pub fn colored_text(&self, text: &str, color: Color) -> String {
        if self.config.use_color {
            match color {
                Color::Red => text.red().to_string(),
                Color::Green => text.green().to_string(),
                Color::Yellow => text.yellow().to_string(),
                Color::Blue => text.blue().to_string(),
                Color::Magenta => text.magenta().to_string(),
                Color::Cyan => text.cyan().to_string(),
                Color::White => text.white().to_string(),
                Color::BrightBlack => text.bright_black().to_string(),
            }
        } else {
            text.to_string()
        }
    }

    /// Display an animated progress indicator with percentage and description
    ///
    /// Creates a visual progress bar that updates in place, showing completion
    /// percentage, current/total counts, and a description. Automatically
    /// adds a newline when progress reaches 100%.
    ///
    /// # Output Format
    /// ```text
    /// [████████████░░░░░░░░] 60% (6/10) Processing files...
    /// ```
    ///
    /// # Progress Bar Components
    ///
    /// - **Bar**: Visual representation using filled (█) and empty (░) blocks
    /// - **Percentage**: Calculated completion percentage
    /// - **Counter**: Current and total item counts
    /// - **Description**: Context about what's being processed
    ///
    /// # Terminal Adaptation
    ///
    /// Bar width automatically adapts to terminal width (1/3 of available width,
    /// maximum 20 characters) to ensure readability across different screen sizes.
    ///
    /// # Usage Pattern
    ///
    /// ```rust
    /// use airs_memspec::utils::output::{OutputConfig, OutputFormatter};
    ///
    /// let config = OutputConfig::new(false, false, false);
    /// let formatter = OutputFormatter::new(config);
    /// let items = vec!["file1.txt", "file2.txt", "file3.txt"];
    /// for (i, item) in items.iter().enumerate() {
    ///     formatter.progress(i + 1, items.len(), "Processing items");
    ///     // ... process item ...
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `current` - Current progress count (0-based indexing common)
    /// * `total` - Total number of items to process
    /// * `description` - Human-readable description of the operation
    pub fn progress(&self, current: usize, total: usize, description: &str) {
        if self.config.quiet {
            return;
        }

        let percentage = if total > 0 {
            (current * 100) / total
        } else {
            0
        };

        let bar_width = (self.config.terminal_width / 3).min(20);
        let filled = (current * bar_width) / total.max(1);
        let empty = bar_width - filled;

        let bar = format!(
            "[{}{}] {}% ({}/{}) {}",
            "█".repeat(filled).green(),
            "░".repeat(empty).bright_black(),
            percentage,
            current,
            total,
            description
        );

        // Use \r to overwrite the current line for animation effect
        eprint!("\r{bar}");
        io::stdout().flush().unwrap_or(());

        if current >= total {
            eprintln!(); // Move to new line when complete
        }
    }
}

/// Color enumeration for consistent terminal output styling
///
/// This enum provides a controlled palette of colors that work well across
/// different terminal themes and provide good contrast and readability.
///
/// # Color Semantics
///
/// - **Red**: Errors, failures, critical issues
/// - **Green**: Success, completion, positive states
/// - **Yellow**: Warnings, cautions, intermediate states
/// - **Blue**: Headers, information, neutral emphasis
/// - **Magenta**: Special emphasis, highlights
/// - **Cyan**: Informational content, metadata
/// - **White**: High contrast text, important content
/// - **BrightBlack**: Subdued text, verbose information, separators
///
/// # Terminal Compatibility
///
/// These colors are chosen to provide good readability across:
/// - Light and dark terminal themes
/// - High and low contrast displays
/// - Color and monochrome terminals
/// - Various accessibility configurations
#[derive(Debug, Clone, Copy)]
pub enum Color {
    /// Red color - for errors and critical issues
    Red,
    /// Green color - for success and positive outcomes
    Green,
    /// Yellow color - for warnings and cautions
    Yellow,
    /// Blue color - for headers and information
    Blue,
    /// Magenta color - for special emphasis
    Magenta,
    /// Cyan color - for informational content
    Cyan,
    /// White color - for high contrast text
    White,
    /// Bright black (dark gray) - for subdued content
    BrightBlack,
}
