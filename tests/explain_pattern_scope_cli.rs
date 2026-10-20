use assura::cli::check::{explain_structure_path, CheckError, PathExplainReport, PathExplainRules};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
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

fn markdown_pattern_project() -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  ./:
    extra: true
    markdown:
      check_links: true
    files:
      markdown_patterns:
        "docs/*.md":
          check_links: false
        "docs/guide.md":
          check_links: true
        "absent/*.md":
          check_links: false
"#,
    )
    .unwrap();
    fs::write(
        project.path().join("README.md"),
        "# Root\n\n[missing](missing.md)\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/guide.md"),
        "# Guide\n\n[missing](missing.md)\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/other.md"),
        "# Other\n\n[missing](missing.md)\n",
    )
    .unwrap();
    project
}

#[test]
fn explain_markdown_selection_matches_check_and_all_cli_renderers() {
    let project = markdown_pattern_project();
    let guide = project.path().join("docs/guide.md");
    let other = project.path().join("docs/other.md");
    let readme = project.path().join("README.md");

    let check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        check.status.code(),
        Some(1),
        "check should report the enabled missing links"
    );
    let check_json: Value = serde_json::from_slice(&check.stdout).expect("check emits JSON");
    let link_failures = check_json["violations"]
        .as_array()
        .expect("violations array")
        .iter()
        .filter(|violation| violation["rule"] == "markdown_link_target")
        .map(|violation| violation["path"].as_str().unwrap().replace('\\', "/"))
        .collect::<Vec<_>>();
    assert!(link_failures.iter().any(|path| path.ends_with("README.md")));
    assert!(link_failures
        .iter()
        .any(|path| path.ends_with("docs/guide.md")));
    assert!(!link_failures
        .iter()
        .any(|path| path.ends_with("docs/other.md")));

    let guide_json = json_from_success(run_assura(&[
        "explain",
        guide.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(guide_json["selected_markdown"]["selection_kind"], "pattern");
    assert_eq!(guide_json["selected_markdown"]["pattern"], "docs/guide.md");
    assert_eq!(guide_json["selected_markdown"]["check_links"], true);
    assert!(!guide_json["skipped_checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|skip| skip["name"] == "markdown_checks" && skip["status"] == "not_configured"));

    let other_json = json_from_success(run_assura(&[
        "explain",
        other.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(other_json["selected_markdown"]["selection_kind"], "pattern");
    assert_eq!(other_json["selected_markdown"]["pattern"], "docs/*.md");
    assert_eq!(other_json["selected_markdown"]["check_links"], false);
    assert!(!other_json["skipped_checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|skip| skip["name"] == "markdown_checks" && skip["status"] == "not_configured"));

    let readme_json = json_from_success(run_assura(&[
        "explain",
        readme.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        readme_json["selected_markdown"]["selection_kind"],
        "directory"
    );
    assert!(readme_json["selected_markdown"].get("pattern").is_none());
    assert_eq!(readme_json["selected_markdown"]["check_links"], true);

    let yaml = run_assura(&["explain", guide.to_str().unwrap(), "--format", "yaml"]);
    assert!(yaml.status.success());
    let yaml_report: serde_yaml::Value = serde_yaml::from_slice(&yaml.stdout).unwrap();
    assert_eq!(yaml_report["selected_markdown"]["pattern"], "docs/guide.md");
    assert_eq!(yaml_report["selected_markdown"]["check_links"], true);

    let agent = run_assura(&["explain", guide.to_str().unwrap(), "--format", "agent"]);
    let agent_report: Value = serde_json::from_slice(&agent.stdout).unwrap();
    assert_eq!(
        agent_report["selected_markdown"]["pattern"],
        "docs/guide.md"
    );
    assert_eq!(agent_report["selected_markdown"]["check_links"], true);

    let text = run_assura(&["explain", guide.to_str().unwrap(), "--format", "text"]);
    assert!(String::from_utf8_lossy(&text.stdout)
        .contains("selected_markdown=pattern:docs/guide.md[check_links=true]"));
}

#[test]
fn public_explain_report_type_and_serialization_remain_unchanged() {
    let _signature: fn(Option<PathBuf>, Option<PathBuf>) -> Result<PathExplainReport, CheckError> =
        explain_structure_path;
    let _public_literal = PathExplainReport {
        schema: "assura.path-explain.v1",
        project_root: String::new(),
        config_path: String::new(),
        path: String::new(),
        relative_path: String::new(),
        exists: true,
        kind: "file",
        excluded: false,
        effective_directory: String::new(),
        applied_scopes: Vec::new(),
        effective_rules: PathExplainRules::default(),
        matched_file_patterns: Vec::new(),
        source_rules: Vec::new(),
        skipped_checks: Vec::new(),
        suppressions: Vec::new(),
        next_actions: Vec::new(),
    };
    let project = markdown_pattern_project();
    let report = explain_structure_path(Some(project.path().join("docs/guide.md")), None).unwrap();
    let serialized = serde_json::to_value(report).unwrap();
    assert_eq!(serialized["schema"], "assura.path-explain.v1");
    assert!(serialized.get("selected_markdown").is_none());
}

#[test]
fn explain_shows_inherited_scope_and_winning_file_directive() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
rules:
  source-file:
    naming: kebab-case
    max_lines: 500
structure:
  ./:
    .ts: $source-file
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
        .contains("matched_file_patterns=*.ts[naming=kebab-case,max_lines=500,status=checked]"));

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
  direct: { max_lines: 1 }
  recursive: { max_lines: 2 }
structure:
  ./:
    "./**/*.tsx": $recursive
    "./*.tsx": $direct
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
