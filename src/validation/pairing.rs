//! File Pairing Validation
//!
//! Validates that paired files exist (e.g., Button.tsx requires Button.test.tsx).
//! Follows Constitution: structure-first, implicit pairing via variables.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Tracks file pairings that need validation
#[derive(Debug, Clone)]
pub struct PairingRequirement {
    /// Source file pattern (e.g., "${name}.tsx")
    pub source_pattern: String,
    /// Required paired file pattern (e.g., "${name}.test.tsx")
    pub target_pattern: String,
    /// Minimum number of targets required
    pub min_count: u64,
    /// Maximum number of targets allowed
    pub max_count: Option<u64>,
}

/// Validates file pairings across the project
pub struct PairingValidator;

impl PairingValidator {
    /// Find all pairing requirements from policy tree
    pub fn find_requirements(
        policy_root: &crate::config::ast::PolicyNode,
    ) -> Vec<PairingRequirement> {
        let mut requirements = Vec::new();
        let mut found_patterns: HashMap<String, String> = HashMap::new();

        // Scan policy tree for variable patterns
        Self::scan_for_patterns(policy_root, Path::new(""), &mut found_patterns);

        // Identify pairings - patterns with same variable but different extensions
        Self::identify_pairings(&found_patterns, &mut requirements);

        requirements
    }

    /// Scan policy tree and collect all file patterns with variables
    fn scan_for_patterns(
        node: &crate::config::ast::PolicyNode,
        current_path: &Path,
        patterns: &mut HashMap<String, String>,
    ) {
        use crate::config::ast::PolicyEntry;

        for (key, entry) in &node.entries {
            let entry_path = current_path.join(key);

            match entry {
                PolicyEntry::Directory(subdir) => {
                    Self::scan_for_patterns(subdir, &entry_path, patterns);
                }
                PolicyEntry::File(_) if key.contains("${") => {
                    // Check if key contains variable pattern
                    let full_pattern = entry_path.to_string_lossy().to_string();
                    patterns.insert(full_pattern, key.clone());
                }
                _ => {}
            }
        }
    }

    /// Identify pairings from collected patterns
    fn identify_pairings(
        patterns: &HashMap<String, String>,
        requirements: &mut Vec<PairingRequirement>,
    ) {
        // Group patterns by their variable base
        let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

        for (full_path, pattern) in patterns {
            // Extract variable base (e.g., "${name}.tsx" -> "${name}")
            if let Some(start) = pattern.find("${") {
                if let Some(end) = pattern.find('}') {
                    let var_base = &pattern[start..=end];
                    let group_key = full_path.replace(pattern, var_base);
                    grouped
                        .entry(group_key)
                        .or_default()
                        .push(full_path.clone());
                }
            }
        }

        // For each group with multiple patterns, create pairing requirements
        for paths in grouped.values() {
            if paths.len() >= 2 {
                // Find source (usually the main file) and targets
                // Heuristic: shorter extension = source
                let mut sorted = paths.clone();
                sorted.sort_by_key(|p| p.len());

                let source = &sorted[0];
                for target in &sorted[1..] {
                    requirements.push(PairingRequirement {
                        source_pattern: source.clone(),
                        target_pattern: target.clone(),
                        min_count: 1,
                        max_count: None,
                    });
                }
            }
        }
    }

    /// Validate that all pairing requirements are satisfied
    pub fn validate_pairings(
        requirements: &[PairingRequirement],
        existing_files: &[PathBuf],
    ) -> Vec<PairingViolation> {
        let mut violations = Vec::new();

        for req in requirements {
            // Extract variable from source pattern
            let source_var = Self::extract_variable(&req.source_pattern);
            let target_var = Self::extract_variable(&req.target_pattern);

            if source_var != target_var {
                continue; // Variables don't match, not a valid pairing
            }

            // Find all source files that exist
            let source_files: Vec<&PathBuf> = existing_files
                .iter()
                .filter(|f| Self::pattern_matches_path(&req.source_pattern, f))
                .collect();

            for source_file in source_files {
                // Extract the variable value from source
                if let Some(var_value) = Self::extract_var_value(&req.source_pattern, source_file) {
                    // Construct expected target path
                    let expected_target = req
                        .target_pattern
                        .replace(&format!("${{{}}}", source_var), &var_value);

                    // Check if target exists
                    let target_exists = existing_files
                        .iter()
                        .any(|f| f.to_string_lossy().ends_with(&expected_target));

                    if !target_exists {
                        violations.push(PairingViolation {
                            source_file: source_file.clone(),
                            expected_target: PathBuf::from(expected_target),
                            requirement: req.clone(),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Extract variable name from pattern (e.g., "${name}.tsx" -> "name")
    fn extract_variable(pattern: &str) -> String {
        if let Some(start) = pattern.find("${") {
            if let Some(end) = pattern.find('}') {
                return pattern[start + 2..end].to_string();
            }
        }
        String::new()
    }

    /// Extract variable value from actual file path
    fn extract_var_value(pattern: &str, file_path: &Path) -> Option<String> {
        let pattern_parts: Vec<&str> = pattern.split("${").collect();
        if pattern_parts.len() != 2 {
            return None;
        }

        let prefix = pattern_parts[0];
        let rest = pattern_parts[1];

        if let Some(end) = rest.find('}') {
            let suffix = &rest[end + 1..];
            let file_name = file_path.file_name()?.to_string_lossy();
            let full_path = file_path.to_string_lossy();
            let candidate = if prefix.contains('/') || prefix.contains('\\') {
                full_path.as_ref()
            } else {
                file_name.as_ref()
            };

            if candidate.starts_with(prefix) && candidate.ends_with(suffix) {
                let var_value = &candidate[prefix.len()..candidate.len() - suffix.len()];
                if var_value.is_empty()
                    || var_value.contains('/')
                    || var_value.contains('\\')
                    || var_value.contains('.')
                {
                    return None;
                }
                return Some(var_value.to_string());
            }
        }

        None
    }

    /// Check if pattern matches actual file path
    fn pattern_matches_path(pattern: &str, file_path: &Path) -> bool {
        let file_str = file_path.to_string_lossy();

        // Handle ${var} pattern
        if pattern.contains("${") {
            let var_name = Self::extract_variable(pattern);
            if var_name.is_empty() {
                return false;
            }

            return Self::extract_var_value(pattern, file_path).is_some();
        }

        file_str.ends_with(pattern)
    }
}

/// A pairing violation
#[derive(Debug, Clone)]
pub struct PairingViolation {
    pub source_file: PathBuf,
    pub expected_target: PathBuf,
    pub requirement: PairingRequirement,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_variable() {
        assert_eq!(PairingValidator::extract_variable("${name}.tsx"), "name");
        assert_eq!(
            PairingValidator::extract_variable("${component}.test.tsx"),
            "component"
        );
    }

    #[test]
    fn test_extract_var_value() {
        assert_eq!(
            PairingValidator::extract_var_value(
                "${name}.tsx",
                &PathBuf::from("src/components/Button.tsx")
            ),
            Some("Button".to_string())
        );
    }

    #[test]
    fn test_pattern_matching() {
        assert!(PairingValidator::pattern_matches_path(
            "${name}.tsx",
            &PathBuf::from("src/components/Button.tsx")
        ));
        assert!(!PairingValidator::pattern_matches_path(
            "${name}.tsx",
            &PathBuf::from("src/components/Button.test.tsx")
        ));
    }

    #[test]
    fn test_validate_pairings_missing() {
        let req = PairingRequirement {
            source_pattern: "src/components/${name}.tsx".to_string(),
            target_pattern: "src/components/${name}.test.tsx".to_string(),
            min_count: 1,
            max_count: None,
        };

        let existing = vec![
            PathBuf::from("src/components/Button.tsx"),
            // Button.test.tsx is missing!
        ];

        let violations = PairingValidator::validate_pairings(&[req], &existing);

        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].expected_target,
            PathBuf::from("src/components/Button.test.tsx")
        );
    }

    #[test]
    fn test_validate_pairings_complete() {
        let req = PairingRequirement {
            source_pattern: "src/components/${name}.tsx".to_string(),
            target_pattern: "src/components/${name}.test.tsx".to_string(),
            min_count: 1,
            max_count: None,
        };

        let existing = vec![
            PathBuf::from("src/components/Button.tsx"),
            PathBuf::from("src/components/Button.test.tsx"),
        ];

        let violations = PairingValidator::validate_pairings(&[req], &existing);

        assert_eq!(violations.len(), 0);
    }
}
