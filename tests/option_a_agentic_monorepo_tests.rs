use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const CONFIG: &str = include_str!("../website/src/data/config-examples/agentic-monorepo.yml");

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write(project: &Path, path: &str, content: &str) {
    let path = project.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn project() -> TempDir {
    let project = tempfile::tempdir().unwrap();
    write(project.path(), ".assura/config.yml", CONFIG);
    for (path, content) in [
        ("AGENTS.md", "# Agent guidance\n"),
        ("package.json", "{}\n"),
        ("pnpm-lock.yaml", "lockfileVersion: 9\n"),
        ("pnpm-workspace.yaml", "packages:\n  - packages/*\n"),
        ("README.md", "# Project\n"),
        ("turbo.json", "{}\n"),
        ("docs/agent-guidance.md", "# Agent guidance\n"),
        ("docs/structure.md", "# Structure\n"),
        ("packages/core/AGENTS.md", "# Core guidance\n"),
        ("packages/core/package.json", "{}\n"),
        (
            "packages/core/src/user-menu.ts",
            "export const menu = [];\n",
        ),
        (
            ".agents/skills/release-check/SKILL.md",
            "---\nname: release-check\n---\n\n# Release check\n",
        ),
    ] {
        write(project.path(), path, content);
    }
    project
}

fn check(project: &Path) -> (std::process::ExitStatus, Value) {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid JSON report: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, report)
}

fn explain(project: &Path, path: &str) -> Value {
    let output = Command::new(assura_bin())
        .arg("explain")
        .arg(project.join(path))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn homepage_agentic_monorepo_policy_accepts_its_documented_shape() {
    let project = project();
    let (status, report) = check(project.path());
    assert!(status.success(), "report was:\n{report:#}");
    assert_eq!(
        report["violations"].as_array().unwrap().len(),
        0,
        "{report:#}"
    );
}

#[test]
fn recursive_defaults_preserve_required_root_policy() {
    let project = project();
    fs::remove_file(project.path().join("AGENTS.md")).unwrap();
    fs::remove_dir_all(project.path().join("packages")).unwrap();

    let (status, report) = check(project.path());
    assert!(!status.success(), "report was:\n{report:#}");
    let messages = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|violation| violation["message"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(messages.contains("AGENTS.md"), "report was:\n{report:#}");
    assert!(messages.contains("packages"), "report was:\n{report:#}");
}

#[test]
fn skill_name_is_allowed_only_at_a_declared_skill_entrypoint() {
    let project = project();
    write(project.path(), "docs/SKILL.md", "# Misplaced skill\n");

    let (status, report) = check(project.path());
    assert!(!status.success(), "report was:\n{report:#}");
    assert!(
        report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|violation| {
                violation["rule"] == "file_naming"
                    && violation["path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with("docs/SKILL.md"))
            }),
        "report was:\n{report:#}"
    );
}

#[test]
fn root_dot_directory_cardinality_survives_recursive_defaults() {
    let project = project();
    for index in 0..11 {
        write(
            project.path(),
            &format!(".tool-{index}/placeholder.txt"),
            "placeholder\n",
        );
    }

    let (status, report) = check(project.path());
    assert!(!status.success(), "report was:\n{report:#}");
    assert!(
        report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|violation| {
                violation["rule"] == "exists_count"
                    && violation["path"].as_str() == Some("")
                    && violation["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("expected 0-10"))
            }),
        "report was:\n{report:#}"
    );
}

#[test]
fn homepage_policy_explains_rebased_rule_threshold_and_repair_context() {
    let project = project();
    let report = explain(project.path(), "packages/core/AGENTS.md");
    assert!(report["source_rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| {
            source["rule"] == "agent-entrypoint"
                && source["effective_selector"] == "packages/*/AGENTS.md"
                && source["status"] == "checked"
        }));
    assert!(report["matched_file_patterns"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| {
            rule["max_lines"] == 160
                && rule["severity"] == "low"
                && rule["message"] == "See docs/agent-guidance.md."
        }));
}

#[test]
fn homepage_agentic_monorepo_policy_reports_repairable_agent_drift() {
    let project = project();
    fs::remove_file(project.path().join("packages/core/AGENTS.md")).unwrap();
    write(
        project.path(),
        ".agents/skills/release-check/notes.md",
        "temporary notes\n",
    );
    write(
        project.path(),
        ".agents/skills/release-check/SKILL.md",
        "# Missing frontmatter\n",
    );
    write(
        project.path(),
        "packages/core/src/BadName.ts",
        &"line\n".repeat(501),
    );
    for index in 0..11 {
        write(
            project.path(),
            &format!("packages/core/generated-{index}.txt"),
            "generated\n",
        );
    }

    let (_status, report) = check(project.path());
    let violations = report["violations"].as_array().unwrap();
    let rules = violations
        .iter()
        .filter_map(|violation| violation["rule"].as_str())
        .collect::<Vec<_>>();
    let messages = violations
        .iter()
        .filter_map(|violation| violation["message"].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rules.contains(&"exists_count"), "report was:\n{report:#}");
    assert!(rules.contains(&"file_naming"), "report was:\n{report:#}");
    assert!(rules.contains(&"max_lines"), "report was:\n{report:#}");
    assert!(rules.contains(&"limit_children"), "report was:\n{report:#}");
    assert!(
        rules.contains(&"markdown_frontmatter"),
        "report was:\n{report:#}"
    );
    assert!(
        messages.contains("docs/agent-guidance.md#layout"),
        "report was:\n{report:#}"
    );
    assert!(
        messages.contains("docs/structure.md"),
        "report was:\n{report:#}"
    );
}

#[test]
fn homepage_policy_direct_and_compiled_checks_are_equivalent() {
    let project = project();
    fs::remove_file(project.path().join("packages/core/AGENTS.md")).unwrap();
    write(
        project.path(),
        "packages/core/src/too-long.ts",
        &"line\n".repeat(501),
    );
    let direct = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    let config_path = project.path().join(".assura/config.yml");
    let config = ConfigLoader::load_validated(&config_path).unwrap();
    let compiled = run_structure_check_with_artifact(
        project.path().to_path_buf(),
        config_path,
        project.path().to_path_buf(),
        CompiledStructureConfigArtifact::new(config),
        false,
    )
    .unwrap();

    let normalize = |report: &assura::cli::StructureCheckReport| {
        let mut findings = report
            .violations
            .iter()
            .map(|violation| {
                (
                    violation
                        .path
                        .strip_prefix(project.path())
                        .unwrap_or(&violation.path)
                        .to_path_buf(),
                    violation.rule.clone(),
                    violation.severity.clone(),
                    violation.message.clone(),
                    violation.blocking,
                )
            })
            .collect::<Vec<_>>();
        findings.sort();
        findings
    };
    assert_eq!(normalize(&direct), normalize(&compiled));
}
use assura::cli::{
    run_structure_check, run_structure_check_with_artifact, CompiledStructureConfigArtifact,
};
use assura::config::config::ConfigLoader;
