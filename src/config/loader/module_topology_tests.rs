//! Module-topology config loader tests.

use super::ConfigLoader;

#[test]
fn test_parse_with_module_topology() {
    let yaml = r#"
extensions:
  module_topologies:
    - id: public_modules
      severity: high
      rust_exports:
        - src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: validation-tests
          purpose: supported CLI implementation
          roots:
            - src/cli
          public_exports:
            - cli
structure: {}
"#;

    let config = ConfigLoader::parse(yaml).unwrap();
    let extensions = config.extensions.unwrap();
    assert_eq!(extensions.module_topologies.len(), 1);
    let policy = &extensions.module_topologies[0];
    assert_eq!(policy.id, "public_modules");
    assert_eq!(policy.modules[0].family, "cli");
    assert_eq!(policy.modules[0].public_exports, vec!["cli"]);
}

#[test]
fn test_parse_rejects_module_topology_invalid_visibility() {
    let yaml = r#"
extensions:
  module_topologies:
    - id: public_modules
      rust_exports:
        - src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: validation-tests
          purpose: supported CLI implementation
          roots:
            - src/cli
          visibility: secret
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("extensions.module_topologies.public_modules.modules.cli.visibility"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_parse_rejects_module_topology_path_escape() {
    let yaml = r#"
extensions:
  module_topologies:
    - id: public_modules
      rust_exports:
        - ../src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: validation-tests
          purpose: supported CLI implementation
          roots:
            - src/cli
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("rust_exports") && error.contains("must be relative"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_parse_rejects_module_topology_conflicting_export_owner() {
    let yaml = r#"
extensions:
  module_topologies:
    - id: public_modules
      rust_exports:
        - src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: validation-tests
          purpose: supported CLI implementation
          roots:
            - src/cli
        - family: facade
          status: supported
          owner: validation-tests
          purpose: public facade
          roots:
            - src/facade
          public_exports:
            - cli
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("public export `cli`") && error.contains("already belongs"),
        "unexpected error: {error}"
    );
}
