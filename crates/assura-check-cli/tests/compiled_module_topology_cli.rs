use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const CLASSIFIED_EXPORTS: &str = r#"
/* A commented-out export must not become public.
pub mod block_comment_only;
*/
// pub mod line_comment_only;
pub mod cli {
    // } must not close the module for the scanner.
    pub mod nested_only;
}
pub use crate::{
    api::Widget,
    cli::Command,
};
pub use api::Other;
"#;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src/cli")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
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
            - api
  support_matrices:
    - id: public_surface
      severity: high
      rust_exports:
        - src/lib.rs
      entries:
        - surface: "rust:api"
          status: supported
        - surface: "rust:cli"
          status: supported
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), CLASSIFIED_EXPORTS).unwrap();
}

#[test]
fn native_and_compiled_checks_agree_on_classified_rust_export_families() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-module-topology-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project);

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    for artifact in [None, Some(compiled_config.as_path())] {
        let valid = check(&project, artifact);
        assert!(valid.status.success(), "{valid:?}");
    }

    fs::write(
        project.join("src/lib.rs"),
        format!(
            "{CLASSIFIED_EXPORTS}\npub mod zebra;\npub mod surprise;\npub use surprise::Again;\n"
        ),
    )
    .unwrap();
    for artifact in [None, Some(compiled_config.as_path())] {
        let invalid = check(&project, artifact);
        assert_eq!(invalid.status.code(), Some(1), "{invalid:?}");
        let stdout = String::from_utf8_lossy(&invalid.stdout);
        for rule in [
            "module_topology:public_modules",
            "support_matrix:public_surface",
        ] {
            let findings: Vec<_> = stdout.lines().filter(|line| line.contains(rule)).collect();
            assert_eq!(findings.len(), 2, "{stdout}");
            assert!(findings[0].contains("surprise"), "{stdout}");
            assert!(findings[1].contains("zebra"), "{stdout}");
            if rule == "support_matrix:public_surface" {
                assert!(findings[0].contains("rust:surprise"), "{stdout}");
                assert!(findings[1].contains("rust:zebra"), "{stdout}");
            }
        }
    }
}

fn check(project: &Path, artifact: Option<&Path>) -> Output {
    let mut command = match artifact {
        Some(path) => {
            let mut command = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"));
            command.arg("--compiled-config").arg(path);
            command
        }
        None => Command::new(env!("CARGO_BIN_EXE_assura-check")),
    };
    command
        .current_dir(project)
        .arg("--quiet")
        .output()
        .unwrap()
}
