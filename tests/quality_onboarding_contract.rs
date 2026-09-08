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
fn onboarding_routes_rust_configuration_to_the_broader_native_plan() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("Cargo manifest");
    assert!(onboard(&project).status.success());
    let expected = serde_json::json!([
        "assura check",
        "cargo fmt --all -- --check",
        "cargo test --locked",
        "cargo clippy --all-targets -- -D warnings"
    ]);
    for changed_path in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        ".assura/config.yml",
        ".github/workflows/ci.yml",
    ] {
        assert_eq!(
            successful_plan_json(&project, &[changed_path], "pr")["checks"],
            expected,
            "wrong native plan for Rust configuration {changed_path}"
        );
    }
    assert_eq!(
        successful_plan_json(&project, &["docs/guide.md"], "pr")["checks"],
        serde_json::json!(["assura check"]),
        "documentation must remain policy-only"
    );
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

#[test]
fn onboarding_reports_unavailable_bun_as_advice_without_runnable_bun_gates() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("package.json"),
        r#"{"packageManager":"bun@1.1.0","scripts":{"lint":"biome check .","test":"bun test"}}"#,
    )
    .expect("package manifest");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .args(["--format", "json"])
        .env("PATH", project.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("report JSON");
    assert!(
        report["quality_advice"]
            .as_array()
            .expect("quality advice array")
            .iter()
            .any(|item| { item["tool"] == "bun" && item["status"] == "unavailable" }),
        "an unavailable Bun runtime must remain setup advice"
    );
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml"))
            .expect("materialized config"),
    )
    .expect("valid config YAML");
    assert!(
        config["quality"]["scopes"]["bun"].is_null(),
        "onboarding must not emit runnable Bun commands without Bun"
    );
}

#[cfg(unix)]
#[test]
fn onboarding_uses_available_bun_test_fallback_when_no_test_script_is_declared() {
    use std::os::unix::fs::PermissionsExt;

    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("package.json"),
        r#"{"packageManager":"bun@1.1.0","scripts":{"lint":"biome check ."}}"#,
    )
    .expect("package manifest");
    let tools = TempDir::new().expect("tool directory");
    let bun = tools.path().join("bun");
    fs::write(&bun, "#!/bin/sh\nexit 0\n").expect("Bun fixture");
    let mut permissions = fs::metadata(&bun).expect("Bun metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&bun, permissions).expect("Bun executable");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .env("PATH", tools.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    assert_eq!(
        successful_plan_json(&project, &["src/index.ts"], "frequent")["checks"],
        serde_json::json!(["assura check", "bun run lint"])
    );
    assert_eq!(
        successful_plan_json(&project, &["src/index.ts"], "pre-push")["checks"],
        serde_json::json!(["assura check", "bun run lint", "bun test"])
    );
}

#[test]
fn onboarding_reports_configured_but_unavailable_python_tool_as_advice() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("pyproject.toml"),
        "[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n",
    )
    .expect("Python project configuration");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .args(["--format", "json"])
        .env("PATH", project.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("report JSON");
    let advice = report["quality_advice"]
        .as_array()
        .expect("quality advice array");
    assert!(
        advice
            .iter()
            .any(|item| { item["tool"] == "pytest" && item["status"] == "unavailable" }),
        "configured pytest must remain visible as unavailable setup advice"
    );
}

#[cfg(unix)]
#[test]
fn onboarding_admits_available_configured_pytest_to_the_pre_push_plan() {
    use std::os::unix::fs::PermissionsExt;

    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("pyproject.toml"),
        "[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n",
    )
    .expect("Python project configuration");
    let tools = TempDir::new().expect("tool directory");
    let pytest = tools.path().join("pytest");
    fs::write(&pytest, "#!/bin/sh\nexit 0\n").expect("pytest fixture");
    let mut permissions = fs::metadata(&pytest)
        .expect("pytest metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&pytest, permissions).expect("pytest executable");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .env("PATH", tools.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    let plan = successful_plan_json(&project, &["src/app.py"], "pre-push");
    assert_eq!(
        plan["checks"],
        serde_json::json!(["assura check", "pytest"])
    );
}

#[cfg(unix)]
#[test]
fn onboarding_does_not_plan_a_nonexecutable_pytest_file() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("pyproject.toml"),
        "[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n",
    )
    .expect("Python project configuration");
    let tools = TempDir::new().expect("tool directory");
    fs::write(tools.path().join("pytest"), "not executable\n").expect("pytest fixture");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .args(["--format", "json"])
        .env("PATH", tools.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("report JSON");
    assert!(
        report["quality_advice"]
            .as_array()
            .expect("quality advice array")
            .iter()
            .any(|item| { item["tool"] == "pytest" && item["status"] == "unavailable" }),
        "non-executable pytest files are setup advice, not runnable quality gates"
    );
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml"))
            .expect("materialized config"),
    )
    .expect("valid config YAML");
    assert!(
        config["quality"]["scopes"]["python"].is_null(),
        "onboarding must not create a Python quality scope for a non-executable tool"
    );
}

#[cfg(unix)]
#[test]
fn onboarding_plans_only_available_configured_python_quality_tools_by_phase() {
    use std::os::unix::fs::PermissionsExt;

    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("pyproject.toml"),
        "[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n\n[tool.ruff]\n\n[tool.mypy]\n",
    )
    .expect("Python project configuration");
    let tools = TempDir::new().expect("tool directory");
    for tool in ["pytest", "ruff", "mypy"] {
        let path = tools.path().join(tool);
        fs::write(&path, "#!/bin/sh\nexit 0\n").expect("quality tool fixture");
        let mut permissions = fs::metadata(&path).expect("tool metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("quality tool executable");
    }

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .env("PATH", tools.path())
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    assert_eq!(
        successful_plan_json(&project, &["src/app.py"], "frequent")["checks"],
        serde_json::json!(["assura check", "ruff check ."])
    );
    assert_eq!(
        successful_plan_json(&project, &["src/app.py"], "pre-push")["checks"],
        serde_json::json!(["assura check", "ruff check .", "pytest"])
    );
    assert_eq!(
        successful_plan_json(&project, &["src/app.py"], "pr")["checks"],
        serde_json::json!(["assura check", "ruff check .", "pytest", "mypy ."])
    );
}

#[cfg(windows)]
#[test]
fn onboarding_admits_configured_pytest_from_a_pathext_entrypoint() {
    let project = TempDir::new().expect("project directory");
    fs::write(
        project.path().join("pyproject.toml"),
        "[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n",
    )
    .expect("Python project configuration");
    let tools = TempDir::new().expect("tool directory");
    fs::write(tools.path().join("pytest.EXE"), "fixture").expect("pytest fixture");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .env("PATH", tools.path())
        .env("PATHEXT", ".EXE")
        .output()
        .expect("assura agent onboard runs");
    assert!(output.status.success());
    assert_eq!(
        successful_plan_json(&project, &["src/app.py"], "pre-push")["checks"],
        serde_json::json!(["assura check", "pytest"])
    );
}
