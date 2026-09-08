use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

fn onboard(project: &TempDir) -> std::process::Output {
    Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .args(["--format", "json"])
        .output()
        .expect("assura agent onboard runs")
}

fn quality_plan(project: &TempDir, changed_paths: &[&str], phase: &str) -> std::process::Output {
    let changed_paths_file = project.path().join(format!("changed-paths-{phase}.txt"));
    fs::write(
        &changed_paths_file,
        format!("{}\n", changed_paths.join("\n")),
    )
    .expect("changed paths");
    Command::new(assura_full_bin())
        .args(["quality", "plan"])
        .arg(project.path())
        .args(["--files-from"])
        .arg(&changed_paths_file)
        .args(["--phase", phase, "--format", "json"])
        .output()
        .expect("assura quality plan runs")
}

fn successful_plan_json(
    project: &TempDir,
    changed_paths: &[&str],
    phase: &str,
) -> serde_json::Value {
    let output = quality_plan(project, changed_paths, phase);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("quality plan JSON")
}

#[test]
fn onboarding_a_cargo_project_plans_cumulative_native_gates_for_rust_sources() {
    let project = TempDir::new().expect("project directory");
    fs::create_dir_all(project.path().join("src")).expect("source directory");
    fs::write(
        project.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\n",
    )
    .expect("Cargo workspace manifest");
    fs::write(project.path().join("src/lib.rs"), "pub fn fixture() {}\n").expect("Rust source");

    let output = onboard(&project);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let phase_contracts = [
        (
            "frequent",
            serde_json::json!(["assura check", "cargo fmt --all -- --check"]),
        ),
        (
            "pre-push",
            serde_json::json!([
                "assura check",
                "cargo fmt --all -- --check",
                "cargo test --locked"
            ]),
        ),
        (
            "pr",
            serde_json::json!([
                "assura check",
                "cargo fmt --all -- --check",
                "cargo test --locked",
                "cargo clippy --all-targets -- -D warnings"
            ]),
        ),
    ];
    for (phase, expected_checks) in phase_contracts {
        let plan = successful_plan_json(&project, &["src/lib.rs"], phase);
        assert_eq!(plan["checks"], expected_checks, "wrong {phase} plan");
    }
}

#[test]
fn onboarding_a_bun_project_uses_only_declared_quality_scripts() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("package.json"),
        r#"{"packageManager":"bun@1.1.0","scripts":{"lint":"biome check .","test":"bun test"}}"#,
    )
    .expect("package manifest");

    let onboard = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(onboard.status.success());

    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml"))
            .expect("materialized config"),
    )
    .expect("valid config YAML");
    let bun = &config["quality"]["scopes"]["bun"];
    assert_eq!(bun["paths"][0], "src/**");
    assert_eq!(
        bun["frequent"],
        serde_yaml::Value::Sequence(vec!["bun run lint".into()])
    );
    assert_eq!(
        bun["pre_push"],
        serde_yaml::Value::Sequence(vec!["bun run test".into()])
    );
    assert!(bun["pr"].is_null(), "type-check gates must not be invented");
}
