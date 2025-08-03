// Data models module
// Defines core data structures for memory bank, tasks, and instructions

pub mod instruction;
pub mod task;

// Domain-specific memory bank modules
pub mod monitoring;
pub mod progress;
pub mod review;
pub mod sub_project;
pub mod system;
pub mod task_management;
pub mod tech;
pub mod testing;
pub mod types;
pub mod workspace;

// Re-export commonly used types for convenience
pub use sub_project::{ActiveContext, ProductContext, SubProject};
pub use task_management::{Task, TaskCollection, TaskProgressSummary};
pub use types::{Priority, ProgressStatus, ProjectStatus, TaskStatus};
pub use workspace::{ContextSnapshot, Workspace, WorkspaceMetadata};
