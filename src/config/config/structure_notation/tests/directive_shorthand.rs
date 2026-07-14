//! Equivalence coverage for concise and expanded node directives.

use super::parse_config;

#[test]
fn reusable_file_directive_shorthand_matches_expanded_attributes() {
    let shorthand = parse_config(
        r#"
rules:
  "@source-file":
    naming: kebab-case
    max_lines: 500
structure:
  src/:
    .ts: "@source-file"
    .tsx: "@source-file"
"#,
    )
    .unwrap();
    let expanded = parse_config(
        r#"
structure:
  src/:
    .ts:
      naming: kebab-case
      max_lines: 500
    .tsx:
      naming: kebab-case
      max_lines: 500
"#,
    )
    .unwrap();

    for config in [&shorthand, &expanded] {
        let files = config
            .structure
            .get("src/")
            .and_then(|node| node.files.as_ref())
            .unwrap();
        assert_eq!(files.max_lines, None);
        let naming = files.naming_patterns.as_ref().unwrap();
        assert_eq!(naming.get("*.ts"), Some(&"kebab-case".to_string()));
        assert_eq!(naming.get("*.tsx"), Some(&"kebab-case".to_string()));
        let line_limits = files.max_lines_patterns.as_ref().unwrap();
        assert_eq!(line_limits.get("*.ts"), Some(&500));
        assert_eq!(line_limits.get("*.tsx"), Some(&500));
    }
}

#[test]
fn pattern_scoped_line_limits_are_semantically_validated() {
    let error = parse_config(
        r#"
structure:
  src/:
    .ts:
      max_lines: 0
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("max_lines_patterns.*.ts"),
        "unexpected error: {error}"
    );
}

#[test]
fn recursive_exists_file_globs_are_rejected_until_recursive_counting_is_supported() {
    let error = parse_config(
        r#"
structure:
  ./:
    "./**/*.ts": exists:1
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("cannot use exists across directories"),
        "unexpected error: {error}"
    );

    let direct = parse_config(
        r#"
structure:
  ./:
    "./*.ts": exists:1
"#,
    )
    .unwrap();
    assert_eq!(
        direct
            .structure
            .get("./")
            .and_then(|node| node.files.as_ref())
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("*.ts")),
        Some(&"1".to_string())
    );
}
