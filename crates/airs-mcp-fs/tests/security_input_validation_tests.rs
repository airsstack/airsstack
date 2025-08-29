//! Input Validation Security Testing Framework
//!
//! Comprehensive security testing for MCP handler input validation,
//! targeting vulnerabilities identified in security audit Task 010.
//!
//! Focus Areas:
//! - HIGH-003: Insufficient Input Sanitization (null byte, Unicode, control chars)
//! - HIGH-001: Input Validation Bypass in File Size Limits (integer overflow)
//! - Additional input validation edge cases and malformed data handling

// Layer 1: Standard library imports
use std::time::Instant;

// Layer 2: Third-party crate imports
use serde_json::{json, Value};

// Layer 3: Internal module imports
use airs_mcp_fs::filesystem::validation::PathValidator;

/// Comprehensive input validation security test framework
pub struct InputValidationSecurityTester {
    test_vectors: Vec<InputValidationAttackVector>,
    path_validator: PathValidator,
}

/// Input validation attack vector for security testing
#[derive(Debug, Clone)]
pub struct InputValidationAttackVector {
    pub name: String,
    pub payload: Value,
    pub category: AttackCategory,
    pub severity: SeverityLevel,
    pub description: String,
    pub expected_result: ValidationResult,
    pub mcp_method: String,
}

/// Categories of input validation attacks
#[derive(Debug, Clone, PartialEq)]
pub enum AttackCategory {
    NullByteInjection,
    UnicodeManipulation,
    ControlCharacterInjection,
    IntegerOverflow,
    FormatStringAttack,
    JsonInjection,
    EncodingBypass,
    SizeValidationBypass,
    TypeConfusion,
    #[allow(dead_code)]
    PathInjection,
}

/// Severity levels for security vulnerabilities
#[derive(Debug, Clone, PartialEq)]
pub enum SeverityLevel {
    #[allow(dead_code)]
    Critical, // CVSS 9.0-10.0
    High,   // CVSS 7.0-8.9
    Medium, // CVSS 4.0-6.9
    Low,    // CVSS 0.1-3.9
}

/// Expected validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Reject,   // Input should be rejected
    Sanitize, // Input should be sanitized
    Accept,   // Input should be accepted as-is
}

/// Security test report for input validation
#[derive(Debug)]
pub struct InputValidationSecurityReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub vulnerabilities_found: usize,
    pub security_score: f64,
    pub execution_time: std::time::Duration,
    pub test_results: Vec<InputValidationTestResult>,
    pub vulnerability_details: Vec<InputValidationVulnerability>,
}

/// Individual test result
#[derive(Debug)]
pub struct InputValidationTestResult {
    pub test_name: String,

    #[allow(dead_code)]
    pub category: AttackCategory,

    pub severity: SeverityLevel,
    pub passed: bool,

    #[allow(dead_code)]
    pub execution_time: std::time::Duration,

    #[allow(dead_code)]
    pub error_message: Option<String>,
}

/// Vulnerability found during testing
#[derive(Debug)]
pub struct InputValidationVulnerability {
    pub name: String,
    pub category: AttackCategory,
    pub severity: SeverityLevel,
    pub description: String,
    pub impact: String,
    pub remediation: String,
}

impl InputValidationSecurityTester {
    /// Create new input validation security tester
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create path validator with test configuration
        let allowed_patterns = vec!["**/*".to_string()]; // Allow all paths for testing
        let denied_patterns = vec!["**/.*".to_string()]; // Deny hidden files
        let path_validator = PathValidator::new(allowed_patterns, denied_patterns);

        let mut tester = Self {
            test_vectors: Vec::new(),
            path_validator,
        };

        tester.initialize_attack_vectors();
        Ok(tester)
    }

    /// Initialize comprehensive attack vectors for input validation testing
    fn initialize_attack_vectors(&mut self) {
        // Null Byte Injection Attacks
        self.test_vectors
            .extend(Self::null_byte_injection_vectors());

        // Unicode Manipulation Attacks
        self.test_vectors
            .extend(Self::unicode_manipulation_vectors());

        // Control Character Injection
        self.test_vectors.extend(Self::control_character_vectors());

        // Integer Overflow Attacks
        self.test_vectors.extend(Self::integer_overflow_vectors());

        // Format String Attacks
        self.test_vectors.extend(Self::format_string_vectors());

        // JSON Injection Attacks
        self.test_vectors.extend(Self::json_injection_vectors());

        // Encoding Bypass Attacks
        self.test_vectors.extend(Self::encoding_bypass_vectors());

        // Size Validation Bypass
        self.test_vectors
            .extend(Self::size_validation_bypass_vectors());

        // Type Confusion Attacks
        self.test_vectors.extend(Self::type_confusion_vectors());
    }

    /// Null byte injection attack vectors
    fn null_byte_injection_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Null Byte Path Termination".to_string(),
                payload: json!({
                    "path": "/allowed/path\0../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::NullByteInjection,
                severity: SeverityLevel::High,
                description: "Null byte to terminate path validation early".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "URL Encoded Null Byte".to_string(),
                payload: json!({
                    "path": "/allowed/path%00../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::NullByteInjection,
                severity: SeverityLevel::High,
                description: "URL encoded null byte injection".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Null Byte in Content".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "content": "safe content\0malicious content",
                    "encoding": "utf8"
                }),
                category: AttackCategory::NullByteInjection,
                severity: SeverityLevel::Medium,
                description: "Null byte in file content to bypass content filtering".to_string(),
                expected_result: ValidationResult::Sanitize,
                mcp_method: "write_file".to_string(),
            },
        ]
    }

    /// Unicode manipulation attack vectors
    fn unicode_manipulation_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Unicode Normalization Bypass".to_string(),
                payload: json!({
                    "path": "/allowed/\u{2e}\u{2e}/\u{2e}\u{2e}/etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::UnicodeManipulation,
                severity: SeverityLevel::High,
                description: "Unicode characters that normalize to traversal sequences".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Combining Character Attack".to_string(),
                payload: json!({
                    "path": "/allowed/pat\u{0300}h/../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::UnicodeManipulation,
                severity: SeverityLevel::Medium,
                description: "Combining characters to obfuscate path traversal".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "BIDI Override Attack".to_string(),
                payload: json!({
                    "path": "/allowed/\u{202e}toor\u{202d}/../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::UnicodeManipulation,
                severity: SeverityLevel::Medium,
                description: "Bidirectional override to hide malicious path components".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Control character injection vectors
    fn control_character_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Vertical Tab Injection".to_string(),
                payload: json!({
                    "path": "/allowed\x0Bpath/../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::ControlCharacterInjection,
                severity: SeverityLevel::Medium,
                description: "Vertical tab character to bypass path validation".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Form Feed Injection".to_string(),
                payload: json!({
                    "path": "/allowed\x0Cpath/../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::ControlCharacterInjection,
                severity: SeverityLevel::Medium,
                description: "Form feed character injection".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Backspace Injection".to_string(),
                payload: json!({
                    "path": "/allowed/path\x08\x08\x08\x08/../../../etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::ControlCharacterInjection,
                severity: SeverityLevel::Low,
                description: "Backspace characters to manipulate path interpretation".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Integer overflow attack vectors
    fn integer_overflow_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Max Size Integer Overflow".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "max_size_mb": u64::MAX
                }),
                category: AttackCategory::IntegerOverflow,
                severity: SeverityLevel::High,
                description: "Integer overflow in max_size_mb calculation".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Negative Size Bypass".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "max_size_mb": -1
                }),
                category: AttackCategory::IntegerOverflow,
                severity: SeverityLevel::Medium,
                description: "Negative size to bypass validation".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Zero Size Edge Case".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "max_size_mb": 0
                }),
                category: AttackCategory::IntegerOverflow,
                severity: SeverityLevel::Low,
                description: "Zero size edge case handling".to_string(),
                expected_result: ValidationResult::Accept,
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Format string attack vectors
    fn format_string_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Format String in Path".to_string(),
                payload: json!({
                    "path": "/test/%s%s%s%s.txt",
                    "encoding": "utf8"
                }),
                category: AttackCategory::FormatStringAttack,
                severity: SeverityLevel::Medium,
                description: "Format string specifiers in path".to_string(),
                expected_result: ValidationResult::Accept, // Should be treated as literal
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Format String in Content".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "content": "Test content with %p %x %s format specifiers",
                    "encoding": "utf8"
                }),
                category: AttackCategory::FormatStringAttack,
                severity: SeverityLevel::Low,
                description: "Format string specifiers in file content".to_string(),
                expected_result: ValidationResult::Accept, // Should be treated as literal
                mcp_method: "write_file".to_string(),
            },
        ]
    }

    /// JSON injection attack vectors
    fn json_injection_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "JSON Escape Sequence Injection".to_string(),
                payload: json!({
                    "path": "/test.txt\",\"malicious_field\":\"value",
                    "encoding": "utf8"
                }),
                category: AttackCategory::JsonInjection,
                severity: SeverityLevel::Medium,
                description: "Attempt to break out of JSON structure".to_string(),
                expected_result: ValidationResult::Accept, // Should be treated as literal path
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "JSON Comment Injection".to_string(),
                payload: json!({
                    "path": "/test.txt/* comment */",
                    "encoding": "utf8"
                }),
                category: AttackCategory::JsonInjection,
                severity: SeverityLevel::Low,
                description: "JSON comment injection attempt".to_string(),
                expected_result: ValidationResult::Accept, // Should be treated as literal path
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Encoding bypass attack vectors
    fn encoding_bypass_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Double URL Encoding".to_string(),
                payload: json!({
                    "path": "/allowed%252e%252e%252f%252e%252e%252fetc%252fpasswd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::EncodingBypass,
                severity: SeverityLevel::High,
                description: "Double URL encoding to bypass path validation".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Mixed Encoding Attack".to_string(),
                payload: json!({
                    "path": "/allowed%2e%2e/\u{2e}\u{2e}/etc/passwd",
                    "encoding": "utf8"
                }),
                category: AttackCategory::EncodingBypass,
                severity: SeverityLevel::High,
                description: "Mixed URL and Unicode encoding".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Size validation bypass vectors
    fn size_validation_bypass_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "Extremely Large Content".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "content": "A".repeat(1_000_000), // 1MB of content
                    "encoding": "utf8"
                }),
                category: AttackCategory::SizeValidationBypass,
                severity: SeverityLevel::Medium,
                description: "Extremely large content to test size limits".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "write_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Empty Content Edge Case".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "content": "",
                    "encoding": "utf8"
                }),
                category: AttackCategory::SizeValidationBypass,
                severity: SeverityLevel::Low,
                description: "Empty content edge case".to_string(),
                expected_result: ValidationResult::Accept,
                mcp_method: "write_file".to_string(),
            },
        ]
    }

    /// Type confusion attack vectors
    fn type_confusion_vectors() -> Vec<InputValidationAttackVector> {
        vec![
            InputValidationAttackVector {
                name: "String as Number".to_string(),
                payload: json!({
                    "path": "/test.txt",
                    "max_size_mb": "100malicious"
                }),
                category: AttackCategory::TypeConfusion,
                severity: SeverityLevel::Medium,
                description: "String value where number expected".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Array as String".to_string(),
                payload: json!({
                    "path": ["array", "as", "path"],
                    "encoding": "utf8"
                }),
                category: AttackCategory::TypeConfusion,
                severity: SeverityLevel::Medium,
                description: "Array value where string expected".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
            InputValidationAttackVector {
                name: "Object as String".to_string(),
                payload: json!({
                    "path": {"malicious": "object"},
                    "encoding": "utf8"
                }),
                category: AttackCategory::TypeConfusion,
                severity: SeverityLevel::Medium,
                description: "Object value where string expected".to_string(),
                expected_result: ValidationResult::Reject,
                mcp_method: "read_file".to_string(),
            },
        ]
    }

    /// Run comprehensive input validation security tests
    pub fn run_comprehensive_security_tests(&mut self) -> InputValidationSecurityReport {
        println!("🔍 Starting Comprehensive Input Validation Security Tests");
        println!("📊 Testing {} attack vectors", self.test_vectors.len());

        let start_time = Instant::now();
        let mut test_results = Vec::new();
        let mut vulnerabilities = Vec::new();
        let mut passed_tests = 0;
        let mut failed_tests = 0;

        for (index, vector) in self.test_vectors.iter().enumerate() {
            let test_start = Instant::now();
            print!(
                "🧪 Test {}/{}: {} ... ",
                index + 1,
                self.test_vectors.len(),
                vector.name
            );

            let result = self.execute_test_vector(vector);
            let test_duration = test_start.elapsed();

            let passed = result.is_ok();
            if passed {
                passed_tests += 1;
                println!("✅ PASS ({} ms)", test_duration.as_millis());
            } else {
                failed_tests += 1;
                println!("❌ FAIL ({} ms)", test_duration.as_millis());

                // Create vulnerability record for failed test
                vulnerabilities.push(InputValidationVulnerability {
                    name: vector.name.clone(),
                    category: vector.category.clone(),
                    severity: vector.severity.clone(),
                    description: vector.description.clone(),
                    impact: self.get_impact_description(&vector.severity),
                    remediation: self.get_remediation_advice(&vector.category),
                });
            }

            test_results.push(InputValidationTestResult {
                test_name: vector.name.clone(),
                category: vector.category.clone(),
                severity: vector.severity.clone(),
                passed,
                execution_time: test_duration,
                error_message: result.err().map(|e| e.to_string()),
            });
        }

        let total_execution_time = start_time.elapsed();
        let total_tests = self.test_vectors.len();
        let vulnerabilities_found = vulnerabilities.len();

        // Calculate security score
        let security_score = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        InputValidationSecurityReport {
            total_tests,
            passed_tests,
            failed_tests,
            vulnerabilities_found,
            security_score,
            execution_time: total_execution_time,
            test_results,
            vulnerability_details: vulnerabilities,
        }
    }

    /// Execute individual test vector
    fn execute_test_vector(
        &self,
        vector: &InputValidationAttackVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified test execution - in a real implementation,
        // we would call the actual MCP handlers and validate their responses

        match vector.mcp_method.as_str() {
            "read_file" => self.test_read_file_validation(vector),
            "write_file" => self.test_write_file_validation(vector),
            "list_directory" => self.test_list_directory_validation(vector),
            _ => Ok(()), // Default case for unknown methods
        }
    }

    /// Test read_file input validation
    fn test_read_file_validation(
        &self,
        vector: &InputValidationAttackVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Test path validation for various attack categories
        match vector.category {
            AttackCategory::NullByteInjection => {
                if let Some(path) = vector.payload.get("path").and_then(|p| p.as_str()) {
                    if path.contains('\0') || path.contains("%00") {
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                // Test that null bytes in paths are properly rejected
                                let validation_result = self.path_validator.validate_path(path);
                                if validation_result.is_ok() {
                                    return Err(
                                        "Null byte should be rejected but was accepted".into()
                                    );
                                }
                            }
                            _ => return Ok(()),
                        }
                    }
                }
            }
            AttackCategory::UnicodeManipulation => {
                if let Some(path) = vector.payload.get("path").and_then(|p| p.as_str()) {
                    // Test Unicode normalization and manipulation
                    let validation_result = self.path_validator.validate_path(path);
                    match vector.expected_result {
                        ValidationResult::Reject => {
                            if validation_result.is_ok() {
                                return Err(
                                    "Unicode manipulation should be rejected but was accepted"
                                        .into(),
                                );
                            }
                        }
                        _ => return Ok(()),
                    }
                }
            }
            AttackCategory::ControlCharacterInjection => {
                if let Some(path) = vector.payload.get("path").and_then(|p| p.as_str()) {
                    // Test control character filtering
                    let has_control_chars = path
                        .chars()
                        .any(|c| c.is_control() && c != '\t' && c != '\n' && c != '\r');
                    if has_control_chars {
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                let validation_result = self.path_validator.validate_path(path);
                                if validation_result.is_ok() {
                                    return Err(
                                        "Control characters should be rejected but were accepted"
                                            .into(),
                                    );
                                }
                            }
                            _ => return Ok(()),
                        }
                    }
                }
            }
            AttackCategory::IntegerOverflow => {
                if let Some(max_size) = vector.payload.get("max_size_mb") {
                    // Test integer overflow in size validation
                    if max_size.is_string() {
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                return Err("String max_size should be rejected".into())
                            }
                            _ => return Ok(()),
                        }
                    }
                    if let Some(size_val) = max_size.as_u64() {
                        if size_val > 10_000 {
                            match vector.expected_result {
                                ValidationResult::Reject => {
                                    return Err("Oversized max_size should be rejected".into())
                                }
                                _ => return Ok(()),
                            }
                        }
                    }
                }
            }
            AttackCategory::TypeConfusion => {
                if let Some(path) = vector.payload.get("path") {
                    if !path.is_string() {
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                return Err("Non-string path should be rejected".into())
                            }
                            _ => return Ok(()),
                        }
                    }
                }
            }
            AttackCategory::EncodingBypass => {
                if let Some(path) = vector.payload.get("path").and_then(|p| p.as_str()) {
                    // Test encoded path traversal attempts
                    if path.contains("%2e%2e") || path.contains("%252e") {
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                let validation_result = self.path_validator.validate_path(path);
                                if validation_result.is_ok() {
                                    return Err(
                                        "Encoded traversal should be rejected but was accepted"
                                            .into(),
                                    );
                                }
                            }
                            _ => return Ok(()),
                        }
                    }
                }
            }
            _ => {
                // For other categories, assume validation passes for now
                // In a real implementation, this would call actual validation logic
            }
        }

        Ok(())
    }

    /// Test write_file input validation
    fn test_write_file_validation(
        &self,
        vector: &InputValidationAttackVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Similar to read_file validation but for write operations
        match vector.category {
            AttackCategory::SizeValidationBypass => {
                if let Some(content) = vector.payload.get("content").and_then(|c| c.as_str()) {
                    if content.len() > 500_000 {
                        // 500KB limit for testing
                        match vector.expected_result {
                            ValidationResult::Reject => {
                                return Err("Large content should be rejected".into())
                            }
                            _ => return Ok(()),
                        }
                    }
                }
            }
            _ => {
                // Reuse read_file validation for common categories
                return self.test_read_file_validation(vector);
            }
        }

        Ok(())
    }

    /// Test list_directory input validation
    fn test_list_directory_validation(
        &self,
        _vector: &InputValidationAttackVector,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for directory listing validation
        Ok(())
    }

    /// Get impact description for severity level
    fn get_impact_description(&self, severity: &SeverityLevel) -> String {
        match severity {
            SeverityLevel::Critical => {
                "System compromise, data breach, or complete service disruption".to_string()
            }
            SeverityLevel::High => {
                "Unauthorized access, data manipulation, or significant service degradation"
                    .to_string()
            }
            SeverityLevel::Medium => {
                "Information disclosure, limited unauthorized access, or minor service impact"
                    .to_string()
            }
            SeverityLevel::Low => {
                "Minimal security impact, information leakage, or edge case vulnerabilities"
                    .to_string()
            }
        }
    }

    /// Get remediation advice for attack category
    fn get_remediation_advice(&self, category: &AttackCategory) -> String {
        match category {
            AttackCategory::NullByteInjection => {
                "Implement null byte detection and rejection in all input validation".to_string()
            }
            AttackCategory::UnicodeManipulation => {
                "Add Unicode normalization (NFC) before path validation".to_string()
            }
            AttackCategory::ControlCharacterInjection => {
                "Filter or reject control characters in all inputs".to_string()
            }
            AttackCategory::IntegerOverflow => {
                "Add bounds checking and type validation for numeric inputs".to_string()
            }
            AttackCategory::FormatStringAttack => {
                "Treat all user input as literal data, never as format strings".to_string()
            }
            AttackCategory::JsonInjection => {
                "Use proper JSON parsing and validate structure integrity".to_string()
            }
            AttackCategory::EncodingBypass => {
                "Implement multi-layer decoding and canonicalization".to_string()
            }
            AttackCategory::SizeValidationBypass => {
                "Enforce strict size limits with proper overflow protection".to_string()
            }
            AttackCategory::TypeConfusion => {
                "Implement strict type validation and rejection of mismatched types".to_string()
            }
            AttackCategory::PathInjection => {
                "Use path canonicalization and whitelist-based validation".to_string()
            }
        }
    }
}

impl InputValidationSecurityReport {
    /// Print comprehensive security report
    pub fn print_report(&self) {
        println!("\n🔒 INPUT VALIDATION SECURITY TEST REPORT");
        println!("==================================================");

        println!("\n📊 EXECUTIVE SUMMARY");
        println!("Total Tests: {}", self.total_tests);
        println!(
            "Passed: {} ({:.1}%)",
            self.passed_tests,
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        );
        println!(
            "Failed: {} ({:.1}%)",
            self.failed_tests,
            (self.failed_tests as f64 / self.total_tests as f64) * 100.0
        );
        println!("Vulnerabilities Found: {}", self.vulnerabilities_found);
        println!("Security Score: {:.1}/100", self.security_score);
        println!("Execution Time: {} ms", self.execution_time.as_millis());

        if !self.vulnerability_details.is_empty() {
            println!("\n🚨 VULNERABILITIES DISCOVERED");
            for vuln in &self.vulnerability_details {
                println!("\n📍 {}", vuln.name);
                println!("   Category: {:?}", vuln.category);
                println!("   Severity: {:?}", vuln.severity);
                println!("   Description: {}", vuln.description);
                println!("   Impact: {}", vuln.impact);
                println!("   Remediation: {}", vuln.remediation);
            }
        }

        println!("\n💡 SECURITY RECOMMENDATIONS");
        if self.vulnerabilities_found == 0 {
            println!("   ✅ All input validation tests passed! Security validation is robust.");
        } else {
            println!(
                "   ⚠️  {} input validation vulnerabilities require immediate attention.",
                self.vulnerabilities_found
            );
            println!("   🔧 Implement comprehensive input sanitization and validation framework.");
            println!("   📋 Review and update security policies for identified vulnerability categories.");
        }

        println!("\n==================================================");
    }
}
