use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

fn init(project: &TempDir, recipes: &[&str]) {
    let mut command = Command::new(assura_full_bin());
    command
        .arg("init")
        .arg(project.path())
        .arg("--no-git-hooks");
    for recipe in recipes {
        command.arg("--recipe").arg(recipe);
    }
    let output = command.output().expect("assura init runs");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn onboard(project: &TempDir) {
    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .args(["--format", "json"])
        .output()
        .expect("assura agent onboard runs");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn config(project: &TempDir) -> serde_yaml::Value {
    serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml"))
            .expect("materialized config"),
    )
    .expect("valid materialized YAML")
}

fn run_check(project: &TempDir) -> serde_json::Value {
    let output = Command::new(assura_full_bin())
        .args(["check", "--format", "json"])
        .arg(project.path())
        .output()
        .expect("assura check runs");
    assert!(
        output.status.success(),
        "advisory source-health findings must not fail check:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("check report JSON")
}

fn write_lines(project: &TempDir, path: &str, count: usize) {
    let path = project.path().join(path);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
    fs::write(path, "line\n".repeat(count)).expect("fixture source");
}

fn max_line_paths(report: &serde_json::Value) -> Vec<String> {
    report["violations"]
        .as_array()
        .expect("violation array")
        .iter()
        .filter(|violation| violation["rule"] == "max_lines")
        .map(|violation| {
            violation["path"]
                .as_str()
                .expect("violation path")
                .replace('\\', "/")
        })
        .collect()
}

fn has_path(paths: &[String], suffix: &str) -> bool {
    paths.iter().any(|path| path.ends_with(suffix))
}

fn assert_selector(config: &serde_yaml::Value) {
    let selector = &config["structure"]["./**/"][".{md,js,jsx,ts,tsx,py,rs}"];
    assert_eq!(selector, "max_lines:500 | severity:low");
    assert!(config["structure"]["./**/"][".py"].is_null());
    assert!(config["structure"]["./**/"][".rs"].is_null());
}

fn assert_source_health_behavior(project: &TempDir) {
    write_lines(project, "packages/alpha/src/module.py", 501);
    write_lines(project, "packages/beta/tests/module.rs", 501);
    write_lines(project, "packages/alpha/src/within-limit.py", 500);
    write_lines(project, "packages/beta/tests/within-limit.rs", 500);
    write_lines(project, "docs/within-limit.md", 500);
    write_lines(project, "docs/oversized.md", 501);
    write_lines(project, "packages/alpha/src/oversized.js", 501);
    write_lines(project, "packages/beta/tests/oversized.ts", 501);
    write_lines(project, "target/ignored.py", 501);

    let report = run_check(project);
    let paths = max_line_paths(&report);
    for expected in [
        "packages/alpha/src/module.py",
        "packages/beta/tests/module.rs",
        "packages/alpha/src/oversized.js",
        "packages/beta/tests/oversized.ts",
        "docs/oversized.md",
    ] {
        assert!(
            has_path(&paths, expected),
            "missing max_lines finding for {expected}: {paths:?}"
        );
    }
    for absent in [
        "within-limit.py",
        "within-limit.rs",
        "within-limit.md",
        "target/ignored.py",
    ] {
        assert!(
            !has_path(&paths, absent),
            "unexpected max_lines finding for {absent}: {paths:?}"
        );
    }
}

fn assert_exclusions(config: &serde_yaml::Value, expected: &[&str]) {
    let exclusions = config["exclude"]
        .as_sequence()
        .expect("materialized exclusions")
        .iter()
        .map(|value| value.as_str().expect("string exclusion"))
        .collect::<Vec<_>>();
    assert_eq!(exclusions, expected);
}

#[test]
fn starter_health_is_consistent_across_init_and_onboarding_materializations() {
    let standalone = TempDir::new().expect("standalone project");
    init(&standalone, &["structure-health"]);
    let standalone_config = config(&standalone);
    assert_selector(&standalone_config);
    assert_exclusions(
        &standalone_config,
        &["**/{.git,node_modules,target,dist,coverage}/**"],
    );
    assert!(standalone_config["quality"].is_null());
    assert_source_health_behavior(&standalone);

    let combined = TempDir::new().expect("combined project");
    init(&combined, &["agentic-core", "structure-health"]);
    let combined_config = config(&combined);
    assert_selector(&combined_config);
    assert_exclusions(
        &combined_config,
        &["**/{.git,node_modules,target,dist,coverage}/**"],
    );
    assert!(combined_config["quality"].is_null());
    assert_source_health_behavior(&combined);

    let onboarding = TempDir::new().expect("onboarding project");
    onboard(&onboarding);
    let onboarding_config = config(&onboarding);
    assert_selector(&onboarding_config);
    assert_exclusions(
        &onboarding_config,
        &[
            ".assura/**",
            ".git/**",
            "target/**",
            "node_modules/**",
            "dist/**",
            "**/dist/**",
        ],
    );
    assert!(onboarding_config["quality"]["scopes"]["python"].is_null());
    assert!(onboarding_config["quality"]["scopes"]["rust"].is_null());
    let rules = fs::read_to_string(onboarding.path().join(".assura/onboarding/rules.md"))
        .expect("generated rules guidance");
    for expected in [
        "When the generated source-health rule is active",
        "`.md`, `.js`, `.jsx`, `.ts`, `.tsx`, `.py`, and `.rs`",
        "low severity above 500 lines",
        "`.assura/**`, `.git/**`, `target/**`, `node_modules/**`, `dist/**`, and `**/dist/**`",
        "it does not exclude `coverage`",
        "init recipes instead exclude `**/{.git,node_modules,target,dist,coverage}/**`",
        "Excluded files receive no source-health check",
        "Generated, vendor, and environment directories are not universally excluded",
        "Existing configuration conflicts may leave the effective rules different from this baseline",
    ] {
        assert!(rules.contains(expected), "missing guidance: {expected}");
    }
    assert_source_health_behavior(&onboarding);
}

#[test]
fn generated_sources_are_selected_unless_the_project_adds_an_opt_out() {
    let project = TempDir::new().expect("project");
    init(&project, &["structure-health"]);
    write_lines(&project, "generated/output.py", 501);

    let selected = max_line_paths(&run_check(&project));
    assert!(
        has_path(&selected, "generated/output.py"),
        "generated output is covered by the recursive extension baseline: {selected:?}"
    );

    let mut config = config(&project);
    config["exclude"]
        .as_sequence_mut()
        .expect("exclusion sequence")
        .push(serde_yaml::Value::String("generated/**".into()));
    fs::write(
        project.path().join(".assura/config.yml"),
        serde_yaml::to_string(&config).expect("serialized fixture config"),
    )
    .expect("fixture-owned opt-out");

    let opted_out = max_line_paths(&run_check(&project));
    assert!(
        !has_path(&opted_out, "generated/output.py"),
        "a project-authored exact exclusion must suppress the finding: {opted_out:?}"
    );
}

fn quality_plan(project: &TempDir, changed_path_file: &Path) -> serde_json::Value {
    let output = Command::new(assura_full_bin())
        .args(["quality", "plan"])
        .arg(project.path())
        .args(["--files-from"])
        .arg(changed_path_file)
        .args(["--phase", "pr", "--format", "json"])
        .output()
        .expect("assura quality plan runs");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("quality plan JSON")
}

#[test]
fn onboarding_preserves_authored_structure_and_effective_custom_quality_plan() {
    let project = TempDir::new().expect("project");
    fs::create_dir_all(project.path().join(".assura")).expect("Assura directory");
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"version: "2.0"
rules:
  team-file:
    max_lines: 80
    severity: low
structure:
  ./:
    extra: true
    TEAM.md: exists:0-1
quality:
  scopes:
    team_policy:
      paths: ["src/**"]
      always: ["assura check"]
      frequent: ["echo team policy"]
      pr: ["echo team policy full"]
"#,
    )
    .expect("authored project policy");
    fs::write(project.path().join("TEAM.md"), "team policy\n").expect("authored file");
    let changed = project.path().join("changed-paths.txt");
    fs::write(&changed, "src/team.py\n").expect("changed path fixture");

    let before_config = config(&project);
    assert_eq!(before_config["structure"]["./"]["extra"], true);
    assert_eq!(before_config["structure"]["./"]["TEAM.md"], "exists:0-1");
    let before_plan = quality_plan(&project, &changed);
    let expected_checks =
        serde_json::json!(["assura check", "echo team policy", "echo team policy full"]);
    assert_eq!(before_plan["checks"], expected_checks);
    assert_eq!(
        before_plan["scopes"],
        serde_json::json!([{
            "id": "team_policy",
            "matched_paths": ["src/team.py"],
            "checks": expected_checks,
        }])
    );
    onboard(&project);

    let after_config = config(&project);
    assert_eq!(after_config["rules"]["team-file"]["max_lines"], 80);
    assert_eq!(after_config["structure"]["extra"], true);
    assert_eq!(after_config["structure"]["TEAM.md"], "exists:0-1");
    assert_eq!(
        after_config["quality"]["scopes"]["team_policy"]["paths"][0],
        "src/**"
    );
    assert_eq!(
        after_config["quality"]["scopes"]["team_policy"]["pr"][0],
        "echo team policy full"
    );
    assert!(before_config["quality"]["scopes"]["team_policy"].is_mapping());
    assert_eq!(before_plan, quality_plan(&project, &changed));
}
