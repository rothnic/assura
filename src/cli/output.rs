use std::io::Write;

use crate::cli::args::OutputFormat;
use crate::constraints::{ConstraintOutput, ValidationFailures};
use crate::maturity::{MaturityLevel, MaturityReport};

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

pub struct StatusReporter {
    maturity_report: MaturityReport,
    active_constraints: Vec<String>,
    last_validation: Option<std::time::SystemTime>,
}

impl StatusReporter {
    pub fn new(
        maturity_report: MaturityReport,
        active_constraints: Vec<String>,
        last_validation: Option<std::time::SystemTime>,
    ) -> Self {
        Self {
            maturity_report,
            active_constraints,
            last_validation,
        }
    }

    pub fn print(&self, format: OutputFormat, writer: &mut dyn Write) -> std::io::Result<()> {
        let output = match format {
            OutputFormat::Text => self.format_text(),
            OutputFormat::Json => self.format_json(),
            OutputFormat::Yaml => self.format_yaml(),
        };
        write!(writer, "{}", output)
    }

    pub fn format_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str("╔═══════════════════════════════════════════════════════════╗\n");
        output.push_str("║                    Assura Project Status                  ║\n");
        output.push_str("╚═══════════════════════════════════════════════════════════╝\n\n");

        let level_emoji = match self.maturity_report.level {
            MaturityLevel::Established => "🏆",
            MaturityLevel::Mature => "✨",
            MaturityLevel::Developing => "🌱",
            MaturityLevel::Raw => "🔧",
        };

        output.push_str(&format!(
            "Maturity Level: {} {}\n",
            level_emoji, self.maturity_report.level
        ));
        output.push_str(&format!(
            "Score: {:.1}%\n",
            self.maturity_report.score * 100.0
        ));
        output.push_str(&format!(
            "Confidence: {:.1}%\n",
            self.maturity_report.confidence * 100.0
        ));
        output.push('\n');

        output.push_str(&format!(
            "Active Constraints: {}\n",
            self.active_constraints.len()
        ));
        for constraint in &self.active_constraints {
            output.push_str(&format!("  • {}\n", constraint));
        }
        output.push('\n');

        if let Some(last_run) = self.last_validation {
            let elapsed = std::time::SystemTime::now()
                .duration_since(last_run)
                .unwrap_or_default();
            output.push_str(&format!(
                "Last validation: {} ago\n",
                format_duration(elapsed)
            ));
        } else {
            output.push_str("Last validation: Never\n");
        }

        if !self.maturity_report.recommendations.is_empty() {
            output.push_str("\n═══ Recommendations ═══\n");
            for rec in &self.maturity_report.recommendations {
                output.push_str(&format!("  💡 {}\n", rec.message));
            }
        }

        output
    }

    pub fn format_json(&self) -> String {
        let report = serde_json::json!({
            "maturity": {
                "level": format!("{:?}", self.maturity_report.level),
                "score": self.maturity_report.score,
                "confidence": self.maturity_report.confidence,
                "assessed_at": self.maturity_report.assessed_at,
            },
            "constraints": {
                "active": self.active_constraints,
                "count": self.active_constraints.len(),
            },
            "last_validation": self.last_validation.map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
            "recommendations": self.maturity_report.recommendations,
        });
        
        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    pub fn format_yaml(&self) -> String {
        let report = serde_json::json!({
            "maturity": {
                "level": format!("{:?}", self.maturity_report.level),
                "score": self.maturity_report.score,
                "confidence": self.maturity_report.confidence,
                "assessed_at": self.maturity_report.assessed_at,
            },
            "constraints": {
                "active": self.active_constraints,
                "count": self.active_constraints.len(),
            },
            "last_validation": self.last_validation.map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
            "recommendations": self.maturity_report.recommendations,
        });
        
        serde_yaml::to_string(&report).unwrap_or_default()
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::{Severity, ValidationFailure};
    use std::path::PathBuf;

    #[test]
    fn test_validation_reporter_empty() {
        let reporter = ValidationReporter::new(vec![], 0, 0);
        assert!(!reporter.has_failures());
        assert_eq!(reporter.failure_count(), 0);
    }

    #[test]
    fn test_validation_reporter_with_failures() {
        let failures = ValidationFailures::new().with_failure(
            ValidationFailure::new(
                "test",
                PathBuf::from("/test"),
                "Test failure"
            )
        );
        
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

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(120)), "2m");
        assert_eq!(format_duration(std::time::Duration::from_secs(7200)), "2h");
        assert_eq!(format_duration(std::time::Duration::from_secs(172800)), "2d");
    }
}
