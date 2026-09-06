//! Output formatting for validation and status reports.
use std::io::Write;

use crate::cli::args::OutputFormat;
use crate::constraints::ConstraintOutput;

pub trait OutputFormatter {
    fn format(&self, format: OutputFormat) -> String;
}

pub struct ValidationReporter {
    results: Vec<ConstraintOutput>,
    duration_ms: u64,
    files_checked: usize,
}

impl ValidationReporter {
    pub fn new(results: Vec<ConstraintOutput>, duration_ms: u64, files_checked: usize) -> Self {
        Self {
            results,
            duration_ms,
            files_checked,
        }
    }

    pub fn has_failures(&self) -> bool {
        self.results.iter().any(|r| !r.passed)
    }

    pub fn failure_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.failures.len())
            .sum()
    }

    pub fn print(&self, format: OutputFormat, writer: &mut dyn Write) -> std::io::Result<()> {
        let output = match format {
            OutputFormat::Text => self.format_text(),
            OutputFormat::Json => self.format_json(),
            OutputFormat::Yaml => self.format_yaml(),
            OutputFormat::Advice | OutputFormat::Status => self.format_text(),
        };
        write!(writer, "{}", output)
    }

    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("╔═══════════════════════════════════════════════════════════╗\n");
        output.push_str("║                    Assura Validation Report               ║\n");
        output.push_str("╚═══════════════════════════════════════════════════════════╝\n\n");

        let passed_count = self.results.iter().filter(|r| r.passed).count();
        let failed_count = self.results.len() - passed_count;
        let total_failures = self.failure_count();

        output.push_str(&format!("Files checked: {}\n", self.files_checked));
        output.push_str(&format!("Constraints run: {}\n", self.results.len()));
        output.push_str(&format!("Passed: {}\n", passed_count));
        output.push_str(&format!("Failed: {}\n", failed_count));
        output.push_str(&format!("Total failures: {}\n", total_failures));
        output.push_str(&format!("Duration: {}ms\n", self.duration_ms));
        output.push('\n');

        if failed_count > 0 {
            output.push_str("═══ Failures ═══\n\n");

            for result in self.results.iter().filter(|r| !r.passed) {
                output.push_str(&format!(
                    "❌ {} ({}ms)\n",
                    result.constraint_name, result.duration_ms
                ));
                output.push_str(&format!("   Path: {}\n", result.path.display()));
                output.push_str(&format!("   Severity: {:?}\n", result.severity));

                for failure in result.failures.failures() {
                    output.push_str(&format!("   • {}\n", failure.message));
                    if let Some(suggestion) = &failure.suggestion {
                        output.push_str(&format!("     Suggestion: {}\n", suggestion));
                    }
                }
                output.push('\n');
            }
        }

        if passed_count > 0 {
            output.push_str("═══ Passed ═══\n\n");
            for result in self.results.iter().filter(|r| r.passed) {
                output.push_str(&format!(
                    "✓ {} ({}ms)\n",
                    result.constraint_name, result.duration_ms
                ));
            }
            output.push('\n');
        }

        if total_failures == 0 {
            output.push_str("✅ All validations passed!\n");
        } else {
            output.push_str(&format!(
                "❌ Validation failed with {} error(s)\n",
                total_failures
            ));
        }

        output
    }

    pub fn format_json(&self) -> String {
        let report = serde_json::json!({
            "summary": {
                "files_checked": self.files_checked,
                "constraints_run": self.results.len(),
                "passed": self.results.iter().filter(|r| r.passed).count(),
                "failed": self.results.iter().filter(|r| !r.passed).count(),
                "total_failures": self.failure_count(),
                "duration_ms": self.duration_ms,
                "success": !self.has_failures(),
            },
            "results": self.results.iter().map(|r| {
                serde_json::json!({
                    "constraint": r.constraint_name,
                    "path": r.path,
                    "passed": r.passed,
                    "severity": format!("{:?}", r.severity),
                    "duration_ms": r.duration_ms,
                    "failures": r.failures.failures().iter().map(|f| {
                        serde_json::json!({
                            "constraint": f.constraint,
                            "message": f.message,
                            "suggestion": f.suggestion,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    pub fn format_yaml(&self) -> String {
        let report = serde_json::json!({
            "summary": {
                "files_checked": self.files_checked,
                "constraints_run": self.results.len(),
                "passed": self.results.iter().filter(|r| r.passed).count(),
                "failed": self.results.iter().filter(|r| !r.passed).count(),
                "total_failures": self.failure_count(),
                "duration_ms": self.duration_ms,
                "success": !self.has_failures(),
            },
            "results": self.results.iter().map(|r| {
                serde_json::json!({
                    "constraint": r.constraint_name,
                    "path": r.path,
                    "passed": r.passed,
                    "severity": format!("{:?}", r.severity),
                    "duration_ms": r.duration_ms,
                    "failures": r.failures.failures().iter().map(|f| {
                        serde_json::json!({
                            "constraint": f.constraint,
                            "message": f.message,
                            "suggestion": f.suggestion,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });

        serde_yaml::to_string(&report).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::{Severity, ValidationFailure, ValidationFailures};
    use std::path::PathBuf;

    #[test]
    fn test_validation_reporter_empty() {
        let reporter = ValidationReporter::new(vec![], 0, 0);
        assert!(!reporter.has_failures());
        assert_eq!(reporter.failure_count(), 0);
    }

    #[test]
    fn test_validation_reporter_with_failures() {
        let failures = ValidationFailures::new().with_failure(ValidationFailure::new(
            "test",
            PathBuf::from("/test"),
            "Test failure",
        ));

        let result = ConstraintOutput {
            constraint_name: "test".to_string(),
            path: PathBuf::from("/test"),
            passed: false,
            severity: Severity::High,
            duration_ms: 10,
            failures,
            metadata: None,
        };

        let reporter = ValidationReporter::new(vec![result], 100, 1);
        assert!(reporter.has_failures());
        assert_eq!(reporter.failure_count(), 1);
    }
}
