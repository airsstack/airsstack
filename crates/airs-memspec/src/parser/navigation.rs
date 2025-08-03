//! Memory bank file system navigation and discovery
//!
//! This module provides functionality for discovering, validating, and navigating
//! Multi-Project Memory Bank directory structures and files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::utils::fs::FsResult;

/// Memory bank directory structure representation
///
/// This structure represents the complete discovered layout of a Multi-Project Memory Bank,
/// including all workspace-level files, sub-projects, and their associated task files.
/// It serves as the primary data structure for navigation and file access operations.
#[derive(Debug, Clone)]
pub struct MemoryBankStructure {
    /// Root path of the memory bank (.copilot/memory_bank/)
    /// This is the base directory from which all other paths are resolved
    pub root_path: PathBuf,

    /// Workspace-level files and configuration
    /// Contains shared patterns, project briefs, and workspace-wide documentation
    pub workspace: WorkspaceFiles,

    /// Current context file (current_context.md)
    /// Tracks the currently active sub-project and workspace state
    pub current_context: Option<PathBuf>,

    /// Context snapshots directory (context_snapshots/)
    /// Contains historical snapshots for restoration and analysis
    pub snapshots_dir: Option<PathBuf>,

    /// Sub-projects directory (sub_projects/)
    /// Root directory containing all individual project folders
    pub sub_projects_dir: Option<PathBuf>,

    /// Discovered sub-projects and their complete file structures
    /// Maps sub-project names to their discovered file layouts
    pub sub_projects: HashMap<String, SubProjectFiles>,
}
/// Workspace-level file structure
///
/// Represents the files found in the workspace/ directory, which contain
/// shared configuration, patterns, and documentation that apply across
/// all sub-projects in the workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceFiles {
    /// workspace/project_brief.md - Overall workspace vision and objectives
    /// Contains the high-level description of what this workspace aims to achieve
    pub project_brief: Option<PathBuf>,

    /// workspace/shared_patterns.md - Cross-project patterns and practices
    /// Documents reusable patterns, coding standards, and architectural decisions
    pub shared_patterns: Option<PathBuf>,

    /// workspace/workspace_architecture.md - High-level system architecture
    /// Describes the overall structure and relationships between components
    pub workspace_architecture: Option<PathBuf>,

    /// workspace/workspace_progress.md - Cross-project progress and milestones
    /// Tracks workspace-wide progress, strategic decisions, and major milestones
    pub workspace_progress: Option<PathBuf>,
}

/// Sub-project file structure
///
/// Represents the complete file layout of an individual sub-project within
/// the workspace. Each sub-project has its own directory with standardized
/// files for project management, technical documentation, and task tracking.
#[derive(Debug, Clone)]
pub struct SubProjectFiles {
    /// Sub-project root directory (sub_projects/{project_name}/)
    /// Base path for all files belonging to this specific sub-project
    pub root_path: PathBuf,

    /// project_brief.md - Sub-project foundation and scope definition
    /// Defines what this specific sub-project does and why it exists
    pub project_brief: Option<PathBuf>,

    /// product_context.md - User experience and product requirements
    /// Documents user needs, functionality, and success criteria
    pub product_context: Option<PathBuf>,

    /// active_context.md - Current work focus and recent changes
    /// Tracks what's currently being worked on and immediate next steps
    pub active_context: Option<PathBuf>,

    /// system_patterns.md - Technical architecture and design patterns
    /// Documents the technical approach, patterns, and architectural decisions
    pub system_patterns: Option<PathBuf>,

    /// tech_context.md - Technology stack and infrastructure details
    /// Lists technologies used, dependencies, constraints, and deployment context
    pub tech_context: Option<PathBuf>,

    /// progress.md - Work completed, current status, and known issues
    /// Tracks what's working, what's left to build, and current challenges
    pub progress: Option<PathBuf>,

    /// tasks/ directory - Contains all task management files
    /// Directory where individual task files and the task index are stored
    pub tasks_dir: Option<PathBuf>,

    /// Individual task files (task_*.md files excluding _index.md)
    /// All discovered task files with their complete paths for direct access
    pub task_files: Vec<PathBuf>,
}

/// Memory bank discovery and navigation functionality
///
/// This struct provides static methods for discovering, analyzing, and validating
/// Multi-Project Memory Bank directory structures. It implements the core logic
/// for finding memory banks, parsing their contents, and providing access to
/// discovered files and metadata.
///
/// The navigator follows these key principles:
/// - Upward directory traversal to find memory bank roots
/// - Comprehensive file discovery with graceful handling of missing files
/// - Detailed validation and diagnostic reporting
/// - Consistent path resolution and accessibility checking
pub struct MemoryBankNavigator;

impl MemoryBankNavigator {
    /// Discover memory bank structure starting from a given path
    ///
    /// This function will traverse the directory structure looking for
    /// `.copilot/memory_bank/` and analyze its contents.
    ///
    /// # Arguments
    /// * `start_path` - Path to start searching from (typically workspace root)
    ///
    /// # Returns
    /// * `Ok(MemoryBankStructure)` - If memory bank structure is found
    /// * `Err(FsError)` - If there are file system errors or structure not found
    ///
    /// # Examples
    /// ```rust
    /// use airs_memspec::parser::navigation::MemoryBankNavigator;
    /// use std::path::PathBuf;
    ///
    /// let structure = MemoryBankNavigator::discover_structure(
    ///     &PathBuf::from("/workspace/project")
    /// )?;
    /// println!("Found {} sub-projects", structure.sub_projects.len());
    /// ```
    pub fn discover_structure(start_path: &Path) -> FsResult<MemoryBankStructure> {
        let memory_bank_path = Self::find_memory_bank_root(start_path)?;
        Self::analyze_structure(&memory_bank_path)
    }

    /// Find the memory bank root directory
    ///
    /// Performs upward directory traversal starting from the given path to locate
    /// the `.copilot/memory_bank/` directory. This allows the navigator to work
    /// from any location within a workspace hierarchy.
    ///
    /// # Arguments
    /// * `start_path` - Starting point for the upward search
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the memory bank root if found
    /// * `Err(FsError::PathNotFound)` - If no memory bank is found in the hierarchy
    ///
    /// # Search Strategy
    /// 1. Check current directory for `.copilot/memory_bank/`
    /// 2. Move up one directory level and repeat
    /// 3. Continue until root directory is reached
    /// 4. Return error if no memory bank directory is found
    fn find_memory_bank_root(start_path: &Path) -> FsResult<PathBuf> {
        let mut current_path = start_path.to_path_buf();

        loop {
            // Construct the expected memory bank path at this level
            let memory_bank_path = current_path.join(".copilot").join("memory_bank");

            // Check if this directory exists and is actually a directory
            if memory_bank_path.exists() && memory_bank_path.is_dir() {
                return Ok(memory_bank_path);
            }

            // Move up one directory level
            match current_path.parent() {
                Some(parent) => current_path = parent.to_path_buf(),
                None => break, // Reached filesystem root without finding memory bank
            }
        }

        // No memory bank found in the entire hierarchy
        Err(crate::utils::fs::FsError::PathNotFound {
            path: PathBuf::from(".copilot/memory_bank"),
        })
    }

    /// Analyze the memory bank directory structure
    ///
    /// Once the memory bank root is found, this method performs a comprehensive
    /// analysis of its contents, discovering all workspace files, sub-projects,
    /// and their associated task files.
    ///
    /// # Arguments
    /// * `memory_bank_path` - Root path of the memory bank directory
    ///
    /// # Returns
    /// * `Ok(MemoryBankStructure)` - Complete structure representation
    /// * `Err(FsError)` - File system errors during discovery
    ///
    /// # Discovery Process
    /// 1. Discover workspace-level files in workspace/ directory
    /// 2. Find current_context.md and context_snapshots/ directory
    /// 3. Locate sub_projects/ directory
    /// 4. Recursively discover all sub-projects and their files
    /// 5. Build complete structure representation
    fn analyze_structure(memory_bank_path: &Path) -> FsResult<MemoryBankStructure> {
        let mut structure = MemoryBankStructure {
            root_path: memory_bank_path.to_path_buf(),
            workspace: Self::discover_workspace_files(memory_bank_path)?,
            current_context: Self::find_file(memory_bank_path, "current_context.md"),
            snapshots_dir: Self::find_directory(memory_bank_path, "context_snapshots"),
            sub_projects_dir: Self::find_directory(memory_bank_path, "sub_projects"),
            sub_projects: HashMap::new(),
        };

        // Discover sub-projects if the directory exists
        // This is optional as some workspaces might not have sub-projects yet
        if let Some(sub_projects_dir) = &structure.sub_projects_dir {
            structure.sub_projects = Self::discover_sub_projects(sub_projects_dir)?;
        }

        Ok(structure)
    }

    /// Discover workspace-level files
    ///
    /// Searches the workspace/ directory for standard Multi-Project Memory Bank
    /// workspace files. These files contain configuration and documentation
    /// that applies across all sub-projects.
    ///
    /// # Arguments
    /// * `memory_bank_path` - Root path of the memory bank
    ///
    /// # Returns
    /// * `Ok(WorkspaceFiles)` - Structure containing paths to found workspace files
    /// * `Err(FsError)` - File system errors during discovery
    ///
    /// # Files Discovered
    /// - project_brief.md: Workspace vision and objectives
    /// - shared_patterns.md: Cross-project patterns and practices
    /// - workspace_architecture.md: High-level system architecture
    /// - workspace_progress.md: Cross-project progress tracking
    fn discover_workspace_files(memory_bank_path: &Path) -> FsResult<WorkspaceFiles> {
        let workspace_dir = memory_bank_path.join("workspace");

        Ok(WorkspaceFiles {
            project_brief: Self::find_file(&workspace_dir, "project_brief.md"),
            shared_patterns: Self::find_file(&workspace_dir, "shared_patterns.md"),
            workspace_architecture: Self::find_file(&workspace_dir, "workspace_architecture.md"),
            workspace_progress: Self::find_file(&workspace_dir, "workspace_progress.md"),
        })
    }

    /// Discover all sub-projects in the sub_projects directory
    ///
    /// Iterates through the sub_projects/ directory to find all individual
    /// project folders, then analyzes each one to discover its file structure.
    ///
    /// # Arguments
    /// * `sub_projects_dir` - Path to the sub_projects directory
    ///
    /// # Returns
    /// * `Ok(HashMap<String, SubProjectFiles>)` - Map of project names to their file structures
    /// * `Err(FsError)` - File system errors during discovery
    ///
    /// # Discovery Process
    /// 1. Read all entries in the sub_projects directory
    /// 2. Filter for directories only (ignore files)
    /// 3. For each directory, discover its internal file structure
    /// 4. Map directory names to their discovered file layouts
    /// 5. Return complete mapping of all sub-projects
    fn discover_sub_projects(
        sub_projects_dir: &Path,
    ) -> FsResult<HashMap<String, SubProjectFiles>> {
        let mut sub_projects = HashMap::new();

        // Gracefully handle case where sub_projects directory doesn't exist
        if !sub_projects_dir.exists() {
            return Ok(sub_projects);
        }

        let entries = std::fs::read_dir(sub_projects_dir).map_err(crate::utils::fs::FsError::Io)?;

        for entry in entries {
            let entry = entry.map_err(crate::utils::fs::FsError::Io)?;
            let path = entry.path();

            // Only process directories (sub-projects), ignore files
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Discover the file structure for this specific sub-project
                    let sub_project_files = Self::discover_sub_project_files(&path)?;
                    sub_projects.insert(name.to_string(), sub_project_files);
                }
            }
        }

        Ok(sub_projects)
    }

    /// Discover files within a specific sub-project directory
    ///
    /// Analyzes a single sub-project directory to find all standard Multi-Project
    /// Memory Bank files and the tasks directory with its associated task files.
    ///
    /// # Arguments
    /// * `sub_project_path` - Root path of the sub-project directory
    ///
    /// # Returns
    /// * `Ok(SubProjectFiles)` - Complete file structure for this sub-project
    /// * `Err(FsError)` - File system errors during discovery
    ///
    /// # Files Discovered
    /// - Core memory bank files (project_brief.md, product_context.md, etc.)
    /// - tasks/ directory if it exists
    /// - Individual task files within the tasks/ directory
    ///
    /// # Discovery Strategy
    /// - Uses standard file discovery for core memory bank files
    /// - Special handling for tasks directory to find individual task files
    /// - Returns complete sub-project structure with all found files
    fn discover_sub_project_files(sub_project_path: &Path) -> FsResult<SubProjectFiles> {
        let tasks_dir = sub_project_path.join("tasks");

        // Discover task files if the tasks directory exists
        let task_files = if tasks_dir.exists() {
            Self::discover_task_files(&tasks_dir)?
        } else {
            Vec::new() // No task files if tasks directory doesn't exist
        };

        Ok(SubProjectFiles {
            root_path: sub_project_path.to_path_buf(),
            // Discover all standard sub-project files
            project_brief: Self::find_file(sub_project_path, "project_brief.md"),
            product_context: Self::find_file(sub_project_path, "product_context.md"),
            active_context: Self::find_file(sub_project_path, "active_context.md"),
            system_patterns: Self::find_file(sub_project_path, "system_patterns.md"),
            tech_context: Self::find_file(sub_project_path, "tech_context.md"),
            progress: Self::find_file(sub_project_path, "progress.md"),
            // Tasks directory (optional)
            tasks_dir: if tasks_dir.exists() {
                Some(tasks_dir)
            } else {
                None
            },
            task_files,
        })
    }

    /// Discover task files in the tasks directory
    ///
    /// Finds all individual task files within a sub-project's tasks/ directory.
    /// Excludes the _index.md file as it's a special index file, not a task file.
    ///
    /// # Arguments
    /// * `tasks_dir` - Path to the tasks directory
    ///
    /// # Returns
    /// * `Ok(Vec<PathBuf>)` - List of task file paths, sorted by filename
    /// * `Err(FsError)` - File system errors during discovery
    ///
    /// # Discovery Rules
    /// - Only include .md files (Markdown task files)
    /// - Exclude _index.md (special index file, not a task)
    /// - Sort results by filename for consistent ordering
    /// - Return empty vector if no task files found
    fn discover_task_files(tasks_dir: &Path) -> FsResult<Vec<PathBuf>> {
        let mut task_files = Vec::new();

        let entries = std::fs::read_dir(tasks_dir).map_err(crate::utils::fs::FsError::Io)?;

        for entry in entries {
            let entry = entry.map_err(crate::utils::fs::FsError::Io)?;
            let path = entry.path();

            // Only process regular files, not directories
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Include .md files that are not the index file
                    // Task files follow the pattern task_*.md
                    if name.ends_with(".md") && name != "_index.md" {
                        task_files.push(path);
                    }
                }
            }
        }

        // Sort task files by name for consistent ordering across discovery runs
        task_files.sort();
        Ok(task_files)
    }

    /// Find a specific file within a directory
    ///
    /// Safely checks for the existence of a named file within a given directory.
    /// Returns the full path if found, or None if the file doesn't exist or
    /// the path refers to a directory instead of a file.
    ///
    /// # Arguments
    /// * `dir` - Directory to search within
    /// * `filename` - Name of the file to find
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Full path to the file if found and is a regular file
    /// * `None` - File not found, doesn't exist, or is not a regular file
    ///
    /// # Safety
    /// This method validates that the found path is actually a file (not a directory)
    /// before returning it, ensuring type safety for file operations.
    fn find_file(dir: &Path, filename: &str) -> Option<PathBuf> {
        let file_path = dir.join(filename);
        if file_path.exists() && file_path.is_file() {
            Some(file_path)
        } else {
            None
        }
    }

    /// Find a specific directory within a parent directory
    ///
    /// Safely checks for the existence of a named directory within a given parent.
    /// Returns the full path if found, or None if the directory doesn't exist or
    /// the path refers to a file instead of a directory.
    ///
    /// # Arguments
    /// * `dir` - Parent directory to search within
    /// * `dirname` - Name of the directory to find
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Full path to the directory if found and is a directory
    /// * `None` - Directory not found, doesn't exist, or is not a directory
    ///
    /// # Safety
    /// This method validates that the found path is actually a directory (not a file)
    /// before returning it, ensuring type safety for directory operations.
    fn find_directory(dir: &Path, dirname: &str) -> Option<PathBuf> {
        let dir_path = dir.join(dirname);
        if dir_path.exists() && dir_path.is_dir() {
            Some(dir_path)
        } else {
            None
        }
    }

    /// Validate memory bank structure completeness
    ///
    /// Checks for required files and provides diagnostics about what's missing.
    ///
    /// # Arguments
    /// * `structure` - The discovered memory bank structure
    ///
    /// # Returns
    /// A vector of validation messages (warnings about missing files)
    pub fn validate_structure(structure: &MemoryBankStructure) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check workspace files
        if structure.workspace.project_brief.is_none() {
            warnings.push("Missing workspace/project_brief.md".to_string());
        }
        if structure.workspace.shared_patterns.is_none() {
            warnings.push("Missing workspace/shared_patterns.md".to_string());
        }

        // Check current context
        if structure.current_context.is_none() {
            warnings.push("Missing current_context.md".to_string());
        }

        // Check sub-projects
        if structure.sub_projects.is_empty() {
            warnings.push("No sub-projects found in sub_projects/ directory".to_string());
        } else {
            for (name, sub_project) in &structure.sub_projects {
                if sub_project.project_brief.is_none() {
                    warnings.push(format!("Missing {}/project_brief.md", name));
                }
                if sub_project.active_context.is_none() {
                    warnings.push(format!("Missing {}/active_context.md", name));
                }
                if sub_project.progress.is_none() {
                    warnings.push(format!("Missing {}/progress.md", name));
                }
            }
        }

        warnings
    }

    /// Extract the active sub-project name from current_context.md
    ///
    /// Parses the current_context.md file to identify which sub-project is currently
    /// active for development work. This enables context-aware operations and helps
    /// tools know which sub-project to focus on for commands and navigation.
    ///
    /// # Arguments
    /// * `structure` - The discovered memory bank structure containing file paths
    ///
    /// # Returns
    /// * `Ok(Some(String))` - Name of the active sub-project if found and valid
    /// * `Ok(None)` - No current context file or no active project specified
    /// * `Err(FsError)` - File reading errors or permission issues
    ///
    /// # File Format
    /// Expects current_context.md to contain a line in this format:
    /// `**active_sub_project:** project_name`
    ///
    /// # Example
    /// ```markdown
    /// # Current Context
    /// **active_sub_project:** analytics_engine
    /// ```
    pub fn get_active_sub_project(structure: &MemoryBankStructure) -> FsResult<Option<String>> {
        if let Some(current_context_path) = &structure.current_context {
            let content = std::fs::read_to_string(current_context_path)
                .map_err(crate::utils::fs::FsError::Io)?;

            // Parse content line by line to find active sub-project declaration
            for line in content.lines() {
                if line.starts_with("**active_sub_project:**") {
                    // Extract project name after the marker
                    if let Some(remainder) = line.strip_prefix("**active_sub_project:**") {
                        let project_name = remainder.trim();
                        if !project_name.is_empty() {
                            return Ok(Some(project_name.to_string()));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Verify path accessibility and existence
    ///
    /// Safely checks whether a path exists and is accessible for read operations.
    /// This is a fundamental utility for validating file system access before
    /// attempting to read files or traverse directories.
    ///
    /// # Arguments
    /// * `path` - File or directory path to verify
    ///
    /// # Returns
    /// * `Ok(true)` - Path exists and is readable/accessible
    /// * `Ok(false)` - Path does not exist (but no access errors)
    /// * `Err(FsError)` - Permission denied or other file system errors
    ///
    /// # Error Handling
    /// - Permission errors are wrapped in FsError::PermissionDenied
    /// - Other I/O errors are wrapped in FsError::Io
    /// - Non-existence is returned as Ok(false), not an error
    ///
    /// # Use Cases
    /// - Validate paths before attempting file operations
    /// - Check accessibility of memory bank directories
    /// - Diagnose permission issues during discovery
    pub fn path_is_accessible(path: &Path) -> FsResult<bool> {
        match path.try_exists() {
            Ok(exists) => Ok(exists),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(crate::utils::fs::FsError::PermissionDenied {
                        path: path.to_path_buf(),
                    })
                }
                _ => Err(crate::utils::fs::FsError::Io(e)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a test memory bank structure for testing
    fn create_test_memory_bank() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let memory_bank_path = temp_dir.path().join(".copilot").join("memory_bank");

        // Create directory structure
        fs::create_dir_all(&memory_bank_path).unwrap();
        fs::create_dir_all(memory_bank_path.join("workspace")).unwrap();
        fs::create_dir_all(
            memory_bank_path
                .join("sub_projects")
                .join("test-project")
                .join("tasks"),
        )
        .unwrap();

        // Create files
        fs::write(
            memory_bank_path.join("current_context.md"),
            "**active_sub_project:** test-project\n",
        )
        .unwrap();

        fs::write(
            memory_bank_path.join("workspace").join("project_brief.md"),
            "# Workspace Brief\n",
        )
        .unwrap();

        fs::write(
            memory_bank_path
                .join("sub_projects")
                .join("test-project")
                .join("project_brief.md"),
            "# Test Project Brief\n",
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_discover_structure() {
        let temp_dir = create_test_memory_bank();
        let structure = MemoryBankNavigator::discover_structure(temp_dir.path()).unwrap();

        assert!(structure.current_context.is_some());
        assert!(structure.workspace.project_brief.is_some());
        assert_eq!(structure.sub_projects.len(), 1);
        assert!(structure.sub_projects.contains_key("test-project"));
    }

    #[test]
    fn test_get_active_sub_project() {
        let temp_dir = create_test_memory_bank();
        let structure = MemoryBankNavigator::discover_structure(temp_dir.path()).unwrap();

        let active_project = MemoryBankNavigator::get_active_sub_project(&structure).unwrap();
        assert_eq!(active_project, Some("test-project".to_string()));
    }

    #[test]
    fn test_validate_structure() {
        let temp_dir = create_test_memory_bank();
        let structure = MemoryBankNavigator::discover_structure(temp_dir.path()).unwrap();

        let warnings = MemoryBankNavigator::validate_structure(&structure);

        // Should have some warnings for missing files
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("shared_patterns.md")));
    }
}
