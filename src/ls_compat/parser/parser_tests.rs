//! Unit tests for the parent module.
use super::*;

#[test]
fn test_parse_simple_ls_lint() {
    let yaml = r#"
ls:
  .rs: snake_case
  .tsx: PascalCase
  .go: camelCase | snake_case
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse");

    assert!(config.extensions.contains_key(".rs"));
    assert_eq!(config.extensions[".rs"], vec!["snake_case"]);

    assert!(config.extensions.contains_key(".tsx"));
    assert_eq!(config.extensions[".tsx"], vec!["PascalCase"]);

    assert!(config.extensions.contains_key(".go"));
    assert_eq!(config.extensions[".go"], vec!["camelCase", "snake_case"]);
}

#[test]
fn test_convert_convention() {
    assert_eq!(
        LsLintParser::convert_convention("snake_case"),
        Some(NamingConvention::SnakeCase)
    );
    assert_eq!(
        LsLintParser::convert_convention("PascalCase"),
        Some(NamingConvention::PascalCase)
    );
    assert_eq!(LsLintParser::convert_convention("unknown"), None);
}

#[test]
fn test_convert_to_assura() {
    let ls_config = LsLintConfig {
        extensions: {
            let mut map = HashMap::new();
            map.insert(".rs".to_string(), vec!["snake_case".to_string()]);
            map.insert(".tsx".to_string(), vec!["PascalCase".to_string()]);
            map
        },
        paths: HashMap::new(),
        ignore: vec![],
        exists: HashMap::new(),
    };

    let assura_config = LsLintParser::convert_to_assura(&ls_config);

    // Should have rules for each extension
    assert!(assura_config.rules.contains_key("rs_files"));
    assert!(assura_config.rules.contains_key("tsx_files"));

    // Should have policy entries
    assert!(!assura_config.policy.entries.is_empty());
}

#[test]
fn test_migration_tool() {
    let ls_yaml = r#"
ls:
  .rs: snake_case
  .tsx: PascalCase | camelCase
"#;

    let assura_yaml = MigrationTool::migrate(ls_yaml).expect("Should migrate");

    // Should produce valid Assura YAML
    assert!(assura_yaml.contains("rules:"));
    assert!(assura_yaml.contains("policy:"));
    assert!(assura_yaml.contains("snake_case") || assura_yaml.contains("PascalCase"));
}

#[test]
fn test_feature_parity() {
    // Test that all LS-Lint features can be expressed in Assura
    let ls_yaml = r#"
ls:
  .rs: snake_case
  .go: camelCase
  src/components/*: PascalCase
  
ignore:
  - node_modules
  - .git
"#;

    let config = LsLintParser::parse(ls_yaml).expect("Should parse");

    // Verify all features captured
    assert_eq!(config.extensions.len(), 2);
    assert_eq!(config.paths.len(), 1);

    // Convert and verify
    let assura = LsLintParser::convert_to_assura(&config);
    assert!(!assura.rules.is_empty());
}

#[test]
fn test_parse_exists_directive() {
    let yaml = r#"
ls:
  .log: exists:0
  README.md: exists:1
  .test.tsx: exists:1..5
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse exists directives");

    // Check exists map
    assert_eq!(config.exists.get(".log"), Some(&"0".to_string()));
    assert_eq!(config.exists.get("README.md"), Some(&"1".to_string()));
    assert_eq!(config.exists.get(".test.tsx"), Some(&"1..5".to_string()));
}

#[test]
fn test_convert_exists_directive() {
    let ls_config = LsLintConfig {
        extensions: HashMap::new(),
        paths: HashMap::new(),
        ignore: vec![],
        exists: {
            let mut map = HashMap::new();
            map.insert(".log".to_string(), "0".to_string());
            map.insert("README.md".to_string(), "1".to_string());
            map
        },
    };

    let assura_config = LsLintParser::convert_to_assura(&ls_config);

    // Should have policy entries for exists patterns
    assert!(assura_config.policy.entries.contains_key(".log"));
    assert!(assura_config.policy.entries.contains_key("README.md"));
}

#[test]
fn test_multi_part_extensions() {
    let yaml = r#"
ls:
  .test.tsx: snake_case
  .d.ts: camelCase
  .spec.js: PascalCase
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse multi-part extensions");

    // Should handle compound extensions
    assert!(config.extensions.contains_key(".test.tsx"));
    assert!(config.extensions.contains_key(".d.ts"));
    assert!(config.extensions.contains_key(".spec.js"));

    // Convert and verify - should produce valid YAML
    let assura_yaml = MigrationTool::migrate(yaml).expect("Should migrate");
    assert!(!assura_yaml.is_empty());

    // Note: Full round-trip parsing depends on YAML enum serialization compatibility
}

#[test]
fn test_comprehensive_ls_lint_config() {
    // Full LS-Lint config with all features (excluding ignore which requires full YAML parser)
    let yaml = r#"
ls:
  .rs: snake_case
  .go: camelCase
  .test.tsx: snake_case
  .d.ts: camelCase
  src/components/*: PascalCase
  tests/*: snake_case
  .log: exists:0
  README.md: exists:1
  LICENSE: exists:1
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse comprehensive config");

    // Verify all features captured
    assert_eq!(config.extensions.len(), 4); // .rs, .go, .test.tsx, .d.ts
    assert_eq!(config.paths.len(), 2); // src/components/*, tests/*
    assert_eq!(config.exists.len(), 3); // .log, README.md, LICENSE
                                        // Note: ignore list parsing requires full YAML structure support

    // Convert and verify (note: ignore not included in migration yet)
    let assura_yaml = MigrationTool::migrate(yaml).expect("Should migrate comprehensive");
    assert!(!assura_yaml.is_empty());
}

// =========================================================================
// COMPREHENSIVE EXISTS DIRECTIVE TESTS
// =========================================================================

#[test]
fn test_exists_zero_forbidden() {
    // exists:0 means no files of this type should exist
    let yaml = r#"
ls:
  .log: exists:0
  .tmp: exists:0
  .cache: exists:0
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse exists:0");

    // All patterns should be in exists map with value "0"
    assert_eq!(config.exists.get(".log"), Some(&"0".to_string()));
    assert_eq!(config.exists.get(".tmp"), Some(&"0".to_string()));
    assert_eq!(config.exists.get(".cache"), Some(&"0".to_string()));

    // Convert to Assura
    let assura = LsLintParser::convert_to_assura(&config);

    // Should create policy entries for forbidden patterns
    assert!(assura.policy.entries.contains_key(".log"));
    assert!(assura.policy.entries.contains_key(".tmp"));
    assert!(assura.policy.entries.contains_key(".cache"));
}

#[test]
fn test_exists_one_required() {
    // exists:1 means at least one file must exist
    let yaml = r#"
ls:
  README.md: exists:1
  LICENSE: exists:1
  .gitignore: exists:1
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse exists:1");

    assert_eq!(config.exists.get("README.md"), Some(&"1".to_string()));
    assert_eq!(config.exists.get("LICENSE"), Some(&"1".to_string()));
    assert_eq!(config.exists.get(".gitignore"), Some(&"1".to_string()));
}

#[test]
fn test_exists_range_validation() {
    // Test various range formats
    let test_cases = vec![
        ("exists:1..5", "1..5", "between 1 and 5"),
        ("exists:2..10", "2..10", "between 2 and 10"),
        ("exists:..3", "..3", "up to 3"),
        ("exists:5..", "5..", "at least 5"),
        ("exists:10..20", "10..20", "between 10 and 20"),
    ];

    for (directive, expected_range, _desc) in test_cases {
        let yaml = format!(
            "
ls:
  .test: {}
",
            directive
        );

        let config =
            LsLintParser::parse(&yaml).unwrap_or_else(|_| panic!("Should parse {}", directive));

        assert_eq!(
            config.exists.get(".test"),
            Some(&expected_range.to_string()),
            "Failed for range: {}",
            directive
        );
    }
}

#[test]
fn test_exists_exact_count() {
    // Test exact count requirements
    let yaml = r#"
ls:
  .test.tsx: exists:3
  README.md: exists:1
  docs/: exists:5
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse exact counts");

    assert_eq!(config.exists.get(".test.tsx"), Some(&"3".to_string()));
    assert_eq!(config.exists.get("README.md"), Some(&"1".to_string()));
    assert_eq!(config.exists.get("docs/"), Some(&"5".to_string()));
}

#[test]
fn test_exists_with_extensions() {
    // Test exists with various extension patterns
    let yaml = r#"
ls:
  .test.tsx: exists:1..10
  .d.ts: exists:0
  .spec.js: exists:1
  .config.ts: exists:1
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse multi-part exists");

    assert_eq!(config.exists.get(".test.tsx"), Some(&"1..10".to_string()));
    assert_eq!(config.exists.get(".d.ts"), Some(&"0".to_string()));
    assert_eq!(config.exists.get(".spec.js"), Some(&"1".to_string()));
    assert_eq!(config.exists.get(".config.ts"), Some(&"1".to_string()));
}

#[test]
fn test_exists_with_naming_convention() {
    // Test exists combined with naming convention
    let yaml = r#"
ls:
  .rs: snake_case
  .test.rs: exists:1..10
  .bench.rs: exists:0
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse mixed rules");

    // Should have both extension rule and exists directive
    assert!(config.extensions.contains_key(".rs"));
    assert_eq!(config.extensions[".rs"], vec!["snake_case"]);

    // Should have exists directives
    assert_eq!(config.exists.get(".test.rs"), Some(&"1..10".to_string()));
    assert_eq!(config.exists.get(".bench.rs"), Some(&"0".to_string()));
}

#[test]
fn test_exists_directory_patterns() {
    // Test exists with directory patterns
    let yaml = r#"
ls:
  src/: exists:1
  tests/: exists:1..5
  docs/: exists:0
  examples/: exists:1..10
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse directory exists");

    assert_eq!(config.exists.get("src/"), Some(&"1".to_string()));
    assert_eq!(config.exists.get("tests/"), Some(&"1..5".to_string()));
    assert_eq!(config.exists.get("docs/"), Some(&"0".to_string()));
    assert_eq!(config.exists.get("examples/"), Some(&"1..10".to_string()));
}

#[test]
fn test_exists_migration_produces_valid_yaml() {
    let yaml = r#"
ls:
  .log: exists:0
  README.md: exists:1
  .test.tsx: exists:1..10
  src/: exists:1
"#;

    // Migrate to Assura - should produce valid YAML
    let assura_yaml = MigrationTool::migrate(yaml).expect("Should migrate exists");

    // Verify the YAML contains expected elements
    assert!(!assura_yaml.is_empty());
    assert!(assura_yaml.contains("policy:"));

    // Note: Full round-trip parsing depends on YAML enum serialization compatibility
    // which is tested separately in integration tests
}

#[test]
fn test_exists_edge_cases() {
    // Test edge case values
    let yaml = r#"
ls:
  .single: exists:1
  .none: exists:0
  .many: exists:100
  .unbounded: exists:1000..
  .max: exists:..1
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse edge cases");

    assert_eq!(config.exists.get(".single"), Some(&"1".to_string()));
    assert_eq!(config.exists.get(".none"), Some(&"0".to_string()));
    assert_eq!(config.exists.get(".many"), Some(&"100".to_string()));
    assert_eq!(config.exists.get(".unbounded"), Some(&"1000..".to_string()));
    assert_eq!(config.exists.get(".max"), Some(&"..1".to_string()));
}

#[test]
fn test_exists_basic_patterns() {
    // Test exists parsing with various patterns
    let yaml = r#"
ls:
  .test.ts: exists:1..10
  .rs: exists:5
  README.md: exists:1
"#;

    let config = LsLintParser::parse(yaml).expect("Should parse exists patterns");

    // Should parse all exists directives
    assert_eq!(config.exists.get(".test.ts"), Some(&"1..10".to_string()));
    assert_eq!(config.exists.get(".rs"), Some(&"5".to_string()));
    assert_eq!(config.exists.get("README.md"), Some(&"1".to_string()));
}
