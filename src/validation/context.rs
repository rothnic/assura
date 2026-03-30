//! Context Matching Engine
//!
//! Matches current execution context against defined contexts.
//! Determines violation levels based on context.

use crate::config::ast::{Config, Context, ViolationEntry};
use std::collections::HashMap;

/// Current execution context
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub hook: Option<String>,      // tool, pre-commit, ci, etc.
    pub branch: Option<String>,    // feature/auth, main, hotfix/123
    pub version: Option<String>,   // 2.1.0, 1.x
    pub env_vars: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create from environment
    pub fn from_env() -> Self {
        Self {
            hook: std::env::var("ASSURA_HOOK").ok(),
            branch: std::env::var("ASSURA_BRANCH")
                .or_else(|_| std::env::var("GIT_BRANCH"))
                .ok(),
            version: std::env::var("ASSURA_VERSION").ok(),
            env_vars: std::env::vars().collect(),
        }
    }
    
    /// Create for CI context
    pub fn ci() -> Self {
        Self {
            hook: Some("ci".to_string()),
            branch: None,
            version: None,
            env_vars: HashMap::new(),
        }
    }
    
    /// Create for tool context (IDE/editor)
    pub fn tool() -> Self {
        Self {
            hook: Some("tool".to_string()),
            branch: None,
            version: None,
            env_vars: HashMap::new(),
        }
    }
}

/// Matches execution context against defined contexts
pub struct ContextMatcher;

impl ContextMatcher {
    /// Find matching context and determine violation level
    pub fn match_context(
        config: &Config,
        execution: &ExecutionContext,
        violation_entries: &Vec<ViolationEntry>,
    ) -> ViolationLevel {
        // Default level
        let mut level = ViolationLevel::Warn;
        
        // Check each violation entry for context-specific override
        for entry in violation_entries {
            match entry {
                ViolationEntry::Level(l) => {
                    // This is the default level
                    level = ViolationLevel::from_str(l);
                }
                ViolationEntry::ContextSpecific { context, level: ctx_level } => {
                    // Check if this context matches current execution
                    if Self::context_matches(
                        config,
                        context,
                        execution,
                    ) {
                        level = ViolationLevel::from_str(ctx_level);
                    }
                }
            }
        }
        
        level
    }
    
    /// Check if a named context matches current execution
    fn context_matches(
        config: &Config,
        context_name: &str,
        execution: &ExecutionContext,
    ) -> bool {
        let context = match config.contexts.get(context_name) {
            Some(c) => c,
            None => return false, // Unknown context name
        };
        
        // Check hook
        if let Some(ref hook) = context.hook {
            if let Some(ref exec_hook) = execution.hook {
                if !Self::hook_matches(hook, exec_hook) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check branch
        if let Some(ref branch_pattern) = context.branch {
            if let Some(ref exec_branch) = execution.branch {
                if !Self::pattern_matches(branch_pattern, exec_branch) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check version
        if let Some(ref version_range) = context.version {
            if let Some(ref exec_version) = execution.version {
                if !Self::version_matches(version_range, exec_version) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check env vars
        if let Some(ref env) = context.env {
            for (key, value) in env {
                match execution.env_vars.get(key) {
                    Some(exec_value) => {
                        if exec_value != value {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
        }
        
        true
    }
    
    /// Check if hook matches (exact or wildcard)
    fn hook_matches(pattern: &str, actual: &str) -> bool {
        if pattern == "*" || pattern == actual {
            return true;
        }
        
        // Handle pre-commit matching pre-commit-hook, etc.
        actual.starts_with(pattern)
    }
    
    /// Match branch pattern (supports wildcards)
    fn pattern_matches(pattern: &str, actual: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return actual.starts_with(parts[0]) && actual.ends_with(parts[1]);
            }
        }
        
        pattern == actual
    }
    
    /// Check if version matches range
    fn version_matches(range: &str, version: &str) -> bool {
        // Simple version matching: "2.x.." means 2.x and above
        // "..1.x" means up to 1.x
        
        if range == "*" {
            return true;
        }
        
        if range.ends_with("..") {
            // Minimum version
            let min = &range[..range.len()-2];
            return Self::version_gte(version, min);
        }
        
        if range.starts_with("..") {
            // Maximum version
            let max = &range[2..];
            return Self::version_lte(version, max);
        }
        
        if range.contains("..") {
            // Range
            let parts: Vec<&str> = range.split("..").collect();
            if parts.len() == 2 {
                return Self::version_gte(version, parts[0]) &&
                       Self::version_lte(version, parts[1]);
            }
        }
        
        // Exact match
        range == version
    }
    
    /// Version comparison: version >= target
    /// Handles wildcards like "1.x" which matches any 1.x version
    fn version_gte(version: &str, target: &str) -> bool {
        // Handle x wildcard in target (e.g., "1.x" means "1.0.0" and above within major version 1)
        if target.ends_with(".x") || target.ends_with('.') {
            // Extract major version from target
            let target_major: u32 = target
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            // Extract major version from version
            let version_major: u32 = version
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            // For "1.x", we want version >= 1.0.0 but < 2.0.0
            // So check: version_major == target_major
            return version_major == target_major;
        }
        
        // Simplified: compare major.minor.patch
        let v_parts: Vec<u32> = version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let t_parts: Vec<u32> = target
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for i in 0..std::cmp::max(v_parts.len(), t_parts.len()) {
            let v = v_parts.get(i).copied().unwrap_or(0);
            let t = t_parts.get(i).copied().unwrap_or(0);

            if v > t {
                return true;
            }
            if v < t {
                return false;
            }
        }

        true // Equal
    }
    
    /// Version comparison: version <= target
    /// Handles wildcards like "1.x" in target
    fn version_lte(version: &str, target: &str) -> bool {
        // Handle x wildcard in target (e.g., "1.x" means any 1.x version)
        if target.ends_with(".x") || target.ends_with('.') {
            // Extract major version from target
            let target_major: u32 = target
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Extract major version from version
            let version_major: u32 = version
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // For target "1.x", version is <= target if version_major <= target_major
            return version_major <= target_major;
        }

        Self::version_gte(target, version)
    }
}

/// Violation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationLevel {
    Info,    // FYI only
    Warn,    // Warning, may block depending on gate
    Block,   // Always blocks
    Notify,  // Silent notification
}

impl ViolationLevel {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => Self::Info,
            "warn" => Self::Warn,
            "block" => Self::Block,
            "notify" => Self::Notify,
            _ => Self::Warn, // Default
        }
    }
    
    /// Should this level block the operation?
    pub fn should_block(&self, gate: &str, allowed_levels: &[ViolationLevel]) -> bool {
        if *self == ViolationLevel::Block {
            return true;
        }
        
        // Check if this level is in the allowed list
        !allowed_levels.contains(self)
    }
}

impl std::fmt::Display for ViolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationLevel::Info => write!(f, "info"),
            ViolationLevel::Warn => write!(f, "warn"),
            ViolationLevel::Block => write!(f, "block"),
            ViolationLevel::Notify => write!(f, "notify"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_pattern_matching() {
        assert!(ContextMatcher::pattern_matches("feature/*", "feature/auth"));
        assert!(ContextMatcher::pattern_matches("feature/*", "feature/login"));
        assert!(!ContextMatcher::pattern_matches("feature/*", "main"));
        assert!(ContextMatcher::pattern_matches("main", "main"));
    }

    #[test]
    fn test_version_comparison() {
        assert!(ContextMatcher::version_gte("2.1.0", "2.0.0"));
        assert!(ContextMatcher::version_gte("2.1.0", "2.1.0"));
        assert!(!ContextMatcher::version_gte("1.9.0", "2.0.0"));

        assert!(ContextMatcher::version_matches("2.x..", "2.1.0"));
        assert!(ContextMatcher::version_matches("..1.x", "1.5.0"));
        assert!(!ContextMatcher::version_matches("..1.x", "2.0.0"));
    }

    #[test]
    fn test_violation_level_parsing() {
        assert_eq!(ViolationLevel::from_str("warn"), ViolationLevel::Warn);
        assert_eq!(ViolationLevel::from_str("block"), ViolationLevel::Block);
        assert_eq!(ViolationLevel::from_str("info"), ViolationLevel::Info);
        assert_eq!(ViolationLevel::from_str("unknown"), ViolationLevel::Warn); // Default
    }

    #[test]
    fn test_context_matching() {
        use crate::config::ast::{Config, PolicyNode};
        
        let mut contexts = std::collections::HashMap::new();
        contexts.insert(
            "ci".to_string(),
            Context {
                hook: Some("ci".to_string()),
                branch: None,
                version: None,
                env: None,
            },
        );
        
        contexts.insert(
            "feature".to_string(),
            Context {
                hook: Some("pre-commit".to_string()),
                branch: Some("feature/*".to_string()),
                version: None,
                env: None,
            },
        );
        
        let config = Config {
            rules: std::collections::HashMap::new(),
            contexts,
            messages: std::collections::HashMap::new(),
            policy: PolicyNode {
                entries: std::collections::HashMap::new(),
            },
        };
        
        let exec = ExecutionContext {
            hook: Some("ci".to_string()),
            branch: Some("feature/auth".to_string()),
            version: None,
            env_vars: std::collections::HashMap::new(),
        };
        
        assert!(ContextMatcher::context_matches(
            &config, "ci", &exec
        ));
    }
}
