//! Rust-first repository maintenance entrypoint.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const AUDIT: &str = "docs/analysis/2026-06-09-assura-best-practice-target-state.md";

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(mode) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    let rest = args.collect::<Vec<_>>();

    match mode.as_str() {
        "fast" => run_fast(),
        "check" => run_check(),
        "test" => run_test(),
        "evidence" => run_evidence(),
        "target-state" => run_target_state(),
        "hygiene" => run_hygiene(),
        "docs" => run_docs(),
        "release-size" => run_release_size(),
        "release-smoke" => run_release_smoke(),
        "release-live" => run_release_live(),
        "changed" => run_changed(&rest),
        "pr" => run_pr(),
        "full" => {
            run_pr()?;
            run_command("cargo", ["test", "--all-targets", "--quiet"])
        }
        "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => {
            print_usage();
            Err(format!("unknown xtask mode: {mode}").into())
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo xtask <fast|check|test|evidence|target-state|hygiene|docs|release-size|release-smoke|release-live|changed|pr|full>"
    );
}

fn run_fast() -> Result<()> {
    run_command("cargo", ["fmt", "--all", "--", "--check"])?;
    run_command("git", ["diff", "--check"])?;
    run_check()?;
    run_test()?;
    run_command(
        "cargo",
        ["run", "--quiet", "--", "check", "--format", "json", "."],
    )?;
    run_evidence()
}

fn run_pr() -> Result<()> {
    run_fast()?;
    run_target_state()?;
    run_command(
        "cargo",
        [
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_docs()
}

fn run_check() -> Result<()> {
    run_command(
        "cargo",
        [
            "check",
            "-p",
            "assura",
            "--bin",
            "assura",
            "--no-default-features",
            "--features",
            "json-output,yaml-config",
        ],
    )?;
    run_command("cargo", ["check", "-p", "assura", "--bin", "assura-full"])
}

fn run_test() -> Result<()> {
    run_command(
        "cargo",
        [
            "test",
            "--workspace",
            "--lib",
            "--tests",
            "--all-features",
            "--quiet",
        ],
    )
}

fn run_hygiene() -> Result<()> {
    if !command_exists("cargo-machete") {
        return Err(
            "cargo-machete is required. Install with: cargo install cargo-machete --version 0.9.2 --locked"
                .into(),
        );
    }
    run_command("cargo-machete", [] as [&str; 0])
}

fn run_docs() -> Result<()> {
    if command_exists("pnpm") {
        run_command("pnpm", ["--dir", "website", "build"])
    } else if command_exists("npm") {
        run_command("npm", ["--prefix", "website", "run", "build"])
    } else {
        let mut command = Command::new("node");
        command.current_dir("website").args(["--run", "build"]);
        run_command_status(command)
    }
}

fn command_exists(program: &str) -> bool {
    matches!(
        Command::new("sh")
        .args(["-c", &format!("command -v {program} >/dev/null 2>&1")])
            .status(),
        Ok(status) if status.success()
    )
}

fn run_release_size() -> Result<()> {
    let archive = run_release_bundle()?;
    let size = fs::metadata(&archive)?.len();
    let max_size = env::var("ASSURA_MAX_RELEASE_ARCHIVE_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(8_388_608);
    println!("Release archive: {}", archive.display());
    println!("Release archive size: {size} bytes (max {max_size})");
    if size > max_size {
        return Err("release archive exceeds size budget".into());
    }
    Ok(())
}

fn run_release_smoke() -> Result<()> {
    let archive = run_release_bundle()?;
    let tmp = env::temp_dir().join(format!("assura-release-smoke-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    let install_dir = tmp.join("bin");

    let mut install = Command::new("./website/public/install.sh");
    install
        .env("ASSURA_ASSET_URL", fs::canonicalize(&archive)?)
        .env("BIN_DIR", &install_dir);
    run_command_status(install)?;

    let assura = install_dir.join("assura");
    run_command_status({
        let mut command = Command::new(&assura);
        command
            .arg("--help")
            .stdout(fs::File::create(tmp.join("assura-help.txt"))?);
        command
    })?;
    let help = read(tmp.join("assura-help.txt"));
    if !help.contains("Usage: assura") {
        return Err("release smoke help output did not contain Usage: assura".into());
    }
    let mut smoke = Command::new("./scripts/smoke-install-adoption.sh");
    smoke
        .env("ASSURA_BIN", &assura)
        .env("ASSURA_SMOKE_DIR", tmp.join("adoption"));
    run_command_status(smoke)
}

fn run_release_live() -> Result<()> {
    let repo = env::var("ASSURA_REPO").unwrap_or_else(|_| "rothnic/assura".to_string());
    let version = env::var("ASSURA_VERSION").unwrap_or_else(|_| "latest".to_string());
    let release_base = if version == "latest" {
        format!("https://github.com/{repo}/releases/latest/download")
    } else {
        format!("https://github.com/{repo}/releases/download/{version}")
    };
    for url in [
        format!("https://raw.githubusercontent.com/{repo}/master/website/public/install.sh"),
        format!("https://raw.githubusercontent.com/{repo}/master/website/public/install.ps1"),
        format!("{release_base}/assura-linux-amd64.tar.gz"),
        format!("{release_base}/assura-linux-amd64.tar.gz.sha256"),
        format!("{release_base}/assura-linux-musl-amd64.tar.gz"),
        format!("{release_base}/assura-linux-musl-amd64.tar.gz.sha256"),
        format!("{release_base}/assura-macos-amd64.tar.gz"),
        format!("{release_base}/assura-macos-amd64.tar.gz.sha256"),
        format!("{release_base}/assura-macos-arm64.tar.gz"),
        format!("{release_base}/assura-macos-arm64.tar.gz.sha256"),
        format!("{release_base}/assura-windows-amd64.zip"),
        format!("{release_base}/assura-windows-amd64.zip.sha256"),
    ] {
        let status = command_stdout(
            "curl",
            [
                "-I",
                "-L",
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                &url,
            ],
        )?;
        println!("{} {url}", status.trim());
        if status.trim() != "200" {
            return Err(format!("public URL is not reachable: {url}").into());
        }
    }
    Ok(())
}

fn run_release_bundle() -> Result<PathBuf> {
    let platform = release_platform()?;
    if platform == "windows" {
        return Err("release-size and release-smoke use Unix tarballs; Windows archive smoke is covered by CI".into());
    }
    let archive = release_archive_path()?;
    if platform == "linux" {
        run_command(
            "cargo",
            [
                "rustc",
                "--release",
                "--bin",
                "assura",
                "--no-default-features",
                "--features",
                "json-output,yaml-config",
                "--",
                "-C",
                "target-feature=+crt-static",
                "-C",
                "link-arg=-lgcc_eh",
            ],
        )?;
    } else {
        run_command(
            "cargo",
            [
                "build",
                "--release",
                "--bin",
                "assura",
                "--no-default-features",
                "--features",
                "json-output,yaml-config",
            ],
        )?;
    }
    run_command("cargo", ["build", "--release", "--bin", "assura-full"])?;

    let bundle_dir = Path::new("target/release-bundle");
    fs::create_dir_all(bundle_dir)?;
    fs::copy("target/release/assura", bundle_dir.join("assura"))?;
    fs::copy("target/release/assura-full", bundle_dir.join("assura-full"))?;
    run_command_status({
        let mut command = Command::new("tar");
        command
            .args(["-C", "target/release-bundle", "-czf"])
            .arg(&archive)
            .args(["assura", "assura-full"]);
        command
    })?;
    write_checksum(&archive)?;
    Ok(archive)
}

fn release_platform() -> Result<&'static str> {
    match env::consts::OS {
        "linux" => Ok("linux"),
        "macos" => Ok("macos"),
        "windows" => Ok("windows"),
        other => Err(format!("unsupported release platform: {other}").into()),
    }
}

fn release_arch() -> &'static str {
    match env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    }
}

fn release_archive_path() -> Result<PathBuf> {
    Ok(PathBuf::from(format!(
        "target/assura-{}-{}-preview.tar.gz",
        release_platform()?,
        release_arch()
    )))
}

fn write_checksum(archive: &Path) -> Result<()> {
    let archive_name = archive
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or("release archive has no file name")?;
    let parent = archive.parent().unwrap_or_else(|| Path::new("."));
    let checksum = archive.with_extension(format!(
        "{}.sha256",
        archive.extension().and_then(OsStr::to_str).unwrap_or("")
    ));
    if command_exists("sha256sum") {
        let output = command_stdout_in(parent, "sha256sum", [archive_name])?;
        fs::write(&checksum, output)?;
        run_command_status({
            let mut command = Command::new("sha256sum");
            command
                .current_dir(parent)
                .args(["-c", &format!("{archive_name}.sha256")]);
            command
        })
    } else {
        let output = command_stdout_in(parent, "shasum", ["-a", "256", archive_name])?;
        fs::write(&checksum, output)?;
        run_command_status({
            let mut command = Command::new("shasum");
            command.current_dir(parent).args([
                "-a",
                "256",
                "-c",
                &format!("{archive_name}.sha256"),
            ]);
            command
        })
    }
}

fn command_stdout_in<I, S>(dir: &Path, program: &str, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .current_dir(dir)
        .args(args)
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(format!("command failed: {program}").into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn run_changed(args: &[String]) -> Result<()> {
    let mut phase = "frequent".to_string();
    let mut dry_run = false;
    let mut plan_args = vec![".".to_string()];
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--phase" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --phase".into());
                };
                phase = value.clone();
                index += 2;
            }
            "--files-from" | "--base" | "--head" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(format!("missing value for {}", args[index]).into());
                };
                plan_args.push(args[index].clone());
                plan_args.push(value.clone());
                index += 2;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--" => index += 1,
            other => return Err(format!("unknown changed-mode option: {other}").into()),
        }
    }

    let mut command = Command::new("cargo");
    command
        .args([
            "run",
            "--quiet",
            "--bin",
            "assura-full",
            "--",
            "quality",
            "plan",
        ])
        .args(&plan_args)
        .args(["--phase", &phase, "--format", "json"]);
    if plan_args
        .windows(2)
        .any(|pair| pair[0] == "--files-from" && pair[1] == "-")
    {
        command.stdin(Stdio::inherit());
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err("assura quality plan failed".into());
    }
    let plan: Value = serde_json::from_slice(&output.stdout)?;
    let scopes = plan
        .get("scopes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let changed_paths = plan
        .get("changed_paths")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    println!(
        "Assura changed-check plan: phase={} changed_paths={} scopes={}",
        plan.get("phase").and_then(Value::as_str).unwrap_or(&phase),
        changed_paths,
        scopes.len()
    );
    for scope in scopes {
        let id = scope
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let matched = scope
            .get("matched_paths")
            .and_then(Value::as_array)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        println!("- {id}: {matched}");
    }
    let checks = plan
        .get("checks")
        .and_then(Value::as_array)
        .map(|checks| {
            checks
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if checks.is_empty() {
        println!("No quality checks selected.");
        return Ok(());
    }
    let has_pr = checks.iter().any(|check| check == "cargo xtask pr");
    let has_full = checks.iter().any(|check| check == "cargo xtask full");
    for check in checks {
        if check == "cargo xtask changed" || check.starts_with("cargo xtask changed ") {
            return Err(format!("refusing recursive changed-check command: {check}").into());
        }
        if (has_pr || has_full)
            && matches!(
                check.as_str(),
                "cargo xtask check"
                    | "cargo xtask test"
                    | "cargo xtask evidence"
                    | "cargo run --quiet -- check --format json ."
                    | "git diff --check"
            )
        {
            println!("Skipping check covered by broader local gate: {check}");
            continue;
        }
        if has_full && check == "cargo xtask pr" {
            println!("Skipping check covered by broader local gate: {check}");
            continue;
        }
        if dry_run {
            println!("[dry-run] {check}");
        } else if is_local_check(&check) {
            println!("\n$ {check}");
            run_command("bash", ["-lc", &check])?;
        } else {
            println!("Skipping non-local check: {check}");
        }
    }
    Ok(())
}

fn is_local_check(check: &str) -> bool {
    ["cargo ", "git ", "npm ", "pnpm ", "scripts/", "./scripts/"]
        .iter()
        .any(|prefix| check.starts_with(prefix))
}

fn run_command<I, S>(program: &str, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    run_command_status(command)
}

fn run_command_status(mut command: Command) -> Result<()> {
    let display = format!("{command:?}");
    let status = command.status()?;
    if !status.success() {
        return Err(format!("command failed: {display}").into());
    }
    Ok(())
}

fn command_stdout<I, S>(program: &str, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program)
        .args(args)
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(format!("command failed: {program}").into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[derive(Default)]
struct Checks {
    errors: Vec<String>,
}

impl Checks {
    fn add(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    fn require(&mut self, condition: bool, message: impl Into<String>) {
        if !condition {
            self.add(message);
        }
    }

    fn finish(self, success: &str) -> Result<()> {
        if self.errors.is_empty() {
            println!("{success}");
            return Ok(());
        }
        for error in self.errors {
            eprintln!("{error}");
        }
        Err("xtask checks failed".into())
    }
}

fn run_evidence() -> Result<()> {
    run_command("bash", ["scripts/check-ci-scope.sh"])?;

    let mut checks = Checks::default();
    check_trellis_state(&mut checks);
    check_evidence_policy(&mut checks);
    checks.finish("Review evidence policy checks passed.")
}

fn run_target_state() -> Result<()> {
    let mut checks = Checks::default();
    check_audit_artifact(&mut checks);
    check_command_surface_support(&mut checks);
    check_manifest_semantics(&mut checks);
    check_test_relationships(&mut checks);
    check_docs_release_performance(&mut checks);
    check_agent_workflow_state(&mut checks);
    check_root_tooling_boundary(&mut checks);
    check_lint_suppression_reasons(&mut checks);
    checks.finish("Target-state audit checks passed.")
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

fn rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn collect_files(root: impl AsRef<Path>, suffix: Option<&str>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_inner(root.as_ref(), suffix, &mut files);
    files.sort();
    files
}

fn collect_files_inner(path: &Path, suffix: Option<&str>, files: &mut Vec<PathBuf>) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.is_file() {
        if suffix.map_or(true, |suffix| path.to_string_lossy().ends_with(suffix)) {
            files.push(path.to_path_buf());
        }
        return;
    }
    if !metadata.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_files_inner(&entry.path(), suffix, files);
    }
}

fn check_trellis_state(checks: &mut Checks) {
    let allowed_task_statuses = ["planning", "in_progress"];
    for task_file in direct_task_files() {
        let Ok(task) = serde_json::from_str::<Value>(&read(&task_file)) else {
            checks.add(format!("{}: task JSON is invalid", rel(&task_file)));
            continue;
        };
        let status = task.get("status").and_then(Value::as_str).unwrap_or("");
        checks.require(
            allowed_task_statuses.contains(&status),
            format!(
                "{}: active task status {status:?} should be archived or in progress/planning",
                rel(&task_file)
            ),
        );
    }

    let mut goal_statuses = BTreeMap::new();
    let allowed_goal_statuses = ["planned", "active", "completed", "archived"];
    for goal_file in collect_files("docs/goals", Some(".md")) {
        let text = read(&goal_file);
        let status = frontmatter_value(&text, "status");
        let Some(status) = status else {
            if text.starts_with("---\n") {
                checks.add(format!("{}: missing frontmatter status", rel(&goal_file)));
            }
            continue;
        };
        checks.require(
            allowed_goal_statuses.contains(&status.as_str()),
            format!(
                "{}: unsupported goal status {status:?}; expected one of {:?}",
                rel(&goal_file),
                allowed_goal_statuses
            ),
        );
        if let Some(file_name) = goal_file.file_name().and_then(OsStr::to_str) {
            goal_statuses.insert(file_name.to_string(), status);
        }
    }

    let phase_plan = Path::new("docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md");
    let phase_goal_files = BTreeMap::from([
        (1, "assura-goal-01-trustworthy-self-enforcement.md"),
        (2, "assura-goal-02-policy-language-completeness.md"),
        (3, "assura-goal-03-agent-feedback-delivery-loop.md"),
        (4, "assura-goal-04-fast-incremental-check-engine.md"),
        (5, "assura-goal-05-installable-adoption-path.md"),
        (6, "assura-goal-06-review-evidence-and-quality-gates.md"),
        (7, "assura-goal-07-extension-and-plugin-foundation.md"),
        (8, "assura-goal-08-release-readiness-and-ecosystem.md"),
    ]);
    if phase_plan.exists() {
        let ledger_statuses = iteration_ledger_statuses(&read(phase_plan));
        for order in phase_goal_files.keys() {
            checks.require(
                ledger_statuses.contains_key(order),
                format!(
                    "{}: missing Iteration 01 ledger row for goal {order}",
                    rel(phase_plan)
                ),
            );
        }
        for (order, file_name) in &phase_goal_files {
            let expected = ledger_statuses.get(order);
            let actual = goal_statuses.get(*file_name);
            match (expected, actual) {
                (_, None) => checks.add(format!("docs/goals/{file_name}: missing Iteration 01 goal file")),
                (Some(expected), Some(actual)) if actual != expected => checks.add(format!(
                    "docs/goals/{file_name}: frontmatter status {actual:?} does not match Iteration 01 ledger status {expected:?}"
                )),
                _ => {}
            }
        }

        let mut allowed_active =
            BTreeSet::from(["assura-roadmap-phase-01-agentic-adoption-foundation.md".to_string()]);
        for (order, file_name) in &phase_goal_files {
            if ledger_statuses.get(order).map(String::as_str) == Some("active") {
                allowed_active.insert((*file_name).to_string());
            }
        }
        for (file_name, status) in &goal_statuses {
            if status == "active" && !allowed_active.contains(file_name) {
                checks.add(format!(
                    "docs/goals/{file_name}: active status is not listed as active in the Phase 01 ledger"
                ));
            }
        }
    }
}

fn frontmatter_value(text: &str, key: &str) -> Option<String> {
    if !text.starts_with("---\n") {
        return None;
    }
    let rest = &text[4..];
    let end = rest.find("\n---")?;
    for line in rest[..end].lines() {
        let (left, right) = line.split_once(':')?;
        if left.trim() == key {
            return Some(right.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn iteration_ledger_statuses(text: &str) -> BTreeMap<i32, String> {
    let mut statuses = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let columns = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if columns.len() < 2 {
            continue;
        }
        let Some((order, _)) = columns[0].split_once('.') else {
            continue;
        };
        let Ok(order) = order.trim().parse::<i32>() else {
            continue;
        };
        statuses.insert(order, columns[1].to_ascii_lowercase());
    }
    statuses
}

fn check_evidence_policy(checks: &mut Checks) {
    for path in [
        "docs/analysis/review-record-template.md",
        "docs/analysis/evidence-and-review-policy.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
    ] {
        checks.require(
            exists(path),
            format!("{path}: required Goal 06 evidence file is missing"),
        );
    }

    let template = "docs/analysis/review-record-template.md";
    let template_text = read(template);
    for heading in [
        "## Scope Review",
        "## Evidence Inventory",
        "## Validation Commands",
        "## Review Tasks",
        "## Review Feedback Closure",
        "## Handoff",
    ] {
        checks.require(
            template_text.contains(heading),
            format!("{template}: missing required heading {heading:?}"),
        );
    }

    let pr_template = ".github/PULL_REQUEST_TEMPLATE.md";
    let pr_template_text = read(pr_template).to_ascii_lowercase();
    for phrase in [
        "Goal",
        "Review record",
        "Evidence",
        "Validation",
        "Review feedback",
        "Next goal",
    ] {
        checks.require(
            pr_template_text.contains(&phrase.to_ascii_lowercase()),
            format!("{pr_template}: missing PR evidence phrase {phrase:?}"),
        );
    }

    let phase_goal_files = [
        "docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md",
        "docs/goals/assura-goal-01-trustworthy-self-enforcement.md",
        "docs/goals/assura-goal-02-policy-language-completeness.md",
        "docs/goals/assura-goal-03-agent-feedback-delivery-loop.md",
        "docs/goals/assura-goal-04-fast-incremental-check-engine.md",
        "docs/goals/assura-goal-05-installable-adoption-path.md",
        "docs/goals/assura-goal-06-review-evidence-and-quality-gates.md",
        "docs/goals/assura-goal-07-extension-and-plugin-foundation.md",
        "docs/goals/assura-goal-08-release-readiness-and-ecosystem.md",
    ];
    let goal_required_keys = ["id", "type", "title", "status", "created", "owners"];
    for goal_file in phase_goal_files {
        if !exists(goal_file) {
            checks.add(format!("{goal_file}: missing Iteration 01 goal file"));
            continue;
        }
        let text = read(goal_file);
        if !text.starts_with("---\n") {
            checks.add(format!("{goal_file}: missing YAML frontmatter"));
            continue;
        }
        for key in goal_required_keys {
            checks.require(
                frontmatter_value(&text, key).is_some(),
                format!("{goal_file}: missing frontmatter key {key:?}"),
            );
        }
    }

    let mut checked_markdown_files = vec![
        "docs/validation.md",
        "docs/release-notes.md",
        "docs/release-candidate-checklist.md",
        "docs/support-policy.md",
        "docs/compatibility-and-surface.md",
        "docs/project-memories.md",
        pr_template,
        "docs/analysis/review-record-template.md",
        "docs/analysis/evidence-and-review-policy.md",
        "docs/analysis/2026-06-02-goal-06-review-evidence-gates-review.md",
        "docs/analysis/2026-06-02-goal-07-extension-plugin-foundation-review.md",
        "docs/analysis/2026-06-02-goal-08-release-readiness-review.md",
        ".trellis/spec/assura/index.md",
        ".trellis/spec/assura/roadmap.md",
        ".trellis/spec/assura/codex-agent-feedback.md",
        ".trellis/spec/assura/tooling-stabilization.md",
        "docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md",
        "website/src/content/docs/reference/release-readiness.md",
    ];
    checked_markdown_files.extend(phase_goal_files);
    for md_file in checked_markdown_files {
        if exists(md_file) {
            check_markdown_links(checks, Path::new(md_file), &read(md_file));
        }
    }

    check_forbidden_surface(checks);
    check_release_surface_claims(checks);
}

fn check_markdown_links(checks: &mut Checks, md_file: &Path, text: &str) {
    let mut remaining = text;
    while let Some(open) = remaining.find("](") {
        let after_open = &remaining[open + 2..];
        let Some(close) = after_open.find(')') else {
            break;
        };
        let link = after_open[..close].trim();
        remaining = &after_open[close + 1..];
        if link.is_empty()
            || link.starts_with('#')
            || link.starts_with("http://")
            || link.starts_with("https://")
            || link.starts_with("mailto:")
            || link.starts_with("target/")
            || link.starts_with('/')
        {
            continue;
        }
        let target = link
            .trim_matches('<')
            .trim_matches('>')
            .split('#')
            .next()
            .unwrap_or("");
        if target.is_empty() {
            continue;
        }
        let target = percent_decode(target);
        let resolved = md_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(target);
        checks.require(
            resolved.exists(),
            format!("{}: broken local markdown link {link:?}", rel(md_file)),
        );
    }
}

fn percent_decode(text: &str) -> String {
    text.replace("%20", " ")
}

fn check_forbidden_surface(checks: &mut Checks) {
    for sample in [
        "assura-codex-feedback --warn",
        "assura check --format codex-hook .",
        "assura check --format opencode .",
        "uses: assura/assura-action@v1",
        "assura check --maturity --strict .",
    ] {
        checks.require(
            forbidden_surface_hits(sample),
            format!("stale-surface self-test failed to reject {sample:?}"),
        );
    }
    for sample in [
        "assura check --format agent --agent codex . --warn",
        "assura check --format json .",
        "<assura-feedback>valid payload marker</assura-feedback>",
        "assura-linux-amd64.tar.gz",
    ] {
        checks.require(
            !forbidden_surface_hits(sample),
            format!("stale-surface self-test rejected valid text {sample:?}"),
        );
    }

    let scan_roots = [
        "README.md",
        ".agents/skills",
        "website/src/content",
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/workflows",
        "docs/validation.md",
        "integrations/agents/README.md",
        "integrations/agents/codex/README.md",
        "integrations/agents/codex/package.json",
    ];
    for scan_root in scan_roots {
        for path in collect_files(scan_root, None) {
            let suffix = path.extension().and_then(OsStr::to_str).unwrap_or("");
            if !["json", "md", "mdx", "astro", "yml", "yaml"].contains(&suffix) {
                continue;
            }
            let text = read(&path);
            if forbidden_surface_hits(&text) {
                checks.add(format!("{}: forbidden stale command surface", rel(&path)));
            }
        }
    }

    for manifest in ["integrations/agents/codex/package.json"] {
        if !exists(manifest) {
            continue;
        }
        let Ok(data) = serde_json::from_str::<Value>(&read(manifest)) else {
            checks.add(format!("{manifest}: invalid package manifest"));
            continue;
        };
        let names = match data.get("bin") {
            Some(Value::String(_)) => vec![data
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("package")],
            Some(Value::Object(map)) => map.keys().map(String::as_str).collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        for bin_name in names {
            if bin_name.starts_with("assura-") && bin_name != "assura-full" {
                checks.add(format!(
                    "{manifest}: forbidden per-agent CLI bin {bin_name:?}"
                ));
            }
        }
    }
}

fn forbidden_surface_hits(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let agent_names = [
        "codex", "claude", "cursor", "opencode", "gemini", "copilot", "qoder", "droid", "pi",
    ];
    agent_names
        .iter()
        .any(|agent| lower.contains(&format!("assura-{agent}-feedback")))
        || lower.contains("--format codex-hook")
        || lower.contains("--format claude-hook")
        || lower.contains("--format cursor-hook")
        || lower.contains("--format opencode")
        || lower.contains("assura/assura-action")
        || lower.contains("assura check --maturity")
        || lower.contains("assura check --strict")
        || lower.contains("--require-frontmatter")
}

fn check_release_surface_claims(checks: &mut Checks) {
    let cargo_text = read("Cargo.toml");
    let description = toml_string_value(&cargo_text, "description").unwrap_or_default();
    if unsupported_release_claim_hits(&description) {
        checks.add("Cargo.toml: package description contains unsupported release claim");
    }
    if cargo_text
        .lines()
        .find(|line| line.trim_start().starts_with("keywords"))
        .is_some_and(|line| line.contains("\"dependencies\""))
    {
        checks.add("Cargo.toml: keywords must not imply dependency graph validation support");
    }

    let args_text = read("src/cli/args.rs");
    if let Some(about) = args_text
        .split("#[command(about = \"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
    {
        if unsupported_release_claim_hits(about) {
            checks.add("src/cli/args.rs: CLI about text contains unsupported release claim");
        }
    } else {
        checks.add("src/cli/args.rs: missing CLI about text");
    }

    let lib_text = read("src/lib.rs");
    for marker in [
        "unstable internal APIs",
        "not a supported dependency graph validation release surface",
        "not a supported maturity detection release surface",
        "No internal stability guarantee applies before Assura reaches both 1.0 and",
        "10 GitHub stars",
    ] {
        checks.require(
            lib_text.contains(marker),
            format!("src/lib.rs: missing public-surface marker {marker:?}"),
        );
    }

    let support_text = read("docs/support-policy.md");
    checks.require(
        support_text.contains("Public Rust module visibility in `src/lib.rs`"),
        "docs/support-policy.md: missing Rust module visibility support-policy language",
    );
    let compatibility_text = read("docs/compatibility-and-surface.md");
    for marker in [
        "## Rust Library Surface",
        "These exports are unstable internal APIs before Assura reaches both",
        "1.0 and 10 GitHub stars",
        "Public module visibility in `src/lib.rs` does not imply release support",
    ] {
        checks.require(
            compatibility_text.contains(marker),
            format!("docs/compatibility-and-surface.md: missing public-surface marker {marker:?}"),
        );
    }
}

fn unsupported_release_claim_hits(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("dependency-aware")
        || lower.contains("circular dependency detection")
        || lower.contains("dependency graph validation")
        || lower.contains("maturity detection")
}

fn check_audit_artifact(checks: &mut Checks) {
    checks.require(
        exists(AUDIT),
        format!("{AUDIT}: target-state audit is missing"),
    );
    if !exists(AUDIT) {
        return;
    }

    let text = read(AUDIT);
    for section in [
        "## Repo Inventory",
        "## Source-of-Truth Classification",
        "## Backlog And Detector Ownership",
        "## Deterministic Detection Strategy",
    ] {
        checks.require(
            text.contains(section),
            format!("{AUDIT}: missing section {section:?}"),
        );
    }
    for label in [
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
    ] {
        checks.require(
            text.lines()
                .any(|line| line.starts_with(&format!("| {label} |"))),
            format!("{AUDIT}: repo inventory missing {label:?}"),
        );
    }
    let backlog_header = "| Priority | Concrete Finding | Affected Files/Surfaces | Expected Target State | Remediation Action | Deterministic Detector | Owner |";
    checks.require(
        text.contains(backlog_header),
        format!("{AUDIT}: backlog table has wrong header"),
    );
    let backlog = text
        .split("## Backlog And Detector Ownership")
        .nth(1)
        .unwrap_or("");
    let p0_rows = backlog
        .lines()
        .filter(|line| {
            line.starts_with("| P0 |")
                && !line.to_ascii_lowercase().contains("human review required")
        })
        .collect::<Vec<_>>();
    checks.require(
        p0_rows.len() >= 5,
        format!("{AUDIT}: expected at least five P0 detector rows"),
    );
    for row in p0_rows {
        let columns = row
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if columns.len() != 7 {
            checks.add(format!("{AUDIT}: malformed P0 backlog row {row:?}"));
            continue;
        }
        let detector = columns[5].to_ascii_lowercase();
        checks.require(
            !detector.is_empty()
                && !["none", "tbd", "todo", "human review"].contains(&detector.as_str()),
            format!("{AUDIT}: P0 row lacks deterministic detector: {row}"),
        );
    }
}

fn check_command_surface_support(checks: &mut Checks) {
    let classified_text = [
        read("docs/support-policy.md"),
        read("docs/compatibility-and-surface.md"),
        read("docs/release-notes.md"),
        read("docs/validation.md"),
    ]
    .join("\n");
    let command_surface = read(".assura/command-surface.yml");
    let commands = command_surface
        .lines()
        .filter_map(|line| line.split("- name: \"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .collect::<Vec<_>>();
    checks.require(
        !commands.is_empty(),
        ".assura/command-surface.yml: no command names found",
    );
    for command in commands {
        let mut candidates = vec![command.to_string()];
        if command == "assura status" {
            candidates.push("assura status --format json".to_string());
        }
        if command.starts_with("assura hooks ") {
            candidates.push("assura hooks".to_string());
        }
        let classified = candidates
            .iter()
            .any(|candidate| classified_text.contains(&format!("`{candidate}`")));
        checks.require(
            classified,
            format!("{command}: command is not classified in support/release docs"),
        );
    }
}

fn check_manifest_semantics(checks: &mut Checks) {
    let cargo_text = read("Cargo.toml");
    for field in [
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
    ] {
        checks.require(
            toml_string_value(&cargo_text, field).is_some(),
            format!("Cargo.toml: missing package.{field}"),
        );
    }
    let version = toml_string_value(&cargo_text, "version").unwrap_or_default();
    checks.require(
        semver_like(&version),
        "Cargo.toml: package.version must be SemVer-like",
    );
    checks.require(
        toml_string_value(&cargo_text, "default-run").as_deref() == Some("assura"),
        "Cargo.toml: default-run must be assura",
    );
    let expected_members = BTreeSet::from([
        ".".to_string(),
        "crates/assura-check-cli".to_string(),
        "crates/assura-stable-hash".to_string(),
        "xtask".to_string(),
    ]);
    checks.require(
        toml_array_value(&cargo_text, "members") == expected_members,
        "Cargo.toml: workspace members drifted",
    );
    checks.require(
        toml_array_value(&cargo_text, "default-members") == expected_members,
        "Cargo.toml: workspace default-members must include all current members",
    );

    for manifest in [
        "crates/assura-check-cli/Cargo.toml",
        "crates/assura-stable-hash/Cargo.toml",
    ] {
        let internal_text = read(manifest);
        for field in [
            "name",
            "version",
            "edition",
            "description",
            "license",
            "rust-version",
        ] {
            checks.require(
                toml_string_value(&internal_text, field).is_some(),
                format!("{manifest}: missing package.{field}"),
            );
        }
        checks.require(
            toml_string_value(&internal_text, "version")
                == toml_string_value(&cargo_text, "version"),
            format!("{manifest}: internal crate version must match root package"),
        );
        checks.require(
            toml_string_value(&internal_text, "rust-version")
                == toml_string_value(&cargo_text, "rust-version"),
            format!("{manifest}: internal crate MSRV must match root package"),
        );
        checks.require(
            toml_bool_value(&internal_text, "publish") == Some(false),
            format!("{manifest}: internal crate must remain publish=false"),
        );
    }
}

fn toml_string_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix(&format!("{key} = "))?;
        Some(rest.trim().trim_matches('"').to_string())
    })
}

fn toml_bool_value(text: &str, key: &str) -> Option<bool> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix(&format!("{key} = "))?;
        match rest.trim() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    })
}

fn toml_array_value(text: &str, key: &str) -> BTreeSet<String> {
    text.lines()
        .find_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix(&format!("{key} = ["))?;
            let rest = rest.trim_end_matches(']');
            Some(
                rest.split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.trim_matches('"').to_string())
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn semver_like(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() >= 3
        && parts[..3]
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

fn check_test_relationships(checks: &mut Checks) {
    let test_text = [
        collect_files("tests", Some(".rs")),
        collect_files("crates/assura-check-cli/tests", Some(".rs")),
    ]
    .concat()
    .into_iter()
    .map(|path| format!("{}\n{}", rel(&path), read(&path)))
    .collect::<Vec<_>>()
    .join("\n");
    let source_text = collect_files("src", Some(".rs"))
        .into_iter()
        .map(|path| format!("{}\n{}", rel(&path), read(&path)))
        .collect::<Vec<_>>()
        .join("\n");
    let coverage: BTreeMap<&str, (Vec<&str>, Vec<&str>)> = BTreeMap::from([
        (
            "assura check",
            (
                vec!["tests/cli_check_tests.rs", "run_check", "--format"],
                vec![],
            ),
        ),
        (
            "assura check --format json",
            (
                vec!["tests/cli_command_surface_tests.rs", "\"json\""],
                vec![],
            ),
        ),
        (
            "assura check --format yaml",
            (
                vec!["tests/cli_command_surface_tests.rs", "\"yaml\""],
                vec![],
            ),
        ),
        (
            "assura check --format agent",
            (
                vec!["tests/cli_command_surface_tests.rs", "--format", "agent"],
                vec![],
            ),
        ),
        (
            "assura check --format agent --agent codex",
            (
                vec!["tests/cli_command_surface_tests.rs", "--agent", "codex"],
                vec![],
            ),
        ),
        (
            "assura init",
            (
                vec![
                    "tests/cli_command_surface_tests.rs",
                    ".arg(\"init\")",
                    "--no-git-hooks",
                ],
                vec![],
            ),
        ),
        (
            "assura status --format json",
            (
                vec![
                    "tests/real_project_agentic_feedback_tests.rs",
                    ".arg(\"status\")",
                    "\"json\"",
                ],
                vec![],
            ),
        ),
        (
            "assura migrate",
            (
                vec!["tests/ls_lint_rule_coverage_tests.rs", ".arg(\"migrate\")"],
                vec![],
            ),
        ),
        (
            "assura performance-report",
            (
                vec![
                    "tests/performance_report_contract_tests.rs",
                    "two_x_claim_status",
                ],
                vec![],
            ),
        ),
        (
            "assura hooks",
            (
                vec!["git_hooks_dir_resolves_regular_git_directory"],
                vec!["GitHooksManager"],
            ),
        ),
        (
            "assura quality plan",
            (
                vec!["plan_uses_cumulative_phase_checks", "QualityPhase::Merge"],
                vec!["QualityPlanCommandOptions"],
            ),
        ),
    ]);
    for (surface, (tests, sources)) in coverage {
        let missing_tests = tests
            .into_iter()
            .filter(|marker| !test_text.contains(marker) && !source_text.contains(marker))
            .collect::<Vec<_>>();
        checks.require(
            missing_tests.is_empty(),
            format!("{surface}: missing test coverage markers {missing_tests:?}"),
        );
        let missing_source = sources
            .into_iter()
            .filter(|marker| !source_text.contains(marker))
            .collect::<Vec<_>>();
        checks.require(
            missing_source.is_empty(),
            format!("{surface}: missing source markers {missing_source:?}"),
        );
    }

    let mut unexpected_ignored = Vec::new();
    for root in ["src", "tests", "crates/assura-check-cli/tests"] {
        for path in collect_files(root, Some(".rs")) {
            for (index, line) in read(&path).lines().enumerate() {
                if line.contains("#[ignore")
                    && !(rel(&path) == "tests/ls_lint_parity_regression_tests.rs"
                        && line.contains("manual performance audit fixture"))
                {
                    unexpected_ignored.push(format!(
                        "{}:{}: {}",
                        rel(&path),
                        index + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    checks.require(
        unexpected_ignored.is_empty(),
        format!(
            "unexpected ignored Rust tests outside audited manual fixtures: {}",
            unexpected_ignored.join(", ")
        ),
    );
}

fn check_docs_release_performance(checks: &mut Checks) {
    let version = toml_string_value(&read("Cargo.toml"), "version").unwrap_or_default();
    checks.require(
        read("docs/release-notes.md").contains(&format!("v{version}")),
        "docs/release-notes.md: release version must match Cargo.toml",
    );
    checks.require(
        read("docs/release-candidate-checklist.md").contains(&format!("v{version}")),
        "docs/release-candidate-checklist.md: tag version must match Cargo.toml",
    );

    let release_text = read("docs/release-notes.md");
    let compatibility_text = read("docs/compatibility-and-surface.md");
    let install_scripts = format!(
        "{}\n{}",
        read("website/public/install.sh"),
        read("website/public/install.ps1")
    );
    for archive in [
        "assura-linux-amd64.tar.gz",
        "assura-linux-musl-amd64.tar.gz",
        "assura-macos-arm64.tar.gz",
        "assura-macos-amd64.tar.gz",
        "assura-windows-amd64.zip",
    ] {
        checks.require(
            compatibility_text.contains(archive),
            format!("docs/compatibility-and-surface.md: missing {archive}"),
        );
        checks.require(
            release_text.contains(archive),
            format!("docs/release-notes.md: missing {archive}"),
        );
    }
    checks.require(
        install_scripts.contains("assura-linux-amd64.tar.gz")
            && install_scripts.contains("assura-windows-amd64.zip"),
        "website install scripts: expected public archive names are missing",
    );

    let Ok(bench_current) = serde_json::from_str::<Value>(&read("benches/history/current.json"))
    else {
        checks.add("benches/history/current.json: invalid JSON");
        return;
    };
    let Ok(website_current) =
        serde_json::from_str::<Value>(&read("website/public/data/performance/current.json"))
    else {
        checks.add("website/public/data/performance/current.json: invalid JSON");
        return;
    };
    checks.require(
        bench_current == website_current,
        "performance current.json drift: benches/history and website/public data must match",
    );
    for field in [
        "schema_version",
        "timestamp",
        "claim_summary",
        "warm_claim_summary",
        "results",
    ] {
        checks.require(
            bench_current.get(field).is_some(),
            format!("performance current.json: missing {field}"),
        );
    }
    checks.require(
        bench_current.get("schema_version").and_then(Value::as_str)
            == Some("assura.performance.v1"),
        "performance current.json: unexpected schema_version",
    );
}

fn check_agent_workflow_state(checks: &mut Checks) {
    let branch = command_stdout("git", ["branch", "--show-current"]).unwrap_or_default();
    let branch = branch.trim();
    let active_tasks = direct_task_files()
        .into_iter()
        .filter_map(|path| {
            let task = serde_json::from_str::<Value>(&read(&path)).ok()?;
            let status = task.get("status").and_then(Value::as_str)?;
            (status == "planning" || status == "in_progress").then_some((path, task))
        })
        .collect::<Vec<_>>();
    if active_tasks.is_empty() {
        let status = command_stdout("git", ["status", "--short"]).unwrap_or_default();
        checks.require(
            status.trim().is_empty(),
            "workflow gate: no active task is acceptable only for a clean repo state",
        );
        return;
    }
    let branch_owned = active_tasks
        .iter()
        .any(|(_, task)| task.get("branch").and_then(Value::as_str) == Some(branch));
    checks.require(
        branch_owned,
        "workflow gate: active task branch must match current branch",
    );
    let has_prd = active_tasks.iter().any(|(path, task)| {
        task.get("branch").and_then(Value::as_str) == Some(branch)
            && path
                .parent()
                .is_some_and(|task_dir| task_dir.join("prd.md").exists())
    });
    checks.require(has_prd, "workflow gate: active task needs prd.md");
}

fn direct_task_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(".trellis/tasks") else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path().join("task.json");
        if path.exists() {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn check_root_tooling_boundary(checks: &mut Checks) {
    checks.require(!exists("package.json"), "package.json: root Node manifest must not be used for Assura quality gates; keep Node under website/ or integrations/");
    checks.require(
        !exists("package-lock.json"),
        "package-lock.json: root npm lockfile must not exist",
    );
    checks.require(
        !exists("scripts/verify-target-state.py"),
        "scripts/verify-target-state.py: target-state checks must live in cargo xtask",
    );

    for path in active_root_quality_files() {
        if !exists(&path) {
            continue;
        }
        let text = read(&path);
        if text.contains("node --run verify") {
            checks.add(format!(
                "{path}: root validation docs/config must use cargo xtask, not node --run verify"
            ));
        }
        if text.contains("scripts/verify-target-state.py") {
            checks.add(format!("{path}: root validation docs/config must not reference the removed Python target-state verifier"));
        }
    }
}

fn active_root_quality_files() -> Vec<String> {
    let mut paths = vec![
        "AGENTS.md".to_string(),
        ".assura/config.yml".to_string(),
        ".github/PULL_REQUEST_TEMPLATE.md".to_string(),
        "README.md".to_string(),
        ".trellis/spec/assura/tooling-stabilization.md".to_string(),
        "docs/validation.md".to_string(),
        "docs/release-candidate-checklist.md".to_string(),
        "docs/release-notes.md".to_string(),
        "docs/github-setup.md".to_string(),
        "docs/project-memories.md".to_string(),
        "docs/analysis/review-record-template.md".to_string(),
        AUDIT.to_string(),
        "scripts/ci-scope.sh".to_string(),
        "scripts/check-ci-scope.sh".to_string(),
    ];
    paths.extend(
        collect_files(".github/workflows", Some(".yml"))
            .into_iter()
            .map(|path| rel(&path)),
    );
    paths
}

fn check_lint_suppression_reasons(checks: &mut Checks) {
    for root in ["src", "crates", "tests", "benches"] {
        for path in collect_files(root, Some(".rs")) {
            let text = read(&path);
            let lines = text.lines().collect::<Vec<_>>();
            for (index, line) in lines.iter().enumerate() {
                if !(line.contains("#[allow(") || line.contains("#![allow(")) {
                    continue;
                }
                let start = index.saturating_sub(3);
                let context = lines[start..=index].join("\n");
                checks.require(
                    context.contains("allow-reason:"),
                    format!(
                        "{}:{}: lint suppression needs an allow-reason comment",
                        rel(&path),
                        index + 1
                    ),
                );
            }
        }
    }
}
