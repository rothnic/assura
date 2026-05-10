//! Unit tests for the parent module.
use super::*;

#[test]
fn test_frontmatter_schema_builder() {
    let schema = FrontmatterSchema::new()
        .required()
        .with_field("title", FieldValidator::new(FieldType::String).required())
        .with_field("date", FieldValidator::new(FieldType::Date).required())
        .allow_additional_fields(true);

    assert!(schema.required);
    assert_eq!(schema.fields.len(), 2);
    assert!(schema.allow_additional_fields);
}

#[test]
fn test_field_validator_builder() {
    let validator = FieldValidator::new(FieldType::String)
        .required()
        .with_pattern(r"^\d{4}$")
        .with_min(4u64)
        .with_max(4u64)
        .with_allowed_values(vec!["2023", "2024"]);

    assert!(validator.required);
    assert!(validator.pattern.is_some());
    assert!(validator.min.is_some());
    assert!(validator.max.is_some());
    assert!(validator.allowed_values.is_some());
}

#[test]
fn test_validate_string_type() {
    let validator = FieldValidator::new(FieldType::String);
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::String("test".to_string());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let invalid = serde_yaml::Value::Number(42.into());
    assert!(validator.validate("field", &invalid, &path).is_err());
}

#[test]
fn test_validate_string_pattern() {
    let validator = FieldValidator::new(FieldType::String).with_pattern(r"^\d{4}-\d{2}-\d{2}$");
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::String("2024-01-15".to_string());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let invalid = serde_yaml::Value::String("invalid".to_string());
    assert!(validator.validate("field", &invalid, &path).is_err());
}

#[test]
fn test_validate_integer() {
    let validator = FieldValidator::new(FieldType::Integer)
        .with_min(0i64)
        .with_max(100i64);
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::Number(50.into());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let too_small = serde_yaml::Value::Number((-1i64).into());
    assert!(validator.validate("field", &too_small, &path).is_err());

    let too_large = serde_yaml::Value::Number(101i64.into());
    assert!(validator.validate("field", &too_large, &path).is_err());
}

#[test]
fn test_validate_date() {
    let validator = FieldValidator::new(FieldType::Date);
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::String("2024-01-15".to_string());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let invalid = serde_yaml::Value::String("not-a-date".to_string());
    assert!(validator.validate("field", &invalid, &path).is_err());
}

#[test]
fn test_validate_email() {
    let validator = FieldValidator::new(FieldType::Email);
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::String("test@example.com".to_string());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let invalid = serde_yaml::Value::String("not-an-email".to_string());
    assert!(validator.validate("field", &invalid, &path).is_err());
}

#[test]
fn test_validate_allowed_values() {
    let validator = FieldValidator::new(FieldType::String).with_allowed_values(vec![
        "draft",
        "published",
        "archived",
    ]);
    let path = std::path::PathBuf::from("/test.md");

    let valid = serde_yaml::Value::String("published".to_string());
    assert!(validator.validate("field", &valid, &path).is_ok());

    let invalid = serde_yaml::Value::String("invalid".to_string());
    assert!(validator.validate("field", &invalid, &path).is_err());
}

#[test]
fn test_frontmatter_schema_validation() {
    let schema = FrontmatterSchema::new()
        .required()
        .with_field("title", FieldValidator::new(FieldType::String).required())
        .with_field("author", FieldValidator::new(FieldType::String));

    // Document with frontmatter
    let doc_with_frontmatter = super::MarkdownDocument {
        content: "---\ntitle: Test\nauthor: John\n---\n\n# Test".to_string(),
        frontmatter: Some("title: Test\nauthor: John".to_string()),
        body: "# Test".to_string(),
        headings: vec![],
        links: vec![],
        code_blocks: vec![],
        text_content: "Test".to_string(),
        line_count: 5,
        word_count: 1,
    };

    let path = std::path::PathBuf::from("/test.md");
    let failures = schema.validate(&doc_with_frontmatter, &path).unwrap();
    assert!(failures.is_empty());

    // Document without frontmatter
    let doc_without_frontmatter = super::MarkdownDocument {
        content: "# Test".to_string(),
        frontmatter: None,
        body: "# Test".to_string(),
        headings: vec![],
        links: vec![],
        code_blocks: vec![],
        text_content: "Test".to_string(),
        line_count: 1,
        word_count: 1,
    };

    let failures = schema.validate(&doc_without_frontmatter, &path).unwrap();
    assert_eq!(failures.len(), 1);
}

#[test]
fn test_field_type_display() {
    assert_eq!(FieldType::String.display_name(), "string");
    assert_eq!(FieldType::Integer.display_name(), "integer");
    assert_eq!(FieldType::Date.display_name(), "date");
}
