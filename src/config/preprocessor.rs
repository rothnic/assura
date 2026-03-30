//! YAML Preprocessor
//! 
//! Converts user-friendly YAML to valid YAML by adding necessary quotes.
//! Follows Constitution: valid YAML/JSON compatible.

use regex::Regex;
use lazy_static::lazy_static;

/// Preprocessor for Assura configuration YAML
pub struct YamlPreprocessor;

lazy_static! {
    // Pattern to match keys that need quoting
    // Keys starting with . (extensions), containing * (globs), or starting with $
    static ref NEEDS_QUOTING: Regex = Regex::new(
        r"^(\s*)(\.\w+|\*\*|\*|\$\w+|\w*\*\w*)(\s*:)$"
    ).unwrap();
    
    // Pattern to match unquoted values that look like numbers but should be strings
    static ref RANGE_VALUE: Regex = Regex::new(
        r"(:\s*)(\.\.\d+|\d+\.\.|\d+\.\.\d+)(\s*$|\s*#)"
    ).unwrap();
}

impl YamlPreprocessor {
    /// Process raw YAML to make it valid
    pub fn process(input: &str) -> String {
        // First pass: normalize ranges
        let normalized = Self::normalize_ranges(input);
        
        // Second pass: quote keys
        let mut result = String::new();
        
        for line in normalized.lines() {
            let processed = Self::process_line(line);
            result.push_str(&processed);
            result.push('\n');
        }
        
        result
    }
    
    /// Process a single line
    fn process_line(line: &str) -> String {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        
        // Check if this is a key-value pair
        if let Some(colon_pos) = trimmed.find(':') {
            let key = &trimmed[..colon_pos];
            let rest = &trimmed[colon_pos..];
            
            // Check if key needs quoting
            let needs_quote = Self::key_needs_quoting(key);
            
            if needs_quote && !key.starts_with('"') && !key.starts_with('\'') {
                return format!("{}\"{}\"{}", indent, key, rest);
            }
        }
        
        line.to_string()
    }
    
    /// Check if a key needs quoting
    fn key_needs_quoting(key: &str) -> bool {
        // Extension patterns (.tsx, .rs)
        if key.starts_with('.') && key.len() > 1 && key[1..].chars().next().map_or(false, |c| c.is_alphabetic()) {
            return true;
        }
        
        // Glob patterns (*, **)
        if key.contains('*') {
            return true;
        }
        
        // Variable patterns (${name})
        if key.contains("${") && key.contains('}') {
            return true;
        }
        
        // Directives starting with context indicators
        if key == "violation" || key == "constraints" || key == "exists" {
            return false; // These are valid unquoted
        }
        
        false
    }
    
    /// Convert range notation for consistency
    /// Handles: lines: ..400, lines: 100.., lines: 100..400, exists: 1..10
    pub fn normalize_ranges(input: &str) -> String {
        let mut result = input.to_string();

        // Pattern: lines: ..number or exists: ..number (e.g., lines: ..400)
        result = Regex::new(r"(:\s*)(\.\.\d+)(\s*$|\s*#)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}\"{}\"{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // Pattern: lines: number.. (e.g., lines: 100..)
        result = Regex::new(r"(:\s*)(\d+\.\.)(\s*$|\s*#|\s+\w)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}\"{}\"{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // Pattern: key: number..number (e.g., lines: 100..400, exists: 1..10)
        result = Regex::new(r"(:\s*)(\d+\.\.\d+)(\s*$|\s*#)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}\"{}\"{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // Pattern: Array elements like "lines:..400", "size:..1MB", "exists:1..10"
        // These need to be quoted to prevent YAML from interpreting the colon as key-value
        result = Regex::new(r"(\[\s*|,\s*)(lines|size|exists)(:[^\],\s]+)(\s*[,\]])")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}\"{}{}\"{}", &caps[1], &caps[2], &caps[3], &caps[4])
            })
            .to_string();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_extensions() {
        let input = "  .tsx: PascalCase";
        let expected = "  \".tsx\": PascalCase\n";
        assert_eq!(YamlPreprocessor::process(input), expected);
    }

    #[test]
    fn test_quote_globs() {
        let input = "  *.tsx: PascalCase";
        let expected = "  \"*.tsx\": PascalCase\n";
        assert_eq!(YamlPreprocessor::process(input), expected);
    }

    #[test]
    fn test_quote_variables() {
        let input = "  ${name}.tsx: exists";
        let expected = "  \"${name}.tsx\": exists\n";
        assert_eq!(YamlPreprocessor::process(input), expected);
    }

    #[test]
    fn test_no_quote_regular_keys() {
        let input = "  rules:";
        let expected = "  rules:\n";
        assert_eq!(YamlPreprocessor::process(input), expected);
    }

    #[test]
    fn test_no_quote_already_quoted() {
        let input = r#"  ".tsx": PascalCase"#;
        let expected = r#"  ".tsx": PascalCase
"#;
        assert_eq!(YamlPreprocessor::process(input), expected);
    }

    #[test]
    fn test_full_config() {
        let input = r#"
rules:
  react:
    .tsx: PascalCase
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn]

policy:
  src/:
    *.tsx: PascalCase
"#;

        let result = YamlPreprocessor::process(input);
        
        // Check key patterns are quoted
        assert!(result.contains("\".tsx\": PascalCase"));
        assert!(result.contains("\"${name}.tsx\":"));
        assert!(result.contains("\"*.tsx\": PascalCase"));
        
        // Check regular keys are not quoted
        assert!(result.contains("rules:"));
        assert!(result.contains("policy:"));
    }
}
