use std::collections::HashSet;
use std::path::Path;

use super::{
    signal::{MaturitySignal, SignalCollector, SignalType},
    MaturityResult,
};

/// Collector for environment-based maturity signals
pub struct EnvironmentSignals;

impl EnvironmentSignals {
    pub fn new() -> Self {
        Self
    }

    fn detect_cicd(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let ci_configs = [
            ("GitHub Actions", ".github/workflows/", true),
            ("GitLab CI", ".gitlab-ci.yml", false),
            ("CircleCI", ".circleci/config.yml", false),
            ("Travis CI", ".travis.yml", false),
            ("Azure Pipelines", "azure-pipelines.yml", false),
            ("Jenkins", "Jenkinsfile", false),
            ("Drone CI", ".drone.yml", false),
            ("Buildkite", ".buildkite/pipeline.yml", true),
        ];

        let mut detected_ci = Vec::new();
        let mut has_advanced_ci = false;

        for (name, config_path, is_advanced) in &ci_configs {
            let full_path = path.join(config_path);
            if full_path.exists() {
                detected_ci.push(*name);
                if *is_advanced {
                    has_advanced_ci = true;
                }
            }
        }

        let ci_count = detected_ci.len();

        // Score based on count and quality
        let score = ci_count as f64 * 0.3
            + if has_advanced_ci { 0.3 } else { 0.0 }
            + if detected_ci.contains(&"GitHub Actions") {
                0.2
            } else {
                0.0
            };

        let normalized = score.min(1.0);

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "cicd_config",
            normalized,
            if detected_ci.is_empty() {
                "no CI/CD detected".to_string()
            } else {
                format!("detected: {}", detected_ci.join(", "))
            },
        )
        .with_confidence(1.0)
        .with_weight(1.3))
    }

    fn detect_package_managers(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let package_managers = [
            ("Cargo (Rust)", "Cargo.toml"),
            ("npm/yarn/pnpm", "package.json"),
            ("Poetry (Python)", "pyproject.toml"),
            ("pip", "requirements.txt"),
            ("Pipenv", "Pipfile"),
            ("Maven", "pom.xml"),
            ("Gradle", "build.gradle"),
            ("sbt", "build.sbt"),
            ("Go Modules", "go.mod"),
            ("RubyGems", "Gemfile"),
            ("Bundler", "Gemfile.lock"),
            ("Composer", "composer.json"),
            ("Swift Package Manager", "Package.swift"),
        ];

        let mut detected_pms = Vec::new();

        for (name, manifest) in &package_managers {
            if path.join(manifest).exists() {
                detected_pms.push(*name);
            }
        }

        // Check for lock files (indicates dependency management maturity)
        let lock_files = [
            "Cargo.lock",
            "package-lock.json",
            "yarn.lock",
            "poetry.lock",
            "go.sum",
        ];
        let has_lock_file = lock_files.iter().any(|f| path.join(f).exists());

        let score = if !detected_pms.is_empty() {
            0.5 + if has_lock_file { 0.3 } else { 0.0 }
                + (detected_pms.len() as f64 * 0.05).min(0.2)
        } else {
            0.0
        };

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "package_manager",
            score.min(1.0),
            if detected_pms.is_empty() {
                "no package manager detected".to_string()
            } else {
                let mut desc = format!("detected: {}", detected_pms.join(", "));
                if has_lock_file {
                    desc.push_str(" (+ lock file)");
                }
                desc
            },
        )
        .with_confidence(1.0)
        .with_weight(1.1))
    }

    fn detect_linters_formatters(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let tools = [
            // Linters
            ("rustfmt", ".rustfmt.toml"),
            ("rustfmt", "rustfmt.toml"),
            ("Clippy", ".clippy.toml"),
            ("ESLint", ".eslintrc.js"),
            ("ESLint", ".eslintrc.json"),
            ("ESLint", ".eslintrc.yaml"),
            ("Prettier", ".prettierrc"),
            ("Prettier", ".prettierrc.js"),
            ("Prettier", ".prettierrc.json"),
            ("Black", "pyproject.toml"), // Could check for [tool.black]
            ("flake8", ".flake8"),
            ("pylint", ".pylintrc"),
            ("mypy", "mypy.ini"),
            ("Checkstyle", "checkstyle.xml"),
            ("SpotBugs", "spotbugs-exclude.xml"),
            ("golangci-lint", ".golangci.yml"),
            ("gofmt", "go.mod"), // Implicit from Go project
            ("RuboCop", ".rubocop.yml"),
            // Formatters
            ("EditorConfig", ".editorconfig"),
        ];

        let mut detected_tools: HashSet<&str> = HashSet::new();

        for (name, config_file) in &tools {
            if path.join(config_file).exists() {
                detected_tools.insert(*name);
            }
        }

        // Check for lint scripts in package.json
        if path.join("package.json").exists() {
            detected_tools.insert("npm scripts");
        }

        let tool_count = detected_tools.len();

        // Score based on tool coverage
        let score = (tool_count as f64 * 0.25).min(1.0);

        let tools_vec: Vec<_> = detected_tools.into_iter().collect();

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "linters_formatters",
            score,
            if tools_vec.is_empty() {
                "no linting/formatting tools".to_string()
            } else {
                format!("detected: {}", tools_vec.join(", "))
            },
        )
        .with_confidence(1.0)
        .with_weight(1.0))
    }

    fn detect_ide_config(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let ide_configs = [
            ("VS Code", ".vscode/"),
            ("IntelliJ IDEA", ".idea/"),
            ("Vim/Neovim", ".vimrc"),
            ("Vim/Neovim", ".nvimrc"),
            ("Emacs", ".emacs"),
            ("Sublime", "*.sublime-project"),
            ("Kate", ".kateproject"),
            ("Eclipse", ".project"),
            ("Eclipse", ".classpath"),
        ];

        let mut detected_ides = Vec::new();

        for (name, config) in &ide_configs {
            let full_path = path.join(config);
            if full_path.exists() {
                if !detected_ides.contains(name) {
                    detected_ides.push(*name);
                }
            }
        }

        // Check for editor-agnostic configs
        let has_editorconfig = path.join(".editorconfig").exists();

        let score = if detected_ides.is_empty() {
            if has_editorconfig {
                0.5
            } else {
                0.0
            }
        } else {
            0.5 + if has_editorconfig { 0.3 } else { 0.0 }
                + (detected_ides.len() as f64 * 0.1).min(0.2)
        };

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "ide_config",
            score.min(1.0),
            if detected_ides.is_empty() {
                if has_editorconfig {
                    "EditorConfig only".to_string()
                } else {
                    "no IDE config".to_string()
                }
            } else {
                format!("detected: {}", detected_ides.join(", "))
            },
        )
        .with_confidence(1.0)
        .with_weight(0.6))
    }

    fn detect_security_tools(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let security_indicators = [
            ("Dependabot", ".github/dependabot.yml"),
            ("Snyk", ".snyk"),
            ("CodeQL", ".github/codeql"),
            ("cargo-audit", "Cargo.lock"),
            ("npm audit", "package-lock.json"),
            ("safety", "requirements.txt"),
            ("Bandit", ".bandit"),
            ("security policy", "SECURITY.md"),
        ];

        let mut detected = Vec::new();

        for (name, indicator) in &security_indicators {
            if path.join(indicator).exists() {
                detected.push(*name);
            }
        }

        let count = detected.len();
        let score = (count as f64 * 0.3).min(1.0);

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "security_tools",
            score,
            if detected.is_empty() {
                "no security tools detected".to_string()
            } else {
                format!("detected: {}", detected.join(", "))
            },
        )
        .with_confidence(1.0)
        .with_weight(0.9))
    }

    fn detect_deployment_config(&self, path: &Path) -> MaturityResult<MaturitySignal> {
        let deployment_indicators = [
            ("Docker", "Dockerfile"),
            ("Docker Compose", "docker-compose.yml"),
            ("Docker Compose", "docker-compose.yaml"),
            ("Kubernetes", "k8s/"),
            ("Kubernetes", "kubernetes/"),
            ("Helm", "Chart.yaml"),
            ("Helm", "charts/"),
            ("Terraform", ".tf"),
            ("Serverless", "serverless.yml"),
            ("Vercel", "vercel.json"),
            ("Netlify", "netlify.toml"),
            ("Heroku", "Procfile"),
            ("Fly.io", "fly.toml"),
            ("AWS SAM", "template.yaml"),
        ];

        let mut detected = Vec::new();

        for (name, indicator) in &deployment_indicators {
            let full_path = path.join(indicator);
            if full_path.exists() {
                detected.push(*name);
            }
        }

        // Check for .tf files
        if path
            .read_dir()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.ends_with(".tf"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
        {
            detected.push("Terraform");
        }

        // Remove duplicates while preserving order
        let mut unique = Vec::new();
        let mut seen = HashSet::new();
        for item in detected {
            if seen.insert(item) {
                unique.push(item);
            }
        }

        let count = unique.len();
        let score = if count > 0 {
            0.3 + (count as f64 * 0.2).min(0.7)
        } else {
            0.0
        };

        Ok(MaturitySignal::new(
            SignalType::Environment,
            "deployment_config",
            score.min(1.0),
            if unique.is_empty() {
                "no deployment config".to_string()
            } else {
                format!("detected: {}", unique.join(", "))
            },
        )
        .with_confidence(1.0)
        .with_weight(0.8))
    }
}

impl SignalCollector for EnvironmentSignals {
    fn signal_type(&self) -> SignalType {
        SignalType::Environment
    }

    fn collect(&self, path: &Path) -> MaturityResult<Vec<MaturitySignal>> {
        let mut signals = Vec::new();

        match self.detect_cicd(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect CI/CD: {}", e),
        }

        match self.detect_package_managers(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect package managers: {}", e),
        }

        match self.detect_linters_formatters(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect linters/formatters: {}", e),
        }

        match self.detect_ide_config(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect IDE config: {}", e),
        }

        match self.detect_security_tools(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect security tools: {}", e),
        }

        match self.detect_deployment_config(path) {
            Ok(signal) => signals.push(signal),
            Err(e) => tracing::warn!("Failed to detect deployment config: {}", e),
        }

        Ok(signals)
    }
}

impl Default for EnvironmentSignals {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_environment_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let collector = EnvironmentSignals::new();

        let signals = collector.collect(temp_dir.path()).unwrap();
        assert!(!signals.is_empty());

        // Should have low values for empty dir
        let cicd = signals.iter().find(|s| s.name == "cicd_config").unwrap();
        assert_eq!(cicd.value, 0.0);
    }

    #[test]
    fn test_cicd_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let github_dir = temp_dir.path().join(".github").join("workflows");
        fs::create_dir_all(&github_dir).unwrap();
        fs::write(github_dir.join("ci.yml"), "name: CI").unwrap();

        let collector = EnvironmentSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let cicd = signals.iter().find(|s| s.name == "cicd_config").unwrap();
        assert!(cicd.value > 0.0);
        assert!(cicd.raw_value.contains("GitHub Actions"));
    }

    #[test]
    fn test_package_manager_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp_dir.path().join("Cargo.lock"), "# lock file").unwrap();

        let collector = EnvironmentSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let pm = signals
            .iter()
            .find(|s| s.name == "package_manager")
            .unwrap();
        assert!(pm.value > 0.0);
        assert!(pm.raw_value.contains("Cargo"));
    }

    #[test]
    fn test_linter_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join(".rustfmt.toml"), "max_width = 100").unwrap();
        fs::write(temp_dir.path().join(".editorconfig"), "root = true").unwrap();

        let collector = EnvironmentSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let linter = signals
            .iter()
            .find(|s| s.name == "linters_formatters")
            .unwrap();
        assert!(linter.value > 0.0);
    }

    #[test]
    fn test_ide_config_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir(temp_dir.path().join(".vscode")).unwrap();

        let collector = EnvironmentSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let ide = signals.iter().find(|s| s.name == "ide_config").unwrap();
        assert!(ide.value > 0.0);
        assert!(ide.raw_value.contains("VS Code"));
    }

    #[test]
    fn test_deployment_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("Dockerfile"), "FROM rust:latest").unwrap();

        let collector = EnvironmentSignals::new();
        let signals = collector.collect(temp_dir.path()).unwrap();

        let deploy = signals
            .iter()
            .find(|s| s.name == "deployment_config")
            .unwrap();
        assert!(deploy.value > 0.0);
        assert!(deploy.raw_value.contains("Docker"));
    }
}
