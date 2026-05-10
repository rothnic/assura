//! Unit tests for the parent module.
use super::*;

#[test]
fn test_config_new() {
    let config = Config::new();
    assert!(config.structure.is_empty());
    assert!(config.exclude.is_empty());
    assert!(config.patterns.is_empty());
}

#[test]
fn test_config_builder() {
    let config = Config::new()
        .with_node(
            "src/",
            DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
        )
        .with_exclude("target/**")
        .with_pattern("**/*.rs", FileBundle::new().with_max_lines(500));

    assert!(config.structure.contains_key("src/"));
    assert_eq!(config.exclude.len(), 1);
    assert!(config.patterns.contains_key("**/*.rs"));
}

#[test]
fn test_validate_naming_convention_valid() {
    assert!(validate_naming_convention("snake_case").is_ok());
    assert!(validate_naming_convention("PascalCase").is_ok());
    assert!(validate_naming_convention("kebab-case").is_ok());
    assert!(validate_naming_convention("regex:^[a-z]+$").is_ok());
}

#[test]
fn test_validate_naming_convention_invalid() {
    assert!(validate_naming_convention("invalid_case").is_err());
    assert!(validate_naming_convention("UnknownCase").is_err());
}

#[test]
fn test_validate_size_string_valid() {
    assert!(validate_size_string("100KB").is_ok());
    assert!(validate_size_string("1MB").is_ok());
    assert!(validate_size_string("10 MB").is_ok());
    assert!(validate_size_string("500B").is_ok());
}

#[test]
fn test_validate_size_string_invalid() {
    assert!(validate_size_string("100").is_err());
    assert!(validate_size_string("large").is_err());
    assert!(validate_size_string("100XB").is_err());
}

#[test]
fn test_directory_node_builder() {
    let node = DirectoryNode::new()
        .with_files(FileBundle::new().with_naming("kebab-case"))
        .with_child(
            "components/",
            DirectoryNode::new().with_files(FileBundle::new().with_naming("PascalCase")),
        )
        .with_inherit(false);

    assert!(node.files.is_some());
    assert!(node.children.is_some());
    assert!(!node.inherit);
}

#[test]
fn test_file_bundle_validation() {
    let bundle = FileBundle::new()
        .with_naming("snake_case")
        .with_max_lines(500)
        .with_max_size("1MB");

    assert!(bundle.validate().is_ok());
}

#[test]
fn test_file_bundle_invalid_naming() {
    let bundle = FileBundle::new().with_naming("invalid_case");
    assert!(bundle.validate().is_err());
}

#[test]
fn test_yaml_serialization() {
    let config = Config::new().with_node(
        "src/",
        DirectoryNode::new().with_files(
            FileBundle::new()
                .with_naming("snake_case")
                .with_max_lines(500),
        ),
    );

    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("structure:"));
    assert!(yaml.contains("src/"));
}

#[test]
fn test_exists_validation() {
    let exists = ExistsValidation::new()
        .with_files(vec!["README.md".to_string(), "LICENSE".to_string()])
        .with_directories(vec!["src".to_string()]);

    assert_eq!(exists.files.as_ref().unwrap().len(), 2);
    assert_eq!(exists.directories.as_ref().unwrap().len(), 1);
}

#[test]
fn test_allowed_names() {
    let bundle =
        FileBundle::new().with_allowed_names(vec!["README.md".to_string(), "LICENSE".to_string()]);

    assert!(bundle.allowed_names.is_some());
    assert_eq!(bundle.allowed_names.unwrap().len(), 2);
}

#[test]
fn test_required_files() {
    let bundle = FileBundle::new().with_required(vec!["README.md".to_string()]);

    assert_eq!(
        bundle.required.as_ref().unwrap(),
        &vec!["README.md".to_string()]
    );
}
