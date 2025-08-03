//! Shared types and enumerations used across the memory bank domain
//!
//! This module contains common types, enumerations, and utilities that are
//! shared across multiple domain modules in the memory bank system.

use serde::{Deserialize, Serialize};

/// Priority enumeration for tasks, work items, and other prioritizable entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Progress status enumeration for tracking overall progress state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    OnTrack,
    AtRisk,
    Blocked,
    Completed,
}

/// Issue severity levels for categorizing problems and bugs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Enhancement,
}

/// Task status enumeration for task lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Abandoned,
    Blocked,
}

/// Subtask status enumeration for granular task tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubtaskStatus {
    /// Not yet started
    NotStarted,
    /// Currently in progress
    InProgress,
    /// Completed successfully
    Complete,
    /// Blocked by dependencies or issues
    Blocked,
    /// Skipped or deemed unnecessary
    Skipped,
}

/// Progress log entry types for categorizing different kinds of updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgressLogType {
    /// Regular progress update
    ProgressUpdate,
    /// Major milestone reached
    Milestone,
    /// Issue or blocker encountered
    Issue,
    /// Issue resolved
    Resolution,
    /// Status change
    StatusChange,
    /// Plan modification
    PlanUpdate,
    /// Decision made
    Decision,
    /// Task completion
    Completion,
}

/// Sub-project status enumeration for project lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    /// Project is actively being worked on
    Active,
    /// Project is temporarily paused
    Paused,
    /// Project has been completed
    Completed,
    /// Project has been archived
    Archived,
    /// Project is in planning phase
    Planning,
}
