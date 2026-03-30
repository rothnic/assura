use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

use super::{
    signal::{MaturitySignal, SignalCollector, SignalType},
    MaturityResult,
};

/// Collector for file system-based maturity signals
pub struct FilesystemSignals;

impl FilesystemSignals {
    pub fn new() -> Self {
        Self
    }

    fn count_files(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in WalkDir::new(path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                file_count += 1;
            } else if entry.file_type().is_dir() {
                dir_count += 1;
            }
        }

        // Normalize file count: <10 files = 0.0, 100+ files = 1.0
        let file_normalized = (file_count as f64 / 100.0).min(1.0);

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "file_count",
                file_normalized,
                format!("{} files, {} directories", file_count, dir_count),
            )
            .with_confidence(1.0)
            .with_weight(0.8),
        )
    }

    fn measure_directory_depth(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let mut max_depth = 0;
        let base_depth = path.components().count();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                let depth = entry.path().components().count() - base_depth;
                max_depth = max_depth.max(depth);
            }
        }

        // Normalize: depth 0-3 = 0.0, depth 8+ = 1.0
        let normalized = ((max_depth as f64 - 3.0) / 5.0).clamp(0.0, 1.0);

        let depth_desc = match max_depth {
            0..=2 => "flat structure",
            3..=5 => "moderate nesting",
            6..=8 => "deep nesting",
            _ => "very deep nesting",
        };

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "directory_depth",
                normalized,
                format!("max depth {} ({})", max_depth, depth_desc),
            )
            .with_confidence(1.0)
            .with_weight(0.6),
        )
    }

    fn detect_config_files(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let config_patterns: HashSet<&str> = [
            // Rust
            "Cargo.toml",
            "Cargo.lock",
            // JavaScript/Node
            "package.json",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            // Python
            "pyproject.toml",
            "setup.py",
            "requirements.txt",
            "Pipfile",
            // Java
            "pom.xml",
            "build.gradle",
            // Go
            "go.mod",
            "go.sum",
            // Ruby
            "Gemfile",
            // General
            "Makefile",
            "justfile",
            "Dockerfile",
            "docker-compose.yml",
            ".gitignore",
        ]
        .iter()
        .cloned()
        .collect();

        let mut detected_configs = Vec::new();

        for pattern in &config_patterns {
            if path.join(pattern).exists() {
                detected_configs.push(pattern.to_string());
            }
        }

        let config_count = detected_configs.len();

        // Score based on count and quality of configs
        let score = config_count as f64 * 0.2 + 
            if detected_configs.contains(&".gitignore".to_string()) { 0.2 } else { 0.0 } +
            if detected_configs.iter().any(|c| c.contains("Dockerfile")) { 0.2 } else { 0.0 } +
            if detected_configs.iter().any(|c| c.contains("lock") || c.contains("sum")) { 0.2 } else { 0.0 };

        let normalized = score.min(1.0);

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "config_files",
                normalized,
                if detected_configs.is_empty() {
                    "no config files detected".to_string()
                } else {
                    format!("{} configs: {}", config_count, detected_configs.join(", "))
                },
            )
            .with_confidence(1.0)
            .with_weight(1.0),
        )
    }

    fn detect_test_coverage(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let test_indicators: Vec<(&str, Box<dyn Fn(&Path) -> bool + Send + Sync>)> = vec![
            ("tests/", Box::new(|p| p.join("tests").is_dir())),
            ("test/", Box::new(|p| p.join("test").is_dir())),
            ("__tests__/", Box::new(|p| {
                p.read_dir()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| e.file_name() == "__tests__")
                    })
                    .unwrap_or(false)
            })),
            ("*.spec.*", Box::new(|p| {
                WalkDir::new(p)
                    .max_depth(5)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.contains(".spec."))
                            .unwrap_or(false)
                    })
            })),
            ("*.test.*", Box::new(|p| {
                WalkDir::new(p)
                    .max_depth(5)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.contains(".test."))
                            .unwrap_or(false)
                    })
            })),
        ];

        let mut detected = Vec::new();
        for (name, checker) in &test_indicators {
            if checker(path) {
                detected.push(*name);
            }
        }

        // Check for test configuration files
        let test_configs = ["jest.config.js", "vitest.config.js", "pytest.ini", ".rspec", "tox.ini"];
        for config in &test_configs {
            if path.join(config).exists() {
                detected.push(*config);
            }
        }

        let test_indicators_count = detected.len();

        // Normalize: 0 indicators = 0.0, 3+ indicators = 1.0
        let normalized = (test_indicators_count as f64 / 3.0).min(1.0);

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "test_coverage",
                normalized,
                if detected.is_empty() {
                    "no test indicators found".to_string()
                } else {
                    format!("found: {}", detected.join(", "))
                },
            )
            .with_confidence(if test_indicators_count > 0 { 1.0 } else { 0.7 })
            .with_weight(1.2),
        )
    }

    fn detect_documentation(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let doc_indicators: Vec<(&str, Box<dyn Fn(&Path) -> bool + Send + Sync>)> = vec![
            ("README", Box::new(|p| {
                p.read_dir()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| {
                                e.file_name()
                                    .to_str()
                                    .map(|n| {
                                        n.to_uppercase().starts_with("README")
                                    })
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap_or(false)
            })),
            ("LICENSE", Box::new(|p| {
                p.read_dir()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| {
                                e.file_name()
                                    .to_str()
                                    .map(|n| {
                                        n.to_uppercase().starts_with("LICENSE") ||
                                        n.to_uppercase().starts_with("LICENCE")
                                    })
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap_or(false)
            })),
            ("CONTRIBUTING", Box::new(|p| {
                p.read_dir()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| {
                                e.file_name()
                                    .to_str()
                                    .map(|n| n.to_uppercase().starts_with("CONTRIBUTING"))
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap_or(false)
            })),
            ("docs/", Box::new(|p| p.join("docs").is_dir())),
            ("doc/", Box::new(|p| p.join("doc").is_dir())),
            ("documentation/", Box::new(|p| p.join("documentation").is_dir())),
            ("CHANGELOG", Box::new(|p| {
                p.read_dir()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .any(|e| {
                                e.file_name()
                                    .to_str()
                                    .map(|n| n.to_uppercase().starts_with("CHANGELOG"))
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap_or(false)
            })),
            ("API docs", Box::new(|p| p.join("src").join("lib.rs").exists())),
        ];

        let mut detected = Vec::new();
        for (name, checker) in &doc_indicators {
            if checker(path) {
                detected.push(*name);
            }
        }

        let doc_count = detected.len();

        // Calculate score
        let score = doc_count as f64 * 0.15 +
            if detected.contains(&"README") { 0.25 } else { 0.0 } +
            if detected.contains(&"LICENSE") { 0.15 } else { 0.0 };

        let normalized = score.min(1.0);

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "documentation",
                normalized,
                if detected.is_empty() {
                    "no documentation found".to_string()
                } else {
                    format!("found: {}", detected.join(", "))
                },
            )
            .with_confidence(1.0)
            .with_weight(1.0),
        )
    }

    fn detect_code_organization(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        // Check for organized source structure
        let src_dirs = ["src", "lib", "source", "app"];
        let has_src = src_dirs.iter().any(|d| path.join(d).is_dir());

        // Check for separate test directories
        let has_test_dir = path.join("tests").is_dir() || path.join("test").is_dir();

        // Check for examples
        let has_examples = path.join("examples").is_dir() || path.join("example").is_dir();

        // Check for CI config
        let has_ci = path.join(".github").join("workflows").exists() ||
            path.join(".gitlab-ci.yml").exists() ||
            path.join(".circleci").exists();

        let score = (has_src as i32) + (has_test_dir as i32) + 
            (has_examples as i32) + (has_ci as i32);

        let normalized = score as f64 / 4.0;

        let indicators = vec![
            if has_src { "src/" } else { "" },
            if has_test_dir { "tests/" } else { "" },
            if has_examples { "examples/" } else { "" },
            if has_ci { "CI/" } else { "" },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

        Ok(
            MaturitySignal::new(
                SignalType::Filesystem,
                "code_organization",
                normalized,
                if indicators.is_empty() {
                    "unstructured".to_string()
                } else {
                    indicators
                },
            )
            .with_confidence(1.0)
            .with_weight(0.9),
        )
    }
}

impl SignalCollector for FilesystemSignals {
    fn signal_type(&self) -> SignalType {
        SignalType::Filesystem
    }

    fn collect(&self, path: &Path) -> MaturityResult<Vec<MaturitySignal>> {
        let mut signals = Vec::new();

        match self.count_files(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to count files: {}", e),
        }

        match self.measure_directory_depth(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to measure directory depth: {}", e),
        }

        match self.detect_config_files(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect config files: {}", e),
        }

        match self.detect_test_coverage(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect test coverage: {}", e),
        }

        match self.detect_documentation(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect documentation: {}", e),
        }

        match self.detect_code_organization(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect code organization: {}", e),
        }

        Ok(signals)
    }
}

impl Default for FilesystemSignals {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_filesystem_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let collector = FilesystemSignals::new();

        let signals = collector.collect(temp_dir.path()).unwrap();
        assert!(!signals.is_empty());

        let file_count = signals.iter().find(|s| s.name == "file_count").unwrap();
        assert_eq!(file_count.value, 0.0);
    }

    #[test]
    fn test_filesystem_with_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "test").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "test").unwrap();

        let collector = FilesystemSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let file_count = signals.iter().find(|s| s.name == "file_count").unwrap();
        assert!(file_count.value > 0.0);
    }

    #[test]
    fn test_config_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();

        let collector = FilesystemSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let config_signal = signals.iter().find(|s| s.name == "config_files").unwrap();
        assert!(config_signal.value > 0.0);
        assert!(config_signal.raw_value.contains("Cargo.toml"));
    }

    #[test]
    fn test_documentation_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();
        fs::write(temp_dir.path().join("LICENSE"), "MIT").unwrap();

        let collector = FilesystemSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let doc_signal = signals.iter().find(|s| s.name == "documentation").unwrap();
        assert!(doc_signal.value > 0.0);
        assert!(doc_signal.raw_value.contains("README"));
    }

    #[test]
    fn test_test_coverage_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir(temp_dir.path().join("tests")).unwrap();
        fs::write(temp_dir.path().join("tests/test.rs"), "# test").unwrap();

        let collector = FilesystemSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let test_signal = signals.iter().find(|s| s.name == "test_coverage").unwrap();
        assert!(test_signal.value > 0.0);
    }
}
