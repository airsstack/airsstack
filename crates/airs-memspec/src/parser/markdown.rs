//! Markdown parsing for Multi-Project Memory Bank files
//!
//! This module provides functionality for parsing markdown content from memory bank files,
//! extracting structured sections, handling YAML frontmatter, and parsing task lists with
//! status information.

use std::collections::HashMap;
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde_yml::Value as YamlValue;

use crate::utils::fs::FsResult;

/// Represents the parsed content of a markdown file
///
/// This structure captures all the important information that can be extracted
/// from a memory bank markdown file, including frontmatter, content sections,
/// and structured data like task lists.
#[derive(Debug, Clone)]
pub struct MarkdownContent {
    /// YAML frontmatter data, if present
    pub frontmatter: Option<HashMap<String, YamlValue>>,

    /// Raw markdown content without frontmatter
    pub content: String,

    /// Structured sections extracted from the content
    /// Maps section headings to their content
    pub sections: HashMap<String, String>,

    /// Extracted task information, if any
    pub tasks: Vec<TaskItem>,

    /// File metadata extracted from content
    pub metadata: FileMetadata,
}

/// Represents a task item found in markdown content
///
/// Task items can appear in various formats within memory bank files,
/// such as task lists, progress tracking tables, or status sections.
#[derive(Debug, Clone)]
pub struct TaskItem {
    /// Task identifier (e.g., "task_001", "TASK-123")
    pub id: Option<String>,

    /// Task title or description
    pub title: String,

    /// Current status of the task
    pub status: TaskStatus,

    /// Additional details or notes
    pub details: Option<String>,

    /// Last updated date, if specified
    pub updated: Option<String>,
}

/// Task status enumeration
///
/// Represents the various states a task can be in, based on common
/// patterns found in memory bank task tracking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Task has not been started yet
    NotStarted,
    /// Task is currently being worked on
    InProgress,
    /// Task has been completed successfully
    Completed,
    /// Task is blocked by dependencies or issues
    Blocked,
    /// Task has been abandoned or cancelled
    Abandoned,
    /// Status could not be determined or is in unknown format
    Unknown(String),
}

/// File metadata extracted from markdown content
///
/// Contains information about the file itself, such as title, description,
/// and other metadata that can be extracted from the content structure.
#[derive(Debug, Clone, Default)]
pub struct FileMetadata {
    /// Main title of the document (usually from first heading)
    pub title: Option<String>,

    /// Document description or summary
    pub description: Option<String>,

    /// Status information, if present
    pub status: Option<String>,

    /// Last updated date mentioned in content
    pub updated: Option<String>,

    /// Any additional metadata found in the content
    pub extra: HashMap<String, String>,
}

/// Markdown parser for memory bank files
///
/// This parser is specifically designed to handle the structured markdown format
/// used in Multi-Project Memory Bank files, extracting relevant information
/// while being resilient to variations in format and missing sections.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse a markdown file from a given path
    ///
    /// Reads the file content and parses it using the comprehensive markdown
    /// parsing pipeline to extract all relevant structured information.
    ///
    /// # Arguments
    /// * `file_path` - Path to the markdown file to parse
    ///
    /// # Returns
    /// * `Ok(MarkdownContent)` - Parsed content with all extracted information
    /// * `Err(FsError)` - File reading or parsing errors
    ///
    /// # Example
    /// ```rust,no_run
    /// use airs_memspec::parser::markdown::MarkdownParser;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = MarkdownParser::parse_file(&PathBuf::from("project_brief.md"))?;
    /// println!("Found {} sections", content.sections.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_file(file_path: &Path) -> FsResult<MarkdownContent> {
        let content = std::fs::read_to_string(file_path).map_err(crate::utils::fs::FsError::Io)?;

        Self::parse_content(&content)
    }

    /// Parse markdown content from a string
    ///
    /// The core parsing function that handles the complete markdown parsing
    /// pipeline including frontmatter extraction, section parsing, and
    /// task item detection.
    ///
    /// # Arguments
    /// * `content` - Raw markdown content as a string
    ///
    /// # Returns
    /// * `Ok(MarkdownContent)` - Parsed content with all extracted information
    /// * `Err(FsError)` - Parsing errors
    ///
    /// # Parsing Process
    /// 1. Extract YAML frontmatter if present
    /// 2. Parse markdown structure using pulldown-cmark
    /// 3. Extract sections based on heading hierarchy
    /// 4. Identify and parse task items
    /// 5. Extract file metadata from content patterns
    pub fn parse_content(content: &str) -> FsResult<MarkdownContent> {
        // Step 1: Extract frontmatter
        let (frontmatter, markdown_content) = Self::extract_frontmatter(content)?;

        // Step 2: Parse markdown structure
        let sections = Self::extract_sections(&markdown_content);

        // Step 3: Extract task items
        let tasks = Self::extract_tasks(&markdown_content, &sections);

        // Step 4: Extract file metadata
        let metadata = Self::extract_metadata(&markdown_content, &sections, &frontmatter);

        Ok(MarkdownContent {
            frontmatter,
            content: markdown_content,
            sections,
            tasks,
            metadata,
        })
    }

    /// Extract YAML frontmatter from markdown content
    ///
    /// Detects and parses YAML frontmatter at the beginning of markdown files,
    /// which is commonly used for metadata in memory bank files.
    ///
    /// # Arguments
    /// * `content` - Raw markdown content
    ///
    /// # Returns
    /// * `(Option<HashMap<String, YamlValue>>, String)` - Frontmatter and remaining content
    /// * `Err(FsError)` - YAML parsing errors
    ///
    /// # Frontmatter Format
    /// Expects frontmatter in the format:
    /// ```yaml
    /// ---
    /// key: value
    /// another_key: another_value
    /// ---
    /// ```
    fn extract_frontmatter(
        content: &str,
    ) -> FsResult<(Option<HashMap<String, YamlValue>>, String)> {
        let content = content.trim_start();

        // Check if content starts with frontmatter delimiter
        if !content.starts_with("---") {
            return Ok((None, content.to_string()));
        }

        // Find the closing delimiter
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 3 {
            return Ok((None, content.to_string()));
        }

        // Look for the closing --- delimiter
        let mut frontmatter_end = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                frontmatter_end = Some(i);
                break;
            }
        }

        let Some(end_line) = frontmatter_end else {
            // No closing delimiter found, treat as regular content
            return Ok((None, content.to_string()));
        };

        // Extract frontmatter YAML
        let frontmatter_lines = &lines[1..end_line];
        let frontmatter_yaml = frontmatter_lines.join("\n");

        // Parse YAML frontmatter
        let frontmatter: HashMap<String, YamlValue> = serde_yml::from_str(&frontmatter_yaml)
            .map_err(|e| {
                crate::utils::fs::FsError::ParseError {
                    message: format!("YAML frontmatter parsing failed: {e}"),
                    suggestion: "Check the YAML syntax in the frontmatter. Common issues: incorrect indentation, missing quotes, or invalid YAML structure.".to_string(),
                }
            })?;

        // Return remaining content
        let remaining_content = lines[(end_line + 1)..].join("\n");

        Ok((Some(frontmatter), remaining_content))
    }

    /// Extract sections from markdown content based on heading hierarchy
    ///
    /// Parses the markdown structure to identify sections defined by headings
    /// and captures the content under each section for structured access.
    ///
    /// # Arguments
    /// * `content` - Markdown content without frontmatter
    ///
    /// # Returns
    /// * `HashMap<String, String>` - Map of section titles to their content
    ///
    /// # Section Extraction Rules
    /// - Sections are defined by markdown headings (# ## ### etc.)
    /// - The first heading is treated as the document title, not a section
    /// - Section content includes everything until the next heading of equal or higher level
    /// - Nested headings are included in parent section content
    fn extract_sections(content: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        let mut first_heading_found = false;

        while i < lines.len() {
            let line = lines[i].trim();

            // Check if this line is a heading
            if line.starts_with('#') {
                let heading_level = line.chars().take_while(|&c| c == '#').count();
                let heading_text = line.trim_start_matches('#').trim().to_string();

                // Skip the first heading (document title)
                if !first_heading_found {
                    first_heading_found = true;
                    i += 1;
                    continue;
                }

                // Find content for this section (everything until next heading of same or higher level)
                let mut section_content = Vec::new();
                i += 1; // Move past heading line

                while i < lines.len() {
                    let next_line = lines[i].trim();

                    // Check if we've hit another heading
                    if next_line.starts_with('#') {
                        let next_heading_level =
                            next_line.chars().take_while(|&c| c == '#').count();
                        if next_heading_level <= heading_level {
                            // This heading is at same or higher level, so our section ends here
                            break;
                        }
                    }

                    section_content.push(lines[i]);
                    i += 1;
                }

                // Store the section
                if !heading_text.is_empty() {
                    let content_text = section_content.join("\n").trim().to_string();
                    sections.insert(heading_text, content_text);
                }
            } else {
                i += 1;
            }
        }

        sections
    }

    /// Extract task items from markdown content and sections
    ///
    /// Identifies and parses task-related information from various formats
    /// commonly found in memory bank files, including task lists, tables,
    /// and status sections.
    ///
    /// # Arguments
    /// * `content` - Full markdown content
    /// * `sections` - Pre-parsed sections for targeted extraction
    ///
    /// # Returns
    /// * `Vec<TaskItem>` - List of all identified task items
    ///
    /// # Task Recognition Patterns
    /// - Markdown task lists with checkboxes: `- [ ] Task description`
    /// - Task tables with status columns
    /// - Task index entries with IDs and status
    /// - Progress tracking sections with task references
    fn extract_tasks(content: &str, sections: &HashMap<String, String>) -> Vec<TaskItem> {
        let mut tasks = Vec::new();

        // Extract from task-specific sections
        for (section_name, section_content) in sections {
            if section_name.to_lowercase().contains("task")
                || section_name.to_lowercase().contains("progress")
                || section_name.to_lowercase().contains("subtask")
            {
                tasks.extend(Self::parse_task_content(section_content));
            }
        }

        // Also check the full content for task patterns
        tasks.extend(Self::parse_task_content(content));

        // Deduplicate tasks based on ID or title
        Self::deduplicate_tasks(tasks)
    }

    /// Parse task items from a content string
    ///
    /// Internal method that handles the actual parsing logic for identifying
    /// task items in various formats within a given content string.
    ///
    /// # Arguments
    /// * `content` - Content string to parse for task items
    ///
    /// # Returns
    /// * `Vec<TaskItem>` - List of task items found in the content
    fn parse_task_content(content: &str) -> Vec<TaskItem> {
        let mut tasks = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Pattern 1: Task index entries (check first to avoid conflict with task list)
            // - [task_001] Task Name - Status description
            if let Some(task) = Self::parse_task_index_item(line) {
                tasks.push(task);
                continue;
            }

            // Pattern 2: Markdown task list items
            // - [ ] Task description
            // - [x] Completed task
            if let Some(task) = Self::parse_task_list_item(line) {
                tasks.push(task);
                continue;
            }

            // Pattern 3: Table rows with task information
            // | task_001 | Task Description | completed | 2025-08-03 |
            if let Some(task) = Self::parse_task_table_row(line) {
                tasks.push(task);
                continue;
            }
        }

        tasks
    }

    /// Parse a markdown task list item
    ///
    /// Handles checkbox-style task items commonly found in markdown files.
    ///
    /// # Arguments
    /// * `line` - Line of text to parse
    ///
    /// # Returns
    /// * `Option<TaskItem>` - Parsed task item if line matches pattern
    fn parse_task_list_item(line: &str) -> Option<TaskItem> {
        // Match patterns like:
        // - [ ] Task description
        // - [x] Completed task
        // - [X] Completed task
        // Note: Must have exactly one character between brackets for checkbox

        if line.starts_with("- [") && line.len() > 5 && line.chars().nth(4) == Some(']') {
            let checkbox = line.chars().nth(3)?;
            let status = match checkbox {
                ' ' => TaskStatus::NotStarted,
                'x' | 'X' => TaskStatus::Completed,
                _ => return None, // Not a valid checkbox, might be something else
            };

            let title = line[5..].trim().to_string();
            if !title.is_empty() {
                return Some(TaskItem {
                    id: None,
                    title,
                    status,
                    details: None,
                    updated: None,
                });
            }
        }

        None
    }

    /// Parse a task index item
    ///
    /// Handles task entries from task index files or sections.
    ///
    /// # Arguments
    /// * `line` - Line of text to parse
    ///
    /// # Returns
    /// * `Option<TaskItem>` - Parsed task item if line matches pattern
    fn parse_task_index_item(line: &str) -> Option<TaskItem> {
        // Match patterns like:
        // - [task_001] Task Name - Status description
        // - [TASK-123] Another task name - In Progress
        // But NOT patterns like:
        // - [ ] Checkbox item
        // - [x] Completed checkbox

        if line.starts_with("- [") {
            if let Some(close_bracket) = line.find(']') {
                // Skip if this looks like a checkbox (single character between brackets)
                if close_bracket == 4 {
                    // This is likely a checkbox like "- [ ]" or "- [x]"
                    return None;
                }

                let id = line[3..close_bracket].trim().to_string();
                let remaining = line[(close_bracket + 1)..].trim();

                // Look for status information after dash
                let (title, status) = if let Some(dash_pos) = remaining.rfind(" - ") {
                    let title = remaining[..dash_pos].trim().to_string();
                    let status_text = remaining[(dash_pos + 3)..].trim();
                    let status = Self::parse_status_text(status_text);
                    (title, status)
                } else {
                    (
                        remaining.to_string(),
                        TaskStatus::Unknown("no status".to_string()),
                    )
                };

                if !title.is_empty() {
                    return Some(TaskItem {
                        id: Some(id),
                        title,
                        status,
                        details: None,
                        updated: None,
                    });
                }
            }
        }

        None
    }

    /// Parse a task table row
    ///
    /// Handles task information presented in markdown table format.
    ///
    /// # Arguments
    /// * `line` - Line of text to parse
    ///
    /// # Returns
    /// * `Option<TaskItem>` - Parsed task item if line matches pattern
    fn parse_task_table_row(line: &str) -> Option<TaskItem> {
        // Match table rows like:
        // | task_001 | Task Description | completed | 2025-08-03 | Notes |

        if line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 4 {
            let parts: Vec<&str> = line[1..line.len() - 1]
                .split('|')
                .map(|s| s.trim())
                .collect();

            if parts.len() >= 3 {
                let id = parts[0].to_string();
                let title = parts[1].to_string();
                let status_text = parts[2];
                let status = Self::parse_status_text(status_text);

                let updated = if parts.len() > 3 && !parts[3].is_empty() {
                    Some(parts[3].to_string())
                } else {
                    None
                };

                let details = if parts.len() > 4 && !parts[4].is_empty() {
                    Some(parts[4].to_string())
                } else {
                    None
                };

                // Only return if we have meaningful content
                // Exclude header rows and separator rows
                if !id.is_empty()
                    && !title.is_empty()
                    && id != "ID"
                    && title != "Description"
                    && !id.chars().all(|c| c == '-' || c == ' ')
                    && !title.chars().all(|c| c == '-' || c == ' ')
                {
                    return Some(TaskItem {
                        id: Some(id),
                        title,
                        status,
                        details,
                        updated,
                    });
                }
            }
        }

        None
    }

    /// Parse status text into TaskStatus enum
    ///
    /// Converts various textual status representations into the standardized
    /// TaskStatus enumeration, handling common variations and formats.
    ///
    /// # Arguments
    /// * `status_text` - Status text to parse
    ///
    /// # Returns
    /// * `TaskStatus` - Parsed status or Unknown with original text
    fn parse_status_text(status_text: &str) -> TaskStatus {
        let normalized = status_text.to_lowercase().trim().replace(['-', '_'], " ");

        match normalized.as_str() {
            "not started" | "pending" | "todo" | "not start" => TaskStatus::NotStarted,
            "in progress" | "active" | "working" | "ongoing" | "in work" => TaskStatus::InProgress,
            "completed" | "done" | "finished" | "complete" | "success" => TaskStatus::Completed,
            "blocked" | "stuck" | "waiting" | "on hold" => TaskStatus::Blocked,
            "abandoned" | "cancelled" | "canceled" | "dropped" | "abort" => TaskStatus::Abandoned,
            _ => TaskStatus::Unknown(status_text.to_string()),
        }
    }

    /// Extract file metadata from content and sections
    ///
    /// Analyzes the parsed content to extract metadata information such as
    /// title, description, status, and other relevant file properties.
    ///
    /// # Arguments
    /// * `content` - Full markdown content
    /// * `sections` - Pre-parsed sections
    /// * `frontmatter` - YAML frontmatter data
    ///
    /// # Returns
    /// * `FileMetadata` - Extracted metadata information
    fn extract_metadata(
        content: &str,
        _sections: &HashMap<String, String>,
        frontmatter: &Option<HashMap<String, YamlValue>>,
    ) -> FileMetadata {
        let mut metadata = FileMetadata::default();

        // Extract title from first heading or frontmatter
        if let Some(frontmatter) = frontmatter {
            if let Some(YamlValue::String(title)) = frontmatter.get("title") {
                metadata.title = Some(title.clone());
            }
            if let Some(YamlValue::String(desc)) = frontmatter.get("description") {
                metadata.description = Some(desc.clone());
            }
            if let Some(YamlValue::String(status)) = frontmatter.get("status") {
                metadata.status = Some(status.clone());
            }
        }

        // If no title from frontmatter, extract from first heading
        if metadata.title.is_none() {
            if let Some(first_heading) = Self::extract_first_heading(content) {
                metadata.title = Some(first_heading);
            }
        }

        // Look for status information in content
        if metadata.status.is_none() {
            for line in content.lines() {
                if line.to_lowercase().contains("status:") {
                    if let Some(status) = line.split(':').nth(1) {
                        metadata.status = Some(status.trim().to_string());
                        break;
                    }
                }
            }
        }

        // Look for update information
        for line in content.lines() {
            if line.to_lowercase().contains("updated:") {
                if let Some(updated) = line.split(':').nth(1) {
                    metadata.updated = Some(updated.trim().to_string());
                    break;
                }
            }
        }

        metadata
    }

    /// Extract the first heading from markdown content
    ///
    /// Helper method to find and extract the first heading in the content,
    /// which is commonly used as the document title.
    ///
    /// # Arguments
    /// * `content` - Markdown content to search
    ///
    /// # Returns
    /// * `Option<String>` - First heading text if found
    fn extract_first_heading(content: &str) -> Option<String> {
        let parser = Parser::new(content);
        let mut in_heading = false;
        let mut heading_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { .. }) => {
                    in_heading = true;
                    heading_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if in_heading && !heading_text.is_empty() {
                        return Some(heading_text.trim().to_string());
                    }
                    in_heading = false;
                }
                Event::Text(text) => {
                    if in_heading {
                        heading_text.push_str(&text);
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Deduplicate tasks based on ID or title
    ///
    /// Removes duplicate task entries that might have been found in multiple
    /// sections or formats, preferring entries with more complete information.
    ///
    /// # Arguments
    /// * `tasks` - Vector of task items to deduplicate
    ///
    /// # Returns
    /// * `Vec<TaskItem>` - Deduplicated list of task items
    fn deduplicate_tasks(tasks: Vec<TaskItem>) -> Vec<TaskItem> {
        let mut seen_ids = std::collections::HashSet::new();
        let mut seen_titles = std::collections::HashSet::new();
        let mut result = Vec::new();

        for task in tasks {
            let mut is_duplicate = false;

            // Check for ID-based duplicates
            if let Some(ref id) = task.id {
                if seen_ids.contains(id) {
                    is_duplicate = true;
                } else {
                    seen_ids.insert(id.clone());
                }
            }

            // Check for title-based duplicates (only if no ID match)
            if !is_duplicate {
                if seen_titles.contains(&task.title) {
                    is_duplicate = true;
                } else {
                    seen_titles.insert(task.title.clone());
                }
            }

            if !is_duplicate {
                result.push(task);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let content = r#"# Test Document

This is a test document.

## Section 1

Some content here.

## Section 2

More content here.
"#;

        let parsed = MarkdownParser::parse_content(content).unwrap();

        assert_eq!(parsed.metadata.title, Some("Test Document".to_string()));
        assert_eq!(parsed.sections.len(), 2);
        assert!(parsed.sections.contains_key("Section 1"));
        assert!(parsed.sections.contains_key("Section 2"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: "Test Document"
status: "completed"
---

# Content

This is the content.
"#;

        let parsed = MarkdownParser::parse_content(content).unwrap();

        assert!(parsed.frontmatter.is_some());
        let frontmatter = parsed.frontmatter.unwrap();
        assert!(frontmatter.contains_key("title"));
        assert!(frontmatter.contains_key("status"));
    }

    #[test]
    fn test_parse_task_list() {
        let content = r#"# Tasks

- [ ] Not started task
- [x] Completed task
- [X] Another completed task
"#;

        let parsed = MarkdownParser::parse_content(content).unwrap();

        assert_eq!(parsed.tasks.len(), 3);
        assert_eq!(parsed.tasks[0].status, TaskStatus::NotStarted);
        assert_eq!(parsed.tasks[1].status, TaskStatus::Completed);
        assert_eq!(parsed.tasks[2].status, TaskStatus::Completed);
    }

    #[test]
    fn test_parse_task_index() {
        let content = r#"# Task Index

- [task_001] First task - completed
- [task_002] Second task - in progress
- [task_003] Third task - blocked
"#;

        let parsed = MarkdownParser::parse_content(content).unwrap();

        assert_eq!(parsed.tasks.len(), 3);
        assert_eq!(parsed.tasks[0].id, Some("task_001".to_string()));
        assert_eq!(parsed.tasks[0].status, TaskStatus::Completed);
        assert_eq!(parsed.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(parsed.tasks[2].status, TaskStatus::Blocked);
    }

    #[test]
    fn test_parse_task_table() {
        let content = r#"# Progress Tracking

| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | First subtask | complete | 2025-08-03 | Working well |
| 1.2 | Second subtask | in progress | 2025-08-03 | Almost done |
"#;

        let parsed = MarkdownParser::parse_content(content).unwrap();

        assert_eq!(parsed.tasks.len(), 2);
        assert_eq!(parsed.tasks[0].id, Some("1.1".to_string()));
        assert_eq!(parsed.tasks[0].title, "First subtask");
        assert_eq!(parsed.tasks[0].status, TaskStatus::Completed);
        assert_eq!(parsed.tasks[0].updated, Some("2025-08-03".to_string()));
        assert_eq!(parsed.tasks[0].details, Some("Working well".to_string()));
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(
            MarkdownParser::parse_status_text("completed"),
            TaskStatus::Completed
        );
        assert_eq!(
            MarkdownParser::parse_status_text("in-progress"),
            TaskStatus::InProgress
        );
        assert_eq!(
            MarkdownParser::parse_status_text("not_started"),
            TaskStatus::NotStarted
        );
        assert_eq!(
            MarkdownParser::parse_status_text("blocked"),
            TaskStatus::Blocked
        );
        assert_eq!(
            MarkdownParser::parse_status_text("abandoned"),
            TaskStatus::Abandoned
        );

        match MarkdownParser::parse_status_text("custom-status") {
            TaskStatus::Unknown(s) => assert_eq!(s, "custom-status"),
            _ => panic!("Expected Unknown status"),
        }
    }
}
