//! Unit tests for the parent module.
use super::*;

#[test]
fn test_multiple_rule_syntax() {
    let syntax = MultipleRuleSyntax::parse("kebab-case | snake_case").unwrap();

    let (passed, _) = syntax.validate("my-file");
    assert!(passed);

    let (passed, _) = syntax.validate("my_file");
    assert!(passed);

    let (passed, failures) = syntax.validate("MyFile");
    assert!(!passed);
    assert_eq!(failures.len(), 2);
}

#[test]
fn test_multiple_rule_three_alternatives() {
    let syntax = MultipleRuleSyntax::new()
        .add_convention(CaseConvention::KebabCase)
        .add_convention(CaseConvention::SnakeCase)
        .add_convention(CaseConvention::CamelCase);

    assert!(syntax.validate("my-file").0);
    assert!(syntax.validate("my_file").0);
    assert!(syntax.validate("myFile").0);
    assert!(!syntax.validate("MyFile").0);
}

#[test]
fn test_parse_case_convention() {
    assert_eq!(
        parse_case_convention("kebab-case").unwrap(),
        CaseConvention::KebabCase
    );
    assert_eq!(
        parse_case_convention("snake_case").unwrap(),
        CaseConvention::SnakeCase
    );
    assert_eq!(
        parse_case_convention("flatcase").unwrap(),
        CaseConvention::FlatCase
    );
    assert_eq!(
        parse_case_convention("FLATCASE").unwrap(),
        CaseConvention::ScreamingFlatCase
    );
    assert_eq!(
        parse_case_convention("COBOL-CASE").unwrap(),
        CaseConvention::CobolCase
    );
    assert_eq!(
        parse_case_convention("Train-Case").unwrap(),
        CaseConvention::TrainCase
    );
}

#[test]
fn test_path_rule_matching() {
    let rule = PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap();

    assert!(rule.matches(Path::new("src/main.rs")));
    assert!(rule.matches(Path::new("src/utils/helpers.rs")));
    assert!(rule.matches(Path::new(r"C:\tmp\project\src\utils\helpers.rs")));
    assert!(!rule.matches(Path::new("tests/main.rs")));
    assert!(!rule.matches(Path::new("src/main.js")));
}

#[test]
fn test_path_rule_validation() {
    let rule = PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap();

    assert!(rule.validate("my_module.rs").is_none());
    assert!(rule.validate("my-module.rs").is_some());
}

#[test]
fn test_path_rule_config() {
    let config = PathRuleConfig::new()
        .with_rule(PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap())
        .with_rule(
            PathRule::new("tests/**/*.rs", CaseConvention::SnakeCase)
                .unwrap()
                .with_severity(Severity::Low),
        )
        .with_default_convention(CaseConvention::KebabCase);

    // Should match src rule
    let error = config.validate(Path::new("src/my-module.rs"));
    assert!(error.is_some());
    assert!(error.unwrap().contains("snake_case"));

    // Should match tests rule
    let error = config.validate(Path::new("tests/my-module.rs"));
    assert!(error.is_some());

    // Should use default for other paths
    let error = config.validate(Path::new("docs/my_file.md"));
    assert!(error.is_some());
    assert!(error.unwrap().contains("kebab-case"));
}

#[test]
fn test_glob_to_regex() {
    let regex = glob_to_regex("src/**/*.rs").unwrap();
    assert!(regex.is_match("src/main.rs"));
    assert!(regex.is_match("src/utils/helpers.rs"));
    assert!(!regex.is_match("tests/main.rs"));

    let regex = glob_to_regex("*.txt").unwrap();
    assert!(regex.is_match("file.txt"));
    assert!(!regex.is_match("file.md"));
}

#[test]
fn test_nested_path_rules() {
    let parent = PathRule::new("src/**", CaseConvention::SnakeCase)
        .unwrap()
        .with_child_rule(PathRule::new("src/components/**", CaseConvention::PascalCase).unwrap());

    // Parent rule should match
    assert!(parent.matches(Path::new("src/utils.rs")));

    // Child rule should be found for more specific path
    let child_rule = parent.find_matching_rule(Path::new("src/components/MyComponent.rs"));
    assert!(child_rule.is_some());
    assert_eq!(child_rule.unwrap().convention, CaseConvention::PascalCase);
}
