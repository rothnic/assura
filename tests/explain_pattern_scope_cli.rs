use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn run_assura(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_assura"))
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

#[test]
fn explain_shows_inherited_scope_and_winning_file_directive() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
rules:
  "@source-file":
    naming: kebab-case
    max_lines: 500
structure:
  ./:
    .ts: "@source-file"
  "vendor/**/generated/":
    inherit: false
"#,
    )
    .unwrap();
    let source = project.path().join("packages/core/src/good-file.ts");
    let generated = project.path().join("vendor/sdk/generated/Unchecked.ts");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(generated.parent().unwrap()).unwrap();
    fs::write(&source, "source\n").unwrap();
    fs::write(&generated, "generated\n").unwrap();

    let explain = json_from_success(run_assura(&[
        "explain",
        source.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(explain["applied_scopes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|scope| scope["scope"] == "." && scope["match_kind"] == "inherited"));
    assert!(explain["matched_file_patterns"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| {
            rule["pattern"] == "*.ts" && rule["naming"] == "kebab-case" && rule["max_lines"] == 500
        }));

    let text = run_assura(&["explain", source.to_str().unwrap(), "--format", "text"]);
    assert!(text.status.success());
    assert!(String::from_utf8_lossy(&text.stdout)
        .contains("matched_file_patterns=*.ts[naming=kebab-case,max_lines=500]"));

    let reset = json_from_success(run_assura(&[
        "explain",
        generated.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(reset["matched_file_patterns"], serde_json::json!([]));
    assert!(
        reset["applied_scopes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|scope| scope["scope"] == "vendor/**/generated"
                && scope["inheritance_reset"] == true)
    );
    let reset_text = run_assura(&["explain", generated.to_str().unwrap(), "--format", "text"]);
    assert!(reset_text.status.success());
    let reset_text = String::from_utf8_lossy(&reset_text.stdout);
    assert!(reset_text.contains("vendor/**/generated:exact(reset)"));
    assert!(reset_text.contains("matched_file_patterns=none"));
}

#[test]
fn explain_distinguishes_direct_and_recursive_file_globs() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
rules:
  "@direct": { max_lines: 1 }
  "@recursive": { max_lines: 2 }
structure:
  ./:
    "./**/*.tsx": "@recursive"
    "./*.tsx": "@direct"
"#,
    )
    .unwrap();
    let direct = project.path().join("view.tsx");
    let recursive = project.path().join("packages/core/view.tsx");
    fs::create_dir_all(recursive.parent().unwrap()).unwrap();
    fs::write(&direct, "view\n").unwrap();
    fs::write(&recursive, "view\n").unwrap();

    let direct_report = json_from_success(run_assura(&[
        "explain",
        direct.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        direct_report["matched_file_patterns"][0]["pattern"],
        "./*.tsx"
    );
    assert_eq!(direct_report["matched_file_patterns"][0]["max_lines"], 1);

    let recursive_report = json_from_success(run_assura(&[
        "explain",
        recursive.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        recursive_report["matched_file_patterns"][0]["pattern"],
        "./**/*.tsx"
    );
    assert_eq!(recursive_report["matched_file_patterns"][0]["max_lines"], 2);
}
