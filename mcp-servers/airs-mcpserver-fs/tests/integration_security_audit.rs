//! Integration test runner for comprehensive security testing
//!
//! This test runs multiple security test suites and generates detailed reports.

#![allow(clippy::assertions_on_constants)]

// Import test modules from individual test files
mod security_input_validation_tests;
mod security_path_traversal_tests;

use security_input_validation_tests::{
    InputValidationSecurityReport, InputValidationSecurityTester,
};
use security_path_traversal_tests::{PathTraversalSecurityTester, SecurityTestReport};

#[tokio::test]
async fn run_comprehensive_path_traversal_security_audit() -> Result<(), Box<dyn std::error::Error>>
{
    println!("🔍 INITIATING COMPREHENSIVE PATH TRAVERSAL SECURITY AUDIT");
    println!("📋 Task 010, Subtask 10.2: Path Traversal Vulnerability Testing");
    println!("🎯 Objective: Validate path validation robustness against 50+ attack vectors\n");

    // Create security tester
    let mut tester = PathTraversalSecurityTester::new()?;

    // Run comprehensive security tests and measure execution time
    let start_time = std::time::Instant::now();
    let report = tester.run_comprehensive_security_tests();
    let _total_execution_time = start_time.elapsed();

    // Print detailed security report
    report.print_report();

    // Security assertions for CI/CD pipeline
    assert_security_requirements(&report);

    // Generate markdown report (for future integration with documentation systems)
    let _markdown_report = generate_markdown_report(&report);

    Ok(())
}

/// Assert security requirements for CI/CD pipeline
fn assert_security_requirements(report: &SecurityTestReport) {
    // Critical security requirements
    assert_eq!(
        report.vulnerabilities_found, 0,
        "SECURITY FAILURE: {} path traversal vulnerabilities found",
        report.vulnerabilities_found
    );

    assert!(
        report.security_score >= 95.0,
        "SECURITY FAILURE: Security score {:.1} below required 95.0",
        report.security_score
    );

    assert!(
        report.passed_tests >= (report.total_tests * 95) / 100,
        "SECURITY FAILURE: Only {}/{} tests passed ({}%)",
        report.passed_tests,
        report.total_tests,
        (report.passed_tests * 100) / report.total_tests
    );
}

/// Generate security documentation for audit trail
#[allow(dead_code)] // Reserved for future audit trail integration
fn generate_security_documentation(report: &SecurityTestReport) {
    println!("\n📝 GENERATING SECURITY AUDIT DOCUMENTATION");

    // Create markdown report
    let _markdown_report = generate_markdown_report(report);

    // In a real implementation, this would write to file
    println!("📄 Security test report generated:");
    println!("   - Total attack vectors tested: {}", report.total_tests);
    println!("   - Security score: {:.1}/100", report.security_score);
    println!(
        "   - Vulnerabilities found: {}",
        report.vulnerabilities_found
    );
    println!(
        "   - Test execution time: {} ms",
        report.total_execution_time_ms
    );

    if report.vulnerabilities_found > 0 {
        println!("\n🚨 CRITICAL: Security vulnerabilities detected!");
        println!("📋 See recommendations in test output above");
    } else {
        println!("\n✅ SUCCESS: All path traversal security tests passed!");
    }
}

/// Generate markdown security report
fn generate_markdown_report(report: &SecurityTestReport) -> String {
    let mut markdown = String::new();

    markdown.push_str("# Path Traversal Security Test Report\n\n");
    markdown.push_str(&format!(
        "**Generated:** {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    markdown.push_str("**Test Suite:** Comprehensive Path Traversal Vulnerability Testing\n");
    markdown.push_str("**Task Reference:** Task 010, Subtask 10.2\n\n");

    markdown.push_str("## Executive Summary\n\n");
    markdown.push_str(&format!("- **Total Tests:** {}\n", report.total_tests));
    markdown.push_str(&format!(
        "- **Passed:** {} ({}%)\n",
        report.passed_tests,
        (report.passed_tests * 100) / report.total_tests
    ));
    markdown.push_str(&format!(
        "- **Failed:** {} ({}%)\n",
        report.failed_tests,
        (report.failed_tests * 100) / report.total_tests
    ));
    markdown.push_str(&format!(
        "- **Vulnerabilities Found:** {}\n",
        report.vulnerabilities_found
    ));
    markdown.push_str(&format!(
        "- **Security Score:** {:.1}/100\n",
        report.security_score
    ));
    markdown.push_str(&format!(
        "- **Execution Time:** {} ms\n\n",
        report.total_execution_time_ms
    ));

    if report.vulnerabilities_found > 0 {
        markdown.push_str("## 🚨 Vulnerabilities Discovered\n\n");
        for result in &report.results {
            if !result.test_passed && result.attack_vector.should_be_rejected {
                markdown.push_str(&format!("### {}\n", result.attack_vector.name));
                markdown.push_str(&format!(
                    "- **Path:** `{}`\n",
                    result.attack_vector.malicious_path
                ));
                markdown.push_str(&format!(
                    "- **Severity:** {:?}\n",
                    result.attack_vector.severity
                ));
                markdown.push_str(&format!(
                    "- **Category:** {:?}\n",
                    result.attack_vector.category
                ));
                markdown.push_str(&format!(
                    "- **Description:** {}\n",
                    result.attack_vector.description
                ));
                if let Some(ref error) = result.error_message {
                    markdown.push_str(&format!("- **Error:** {error}\n"));
                }
                markdown.push('\n');
            }
        }
    }

    markdown.push_str("## Recommendations\n\n");
    for rec in &report.recommendations {
        markdown.push_str(&format!("- {rec}\n"));
    }

    markdown
}

#[tokio::test]
async fn test_specific_critical_vectors() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 TESTING SPECIFIC CRITICAL ATTACK VECTORS");

    let tester = PathTraversalSecurityTester::new()?;

    // Test the most critical vectors that were identified in manual review
    let critical_vectors = vec![
        (
            "URL Encoded Traversal",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        ),
        ("Mixed Encoding", "..%2f..%2f..%2fetc%2fpasswd"),
        ("Unicode Fullwidth", "％２ｅ％２ｅ％２ｆetc％２ｆpasswd"),
        ("Double Encoding", "%252e%252e%252f%252e%252e%252f"),
        ("Null Byte Injection", "../../../etc/passwd\0.jpg"),
    ];

    for (name, path) in critical_vectors {
        println!("🧪 Testing {name}: {path}");
        let result = tester.validator().validate_path(path);

        // These should ALL be rejected for security
        assert!(
            result.is_err(),
            "CRITICAL SECURITY FAILURE: {name} was not rejected: {path}"
        );
        println!("✅ {name} correctly rejected");
    }

    println!("✅ All critical attack vectors properly rejected");

    Ok(())
}

#[tokio::test]
async fn test_legitimate_paths_still_work() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ TESTING LEGITIMATE PATH ACCESS");

    let tester = PathTraversalSecurityTester::new()?;

    // Test legitimate paths that should be allowed
    let legitimate_paths = vec![
        "allowed/document.txt",
        "public/image.jpg",
        "public/subdir/file.pdf",
        "allowed/nested/deep/file.md",
    ];

    for path in legitimate_paths {
        println!("🧪 Testing legitimate path: {path}");
        let result = tester.validator().validate_path(path);

        // These should be allowed (though they may fail for file not found, that's OK)
        // We're testing that they pass path validation, not file existence
        match result {
            Ok(_) => println!("✅ {path} correctly allowed"),
            Err(e) => {
                // Check if it's a path validation error vs file not found
                let error_msg = e.to_string();
                if error_msg.contains("Path traversal")
                    || error_msg.contains("not in allowed list")
                    || error_msg.contains("denied by security policy")
                {
                    assert!(
                        false,
                        "SECURITY ERROR: Legitimate path {path} was incorrectly rejected: {error_msg}"
                    );
                } else {
                    // Other errors (like file not found) are acceptable for this test
                    println!("✅ {path} passed path validation ({error_msg})");
                }
            }
        }
    }

    println!("✅ All legitimate paths properly handled");

    Ok(())
}

#[tokio::test]
async fn run_comprehensive_input_validation_security_audit(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 INITIATING COMPREHENSIVE INPUT VALIDATION SECURITY AUDIT");
    println!("📋 Task 010, Subtask 10.3: Input Validation Security Testing");
    println!("🎯 Objective: Validate input sanitization against injection and bypass attacks\n");

    // Create input validation security tester
    let mut tester = InputValidationSecurityTester::new()?;

    // Run comprehensive input validation security tests
    let start_time = std::time::Instant::now();
    let report = tester.run_comprehensive_security_tests();
    let _total_execution_time = start_time.elapsed();

    // Print detailed security report
    report.print_report();

    // Security assertions for CI/CD pipeline
    assert_input_validation_security_requirements(&report);

    // Generate markdown report (for future integration with documentation systems)
    let _markdown_report = generate_input_validation_markdown_report(&report);

    Ok(())
}

/// Assert input validation security requirements for CI/CD pipeline
fn assert_input_validation_security_requirements(report: &InputValidationSecurityReport) {
    // Critical security requirements
    assert_eq!(
        report.vulnerabilities_found, 0,
        "SECURITY FAILURE: {} input validation vulnerabilities found",
        report.vulnerabilities_found
    );

    assert!(
        report.security_score >= 85.0,
        "SECURITY FAILURE: Input validation security score {:.1} below required 85.0",
        report.security_score
    );

    // Ensure all critical and high severity tests passed
    let critical_high_failures: Vec<_> = report
        .test_results
        .iter()
        .filter(|result| {
            !result.passed
                && matches!(
                    result.severity,
                    security_input_validation_tests::SeverityLevel::Critical
                        | security_input_validation_tests::SeverityLevel::High
                )
        })
        .collect();

    assert!(
        critical_high_failures.is_empty(),
        "SECURITY FAILURE: Critical/High severity input validation tests failed: {:?}",
        critical_high_failures
            .iter()
            .map(|r| &r.test_name)
            .collect::<Vec<_>>()
    );
}

/// Generate markdown report for input validation security testing
fn generate_input_validation_markdown_report(report: &InputValidationSecurityReport) -> String {
    format!(
        r#"# Input Validation Security Test Report

## Executive Summary
- **Total Tests**: {}
- **Passed**: {} ({:.1}%)
- **Failed**: {} ({:.1}%)
- **Vulnerabilities Found**: {}
- **Security Score**: {:.1}/100
- **Execution Time**: {} ms

## Test Results
{}

## Recommendations
{}
"#,
        report.total_tests,
        report.passed_tests,
        (report.passed_tests as f64 / report.total_tests as f64) * 100.0,
        report.failed_tests,
        (report.failed_tests as f64 / report.total_tests as f64) * 100.0,
        report.vulnerabilities_found,
        report.security_score,
        report.execution_time.as_millis(),
        if report.vulnerabilities_found == 0 {
            "All input validation tests passed successfully."
        } else {
            "Input validation vulnerabilities detected. Immediate remediation required."
        },
        if report.vulnerabilities_found == 0 {
            "✅ Input validation security is robust and production-ready."
        } else {
            "⚠️ Implement comprehensive input sanitization framework before production deployment."
        }
    )
}
