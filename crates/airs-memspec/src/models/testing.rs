//! Testing and quality assurance domain
//!
//! This module contains data structures for managing test information,
//! performance testing, manual testing checklists, and quality metrics.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Testing information for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestingInfo {
    /// Test coverage percentage
    pub coverage_percentage: Option<f64>,

    /// Test types implemented
    pub test_types: Vec<TestType>,

    /// Test results summary
    pub test_results: TestResults,

    /// Performance test results
    pub performance_results: Option<PerformanceResults>,

    /// Manual testing checklist
    pub manual_testing: Vec<ManualTestItem>,

    /// Testing notes or observations
    pub testing_notes: Vec<String>,
}

/// Types of tests implemented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
    /// End-to-end tests
    EndToEnd,
    /// Performance tests
    Performance,
    /// Security tests
    Security,
    /// Manual tests
    Manual,
}

/// Test execution results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestResults {
    /// Total tests run
    pub total_tests: usize,

    /// Number of passing tests
    pub passed: usize,

    /// Number of failing tests
    pub failed: usize,

    /// Number of skipped tests
    pub skipped: usize,

    /// Test execution time
    pub execution_time: Option<chrono::Duration>,

    /// Last test run timestamp
    pub last_run: Option<DateTime<Utc>>,

    /// Detailed failure information
    pub failures: Vec<TestFailure>,
}

/// Individual test failure details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestFailure {
    /// Test name or identifier
    pub test_name: String,

    /// Failure message
    pub failure_message: String,

    /// Stack trace or error details
    pub error_details: Option<String>,

    /// When the failure occurred
    pub failed_at: DateTime<Utc>,
}

/// Performance test results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceResults {
    /// Response time metrics
    pub response_times: HashMap<String, f64>,

    /// Throughput metrics
    pub throughput: HashMap<String, f64>,

    /// Resource utilization
    pub resource_usage: HashMap<String, f64>,

    /// Benchmark comparisons
    pub benchmarks: Vec<BenchmarkResult>,

    /// Performance regression status
    pub regression_status: PerformanceStatus,
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,

    /// Current value
    pub current_value: f64,

    /// Baseline value for comparison
    pub baseline_value: Option<f64>,

    /// Percentage change from baseline
    pub change_percentage: Option<f64>,

    /// Units of measurement
    pub units: String,

    /// Whether this is better or worse
    pub improvement: Option<bool>,
}

/// Performance regression status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceStatus {
    /// Performance improved
    Improved,
    /// Performance maintained
    Stable,
    /// Minor performance regression
    MinorRegression,
    /// Significant performance regression
    MajorRegression,
    /// No baseline for comparison
    NoBaseline,
}

/// Manual testing checklist item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManualTestItem {
    /// Test item description
    pub description: String,

    /// Test status
    pub status: ManualTestStatus,

    /// Test results or observations
    pub results: Option<String>,

    /// Who performed the test
    pub tested_by: Option<String>,

    /// When the test was performed
    pub tested_at: Option<DateTime<Utc>>,

    /// Any issues found
    pub issues_found: Vec<String>,
}

/// Manual test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ManualTestStatus {
    /// Not yet tested
    NotTested,
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
    /// Test blocked
    Blocked,
}
