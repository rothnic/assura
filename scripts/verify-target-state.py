#!/usr/bin/env python3
"""Verify Assura's target-state audit and highest-risk drift detectors."""

from __future__ import annotations

import json
import pathlib
import re
import subprocess
import sys
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUDIT = ROOT / "docs/analysis/2026-06-09-assura-best-practice-target-state.md"


class Checks:
    def __init__(self) -> None:
        self.errors: list[str] = []

    def add(self, message: str) -> None:
        self.errors.append(message)

    def require(self, condition: bool, message: str) -> None:
        if not condition:
            self.add(message)


def read(path: pathlib.Path) -> str:
    return path.read_text(errors="ignore")


def load_toml(path: pathlib.Path) -> dict:
    return tomllib.loads(path.read_text())


def command_output(args: list[str]) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True)


def check_audit_artifact(checks: Checks) -> None:
    checks.require(AUDIT.exists(), f"{AUDIT.relative_to(ROOT)}: target-state audit is missing")
    if not AUDIT.exists():
        return

    text = read(AUDIT)
    required_sections = [
        "## Repo Inventory",
        "## Source-of-Truth Classification",
        "## Backlog And Detector Ownership",
        "## Deterministic Detection Strategy",
    ]
    for section in required_sections:
        checks.require(section in text, f"{AUDIT.relative_to(ROOT)}: missing section {section!r}")

    inventory_labels = [
        "src",
        "crates",
        "tests",
        "benches",
        "docs",
        ".agents",
        ".trellis",
        ".github",
        ".assura",
        "release files",
        "website-facing claims",
    ]
    for label in inventory_labels:
        checks.require(
            re.search(rf"^\|\s*{re.escape(label)}\s*\|", text, re.MULTILINE) is not None,
            f"{AUDIT.relative_to(ROOT)}: repo inventory missing {label!r}",
        )

    allowed_statuses = {
        "aligned",
        "misaligned",
        "contained/acceptable",
        "uncertain",
        "remove/refactor candidate",
    }
    classification_rows = [
        line
        for line in text.splitlines()
        if line.startswith("| ") and not line.startswith("| ---") and " | " in line
    ]
    for row in classification_rows:
        columns = [column.strip() for column in row.strip("|").split("|")]
        if len(columns) >= 2 and columns[1] in allowed_statuses:
            continue
        if len(columns) >= 4 and columns[3] in allowed_statuses:
            continue

    backlog_header = (
        "| Priority | Concrete Finding | Affected Files/Surfaces | Expected Target State | "
        "Remediation Action | Deterministic Detector | Owner |"
    )
    checks.require(backlog_header in text, f"{AUDIT.relative_to(ROOT)}: backlog table has wrong header")
    backlog_text = text.split("## Backlog And Detector Ownership", 1)[-1]
    p0_rows = [
        row
        for row in backlog_text.splitlines()
        if row.startswith("| P0 |") and "human review required" not in row.lower()
    ]
    checks.require(len(p0_rows) >= 5, f"{AUDIT.relative_to(ROOT)}: expected at least five P0 detector rows")
    for row in p0_rows:
        columns = [column.strip() for column in row.strip("|").split("|")]
        if len(columns) != 7:
            checks.add(f"{AUDIT.relative_to(ROOT)}: malformed P0 backlog row {row!r}")
            continue
        detector = columns[5]
        checks.require(
            detector and detector.lower() not in {"none", "tbd", "todo", "human review"},
            f"{AUDIT.relative_to(ROOT)}: P0 row lacks deterministic detector: {row}",
        )


def check_command_surface_support(checks: Checks) -> None:
    support_text = read(ROOT / "docs/support-policy.md")
    compatibility_text = read(ROOT / "docs/compatibility-and-surface.md")
    release_text = read(ROOT / "docs/release-notes.md")
    validation_text = read(ROOT / "docs/validation.md")
    command_surface = read(ROOT / ".assura/command-surface.yml")

    classified_text = "\n".join([support_text, compatibility_text, release_text, validation_text])
    commands = re.findall(r'^\s+- name:\s*"([^"]+)"', command_surface, re.MULTILINE)
    checks.require(bool(commands), ".assura/command-surface.yml: no command names found")
    for command in commands:
        candidates = [command]
        if command == "assura status":
            candidates.append("assura status --format json")
        if command.startswith("assura hooks "):
            candidates.append("assura hooks")
        if command == "assura quality plan":
            candidates.append("assura quality plan")
        classified = any(f"`{candidate}`" in classified_text for candidate in candidates)
        checks.require(classified, f"{command}: command is not classified in support/release docs")

    public_exports = read(ROOT / "src/lib.rs")
    for marker in [
        "unstable internal APIs",
        "not a supported dependency graph validation release surface",
        "not a supported maturity detection release surface",
        "do not carry a pre-1.0 compatibility guarantee",
    ]:
        checks.require(marker in public_exports, f"src/lib.rs: missing public-surface marker {marker!r}")


def check_manifest_semantics(checks: Checks) -> None:
    root_manifest = load_toml(ROOT / "Cargo.toml")
    package = root_manifest.get("package", {})
    required_package_fields = [
        "name",
        "version",
        "edition",
        "default-run",
        "description",
        "license",
        "repository",
        "homepage",
        "documentation",
        "rust-version",
        "readme",
    ]
    for field in required_package_fields:
        checks.require(field in package, f"Cargo.toml: missing package.{field}")
    checks.require(
        re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][A-Za-z0-9._-]+)?", str(package.get("version", ""))) is not None,
        "Cargo.toml: package.version must be SemVer-like",
    )
    checks.require(package.get("default-run") == "assura", "Cargo.toml: default-run must be assura")

    workspace = root_manifest.get("workspace", {})
    expected_members = {".", "crates/assura-check-cli"}
    checks.require(set(workspace.get("members", [])) == expected_members, "Cargo.toml: workspace members drifted")
    checks.require(
        set(workspace.get("default-members", [])) == expected_members,
        "Cargo.toml: workspace default-members must include all current members",
    )

    internal_manifest = load_toml(ROOT / "crates/assura-check-cli/Cargo.toml")
    internal_package = internal_manifest.get("package", {})
    for field in ["name", "version", "edition", "description", "license", "rust-version", "publish"]:
        checks.require(field in internal_package, f"crates/assura-check-cli/Cargo.toml: missing package.{field}")
    checks.require(
        internal_package.get("version") == package.get("version"),
        "crates/assura-check-cli/Cargo.toml: internal crate version must match root package",
    )
    checks.require(
        internal_package.get("rust-version") == package.get("rust-version"),
        "crates/assura-check-cli/Cargo.toml: internal crate MSRV must match root package",
    )
    checks.require(
        internal_package.get("publish") is False,
        "crates/assura-check-cli/Cargo.toml: internal crate must remain publish=false",
    )


def check_test_relationships(checks: Checks) -> None:
    test_text = "\n".join(
        f"{path.relative_to(ROOT)}\n{read(path)}"
        for root in [ROOT / "tests", ROOT / "crates/assura-check-cli/tests"]
        for path in sorted(root.rglob("*.rs"))
    )
    source_text = "\n".join(
        f"{path.relative_to(ROOT)}\n{read(path)}" for path in sorted((ROOT / "src").rglob("*.rs"))
    )
    coverage = {
        "assura check": {"tests": ["tests/cli_check_tests.rs", "run_check", "--format"]},
        "assura check --format json": {"tests": ["tests/cli_command_surface_tests.rs", '"json"']},
        "assura check --format yaml": {"tests": ["tests/cli_command_surface_tests.rs", '"yaml"']},
        "assura check --format agent": {"tests": ["tests/cli_command_surface_tests.rs", "--format", "agent"]},
        "assura check --format agent --agent codex": {
            "tests": ["tests/cli_command_surface_tests.rs", "--agent", "codex"]
        },
        "assura init": {"tests": ["tests/cli_command_surface_tests.rs", '.arg("init")', "--no-git-hooks"]},
        "assura status --format json": {"tests": ["tests/real_project_agentic_feedback_tests.rs", '.arg("status")', '"json"']},
        "assura migrate": {"tests": ["tests/ls_lint_rule_coverage_tests.rs", '.arg("migrate")']},
        "assura performance-report": {
            "tests": ["tests/performance_report_contract_tests.rs", "two_x_claim_status"]
        },
        "assura hooks": {
            "tests": ["git_hooks_dir_resolves_regular_git_directory"],
            "source": ["GitHooksManager"],
        },
        "assura quality plan": {
            "tests": ["plan_uses_cumulative_phase_checks", "QualityPhase::Merge"],
            "source": ["QualityPlanCommandOptions"],
        },
    }
    for surface, marker_sets in coverage.items():
        missing_tests = [marker for marker in marker_sets["tests"] if marker not in test_text and marker not in source_text]
        checks.require(not missing_tests, f"{surface}: missing test coverage markers {missing_tests}")
        missing_source = [marker for marker in marker_sets.get("source", []) if marker not in source_text]
        checks.require(not missing_source, f"{surface}: missing source markers {missing_source}")

    ignored_test_hits = []
    for root in [ROOT / "src", ROOT / "tests", ROOT / "crates/assura-check-cli/tests"]:
        for path in sorted(root.rglob("*.rs")):
            for line_number, line in enumerate(read(path).splitlines(), start=1):
                if "#[ignore" in line:
                    ignored_test_hits.append((path.relative_to(ROOT), line_number, line.strip()))
    allowed_ignored_tests = {
        ("tests/ls_lint_parity_regression_tests.rs", "manual performance audit fixture"),
    }
    unexpected_ignored = []
    for path, line_number, line in ignored_test_hits:
        if not any(str(path) == allowed_path and marker in line for allowed_path, marker in allowed_ignored_tests):
            unexpected_ignored.append(f"{path}:{line_number}: {line}")
    checks.require(
        not unexpected_ignored,
        "unexpected ignored Rust tests outside audited manual fixtures: " + ", ".join(unexpected_ignored),
    )


def check_docs_release_performance(checks: Checks) -> None:
    root_manifest = load_toml(ROOT / "Cargo.toml")
    version = root_manifest["package"]["version"]
    release_text = read(ROOT / "docs/release-notes.md")
    checklist_text = read(ROOT / "docs/release-candidate-checklist.md")
    compatibility_text = read(ROOT / "docs/compatibility-and-surface.md")
    checks.require(f"v{version}" in release_text, "docs/release-notes.md: release version must match Cargo.toml")
    checks.require(f"v{version}" in checklist_text, "docs/release-candidate-checklist.md: tag version must match Cargo.toml")

    archives = [
        "assura-linux-amd64.tar.gz",
        "assura-linux-musl-amd64.tar.gz",
        "assura-macos-arm64.tar.gz",
        "assura-macos-amd64.tar.gz",
        "assura-windows-amd64.zip",
    ]
    install_scripts = read(ROOT / "website/public/install.sh") + "\n" + read(ROOT / "website/public/install.ps1")
    for archive in archives:
        checks.require(archive in compatibility_text, f"docs/compatibility-and-surface.md: missing {archive}")
        checks.require(archive in release_text, f"docs/release-notes.md: missing {archive}")
    checks.require(
        "assura-linux-amd64.tar.gz" in install_scripts and "assura-windows-amd64.zip" in install_scripts,
        "website install scripts: expected public archive names are missing",
    )

    bench_current = json.loads(read(ROOT / "benches/history/current.json"))
    website_current = json.loads(read(ROOT / "website/public/data/performance/current.json"))
    checks.require(
        bench_current == website_current,
        "performance current.json drift: benches/history and website/public data must match",
    )
    for field in ["schema_version", "timestamp", "claim_summary", "warm_claim_summary", "results"]:
        checks.require(field in bench_current, f"performance current.json: missing {field}")
    checks.require(
        bench_current.get("schema_version") == "assura.performance.v1",
        "performance current.json: unexpected schema_version",
    )


def check_agent_workflow_state(checks: Checks) -> None:
    workflow_json = json.loads(command_output(["python3", "-B", ".trellis/scripts/workflow_gate.py", "--platform", "codex", "--json"]))
    task = workflow_json.get("task") or {}
    git = workflow_json.get("git") or {}
    if task.get("path"):
        checks.require(
            task.get("status") in {"planning", "in_progress"},
            "workflow gate: active task must be planning or in_progress",
        )
        checks.require(
            task.get("branch") == git.get("branch"),
            "workflow gate: active task branch must match current branch",
        )
        checks.require(task.get("artifacts", {}).get("prd") is True, "workflow gate: active task needs prd.md")
    else:
        checks.require(
            git.get("clean", False),
            "workflow gate: no active task is acceptable only for a clean repo state",
        )
    # During implementation, changed files may be outside the task path. The
    # target-state invariant is that an active task and branch own the work.
    # Final handoff still requires a clean `git status --short --branch`.


def main() -> int:
    checks = Checks()
    check_audit_artifact(checks)
    check_command_surface_support(checks)
    check_manifest_semantics(checks)
    check_test_relationships(checks)
    check_docs_release_performance(checks)
    check_agent_workflow_state(checks)

    if checks.errors:
        for error in checks.errors:
            print(error, file=sys.stderr)
        return 1

    print("Target-state audit checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
