//! Code review and quality assurance domain
//!
//! This module contains data structures for managing code review processes,
//! reviewer assignments, feedback tracking, and approval workflows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Code review information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeReviewInfo {
    /// Review status
    pub status: ReviewStatus,

    /// Reviewer assignments
    pub reviewers: Vec<String>,

    /// Review comments or feedback
    pub feedback: Vec<String>,

    /// Files under review
    pub files_reviewed: Vec<String>,

    /// Review completion date
    pub reviewed_at: Option<DateTime<Utc>>,

    /// Approval status
    pub approved: bool,
}

/// Code review status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    /// Not yet submitted for review
    NotSubmitted,
    /// Pending review
    Pending,
    /// Under active review
    InReview,
    /// Changes requested
    ChangesRequested,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
}
