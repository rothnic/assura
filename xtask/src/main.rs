//! Rust-first repository maintenance entrypoint.

mod website_demo;

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

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
        "release-readiness" => run_release_readiness(&rest),
        "perf-vps-ls-lint-compare" => run_perf_vps_ls_lint_compare(&rest),
        "performance-no-slower" => run_performance_no_slower(&rest),
        "native-performance-no-regression" => run_native_performance_no_regression(&rest),
        "warm-loop-benchmark" => run_warm_loop_benchmark(&rest),
        "warm-loop-no-regression" => run_warm_loop_no_regression(&rest),
        "website-demo-data" => website_demo::run(&rest),
        "website-config-examples" => website_demo::validate_config_examples(&rest),
        "markdown-engine-probe" => run_markdown_engine_probe(&rest),
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
        "Usage: cargo xtask <fast|check|test|evidence|target-state|hygiene|docs|release-size|release-smoke|release-live|release-readiness|perf-vps-ls-lint-compare|performance-no-slower|native-performance-no-regression|warm-loop-benchmark|warm-loop-no-regression|website-demo-data|website-config-examples|markdown-engine-probe|changed|pr|full>"
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
    website_demo::run(&["--check".to_string()])?;
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

fn run_release_readiness(args: &[String]) -> Result<()> {
    let mut format = "text";
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --format".into());
                };
                format = value;
                index += 2;
            }
            other => return Err(format!("unknown release-readiness option: {other}").into()),
        }
    }
    if !matches!(format, "json" | "text") {
        return Err(format!("unsupported release-readiness format: {format}").into());
    }

    let report = release_readiness_report();
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let verdict = report
            .get("verdict")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("Release readiness: {verdict}");
        if let Some(reasons) = report.get("reasons").and_then(Value::as_array) {
            for reason in reasons {
                if let Some(reason) = reason.as_str() {
                    println!("- {reason}");
                }
            }
        }
    }

    if report.get("ready").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err("release readiness failed".into())
    }
}

fn run_perf_vps_ls_lint_compare(args: &[String]) -> Result<()> {
    let args = if args.first().is_some_and(|arg| arg == "--") {
        &args[1..]
    } else {
        args
    };

    if args.is_empty() {
        eprintln!(
            "Usage: cargo xtask perf-vps-ls-lint-compare -- <label> <repo-path> [<repo-path>...]"
        );
        eprintln!("       cargo xtask perf-vps-ls-lint-compare -- --help");
        std::process::exit(2);
    }

    run_command(
        "./scripts/perf-vps-ls-lint-compare.sh",
        args.iter().map(String::as_str),
    )
}

fn release_readiness_report() -> Value {
    let local_version = toml_string_value(&read("Cargo.toml"), "version").unwrap_or_default();
    let release_notes_text = read("docs/release-notes.md");
    let release_notes_version = release_notes_version(&release_notes_text).unwrap_or_default();
    release_readiness_report_from_inputs(
        &local_version,
        &release_notes_version,
        &read("docs/support-policy.md"),
        &read("docs/release-candidate-checklist.md"),
        &read("docs/compatibility-and-surface.md"),
        release_surfaces_report(
            "docs/data/release-surfaces.json",
            Some(&format!("v{local_version}")),
        ),
        latest_github_release(),
    )
}

fn release_readiness_report_from_inputs(
    local_version: &str,
    release_notes_version: &str,
    support_policy_text: &str,
    release_checklist_text: &str,
    compatibility_text: &str,
    release_surfaces: Value,
    latest_release: Value,
) -> Value {
    let local_tag = format!("v{local_version}");
    let mut reasons = Vec::new();
    let mut missing_checklist_items = Vec::new();
    for required in [
        "cargo fmt --all -- --check",
        "cargo test --all-targets --quiet",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo xtask release-readiness --format json",
        "cargo xtask release-smoke",
        "cargo xtask release-live",
    ] {
        if !release_checklist_text.contains(required) {
            missing_checklist_items.push(required.to_string());
        }
    }
    if !missing_checklist_items.is_empty() {
        reasons.push("release checklist is missing required gates".to_string());
    }

    if release_notes_version != local_version {
        reasons.push(format!(
            "release notes version {release_notes_version:?} does not match Cargo.toml version {local_version:?}"
        ));
    }
    if !support_policy_text.contains("A release PR cannot close if") {
        reasons.push("support policy is missing release PR blocking criteria".to_string());
    }
    if !compatibility_text.contains("Compatibility And Public Surface") {
        reasons.push("compatibility matrix is missing release surface source of truth".to_string());
    }

    let unreleased_user_facing_changes = release_surfaces
        .get("unreleased_user_facing_changes")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    if let Some(error) = release_surfaces.get("error").and_then(Value::as_str) {
        reasons.push(format!(
            "release surface manifest could not be checked: {error}"
        ));
    }
    if let Some(error) = latest_release.get("error").and_then(Value::as_str) {
        reasons.push(format!(
            "latest GitHub release could not be checked: {error}"
        ));
    }
    let latest_tag = latest_release
        .get("tagName")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if latest_tag == local_tag
        && unreleased_user_facing_changes
            .as_array()
            .map(|changes| !changes.is_empty())
            .unwrap_or(false)
    {
        reasons.push(format!(
            "{local_tag} is already the latest GitHub release but current branch release notes describe unreleased user-facing changes"
        ));
    }

    let ready = reasons.is_empty();
    serde_json::json!({
        "schema_version": "assura.release-readiness.v1",
        "latest_github_release": latest_release,
        "local_package_version": local_version,
        "local_tag": local_tag,
        "release_notes_version": release_notes_version,
        "unreleased_user_facing_changes": unreleased_user_facing_changes,
        "release_surfaces": release_surfaces,
        "missing_checklist_items": missing_checklist_items,
        "ready": ready,
        "verdict": if ready { "pass" } else { "fail" },
        "reasons": reasons,
    })
}

fn latest_github_release() -> Value {
    match Command::new("gh")
        .args([
            "release",
            "view",
            "--json",
            "tagName,publishedAt,name,isDraft,isPrerelease,url",
        ])
        .output()
    {
        Ok(output) if output.status.success() => serde_json::from_slice::<Value>(&output.stdout)
            .unwrap_or_else(|error| serde_json::json!({ "error": error.to_string() })),
        Ok(output) => serde_json::json!({
            "error": String::from_utf8_lossy(&output.stderr).trim().to_string()
        }),
        Err(error) => serde_json::json!({ "error": error.to_string() }),
    }
}

fn release_notes_version(text: &str) -> Option<String> {
    let after_marker = text.split_once("Assura v")?.1;
    let version = after_marker
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    if version.is_empty() {
        return None;
    }
    Some(version)
}

fn release_surfaces_report(path: &str, current_tag: Option<&str>) -> Value {
    let text = read(path);
    let Ok(manifest) = serde_json::from_str::<Value>(&text) else {
        return serde_json::json!({ "error": format!("{path}: invalid JSON") });
    };
    if manifest.get("schema_version").and_then(Value::as_str) != Some("assura.release-surfaces.v1")
    {
        return serde_json::json!({ "error": format!("{path}: unexpected schema_version") });
    }
    let Some(surfaces) = manifest.get("surfaces").and_then(Value::as_array) else {
        return serde_json::json!({ "error": format!("{path}: surfaces must be an array") });
    };

    let mut errors = Vec::new();
    let mut unreleased = Vec::new();
    for surface in surfaces {
        let id = surface
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let status = surface
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let first_release = surface
            .get("first_release")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let detail_path = surface
            .get("detail_path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if id.is_empty() {
            errors.push("surface missing id".to_string());
        }
        if !matches!(
            status,
            "supported" | "experimental" | "internal" | "roadmap" | "unsupported"
        ) {
            errors.push(format!("{id}: invalid status {status:?}"));
        }
        if first_release.is_empty() {
            errors.push(format!("{id}: missing first_release"));
        }
        if matches!(status, "supported" | "experimental") && first_release != "unreleased" {
            if !release_tag_like(first_release) {
                errors.push(format!(
                    "{id}: supported or experimental surface has invalid first_release {first_release:?}"
                ));
            } else if let Some(current_tag) = current_tag {
                if release_tag_tuple(first_release) > release_tag_tuple(current_tag) {
                    errors.push(format!(
                        "{id}: first_release {first_release:?} is after local release tag {current_tag:?}"
                    ));
                }
            }
        }
        if !detail_path.is_empty() && !exists(detail_path) {
            errors.push(format!("{id}: detail_path {detail_path} does not exist"));
        }
        if matches!(status, "supported" | "experimental") && first_release == "unreleased" {
            unreleased.push(serde_json::json!({
                "id": id,
                "status": status,
                "first_release": first_release,
                "detail_path": detail_path,
            }));
        }
    }

    let mut report = serde_json::json!({
        "schema_version": "assura.release-surfaces.v1",
        "path": path,
        "surface_count": surfaces.len(),
        "unreleased_user_facing_changes": unreleased,
    });
    if !errors.is_empty() {
        report["error"] = serde_json::json!(errors.join("; "));
    }
    report
}

fn release_tag_like(value: &str) -> bool {
    release_tag_tuple(value).is_some()
}

fn release_tag_tuple(value: &str) -> Option<(u64, u64, u64)> {
    let version = value.strip_prefix('v')?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((major, minor, patch))
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
    check_extension_api_boundaries(&mut checks);
    check_document_graph_support_claims(&mut checks);
    check_post_beta_release_hardening(&mut checks);
    check_manifest_semantics(&mut checks);
    check_test_relationships(&mut checks);
    check_docs_release_performance(&mut checks);
    check_public_roadmap(&mut checks);
    check_agent_onboarding_website(&mut checks);
    check_agent_workflow_state(&mut checks);
    check_goal_revalidation_route(&mut checks);
    check_root_tooling_boundary(&mut checks);
    check_lint_suppression_reasons(&mut checks);
    checks.finish("Target-state audit checks passed.")
}

#[derive(Debug)]
struct PerformanceNoSlowerOptions {
    report_path: String,
    cohort: String,
    assura_row: String,
    ls_lint_row: String,
}

impl Default for PerformanceNoSlowerOptions {
    fn default() -> Self {
        Self {
            report_path: "benches/history/current.json".to_string(),
            cohort: "realistic-equivalent".to_string(),
            assura_row: "assura-cli".to_string(),
            ls_lint_row: "ls-lint-cli".to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct FixtureTiming {
    assura: Option<RowTiming>,
    ls_lint: Option<RowTiming>,
}

#[derive(Debug, PartialEq)]
enum RowTiming {
    Pass(f64),
    Invalid(String),
}

#[derive(Debug, PartialEq)]
enum NoSlowerFailure {
    InvalidAcceptance {
        fixture_id: String,
        reason: String,
    },
    MissingAssura {
        fixture_id: String,
    },
    MissingLsLint {
        fixture_id: String,
    },
    InvalidAssura {
        fixture_id: String,
        reason: String,
    },
    InvalidLsLint {
        fixture_id: String,
        reason: String,
    },
    Slower {
        fixture_id: String,
        assura_ms: f64,
        ls_lint_ms: f64,
    },
}

fn run_performance_no_slower(args: &[String]) -> Result<()> {
    let options = parse_performance_no_slower_options(args)?;
    let report_text = fs::read_to_string(&options.report_path)?;
    let report = serde_json::from_str::<Value>(&report_text)?;
    let failures = performance_no_slower_failures(
        &report,
        &options.cohort,
        &options.assura_row,
        &options.ls_lint_row,
    )?;

    if failures.is_empty() {
        println!(
            "Performance no-slower gate passed for accepted fixtures in cohort {} ({} <= {}).",
            options.cohort, options.assura_row, options.ls_lint_row
        );
        return Ok(());
    }

    eprintln!(
        "Performance no-slower gate failed for accepted fixtures in cohort {} ({} must be <= {}).",
        options.cohort, options.assura_row, options.ls_lint_row
    );
    for failure in failures {
        match failure {
            NoSlowerFailure::InvalidAcceptance { fixture_id, reason } => {
                eprintln!("{fixture_id}: invalid fixture acceptance: {reason}");
            }
            NoSlowerFailure::MissingAssura { fixture_id } => {
                eprintln!("{fixture_id}: missing {}", options.assura_row);
            }
            NoSlowerFailure::MissingLsLint { fixture_id } => {
                eprintln!("{fixture_id}: missing {}", options.ls_lint_row);
            }
            NoSlowerFailure::InvalidAssura { fixture_id, reason } => {
                eprintln!("{fixture_id}: invalid {} row: {reason}", options.assura_row);
            }
            NoSlowerFailure::InvalidLsLint { fixture_id, reason } => {
                eprintln!(
                    "{fixture_id}: invalid {} row: {reason}",
                    options.ls_lint_row
                );
            }
            NoSlowerFailure::Slower {
                fixture_id,
                assura_ms,
                ls_lint_ms,
            } => {
                eprintln!(
                    "{fixture_id}: {} {assura_ms:.3} ms > {} {ls_lint_ms:.3} ms",
                    options.assura_row, options.ls_lint_row
                );
            }
        }
    }
    Err("performance no-slower gate failed".into())
}

fn run_native_performance_no_regression(args: &[String]) -> Result<()> {
    let report_path = parse_native_performance_report_path(args)?;
    let report_text = fs::read_to_string(&report_path)?;
    let report = serde_json::from_str::<Value>(&report_text)?;
    let failures = native_performance_failures(&report)?;

    if failures.is_empty() {
        println!("Native performance gate passed for {report_path}.");
        return Ok(());
    }

    eprintln!("Native performance gate failed for {report_path}.");
    for failure in failures {
        eprintln!("{failure}");
    }
    Err("native performance gate failed".into())
}

fn parse_native_performance_report_path(args: &[String]) -> Result<String> {
    let mut report_path = "benches/history/native-current.json".to_string();
    for value in args {
        match value.as_str() {
            "--help" | "-h" => {
                return Err(
                    "Usage: cargo xtask native-performance-no-regression [report.json]".into(),
                );
            }
            value if value.starts_with("--") => {
                return Err(
                    format!("unknown native-performance-no-regression option: {value}").into(),
                );
            }
            value => report_path = value.to_string(),
        }
    }
    Ok(report_path)
}

const WARM_LOOP_BUDGETS: &str = "benches/history/warm-loop-budgets.v1.json";
const WARM_LOOP_CURRENT: &str = "target/performance/warm-loop-current.json";
const WARM_LOOP_MIN_ITERATIONS: usize = 20;

#[derive(Debug)]
struct WarmLoopOptions {
    binary: PathBuf,
    budgets: PathBuf,
    output: PathBuf,
    history: Option<PathBuf>,
    iterations: usize,
}

fn run_warm_loop_benchmark(args: &[String]) -> Result<()> {
    let options = parse_warm_loop_options(args)?;
    if options.iterations < WARM_LOOP_MIN_ITERATIONS {
        return Err(format!(
            "warm-loop benchmark requires at least {WARM_LOOP_MIN_ITERATIONS} iterations"
        )
        .into());
    }
    let binary = fs::canonicalize(&options.binary).map_err(|error| {
        format!(
            "warm-loop binary {} is unavailable: {error}",
            options.binary.display()
        )
    })?;
    let budgets = load_warm_loop_budgets(&options.budgets)?;
    let started_at = command_output_lossy("date", ["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .unwrap_or_else(|| "unknown".to_string());
    let mut rows = Vec::new();

    for scenario in warm_loop_scenarios() {
        let budget_ms = budgets
            .get(scenario.id)
            .copied()
            .ok_or_else(|| format!("missing warm-loop budget row {}", scenario.id))?;
        rows.push(measure_warm_loop_scenario(
            &binary,
            scenario,
            options.iterations,
            budget_ms,
        )?);
    }

    let report = serde_json::json!({
        "schema_version": "assura.warm-loop-performance.v1",
        "timestamp": started_at,
        "commit_sha": command_output_lossy("git", ["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string()),
        "branch": command_output_lossy("git", ["branch", "--show-current"]).unwrap_or_else(|| "unknown".to_string()),
        "source_worktree_dirty": command_output_lossy("git", ["status", "--porcelain"])
            .is_some_and(|status| !status.trim().is_empty()),
        "environment": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "rust_version": command_output_lossy("rustc", ["--version"]).unwrap_or_else(|| "unknown".to_string()),
        },
        "binary": options.binary,
        "iterations": options.iterations,
        "budget_source": options.budgets,
        "rows": rows,
    });
    write_pretty_json(&options.output, &report)?;
    if let Some(history) = &options.history {
        append_json_line(history, &report)?;
    }

    println!(
        "Warm-loop benchmark wrote {} measured rows to {}.",
        warm_loop_scenarios().len(),
        options.output.display()
    );
    Ok(())
}

fn run_warm_loop_no_regression(args: &[String]) -> Result<()> {
    let (report_path, budget_path) = parse_warm_loop_gate_options(args)?;
    let report = serde_json::from_str::<Value>(&fs::read_to_string(&report_path)?)?;
    let budgets = load_warm_loop_budgets(&budget_path)?;
    let failures = warm_loop_regression_failures(&report, &budgets)?;
    if failures.is_empty() {
        println!(
            "Warm-loop p95 gate passed for all {} budget rows.",
            budgets.len()
        );
        return Ok(());
    }
    eprintln!("Warm-loop p95 gate failed:");
    for failure in failures {
        eprintln!("- {failure}");
    }
    Err("warm-loop performance gate failed".into())
}

fn parse_warm_loop_options(args: &[String]) -> Result<WarmLoopOptions> {
    let mut options = WarmLoopOptions {
        binary: PathBuf::from("target/release/assura-full"),
        budgets: PathBuf::from(WARM_LOOP_BUDGETS),
        output: PathBuf::from(WARM_LOOP_CURRENT),
        history: None,
        iterations: WARM_LOOP_MIN_ITERATIONS,
    };
    let mut index = 0;
    while index < args.len() {
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("{} requires a value", args[index]))?;
        match args[index].as_str() {
            "--binary" => options.binary = PathBuf::from(value),
            "--budgets" => options.budgets = PathBuf::from(value),
            "--output" => options.output = PathBuf::from(value),
            "--history" => options.history = Some(PathBuf::from(value)),
            "--iterations" => options.iterations = value.parse()?,
            unknown => return Err(format!("unknown warm-loop benchmark option: {unknown}").into()),
        }
        index += 2;
    }
    Ok(options)
}

fn parse_warm_loop_gate_options(args: &[String]) -> Result<(PathBuf, PathBuf)> {
    let mut report = PathBuf::from(WARM_LOOP_CURRENT);
    let mut budgets = PathBuf::from(WARM_LOOP_BUDGETS);
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--budgets" => {
                budgets = PathBuf::from(args.get(index + 1).ok_or("--budgets requires a value")?);
                index += 2;
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown warm-loop gate option: {value}").into());
            }
            value => {
                report = PathBuf::from(value);
                index += 1;
            }
        }
    }
    Ok((report, budgets))
}

struct WarmLoopScenario {
    id: &'static str,
    label: &'static str,
    kind: WarmLoopScenarioKind,
}

#[derive(Clone, Copy)]
enum WarmLoopScenarioKind {
    NoChangeReview,
    OneFileChange,
    DirectoryCreateDelete,
    ConfigChange,
    AgentNudge,
}

fn warm_loop_scenarios() -> &'static [WarmLoopScenario] {
    &[
        WarmLoopScenario {
            id: "no-change-warm-review",
            label: "No-change warm review",
            kind: WarmLoopScenarioKind::NoChangeReview,
        },
        WarmLoopScenario {
            id: "one-file-change",
            label: "One-file change",
            kind: WarmLoopScenarioKind::OneFileChange,
        },
        WarmLoopScenario {
            id: "directory-create-delete",
            label: "Directory create/delete",
            kind: WarmLoopScenarioKind::DirectoryCreateDelete,
        },
        WarmLoopScenario {
            id: "config-change",
            label: "Config change",
            kind: WarmLoopScenarioKind::ConfigChange,
        },
        WarmLoopScenario {
            id: "agent-nudge",
            label: "Agent nudge",
            kind: WarmLoopScenarioKind::AgentNudge,
        },
    ]
}

fn measure_warm_loop_scenario(
    binary: &Path,
    scenario: &WarmLoopScenario,
    iterations: usize,
    budget_ms: f64,
) -> Result<Value> {
    let fixture = create_warm_loop_fixture(scenario.kind)?;
    let args = warm_loop_command(scenario.kind, &fixture);
    mutate_warm_loop_fixture(scenario.kind, &fixture, 0)?;
    run_measured_command(binary, &args)
        .map_err(|error| format!("{} warmup: {error}", scenario.id))?;
    let mut samples = Vec::with_capacity(iterations);
    for iteration in 0..iterations {
        mutate_warm_loop_fixture(scenario.kind, &fixture, iteration + 1)?;
        samples.push(run_measured_command(binary, &args)?);
    }
    samples.sort_by(|left, right| left.total_cmp(right));
    let p95_ms = percentile(&samples, 0.95).ok_or("warm-loop scenario emitted no samples")?;
    let median_ms = percentile(&samples, 0.50).ok_or("warm-loop scenario emitted no samples")?;
    let row = serde_json::json!({
        "id": scenario.id,
        "label": scenario.label,
        "command": warm_loop_display_command(scenario.kind),
        "fixture_profile": warm_loop_kind_name(scenario.kind),
        "iterations": iterations,
        "median_ms": median_ms,
        "p95_ms": p95_ms,
        "budget_ms": budget_ms,
        "within_budget": p95_ms <= budget_ms,
        "samples_ms": samples,
    });
    let _ = fs::remove_dir_all(&fixture);
    Ok(row)
}

fn create_warm_loop_fixture(kind: WarmLoopScenarioKind) -> Result<PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "assura-warm-loop-{}-{}",
        std::process::id(),
        warm_loop_kind_name(kind)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".assura"))?;
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("old"))?;
    fs::write(
        root.join(".assura/config.yml"),
        "structure:\n  ./:\n    required: false\n",
    )?;
    fs::write(root.join("src/lib.rs"), "pub fn value() -> usize { 1 }\n")?;
    fs::write(root.join("old/README.md"), "# Existing directory\n")?;
    run_in(&root, "git", ["init", "--quiet"])?;
    run_in(
        &root,
        "git",
        ["config", "user.email", "benchmark@assura.dev"],
    )?;
    run_in(&root, "git", ["config", "user.name", "Assura Benchmark"])?;
    run_in(&root, "git", ["add", "."])?;
    run_in(&root, "git", ["commit", "--quiet", "-m", "baseline"])?;

    Ok(root)
}

fn mutate_warm_loop_fixture(
    kind: WarmLoopScenarioKind,
    root: &Path,
    iteration: usize,
) -> Result<()> {
    let variant = iteration % 2;
    match kind {
        WarmLoopScenarioKind::NoChangeReview => {}
        WarmLoopScenarioKind::OneFileChange | WarmLoopScenarioKind::AgentNudge => {
            fs::write(
                root.join("src/lib.rs"),
                format!("pub fn value() -> usize {{ {} }}\n", variant + 2),
            )?;
        }
        WarmLoopScenarioKind::DirectoryCreateDelete => {
            let _ = fs::remove_dir_all(root.join("old"));
            let _ = fs::remove_dir_all(root.join("new"));
            let directory = if variant == 0 {
                root.join("new/nested")
            } else {
                root.join("old")
            };
            fs::create_dir_all(&directory)?;
            fs::write(directory.join("README.md"), "# Changed directory\n")?;
        }
        WarmLoopScenarioKind::ConfigChange => {
            fs::write(
                root.join(".assura/config.yml"),
                format!(
                    "structure:\n  ./:\n    required: false\n# warm-loop config variant {variant}\n"
                ),
            )?;
        }
    }
    Ok(())
}

fn warm_loop_command(kind: WarmLoopScenarioKind, fixture: &Path) -> Vec<String> {
    let path = fixture.to_string_lossy().into_owned();
    match kind {
        WarmLoopScenarioKind::AgentNudge => vec![
            "agent".to_string(),
            "nudge".to_string(),
            path,
            "--event".to_string(),
            "after-tool".to_string(),
            "--changed".to_string(),
            "src/lib.rs".to_string(),
            "--agent".to_string(),
            "codex".to_string(),
            "--cooldown-seconds".to_string(),
            "0".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
        _ => vec![
            "review".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--base".to_string(),
            "HEAD".to_string(),
            path,
        ],
    }
}

fn warm_loop_kind_name(kind: WarmLoopScenarioKind) -> &'static str {
    match kind {
        WarmLoopScenarioKind::NoChangeReview => "no-change",
        WarmLoopScenarioKind::OneFileChange => "one-file",
        WarmLoopScenarioKind::DirectoryCreateDelete => "directory-change",
        WarmLoopScenarioKind::ConfigChange => "config-change",
        WarmLoopScenarioKind::AgentNudge => "agent-nudge",
    }
}

fn warm_loop_display_command(kind: WarmLoopScenarioKind) -> &'static str {
    match kind {
        WarmLoopScenarioKind::AgentNudge => {
            "assura-full agent nudge <fixture> --event after-tool --changed src/lib.rs --agent codex --cooldown-seconds 0 --format json"
        }
        _ => "assura-full review --format json --base HEAD <fixture>",
    }
}

fn run_measured_command(binary: &Path, args: &[String]) -> Result<f64> {
    let started = Instant::now();
    let output = Command::new(binary).args(args).output()?;
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    if !output.status.success() {
        return Err(format!(
            "{} exited {:?}: {}",
            shell_join(binary, args),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(elapsed)
}

fn run_in<const N: usize>(directory: &Path, program: &str, args: [&str; N]) -> Result<()> {
    let output = Command::new(program)
        .current_dir(directory)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "{program} failed in {}: {}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

fn shell_join(binary: &Path, args: &[String]) -> String {
    std::iter::once(binary.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_warm_loop_budgets(path: &Path) -> Result<BTreeMap<String, f64>> {
    let value = serde_json::from_str::<Value>(&fs::read_to_string(path)?)?;
    if value.get("schema_version").and_then(Value::as_str) != Some("assura.warm-loop-budgets.v1") {
        return Err(format!("{}: unexpected warm-loop budget schema", path.display()).into());
    }
    if value.get("minimum_iterations").and_then(Value::as_u64)
        != Some(WARM_LOOP_MIN_ITERATIONS as u64)
    {
        return Err(format!(
            "{}: minimum_iterations must be {WARM_LOOP_MIN_ITERATIONS}",
            path.display()
        )
        .into());
    }
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("warm-loop budgets must contain rows")?;
    let mut budgets = BTreeMap::new();
    for row in rows {
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .ok_or("budget row missing id")?;
        let budget = row
            .get("p95_budget_ms")
            .and_then(Value::as_f64)
            .filter(|value| *value > 0.0)
            .ok_or_else(|| format!("{id}: invalid p95_budget_ms"))?;
        if budgets.insert(id.to_string(), budget).is_some() {
            return Err(format!("duplicate warm-loop budget row: {id}").into());
        }
    }
    Ok(budgets)
}

fn warm_loop_regression_failures(
    report: &Value,
    budgets: &BTreeMap<String, f64>,
) -> Result<Vec<String>> {
    if report.get("schema_version").and_then(Value::as_str)
        != Some("assura.warm-loop-performance.v1")
    {
        return Err("unexpected warm-loop report schema".into());
    }
    let rows = report
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("warm-loop report must contain rows")?;
    let mut failures = Vec::new();
    let mut seen = BTreeSet::new();
    if rows.len() != budgets.len() {
        failures.push(format!(
            "row count {} does not match budget count {}",
            rows.len(),
            budgets.len()
        ));
    }
    for row in rows {
        if let Some(id) = row.get("id").and_then(Value::as_str) {
            if !seen.insert(id) {
                failures.push(format!("{id}: duplicate measured row"));
            }
        }
    }
    for (id, budget) in budgets {
        let Some(row) = rows
            .iter()
            .find(|row| row.get("id").and_then(Value::as_str) == Some(id))
        else {
            failures.push(format!("{id}: missing measured row"));
            continue;
        };
        let Some(p95) = row.get("p95_ms").and_then(Value::as_f64) else {
            failures.push(format!("{id}: missing p95_ms"));
            continue;
        };
        let iterations = row.get("iterations").and_then(Value::as_u64).unwrap_or(0);
        if iterations < WARM_LOOP_MIN_ITERATIONS as u64 {
            failures.push(format!(
                "{id}: fewer than {WARM_LOOP_MIN_ITERATIONS} measured iterations"
            ));
        }
        let samples = row
            .get("samples_ms")
            .and_then(Value::as_array)
            .map(|samples| samples.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
            .unwrap_or_default();
        if samples.len() != iterations as usize {
            failures.push(format!(
                "{id}: sample count {} does not match iterations {iterations}",
                samples.len()
            ));
        } else {
            let mut sorted = samples;
            sorted.sort_by(|left, right| left.total_cmp(right));
            let derived_p95 = percentile(&sorted, 0.95).unwrap_or_default();
            let derived_median = percentile(&sorted, 0.50).unwrap_or_default();
            let reported_median = row
                .get("median_ms")
                .and_then(Value::as_f64)
                .unwrap_or(f64::NAN);
            if (derived_p95 - p95).abs() > 0.000_001 {
                failures.push(format!(
                    "{id}: reported p95 {p95:.6} does not match samples {derived_p95:.6}"
                ));
            }
            if !reported_median.is_finite() || (derived_median - reported_median).abs() > 0.000_001
            {
                failures.push(format!(
                    "{id}: reported median does not match samples {derived_median:.6}"
                ));
            }
        }
        if let Some(scenario) = warm_loop_scenarios()
            .iter()
            .find(|scenario| scenario.id == id)
        {
            let expected = warm_loop_display_command(scenario.kind);
            if row.get("command").and_then(Value::as_str) != Some(expected) {
                failures.push(format!("{id}: command does not match `{expected}`"));
            }
        }
        if p95 > *budget {
            failures.push(format!(
                "{id}: p95 {p95:.3} ms exceeds budget {budget:.3} ms"
            ));
        }
    }
    Ok(failures)
}

fn write_pretty_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn append_json_line(path: &Path, value: &Value) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

#[derive(Default)]
struct MarkdownEngineProbeOptions {
    candidate: Option<String>,
    fixture: Option<String>,
    run_external: bool,
    measure: bool,
    iterations: Option<usize>,
}

#[derive(Debug)]
struct MarkdownEngineProbeFixture {
    name: String,
    root: PathBuf,
    markdown_scope: PathBuf,
    relative_root: String,
    relative_markdown_scope: String,
}

fn run_markdown_engine_probe(args: &[String]) -> Result<()> {
    let options = parse_markdown_engine_probe_options(args)?;
    let root = repo_root();
    let fixture_root = root.join("tests/fixtures/markdown_engine_candidates");
    let matrix_path = fixture_root.join("matrix.json");
    let matrix_text = fs::read_to_string(&matrix_path)?;
    let matrix = serde_json::from_str::<Value>(&matrix_text)?;
    if matrix.get("schema_version").and_then(Value::as_str)
        != Some("assura.markdown-engine-candidate-fixtures.v1")
    {
        return Err(format!("{}: unexpected schema_version", matrix_path.display()).into());
    }

    let fixture = markdown_engine_probe_fixture(&root, &fixture_root, &matrix, &options)?;
    let markdown_files = collect_files(&fixture.markdown_scope, Some(".md"))
        .into_iter()
        .map(|path| rel_from_root(&root, &path))
        .collect::<Vec<_>>();

    let mut candidates = Vec::new();
    for candidate in markdown_engine_candidates() {
        if options
            .candidate
            .as_deref()
            .is_some_and(|name| name != candidate.name)
        {
            continue;
        }
        candidates.push(probe_markdown_candidate(
            candidate,
            &root,
            &fixture,
            &markdown_files,
            &options,
        ));
    }

    if options.candidate.is_some() && candidates.is_empty() {
        return Err("unknown markdown engine candidate".into());
    }

    let report = serde_json::json!({
        "schema": "assura.markdown-engine-probe.v1",
        "fixture_root": rel_from_root(&root, &fixture_root),
        "matrix": rel_from_root(&root, &matrix_path),
        "fixture": fixture.name,
        "fixture_path": fixture.relative_root,
        "markdown_scope": fixture.relative_markdown_scope,
        "run_external": options.run_external,
        "measure": options.measure,
        "iterations": markdown_engine_probe_iterations(&options),
        "markdown_files": markdown_files,
        "candidates": candidates,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_markdown_engine_probe_options(args: &[String]) -> Result<MarkdownEngineProbeOptions> {
    let mut options = MarkdownEngineProbeOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--candidate" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --candidate".into());
                };
                options.candidate = Some(value.clone());
                index += 2;
            }
            "--fixture" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --fixture".into());
                };
                options.fixture = Some(value.clone());
                index += 2;
            }
            "--run-external" => {
                options.run_external = true;
                index += 1;
            }
            "--measure" => {
                options.measure = true;
                index += 1;
            }
            "--iterations" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --iterations".into());
                };
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| "--iterations must be a positive integer")?;
                if parsed == 0 {
                    return Err("--iterations must be greater than zero".into());
                }
                options.iterations = Some(parsed);
                index += 2;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: cargo xtask markdown-engine-probe [--candidate <name>] [--fixture <name>] [--run-external] [--measure] [--iterations <n>]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown markdown-engine-probe option: {other}").into()),
        }
    }
    Ok(options)
}

fn markdown_engine_probe_fixture(
    root: &Path,
    fixture_root: &Path,
    matrix: &Value,
    options: &MarkdownEngineProbeOptions,
) -> Result<MarkdownEngineProbeFixture> {
    let fixture_name = options.fixture.as_deref().unwrap_or("invalid");
    let fixture_value = matrix
        .get("probe_profiles")
        .and_then(|profiles| profiles.get(fixture_name))
        .or_else(|| {
            matrix
                .get("variants")
                .and_then(|variants| variants.get(fixture_name))
        })
        .ok_or_else(|| format!("unknown markdown engine probe fixture: {fixture_name}"))?;
    let fixture_path = fixture_value
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("markdown engine probe fixture {fixture_name}: missing path"))?;
    let markdown_scope = fixture_value
        .get("markdown_scope")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("{fixture_path}/docs"));
    let source_root = fixture_root.join(fixture_path);
    let markdown_scope_root = fixture_root.join(&markdown_scope);
    if !source_root.exists() {
        return Err(format!(
            "markdown engine probe fixture {fixture_name}: {} does not exist",
            source_root.display()
        )
        .into());
    }
    if !markdown_scope_root.exists() {
        return Err(format!(
            "markdown engine probe fixture {fixture_name}: markdown scope {} does not exist",
            markdown_scope_root.display()
        )
        .into());
    }
    let canonical_source_root = source_root.canonicalize().map_err(|error| {
        format!("markdown engine probe fixture {fixture_name}: canonicalize source root: {error}")
    })?;
    let canonical_markdown_scope = markdown_scope_root.canonicalize().map_err(|error| {
        format!(
            "markdown engine probe fixture {fixture_name}: canonicalize markdown scope: {error}"
        )
    })?;
    if !canonical_markdown_scope.starts_with(&canonical_source_root) {
        return Err(format!(
            "markdown engine probe fixture {fixture_name}: markdown_scope must stay under fixture path"
        )
        .into());
    }

    Ok(MarkdownEngineProbeFixture {
        name: fixture_name.to_string(),
        relative_root: rel_from_root(root, &source_root),
        relative_markdown_scope: rel_from_root(root, &markdown_scope_root),
        root: source_root,
        markdown_scope: markdown_scope_root,
    })
}

struct MarkdownEngineCandidate {
    name: &'static str,
    binary: Option<&'static str>,
    probe_args: &'static [&'static str],
    fix_args: Option<&'static [&'static str]>,
}

fn markdown_engine_candidates() -> &'static [MarkdownEngineCandidate] {
    &[
        MarkdownEngineCandidate {
            name: "assura-current",
            binary: None,
            probe_args: &[],
            fix_args: None,
        },
        MarkdownEngineCandidate {
            name: "rumdl",
            binary: Some("rumdl"),
            probe_args: &["check", "--output-format", "json", "--no-cache"],
            fix_args: Some(&["check", "--fix", "--output-format", "json", "--no-cache"]),
        },
        MarkdownEngineCandidate {
            name: "mdlint",
            binary: Some("mdlint"),
            probe_args: &["check", "--output-format", "json"],
            fix_args: Some(&["check", "--fix", "--output-format", "json"]),
        },
        MarkdownEngineCandidate {
            name: "mado",
            binary: Some("mado"),
            probe_args: &["check", "--output-format", "markdownlint"],
            fix_args: None,
        },
        MarkdownEngineCandidate {
            name: "markdownlint-cli2",
            binary: Some("markdownlint-cli2"),
            probe_args: &["--json"],
            fix_args: Some(&["--fix", "--json"]),
        },
    ]
}

fn probe_markdown_candidate(
    candidate: &MarkdownEngineCandidate,
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    markdown_files: &[String],
    options: &MarkdownEngineProbeOptions,
) -> Value {
    if candidate.name == "assura-current" {
        return probe_current_assura(root, fixture, options);
    }

    let Some(binary) = candidate.binary else {
        return serde_json::json!({
            "name": candidate.name,
            "status": "invalid_candidate",
        });
    };

    if !command_exists(binary) {
        return serde_json::json!({
            "name": candidate.name,
            "binary": binary,
            "status": "unavailable",
            "available": false,
        });
    }

    let version = command_output_lossy(binary, ["--version"]);
    if !options.run_external {
        return serde_json::json!({
            "name": candidate.name,
            "binary": binary,
            "status": "available_not_run",
            "available": true,
            "version": version,
            "probe_args": candidate.probe_args,
        });
    }

    let (probe_fixture_root, probe_markdown_files) =
        match prepare_external_probe_fixture(root, fixture, candidate.name, markdown_files) {
            Ok(value) => value,
            Err(error) => {
                return serde_json::json!({
                    "name": candidate.name,
                    "binary": binary,
                    "status": "probe_error",
                    "available": true,
                    "version": version,
                    "error": error,
                });
            }
        };
    let mut command = external_candidate_command(binary, candidate, root, &probe_markdown_files);
    let output = command.output();
    match output {
        Ok(output) => {
            let exit_code = output.status.code();
            let status = markdown_probe_status(exit_code);
            let mut candidate_report = serde_json::json!({
                "name": candidate.name,
                "binary": binary,
                "status": status,
                "available": true,
                "version": version,
                "exit_code": exit_code,
                "probe_fixture_root": probe_fixture_root,
                "probe_markdown_files": probe_markdown_files,
                "fix_supported": candidate.fix_args.is_some(),
                "stdout_bytes": output.stdout.len(),
                "stderr_bytes": output.stderr.len(),
                "stdout_snippet": truncate_utf8(&output.stdout, 6000),
                "stderr_snippet": truncate_utf8(&output.stderr, 6000),
            });
            if options.measure {
                candidate_report["timing"] = measure_external_candidate(
                    root,
                    fixture,
                    candidate,
                    binary,
                    markdown_files,
                    markdown_engine_probe_iterations(options),
                );
                candidate_report["fix_timing"] = measure_external_candidate_fix(
                    root,
                    fixture,
                    candidate,
                    binary,
                    markdown_files,
                    markdown_engine_probe_iterations(options),
                );
                candidate_report["fix_validation"] = validate_external_candidate_fix(
                    root,
                    fixture,
                    candidate,
                    binary,
                    markdown_files,
                );
            }
            candidate_report
        }
        Err(error) => serde_json::json!({
            "name": candidate.name,
            "binary": binary,
            "status": "probe_error",
            "available": true,
            "version": version,
            "error": error.to_string(),
        }),
    }
}

fn prepare_external_probe_fixture(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    candidate_name: &str,
    markdown_files: &[String],
) -> std::result::Result<(String, Vec<String>), String> {
    let probe_invalid_root = root
        .join("target/markdown-engine-probe")
        .join(candidate_name)
        .join(&fixture.name);
    match fs::remove_dir_all(&probe_invalid_root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("remove probe fixture: {error}")),
    }
    copy_dir_recursive(&fixture.root, &probe_invalid_root)
        .map_err(|error| format!("copy probe fixture: {error}"))?;

    let source_prefix = rel_from_root(root, &fixture.root);
    let probe_prefix = rel_from_root(root, &probe_invalid_root);
    let probe_markdown_files = markdown_files
        .iter()
        .map(|file| file.replacen(&source_prefix, &probe_prefix, 1))
        .collect::<Vec<_>>();
    Ok((probe_prefix, probe_markdown_files))
}

fn prepare_external_probe_fixture_at(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    destination: &Path,
    markdown_files: &[String],
) -> std::result::Result<(String, Vec<String>), String> {
    match fs::remove_dir_all(destination) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("remove probe fixture: {error}")),
    }
    copy_dir_recursive(&fixture.root, destination)
        .map_err(|error| format!("copy probe fixture: {error}"))?;

    let source_prefix = rel_from_root(root, &fixture.root);
    let probe_prefix = rel_from_root(root, destination);
    let probe_markdown_files = markdown_files
        .iter()
        .map(|file| file.replacen(&source_prefix, &probe_prefix, 1))
        .collect::<Vec<_>>();
    Ok((probe_prefix, probe_markdown_files))
}

fn external_candidate_command(
    binary: &str,
    candidate: &MarkdownEngineCandidate,
    root: &Path,
    markdown_files: &[String],
) -> Command {
    external_candidate_command_with_args(binary, candidate.probe_args, root, markdown_files)
}

fn external_candidate_fix_command(
    binary: &str,
    candidate: &MarkdownEngineCandidate,
    root: &Path,
    markdown_files: &[String],
) -> Option<Command> {
    candidate
        .fix_args
        .map(|args| external_candidate_command_with_args(binary, args, root, markdown_files))
}

fn external_candidate_command_with_args(
    binary: &str,
    args: &[&str],
    root: &Path,
    markdown_files: &[String],
) -> Command {
    let mut command = Command::new(binary);
    command.current_dir(root).args(args);
    for file in markdown_files {
        command.arg(file);
    }
    command
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn truncate_utf8(bytes: &[u8], max_chars: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut truncated = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        truncated.push_str("...");
    }
    truncated.trim().to_string()
}

fn probe_current_assura(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    options: &MarkdownEngineProbeOptions,
) -> Value {
    let (mut command, execution_mode) = assura_current_probe_command(root, fixture);
    let output = command.output();
    match output {
        Ok(output) => {
            let exit_code = output.status.code();
            let status = markdown_probe_status(exit_code);
            let rules = serde_json::from_slice::<Value>(&output.stdout)
                .ok()
                .and_then(|report| {
                    report
                        .get("violations")
                        .and_then(Value::as_array)
                        .map(|items| {
                            let mut rules = items
                                .iter()
                                .filter_map(|violation| {
                                    violation.get("rule").and_then(Value::as_str)
                                })
                                .map(str::to_string)
                                .collect::<Vec<_>>();
                            rules.sort();
                            rules.dedup();
                            rules
                        })
                })
                .unwrap_or_default();
            let mut current_report = serde_json::json!({
                "name": "assura-current",
                "status": status,
                "execution_mode": execution_mode,
                "exit_code": exit_code,
                "rules": rules,
                "stdout_bytes": output.stdout.len(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
            });
            if options.measure {
                current_report["timing"] = measure_assura_current(
                    root,
                    fixture,
                    markdown_engine_probe_iterations(options),
                );
                current_report["safe_fix_timing"] = measure_assura_safe_fix(
                    root,
                    fixture,
                    markdown_engine_probe_iterations(options),
                );
            }
            current_report
        }
        Err(error) => serde_json::json!({
            "name": "assura-current",
            "status": "probe_error",
            "error": error.to_string(),
        }),
    }
}

fn assura_current_base_command(root: &Path) -> (Command, &'static str) {
    let assura_binary = root
        .join("target")
        .join("debug")
        .join(format!("assura{}", env::consts::EXE_SUFFIX));
    let use_debug_binary = assura_binary.exists();
    let command = if use_debug_binary {
        let mut command = Command::new(&assura_binary);
        command.current_dir(root);
        command
    } else {
        let mut command = Command::new("cargo");
        command.current_dir(root).args(["run", "--quiet", "--"]);
        command
    };
    let execution_mode = if use_debug_binary {
        "target-debug-binary"
    } else {
        "cargo-run"
    };
    (command, execution_mode)
}

fn assura_current_probe_command(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
) -> (Command, &'static str) {
    let (mut command, execution_mode) = assura_current_base_command(root);
    command.arg("check");
    command.arg(markdown_engine_probe_fixture_path(fixture));
    command.args(["--format", "json"]);
    (command, execution_mode)
}

fn assura_current_fix_command(
    root: &Path,
    fixture_root: &Path,
    apply: bool,
) -> (Command, &'static str) {
    let (mut command, execution_mode) = assura_current_base_command(root);
    command.args(["fix", "markdown"]);
    command.arg(fixture_root);
    if apply {
        command.arg("--apply");
    } else {
        command.arg("--dry-run");
    }
    command.args(["--format", "json"]);
    (command, execution_mode)
}

fn markdown_engine_probe_fixture_path(fixture: &MarkdownEngineProbeFixture) -> PathBuf {
    fixture.root.clone()
}

fn markdown_engine_probe_iterations(options: &MarkdownEngineProbeOptions) -> usize {
    options.iterations.unwrap_or(5)
}

fn measure_assura_current(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    iterations: usize,
) -> Value {
    let mut samples_ms = Vec::new();
    let mut errors = Vec::new();
    for _ in 0..iterations {
        let (mut command, _) = assura_current_probe_command(root, fixture);
        let started = Instant::now();
        match command.output() {
            Ok(output) if expected_markdown_probe_exit(output.status.code()) => {
                samples_ms.push(elapsed_ms(started));
            }
            Ok(output) => errors.push(format!(
                "unexpected exit code {:?}: {}",
                output.status.code(),
                truncate_utf8(&output.stderr, 400)
            )),
            Err(error) => errors.push(error.to_string()),
        }
    }
    timing_report(iterations, samples_ms, errors)
}

fn measure_assura_safe_fix(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    iterations: usize,
) -> Value {
    serde_json::json!({
        "dry_run": measure_assura_safe_fix_mode(root, fixture, iterations, false),
        "apply": measure_assura_safe_fix_mode(root, fixture, iterations, true),
    })
}

fn measure_assura_safe_fix_mode(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    iterations: usize,
    apply: bool,
) -> Value {
    let mut samples_ms = Vec::new();
    let mut errors = Vec::new();
    let mode = if apply { "apply" } else { "dry-run" };
    for index in 0..iterations {
        let probe_fixture_root = root
            .join("target/markdown-engine-probe/assura-current-safe-fix")
            .join(&fixture.name)
            .join(mode)
            .join(format!("run-{index}"));
        match fs::remove_dir_all(&probe_fixture_root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                errors.push(format!("remove safe-fix fixture: {error}"));
                continue;
            }
        }
        if let Err(error) = copy_dir_recursive(&fixture.root, &probe_fixture_root) {
            errors.push(format!("copy safe-fix fixture: {error}"));
            continue;
        }

        let (mut command, _) = assura_current_fix_command(root, &probe_fixture_root, apply);
        let started = Instant::now();
        match command.output() {
            Ok(output) if output.status.success() => samples_ms.push(elapsed_ms(started)),
            Ok(output) => errors.push(format!(
                "unexpected exit code {:?}: {}",
                output.status.code(),
                truncate_utf8(&output.stderr, 400)
            )),
            Err(error) => errors.push(error.to_string()),
        }
    }
    timing_report(iterations, samples_ms, errors)
}

fn measure_external_candidate(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    candidate: &MarkdownEngineCandidate,
    binary: &str,
    markdown_files: &[String],
    iterations: usize,
) -> Value {
    let mut samples_ms = Vec::new();
    let mut errors = Vec::new();
    for _ in 0..iterations {
        let (_, probe_markdown_files) =
            match prepare_external_probe_fixture(root, fixture, candidate.name, markdown_files) {
                Ok(value) => value,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
        let mut command =
            external_candidate_command(binary, candidate, root, &probe_markdown_files);
        let started = Instant::now();
        match command.output() {
            Ok(output) if expected_markdown_probe_exit(output.status.code()) => {
                samples_ms.push(elapsed_ms(started));
            }
            Ok(output) => errors.push(format!(
                "unexpected exit code {:?}: {}",
                output.status.code(),
                truncate_utf8(&output.stderr, 400)
            )),
            Err(error) => errors.push(error.to_string()),
        }
    }
    timing_report(iterations, samples_ms, errors)
}

fn measure_external_candidate_fix(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    candidate: &MarkdownEngineCandidate,
    binary: &str,
    markdown_files: &[String],
    iterations: usize,
) -> Value {
    if candidate.fix_args.is_none() {
        return serde_json::json!({
            "supported": false,
            "reason": "candidate has no observed deterministic fix command for this probe"
        });
    }

    let mut samples_ms = Vec::new();
    let mut errors = Vec::new();
    for _ in 0..iterations {
        let (_, probe_markdown_files) =
            match prepare_external_probe_fixture(root, fixture, candidate.name, markdown_files) {
                Ok(value) => value,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
        let Some(mut command) =
            external_candidate_fix_command(binary, candidate, root, &probe_markdown_files)
        else {
            errors.push("candidate has no fix command".to_string());
            continue;
        };
        let started = Instant::now();
        match command.output() {
            Ok(output) if expected_markdown_fix_output(candidate, &output) => {
                samples_ms.push(elapsed_ms(started));
            }
            Ok(output) => errors.push(format!(
                "unexpected exit code {:?}: {}",
                output.status.code(),
                truncate_utf8(&output.stderr, 400)
            )),
            Err(error) => errors.push(error.to_string()),
        }
    }

    let mut report = timing_report(iterations, samples_ms, errors);
    report["supported"] = serde_json::json!(true);
    report["evidence_scope"] = serde_json::json!("candidate_fix_command_timing_only");
    report["post_fix_validation"] = serde_json::json!(false);
    report["success_semantics"] =
        serde_json::json!("expected_exit_status_and_no_known_fix_failure_marker");
    report
}

fn validate_external_candidate_fix(
    root: &Path,
    fixture: &MarkdownEngineProbeFixture,
    candidate: &MarkdownEngineCandidate,
    binary: &str,
    markdown_files: &[String],
) -> Value {
    if candidate.fix_args.is_none() {
        return serde_json::json!({
            "supported": false,
            "reason": "candidate has no observed deterministic fix command for this probe"
        });
    }

    let validation_root = root
        .join("target/markdown-engine-probe")
        .join(candidate.name)
        .join(&fixture.name)
        .join("fix-validation");
    let (probe_fixture_root, probe_markdown_files) =
        match prepare_external_probe_fixture_at(root, fixture, &validation_root, markdown_files) {
            Ok(value) => value,
            Err(error) => {
                return serde_json::json!({
                    "supported": true,
                    "validation_passed": false,
                    "error": error,
                });
            }
        };

    let before = match markdown_file_snapshots(root, &probe_markdown_files) {
        Ok(value) => value,
        Err(error) => {
            return serde_json::json!({
                "supported": true,
                "validation_passed": false,
                "probe_fixture_root": probe_fixture_root,
                "error": error,
            });
        }
    };

    let first_fix = run_external_candidate_fix(binary, candidate, root, &probe_markdown_files);
    let after_first = markdown_file_snapshots(root, &probe_markdown_files).unwrap_or_default();
    let second_fix = run_external_candidate_fix(binary, candidate, root, &probe_markdown_files);
    let after_second = markdown_file_snapshots(root, &probe_markdown_files).unwrap_or_default();
    let post_fix_check =
        run_external_candidate_check(binary, candidate, root, &probe_markdown_files);

    let changed_files = changed_markdown_files(&before, &after_first);
    let second_run_changed_files = changed_markdown_files(&after_first, &after_second);
    let idempotent = second_run_changed_files.is_empty();
    let frontmatter_preserved = markdown_frontmatter_preserved(&before, &after_first);
    let line_endings_preserved = markdown_line_endings_preserved(&before, &after_first);
    let fix_command_accepted = first_fix.accepted;
    let validation_passed = fix_command_accepted
        && idempotent
        && frontmatter_preserved
        && line_endings_preserved
        && post_fix_check.accepted;

    serde_json::json!({
        "supported": true,
        "evidence_scope": "candidate_fix_validation_on_isolated_copy",
        "probe_fixture_root": probe_fixture_root,
        "probe_markdown_files": probe_markdown_files,
        "validation_passed": validation_passed,
        "fix_command_accepted": fix_command_accepted,
        "accepted_first_fix_command": fix_command_accepted,
        "frontmatter_preserved": frontmatter_preserved,
        "line_endings_preserved": line_endings_preserved,
        "idempotent": idempotent,
        "second_run_idempotent": idempotent,
        "changed_files": changed_files,
        "second_run_changed_files": second_run_changed_files,
        "post_fix_check_status": markdown_probe_status(post_fix_check.exit_code),
        "post_fix_check_accepted": post_fix_check.accepted,
        "first_fix": first_fix.to_json(),
        "post_fix_check": post_fix_check.to_json(),
        "second_fix": second_fix.to_json(),
    })
}

struct MarkdownCandidateCommandResult {
    exit_code: Option<i32>,
    accepted: bool,
    stdout_bytes: usize,
    stderr_bytes: usize,
    stdout_snippet: String,
    stderr_snippet: String,
    error: Option<String>,
}

impl MarkdownCandidateCommandResult {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "exit_code": self.exit_code,
            "accepted": self.accepted,
            "stdout_bytes": self.stdout_bytes,
            "stderr_bytes": self.stderr_bytes,
            "stdout_snippet": self.stdout_snippet,
            "stderr_snippet": self.stderr_snippet,
            "error": self.error,
        })
    }
}

#[derive(Clone)]
struct MarkdownFileSnapshot {
    path: String,
    content: String,
    frontmatter: Option<String>,
    line_endings: &'static str,
}

fn run_external_candidate_fix(
    binary: &str,
    candidate: &MarkdownEngineCandidate,
    root: &Path,
    markdown_files: &[String],
) -> MarkdownCandidateCommandResult {
    let Some(mut command) = external_candidate_fix_command(binary, candidate, root, markdown_files)
    else {
        return MarkdownCandidateCommandResult {
            exit_code: None,
            accepted: false,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdout_snippet: String::new(),
            stderr_snippet: String::new(),
            error: Some("candidate has no fix command".to_string()),
        };
    };
    match command.output() {
        Ok(output) => MarkdownCandidateCommandResult {
            exit_code: output.status.code(),
            accepted: expected_markdown_fix_output(candidate, &output),
            stdout_bytes: output.stdout.len(),
            stderr_bytes: output.stderr.len(),
            stdout_snippet: truncate_utf8(&output.stdout, 2000),
            stderr_snippet: truncate_utf8(&output.stderr, 2000),
            error: None,
        },
        Err(error) => MarkdownCandidateCommandResult {
            exit_code: None,
            accepted: false,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdout_snippet: String::new(),
            stderr_snippet: String::new(),
            error: Some(error.to_string()),
        },
    }
}

fn run_external_candidate_check(
    binary: &str,
    candidate: &MarkdownEngineCandidate,
    root: &Path,
    markdown_files: &[String],
) -> MarkdownCandidateCommandResult {
    let mut command = external_candidate_command(binary, candidate, root, markdown_files);
    match command.output() {
        Ok(output) => MarkdownCandidateCommandResult {
            exit_code: output.status.code(),
            accepted: expected_markdown_probe_exit(output.status.code()),
            stdout_bytes: output.stdout.len(),
            stderr_bytes: output.stderr.len(),
            stdout_snippet: truncate_utf8(&output.stdout, 2000),
            stderr_snippet: truncate_utf8(&output.stderr, 2000),
            error: None,
        },
        Err(error) => MarkdownCandidateCommandResult {
            exit_code: None,
            accepted: false,
            stdout_bytes: 0,
            stderr_bytes: 0,
            stdout_snippet: String::new(),
            stderr_snippet: String::new(),
            error: Some(error.to_string()),
        },
    }
}

fn markdown_file_snapshots(
    root: &Path,
    markdown_files: &[String],
) -> std::result::Result<Vec<MarkdownFileSnapshot>, String> {
    let mut snapshots = Vec::new();
    for file in markdown_files {
        let path = root.join(file);
        let content =
            fs::read_to_string(&path).map_err(|error| format!("read markdown {file}: {error}"))?;
        snapshots.push(MarkdownFileSnapshot {
            path: file.clone(),
            frontmatter: leading_frontmatter_block(&content),
            line_endings: line_ending_style(&content),
            content,
        });
    }
    Ok(snapshots)
}

fn leading_frontmatter_block(content: &str) -> Option<String> {
    if !(content.starts_with("---\n") || content.starts_with("---\r\n")) {
        return None;
    }
    let newline = if content.starts_with("---\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let start = 3 + newline.len();
    let marker = format!("{newline}---{newline}");
    let empty_frontmatter_marker = format!("---{newline}");
    content
        .get(start..)
        .and_then(|rest| match rest.find(&marker) {
            Some(index) => Some(start + index + marker.len()),
            None if rest.starts_with(&empty_frontmatter_marker) => {
                Some(start + empty_frontmatter_marker.len())
            }
            None => None,
        })
        .and_then(|end| content.get(..end).map(str::to_string))
}

fn line_ending_style(content: &str) -> &'static str {
    let crlf = content.matches("\r\n").count();
    let lf = content.matches('\n').count();
    match (crlf, lf.saturating_sub(crlf)) {
        (0, 0) => "none",
        (0, _) => "lf",
        (_, 0) => "crlf",
        _ => "mixed",
    }
}

fn changed_markdown_files(
    before: &[MarkdownFileSnapshot],
    after: &[MarkdownFileSnapshot],
) -> Vec<String> {
    before
        .iter()
        .filter(|before_file| {
            match after
                .iter()
                .find(|after_file| after_file.path == before_file.path)
            {
                Some(after_file) => after_file.content != before_file.content,
                None => true,
            }
        })
        .map(|snapshot| snapshot.path.clone())
        .collect()
}

fn markdown_frontmatter_preserved(
    before: &[MarkdownFileSnapshot],
    after: &[MarkdownFileSnapshot],
) -> bool {
    before.iter().all(|before_file| {
        after
            .iter()
            .find(|after_file| after_file.path == before_file.path)
            .is_some_and(|after_file| after_file.frontmatter == before_file.frontmatter)
    })
}

fn markdown_line_endings_preserved(
    before: &[MarkdownFileSnapshot],
    after: &[MarkdownFileSnapshot],
) -> bool {
    before.iter().all(|before_file| {
        after
            .iter()
            .find(|after_file| after_file.path == before_file.path)
            .is_some_and(|after_file| after_file.line_endings == before_file.line_endings)
    })
}

fn expected_markdown_fix_output(
    candidate: &MarkdownEngineCandidate,
    output: &std::process::Output,
) -> bool {
    if !expected_markdown_probe_exit(output.status.code()) {
        return false;
    }
    if candidate.name == "mdlint" {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Failed to apply fixes") || stderr.contains("Cannot apply fixes") {
            return false;
        }
    }
    true
}

fn expected_markdown_probe_exit(exit_code: Option<i32>) -> bool {
    matches!(exit_code, Some(0) | Some(1))
}

fn markdown_probe_status(exit_code: Option<i32>) -> &'static str {
    match exit_code {
        Some(0) => "ran",
        Some(1) => "ran_with_findings",
        _ => "probe_failed",
    }
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

fn timing_report(iterations: usize, mut samples_ms: Vec<f64>, errors: Vec<String>) -> Value {
    samples_ms.sort_by(|left, right| left.total_cmp(right));
    let median_ms = percentile(&samples_ms, 0.50);
    let p95_ms = percentile(&samples_ms, 0.95);
    let min_ms = samples_ms.first().copied();
    let max_ms = samples_ms.last().copied();
    serde_json::json!({
        "iterations": iterations,
        "successful_runs": samples_ms.len(),
        "failed_runs": errors.len(),
        "median_ms": median_ms,
        "p95_ms": p95_ms,
        "min_ms": min_ms,
        "max_ms": max_ms,
        "samples_ms": samples_ms,
        "errors": errors,
    })
}

fn percentile(sorted_samples: &[f64], percentile: f64) -> Option<f64> {
    if sorted_samples.is_empty() {
        return None;
    }
    let rank = (percentile * sorted_samples.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_samples.len() - 1);
    Some(sorted_samples[index])
}

fn command_output_lossy<const N: usize>(program: &str, args: [&str; N]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|output| {
            let text = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };
            String::from_utf8_lossy(text).trim().to_string()
        })
}

fn parse_performance_no_slower_options(args: &[String]) -> Result<PerformanceNoSlowerOptions> {
    let mut options = PerformanceNoSlowerOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--cohort" => {
                index += 1;
                options.cohort = args
                    .get(index)
                    .ok_or("--cohort requires a value")?
                    .to_string();
            }
            "--assura-row" => {
                index += 1;
                options.assura_row = args
                    .get(index)
                    .ok_or("--assura-row requires a value")?
                    .to_string();
            }
            "--ls-lint-row" => {
                index += 1;
                options.ls_lint_row = args
                    .get(index)
                    .ok_or("--ls-lint-row requires a value")?
                    .to_string();
            }
            "--help" | "-h" => {
                return Err("Usage: cargo xtask performance-no-slower [report.json] [--cohort realistic-equivalent] [--assura-row assura-cli] [--ls-lint-row ls-lint-cli]".into());
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown performance-no-slower option: {value}").into());
            }
            value => {
                options.report_path = value.to_string();
            }
        }
        index += 1;
    }
    Ok(options)
}

fn performance_no_slower_failures(
    report: &Value,
    cohort: &str,
    assura_row: &str,
    ls_lint_row: &str,
) -> Result<Vec<NoSlowerFailure>> {
    let rows = report
        .get("results")
        .and_then(Value::as_array)
        .ok_or("performance report missing results array")?;
    let mut accepted_fixtures = BTreeSet::<String>::new();
    let mut timings = BTreeMap::<String, FixtureTiming>::new();
    let mut failures = Vec::new();

    for row in rows {
        if row.get("fixture_cohort").and_then(Value::as_str) != Some(cohort) {
            continue;
        }
        let fixture_id = row
            .get("fixture_id")
            .and_then(Value::as_str)
            .ok_or("performance row missing fixture_id")?;

        match accepted_fixture_row(row) {
            Ok(false) => continue,
            Ok(true) => {
                accepted_fixtures.insert(fixture_id.to_string());
            }
            Err(reason) => {
                failures.push(NoSlowerFailure::InvalidAcceptance {
                    fixture_id: fixture_id.to_string(),
                    reason: reason.to_string(),
                });
                continue;
            }
        }
        let Some(row_family) = row.get("row_family").and_then(Value::as_str) else {
            continue;
        };
        if row_family != assura_row && row_family != ls_lint_row {
            continue;
        }
        let timing = timings.entry(fixture_id.to_string()).or_default();
        if row_family == assura_row {
            timing.assura = Some(timing_from_row(row));
        } else if row_family == ls_lint_row {
            timing.ls_lint = Some(native_ls_lint_timing_from_row(row));
        }
    }
    for fixture_id in accepted_fixtures {
        timings.entry(fixture_id).or_default();
    }

    if timings.is_empty() && failures.is_empty() {
        return Err(
            format!("performance report has no accepted fixture rows for cohort {cohort}").into(),
        );
    }

    for (fixture_id, timing) in timings {
        match (timing.assura, timing.ls_lint) {
            (None, None) => {
                failures.push(NoSlowerFailure::MissingAssura {
                    fixture_id: fixture_id.clone(),
                });
                failures.push(NoSlowerFailure::MissingLsLint { fixture_id });
            }
            (None, _) => failures.push(NoSlowerFailure::MissingAssura { fixture_id }),
            (_, None) => failures.push(NoSlowerFailure::MissingLsLint { fixture_id }),
            (Some(RowTiming::Invalid(reason)), _) => {
                failures.push(NoSlowerFailure::InvalidAssura { fixture_id, reason });
            }
            (_, Some(RowTiming::Invalid(reason))) => {
                failures.push(NoSlowerFailure::InvalidLsLint { fixture_id, reason });
            }
            (Some(RowTiming::Pass(assura_ms)), Some(RowTiming::Pass(ls_lint_ms)))
                if assura_ms > ls_lint_ms =>
            {
                failures.push(NoSlowerFailure::Slower {
                    fixture_id,
                    assura_ms,
                    ls_lint_ms,
                });
            }
            (Some(RowTiming::Pass(_)), Some(RowTiming::Pass(_))) => {}
        }
    }
    Ok(failures)
}

const NATIVE_PERFORMANCE_FIXTURES: &[&str] = &[
    "native_adapter_mix",
    "native_large",
    "native_medium",
    "native_real_project",
    "native_reference_heavy",
    "native_small",
];

const NATIVE_PERFORMANCE_ROW_FAMILIES: &[&str] = &[
    "native:agent-query-keyword-search-cli",
    "native:agent-query-missing-relations-cli",
    "native:content-check-cli",
    "native:content-collections-cli",
    "native:content-expand-cli",
    "native:content-instances-cli",
    "native:content-missing-relations-cli",
    "native:content-references-cli",
    "native:content-search-cli",
    "native:content-show-cli",
    "native:context-pack-cli",
    "native:daemon-status-cli",
    "native:markdown-safe-fix-dry-run-cli",
    "native:phase:config-model-load",
    "native:phase:edge-collect",
    "native:phase:fact-ingest-load",
    "native:phase:factset-serialize-json",
    "native:phase:file-index",
    "native:phase:incremental-replace-generation",
    "native:phase:object-load-validate",
    "native:phase:reference-validate",
    "native:phase:repository-validate-total",
    "native:phase:schema-compile",
    "native:phase:warm-keyword-query",
    "native:session-agent-context-cli",
];

fn native_performance_failures(report: &Value) -> Result<Vec<String>> {
    const EPSILON_MS: f64 = 0.000_001;

    if report.get("schema_version").and_then(Value::as_str) != Some("assura.performance.v1") {
        return Ok(vec![
            "schema_version must be assura.performance.v1".to_string()
        ]);
    }
    let rows = report
        .get("results")
        .and_then(Value::as_array)
        .ok_or("performance report missing results array")?;

    let mut failures = Vec::new();
    let mut matrix = BTreeSet::<(String, String)>::new();
    let native_rows = rows
        .iter()
        .filter(|row| row.get("fixture_cohort").and_then(Value::as_str) == Some("assura-native"))
        .collect::<Vec<_>>();

    if native_rows.is_empty() {
        failures.push("report has no assura-native rows".to_string());
    }

    if report
        .get("ls_lint_package")
        .and_then(Value::as_str)
        .is_some_and(|value| value != "not-applicable")
    {
        failures.push("native report ls_lint_package must be not-applicable".to_string());
    }
    if !command_line_contains(report, "--suite", "native") {
        failures.push("native report command_line must include --suite native".to_string());
    }

    for row in native_rows {
        let fixture_id = row
            .get("fixture_id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-fixture>");
        let row_family = row
            .get("row_family")
            .and_then(Value::as_str)
            .unwrap_or("<missing-row-family>");
        let label = format!("{fixture_id} {row_family}");

        matrix.insert((fixture_id.to_string(), row_family.to_string()));

        if row.get("fixture_acceptance").and_then(Value::as_str) != Some("assura-native-diagnostic")
        {
            failures.push(format!(
                "{label}: fixture_acceptance must be assura-native-diagnostic"
            ));
        }
        if row.get("status").and_then(Value::as_str) != Some("pass") {
            let status = row
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            let details = row
                .get("details")
                .and_then(Value::as_str)
                .unwrap_or("<no details>");
            failures.push(format!("{label}: status {status}: {details}"));
        }
        let expected_assura_exit_status = row
            .get("expected_assura_exit_status")
            .and_then(Value::as_i64);
        let expected_native_status = expected_native_assura_exit_status(fixture_id, row_family);
        if expected_assura_exit_status != Some(expected_native_status) {
            failures.push(format!(
                "{label}: expected_assura_exit_status must be {expected_native_status}"
            ));
        }
        if row
            .get("median_runtime_ms")
            .and_then(Value::as_f64)
            .is_none()
        {
            failures.push(format!("{label}: missing median_runtime_ms"));
        }
        let median_runtime_ms = row.get("median_runtime_ms").and_then(Value::as_f64);
        let samples = row
            .pointer("/distribution/samples")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if samples == 0 {
            failures.push(format!(
                "{label}: distribution.samples must be greater than zero"
            ));
        }
        if row
            .get("latency_threshold_met")
            .and_then(Value::as_bool)
            .is_some_and(|met| !met)
        {
            failures.push(format!("{label}: latency threshold was not met"));
        }
        let native_regression_status = row
            .get("native_regression_status")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        let native_regression_threshold_ms = row
            .get("native_regression_threshold_ms")
            .and_then(Value::as_f64);
        let native_regression_baseline_median_ms = row
            .get("native_regression_baseline_median_ms")
            .and_then(Value::as_f64);
        let native_regression_baseline_report_count = row
            .get("native_regression_baseline_report_count")
            .and_then(Value::as_u64);
        let native_regression_baseline_sample_count = row
            .get("native_regression_baseline_sample_count")
            .and_then(Value::as_u64);
        let native_regression_delta_ms = row
            .get("native_regression_delta_ms")
            .and_then(Value::as_f64);
        if native_regression_baseline_report_count == Some(0) {
            failures.push(format!(
                "{label}: native_regression_baseline_report_count must be greater than zero when present"
            ));
        }
        if native_regression_baseline_sample_count == Some(0) {
            failures.push(format!(
                "{label}: native_regression_baseline_sample_count must be greater than zero when present"
            ));
        }
        let baseline_scope_note = match (
            native_regression_baseline_report_count,
            native_regression_baseline_sample_count,
        ) {
            (Some(report_count), Some(sample_count)) => {
                format!(" (baseline reports={report_count}, samples={sample_count})")
            }
            (Some(report_count), None) => format!(" (baseline reports={report_count})"),
            (None, Some(sample_count)) => format!(" (baseline samples={sample_count})"),
            (None, None) => String::new(),
        };
        match native_regression_status {
            "within-calibrated-baseline" | "within-provisional-baseline" => {
                if native_regression_threshold_ms.is_none() {
                    failures.push(format!("{label}: missing native_regression_threshold_ms"));
                }
                if native_regression_baseline_median_ms.is_none() {
                    failures.push(format!(
                        "{label}: missing native_regression_baseline_median_ms"
                    ));
                }
                if native_regression_delta_ms.is_none() {
                    failures.push(format!("{label}: missing native_regression_delta_ms"));
                }
                if let (Some(median_ms), Some(threshold_ms)) =
                    (median_runtime_ms, native_regression_threshold_ms)
                {
                    if median_ms > threshold_ms + EPSILON_MS {
                        failures.push(format!(
                            "{label}: native_regression_status {native_regression_status} disagrees with median_runtime_ms ({median_ms}) > native_regression_threshold_ms ({threshold_ms})"
                        ));
                    }
                }
                if let (Some(median_ms), Some(baseline_ms), Some(delta_ms)) = (
                    median_runtime_ms,
                    native_regression_baseline_median_ms,
                    native_regression_delta_ms,
                ) {
                    let expected_delta_ms = median_ms - baseline_ms;
                    if (expected_delta_ms - delta_ms).abs() > EPSILON_MS {
                        failures.push(format!(
                            "{label}: native_regression_delta_ms ({delta_ms}) does not match median_runtime_ms - native_regression_baseline_median_ms ({expected_delta_ms})"
                        ));
                    }
                }
            }
            "regressed-vs-calibrated-baseline" | "regressed-vs-provisional-baseline" => {
                if native_regression_threshold_ms.is_none() {
                    failures.push(format!("{label}: missing native_regression_threshold_ms"));
                }
                if native_regression_baseline_median_ms.is_none() {
                    failures.push(format!(
                        "{label}: missing native_regression_baseline_median_ms"
                    ));
                }
                if native_regression_delta_ms.is_none() {
                    failures.push(format!("{label}: missing native_regression_delta_ms"));
                }
                if let (Some(median_ms), Some(threshold_ms)) =
                    (median_runtime_ms, native_regression_threshold_ms)
                {
                    if median_ms <= threshold_ms + EPSILON_MS {
                        failures.push(format!(
                            "{label}: native_regression_status {native_regression_status} disagrees with median_runtime_ms ({median_ms}) <= native_regression_threshold_ms ({threshold_ms})"
                        ));
                    }
                }
                let baseline_kind =
                    if native_regression_status == "regressed-vs-provisional-baseline" {
                        "provisional checked baseline"
                    } else {
                        "checked baseline"
                    };
                failures.push(format!(
                    "{label}: native regression gate failed versus {baseline_kind}{baseline_scope_note}"
                ));
            }
            "baseline-missing" => {
                failures.push(format!("{label}: missing checked native baseline report"))
            }
            "baseline-row-missing" => failures.push(format!(
                "{label}: missing matching row in checked native baseline"
            )),
            "baseline-row-unusable" => {
                failures.push(format!("{label}: checked native baseline row was unusable"))
            }
            other => failures.push(format!(
                "{label}: unsupported native_regression_status {other}"
            )),
        }
    }

    for fixture_id in NATIVE_PERFORMANCE_FIXTURES {
        for row_family in NATIVE_PERFORMANCE_ROW_FAMILIES {
            if !matrix.contains(&(fixture_id.to_string(), row_family.to_string())) {
                failures.push(format!(
                    "{fixture_id} {row_family}: missing native matrix row"
                ));
            }
        }
    }

    Ok(failures)
}

fn expected_native_assura_exit_status(fixture_id: &str, row_family: &str) -> i64 {
    if matches!(fixture_id, "native_reference_heavy" | "native_real_project")
        && row_family == "native:content-check-cli"
    {
        1
    } else {
        0
    }
}

fn command_line_contains(report: &Value, option: &str, value: &str) -> bool {
    report
        .get("command_line")
        .and_then(Value::as_str)
        .and_then(|command_line| command_option_value(command_line, option))
        == Some(value)
}

fn accepted_fixture_row(row: &Value) -> Result<bool> {
    match row.get("fixture_acceptance").and_then(Value::as_str) {
        Some("accepted-ls-lint-equivalent") => Ok(true),
        Some("diagnostic" | "experimental" | "retired" | "assura-native-diagnostic") => Ok(false),
        Some(value) => Err(format!("unknown fixture_acceptance {value:?}").into()),
        None => Err("missing fixture_acceptance".into()),
    }
}

fn timing_from_row(row: &Value) -> RowTiming {
    if row.get("status").and_then(Value::as_str) != Some("pass") {
        let status = row
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        return RowTiming::Invalid(format!("status {status:?}"));
    }
    let Some(median_runtime_ms) = row.get("median_runtime_ms").and_then(Value::as_f64) else {
        return RowTiming::Invalid("missing numeric median_runtime_ms".to_string());
    };
    RowTiming::Pass(median_runtime_ms)
}

fn native_ls_lint_timing_from_row(row: &Value) -> RowTiming {
    if let RowTiming::Invalid(reason) = timing_from_row(row) {
        return RowTiming::Invalid(reason);
    }
    if row.get("tool_name").and_then(Value::as_str) != Some("ls-lint-native-cli") {
        let tool_name = row
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        return RowTiming::Invalid(format!("tool_name {tool_name:?} is not native LS-Lint"));
    }
    if row.get("ls_lint_execution_mode").and_then(Value::as_str)
        != Some("native-binary-from-pinned-npm-package")
    {
        let mode = row
            .get("ls_lint_execution_mode")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        return RowTiming::Invalid(format!(
            "ls_lint_execution_mode {mode:?} is not native binary"
        ));
    }
    timing_from_row(row)
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path)
        .or_else(|error| {
            if path.is_relative() {
                if let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
                    return fs::read_to_string(root.join(path));
                }
            }
            Err(error)
        })
        .unwrap_or_default()
}

fn exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists()
        || (path.is_relative()
            && Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .map(|root| root.join(path).exists())
                .unwrap_or(false))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn rel_from_root(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(path) => rel(path),
        Err(_) => rel(path),
    }
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
        "docs/release-train.md",
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
        "do not carry a pre-1.0 compatibility guarantee",
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
        "These exports are unstable internal APIs before 1.0",
        "Public module visibility in `src/lib.rs` does not imply release support",
    ] {
        checks.require(
            compatibility_text.contains(marker),
            format!("docs/compatibility-and-surface.md: missing public-surface marker {marker:?}"),
        );
    }
}

fn check_extension_api_boundaries(checks: &mut Checks) {
    let boundary_path = "docs/extension-api-boundaries.md";
    let boundary_text = read(boundary_path);
    checks.require(
        !boundary_text.trim().is_empty(),
        format!("{boundary_path}: missing canonical extension/API boundary doc"),
    );

    let support_text = read("docs/support-policy.md");
    let compatibility_text = read("docs/compatibility-and-surface.md");
    let api_text = read("website/src/content/docs/reference/api.md");
    let config_text = read("website/src/content/docs/reference/configuration.md");
    let release_readiness_text = read("website/src/content/docs/reference/release-readiness.md");
    let website_boundary_text =
        read("website/src/content/docs/reference/extension-api-boundaries.md");
    let config_notation_text = read(".trellis/spec/assura/config-notation.md");
    let release_surfaces_text = read("docs/data/release-surfaces.json");

    for marker in [
        "First-party config extension policies",
        "Supported local CLI",
        "Internal Rust APIs",
        "Deferred Public Plugin API",
        "does not currently provide a public third-party plugin API",
        "remote plugin loading",
        "shell-executed validator system",
        "plugin marketplace",
        "TypeScript plugin APIs",
        "semver-stable Rust library API",
    ] {
        checks.require(
            boundary_text.contains(marker),
            format!("{boundary_path}: missing boundary marker {marker:?}"),
        );
    }

    let extension_families = [
        "extensions.custom_constraints",
        "extensions.release_contracts",
        "extensions.support_matrices",
        "extensions.manifest_semantics",
        "extensions.test_relationships",
        "extensions.module_topologies",
        "extensions.docs_lifecycles",
        "extensions.repository_references",
        "extensions.relationships",
    ];
    for family in extension_families {
        checks.require(
            boundary_text.contains(family),
            format!("{boundary_path}: missing {family}"),
        );
        checks.require(
            support_text.contains(family),
            format!("docs/support-policy.md: missing support row for {family}"),
        );
        checks.require(
            compatibility_text.contains(&format!("config:{family}")),
            format!("docs/compatibility-and-surface.md: missing compatibility row for {family}"),
        );
        checks.require(
            config_text.contains(family),
            format!("website configuration reference: missing {family}"),
        );
    }

    for (path, text) in [
        ("docs/support-policy.md", &support_text),
        ("docs/compatibility-and-surface.md", &compatibility_text),
        ("website API reference", &api_text),
        ("website configuration reference", &config_text),
        ("website release readiness", &release_readiness_text),
        ("website extension boundary", &website_boundary_text),
        (
            ".trellis/spec/assura/config-notation.md",
            &config_notation_text,
        ),
    ] {
        checks.require(
            text.contains("Extension API Boundaries")
                || text.contains("extension-api-boundaries")
                || text.contains("extension/API boundary")
                || text.contains("extension and plugin language"),
            format!("{path}: missing link or marker for extension/API boundary"),
        );
    }

    for marker in [
        "Public plugin API or SDK",
        "Roadmap only",
        "remote plugin loading",
        "shell-executed",
        "plugin marketplaces",
        "TypeScript plugin APIs",
        "semver-stable Rust",
    ] {
        checks.require(
            support_text.contains(marker)
                || boundary_text.contains(marker)
                || website_boundary_text.contains(marker),
            format!("extension/API docs: missing unsupported plugin marker {marker:?}"),
        );
    }

    checks.require(
        release_surfaces_text.contains("\"extension-api-boundaries\"")
            && release_surfaces_text
                .contains("\"detail_path\": \"docs/extension-api-boundaries.md\""),
        "docs/data/release-surfaces.json: missing extension API boundary release surface",
    );

    for path in public_claim_files() {
        let text = read(&path);
        for (line_index, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            let mentions_plugin_api = lower.contains("public plugin api")
                || lower.contains("third-party plugin api")
                || lower.contains("remote plugin")
                || lower.contains("shell-executed")
                || lower.contains("plugin marketplace")
                || lower.contains("typescript plugin api")
                || lower.contains("semver-stable rust");
            let is_bounded = lower.contains("unsupported")
                || lower.contains("not support")
                || lower.contains("does not currently support")
                || lower.contains("not currently support")
                || lower.contains("roadmap only")
                || lower.contains("deferred")
                || lower.contains("future goal")
                || lower.contains("not a public");
            if mentions_plugin_api && lower.contains("supported") && !is_bounded {
                checks.add(format!(
                    "{}:{}: plugin/API language may imply unsupported public extension support",
                    path,
                    line_index + 1
                ));
            }
        }
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
    let support_text = read("docs/support-policy.md");
    let compatibility_text = read("docs/compatibility-and-surface.md");
    let args_text = [
        read("src/cli/args.rs"),
        read("src/cli/content_args.rs"),
        read("src/cli/daemon.rs"),
    ]
    .join("\n");
    check_cli_command_inventory(checks, &args_text);
    check_public_support_claim_consistency(checks);

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

    let mut matrix_commands = BTreeSet::new();
    for row in support_matrix_rows() {
        for command in row.command_surface_names {
            matrix_commands.insert(*command);
        }
        for marker in row.support_policy_markers {
            checks.require(
                support_text.contains(marker),
                format!("{}: support policy missing marker {marker:?}", row.surface),
            );
        }
        for marker in row.compatibility_markers {
            checks.require(
                compatibility_text.contains(marker),
                format!(
                    "{}: compatibility docs missing marker {marker:?}",
                    row.surface
                ),
            );
        }
    }

    for command in commands {
        checks.require(
            matrix_commands.contains(command),
            format!("{command}: command surface is missing from the support matrix"),
        );
    }
}

fn check_cli_command_inventory(checks: &mut Checks, args_text: &str) {
    let mut expected = BTreeMap::<&str, BTreeSet<&str>>::new();
    for row in cli_command_variant_rows() {
        expected
            .entry(row.enum_name)
            .or_default()
            .insert(row.variant_name);
        for command in row.command_surface_names {
            checks.require(
                support_matrix_rows()
                    .iter()
                    .any(|matrix_row| matrix_row.command_surface_names.contains(command)),
                format!(
                    "{}::{} maps {command:?} but the support matrix has no matching command row",
                    row.enum_name, row.variant_name
                ),
            );
        }
    }

    for (enum_name, expected_variants) in expected {
        let actual_variants = enum_variant_names(args_text, enum_name);
        checks.require(
            actual_variants == expected_variants,
            format!(
                "src/cli/args.rs: {enum_name} variants {actual_variants:?} do not match support matrix inventory {expected_variants:?}"
            ),
        );
    }
}

fn enum_variant_names<'a>(text: &'a str, enum_name: &str) -> BTreeSet<&'a str> {
    let marker = format!("pub enum {enum_name} {{");
    let Some(rest) = text.split_once(&marker).map(|(_, rest)| rest) else {
        return BTreeSet::new();
    };
    let mut depth = 1i32;
    let mut variants = BTreeSet::new();
    for line in rest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        if depth == 1
            && trimmed
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_uppercase())
        {
            let name = trimmed.split([' ', '{', ',', '(']).next().unwrap_or("");
            if !name.is_empty() {
                variants.insert(name);
            }
        }
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        if depth <= 0 {
            break;
        }
    }
    variants
}

fn check_public_support_claim_consistency(checks: &mut Checks) {
    let experimental_surfaces = [
        ("assura info", "experimental diagnostic"),
        ("assura watch", "experimental"),
    ];
    for path in public_claim_files() {
        let text = read(&path);
        for (line_index, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            for (surface, expected_level) in experimental_surfaces {
                if lower.contains(surface)
                    && lower.contains("supported")
                    && !lower.contains("unsupported")
                    && !lower.contains("not supported")
                    && !lower.contains("not yet supported")
                    && !lower.contains(expected_level)
                {
                    checks.add(format!(
                        "{}:{}: {surface} is experimental but this line claims supported status",
                        path,
                        line_index + 1
                    ));
                }
            }
        }
    }
}

fn check_document_graph_support_claims(checks: &mut Checks) {
    let support_text = read("docs/support-policy.md");
    let compatibility_text = read("docs/compatibility-and-surface.md");
    let content_runtime_text = read("docs/content-runtime.md");

    for marker in [
        "Repository reference facts and queries | Supported project-intelligence graph",
        "`assura content` collection validation and query commands | Supported first project-intelligence query surface",
        "`assura content semantic-search`, `symbols`, and `symbol-refs` | Experimental candidate enrichment",
        "without requiring semantic search, code-symbol providers, a daemon, or a hosted service",
    ] {
        checks.require(
            support_text.contains(marker),
            format!("docs/support-policy.md: missing supported document graph marker {marker:?}"),
        );
    }
    for marker in [
        "| `assura content references` | Supported repository-reference graph query |",
        "| `assura content semantic-search` | Experimental optional local candidate search |",
        "| `project-intelligence:repository-reference-facts` | Supported |",
        "Semantic search and code-symbol queries are",
        "candidate-enrichment surfaces.",
    ] {
        checks.require(
            compatibility_text.contains(marker),
            format!(
                "docs/compatibility-and-surface.md: missing supported document graph marker {marker:?}"
            ),
        );
    }
    for marker in [
        "The supported document graph is the local, deterministic layer built from",
        "repository-reference edges from Markdown links, comments, docstrings, and",
        "Object-mode context packs",
        "bounded `repository_references.inbound`",
        "Semantic search and code-symbol queries remain optional candidate enrichment.",
    ] {
        checks.require(
            content_runtime_text.contains(marker),
            format!("docs/content-runtime.md: missing supported document graph marker {marker:?}"),
        );
    }

    for path in public_claim_files() {
        let text = read(&path);
        for (line_index, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            let mentions_candidate_enrichment = lower.contains("semantic search")
                || lower.contains("semantic-search")
                || lower.contains("code-symbol")
                || lower.contains("symbol-refs");
            let makes_validation_truth_claim = lower.contains("validation truth")
                || lower.contains("decide validation")
                || lower.contains("decides validation")
                || lower.contains("required")
                || lower.contains("requires")
                || lower.contains("must use")
                || lower.contains("must run");
            let clearly_scoped = lower.contains("experimental")
                || lower.contains("candidate")
                || lower.contains("optional")
                || lower.contains("not required")
                || lower.contains("without requiring")
                || lower.contains("do not decide validation")
                || lower.contains("does not decide validation");
            if mentions_candidate_enrichment && makes_validation_truth_claim && !clearly_scoped {
                checks.add(format!(
                    "{}:{}: semantic/code-symbol candidate enrichment must not be claimed as supported graph validation truth",
                    path,
                    line_index + 1
                ));
            }

            let mentions_hosted_requirement = lower.contains("hosted")
                && (lower.contains("required")
                    || lower.contains("requires")
                    || lower.contains("prerequisite"));
            let clearly_local = lower.contains("not required")
                || lower.contains("must not require")
                || lower.contains("does not require")
                || lower.contains("without requiring")
                || lower.contains("no hosted")
                || lower.contains("unsupported");
            if mentions_hosted_requirement && !clearly_local {
                checks.add(format!(
                    "{}:{}: supported document graph must not require hosted services",
                    path,
                    line_index + 1
                ));
            }
        }
    }
}

fn check_post_beta_release_hardening(checks: &mut Checks) {
    let release_notes = read("docs/release-notes.md");
    let release_surfaces = read("docs/data/release-surfaces.json");
    let support_goal = read("docs/goals/assura-post-beta-support-release-hardening.md");
    let parent_goal = read("docs/goals/assura-post-beta-capabilities-program.md");
    let roadmap = read(".trellis/spec/assura/roadmap.md");
    let release_checklist = read("docs/release-candidate-checklist.md");
    let release_readiness = read("website/src/content/docs/reference/release-readiness.md");

    for marker in [
        "Assura v0.3.0 Release Notes",
        "published `v0.3.0` beta increment",
        "still remains pre-1.0 beta software",
    ] {
        checks.require(
            release_notes.contains(marker),
            format!("docs/release-notes.md: missing v0.3.0 beta marker {marker:?}"),
        );
    }

    for marker in [
        "\"daemon-mode\"",
        "\"vscode-extension\"",
        "\"extension-api-boundaries\"",
        "\"agent-integration-lifecycle\"",
    ] {
        checks.require(
            release_surfaces.contains(marker),
            format!("docs/data/release-surfaces.json: missing release surface {marker:?}"),
        );
    }
    for marker in [
        "\"id\": \"daemon-mode\",\n      \"label\": \"Daemon mode\",\n      \"status\": \"experimental\",\n      \"first_release\": \"v0.3.0\"",
        "\"id\": \"vscode-extension\",\n      \"label\": \"VS Code beta local package\",\n      \"status\": \"supported\",\n      \"first_release\": \"v0.3.0\"",
        "\"id\": \"extension-api-boundaries\",\n      \"label\": \"Extension API boundaries\",\n      \"status\": \"supported\",\n      \"first_release\": \"v0.3.0\"",
        "\"id\": \"agent-integration-lifecycle\",\n      \"label\": \"Agent integration lifecycle\",\n      \"status\": \"experimental\",\n      \"first_release\": \"v0.3.0\"",
    ] {
        checks.require(
            release_surfaces.contains(marker),
            format!("docs/data/release-surfaces.json: missing v0.3.0 marker {marker:?}"),
        );
    }

    for marker in [
        "North-Star Verification Scenario",
        "release-blocking reason",
        "merge, block, or targeted-repair decision",
    ] {
        checks.require(
            support_goal.contains(marker) || parent_goal.contains(marker),
            format!("post-beta goals: missing north-star release-hardening marker {marker:?}"),
        );
    }

    checks.require(
        roadmap.contains("docs/goals/assura-post-beta-support-release-hardening.md"),
        ".trellis/spec/assura/roadmap.md: support hardening is not routed",
    );
    for marker in [
        "Experimental daemon surface",
        "assura daemon status",
        "assura daemon check-path",
        "Experimental local agent integration lifecycle",
        "assura agent integration",
        "Codex, OpenCode, Claude, and Pi",
        "Supported beta local editor package",
        "integrations/editors/vscode",
        "pnpm --dir integrations/editors/vscode test",
        "pnpm --dir integrations/editors/vscode run build",
        "pnpm --dir integrations/editors/vscode run doctor",
        "pnpm --dir integrations/editors/vscode run package",
        "Supported extension-boundary documentation",
        "public third-party plugin APIs remain roadmap-only",
        "cargo test --test daemon_cli_tests --quiet",
        "cargo test --test agent_surface_cli --quiet",
    ] {
        checks.require(
            release_checklist.contains(marker),
            format!("docs/release-candidate-checklist.md: missing v0.3.0 surface gate {marker:?}"),
        );
    }
    checks.require(
        release_readiness.contains("pre-1.0")
            && release_readiness.contains("integrations/editors/vscode")
            && release_readiness.contains("Extension API Boundaries"),
        "website release readiness page: missing beta support/readiness markers",
    );
}

fn public_claim_files() -> Vec<String> {
    let mut paths = vec![
        "README.md".to_string(),
        "docs/release-notes.md".to_string(),
        "docs/support-policy.md".to_string(),
        "docs/compatibility-and-surface.md".to_string(),
        "docs/validation.md".to_string(),
    ];
    paths.extend(
        collect_files("website/src/content", None)
            .into_iter()
            .filter(|path| {
                matches!(
                    path.extension().and_then(OsStr::to_str),
                    Some("md" | "mdx" | "astro")
                )
            })
            .map(|path| rel(&path)),
    );
    paths
}

struct CliCommandVariantRow {
    enum_name: &'static str,
    variant_name: &'static str,
    command_surface_names: &'static [&'static str],
}

const CLI_COMMAND_VARIANT_ROWS: &[CliCommandVariantRow] = &[
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Check",
        command_surface_names: &["assura check"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Status",
        command_surface_names: &["assura status"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Doctor",
        command_surface_names: &["assura doctor"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Review",
        command_surface_names: &["assura review"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Cache",
        command_surface_names: &["assura cache"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Explain",
        command_surface_names: &["assura explain"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Init",
        command_surface_names: &["assura init"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Config",
        command_surface_names: &["assura config"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Watch",
        command_surface_names: &["assura watch"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Migrate",
        command_surface_names: &["assura migrate"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Fix",
        command_surface_names: &["assura fix markdown"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Agent",
        command_surface_names: &["assura agent"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Editor",
        command_surface_names: &["assura editor"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Content",
        command_surface_names: &["assura content"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Daemon",
        command_surface_names: &["assura daemon"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Info",
        command_surface_names: &["assura info"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "PerformanceReport",
        command_surface_names: &["assura performance-report"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Hooks",
        command_surface_names: &["assura hooks"],
    },
    CliCommandVariantRow {
        enum_name: "Commands",
        variant_name: "Quality",
        command_surface_names: &["assura quality plan"],
    },
    CliCommandVariantRow {
        enum_name: "HookCommands",
        variant_name: "Install",
        command_surface_names: &["assura hooks install"],
    },
    CliCommandVariantRow {
        enum_name: "HookCommands",
        variant_name: "Uninstall",
        command_surface_names: &["assura hooks uninstall"],
    },
    CliCommandVariantRow {
        enum_name: "HookCommands",
        variant_name: "Status",
        command_surface_names: &["assura hooks status"],
    },
    CliCommandVariantRow {
        enum_name: "HookCommands",
        variant_name: "Verify",
        command_surface_names: &["assura hooks verify"],
    },
    CliCommandVariantRow {
        enum_name: "QualityCommands",
        variant_name: "Plan",
        command_surface_names: &["assura quality plan"],
    },
    CliCommandVariantRow {
        enum_name: "FixCommands",
        variant_name: "Markdown",
        command_surface_names: &["assura fix markdown"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "AgentContext",
        command_surface_names: &["assura content agent-context"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "AgentQuery",
        command_surface_names: &["assura content agent-query"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "ContextPack",
        command_surface_names: &["assura content context-pack"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Session",
        command_surface_names: &["assura content session"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Collections",
        command_surface_names: &["assura content collections"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Instances",
        command_surface_names: &["assura content instances"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Show",
        command_surface_names: &["assura content show"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Search",
        command_surface_names: &["assura content search"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "SemanticSearch",
        command_surface_names: &["assura content semantic-search"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Symbols",
        command_surface_names: &["assura content symbols"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "SymbolRefs",
        command_surface_names: &["assura content symbol-refs"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "MissingRelations",
        command_surface_names: &["assura content missing-relations"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "References",
        command_surface_names: &["assura content references"],
    },
    CliCommandVariantRow {
        enum_name: "ContentCommands",
        variant_name: "Expand",
        command_surface_names: &["assura content expand"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Status",
        command_surface_names: &["assura daemon status"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Start",
        command_surface_names: &["assura daemon start"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Stop",
        command_surface_names: &["assura daemon stop"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Restart",
        command_surface_names: &["assura daemon restart"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Doctor",
        command_surface_names: &["assura daemon doctor"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Logs",
        command_surface_names: &["assura daemon logs"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Health",
        command_surface_names: &["assura daemon health"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "CheckPath",
        command_surface_names: &["assura daemon check-path"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "References",
        command_surface_names: &["assura daemon references"],
    },
    CliCommandVariantRow {
        enum_name: "DaemonCommands",
        variant_name: "Serve",
        command_surface_names: &[],
    },
];

fn cli_command_variant_rows() -> &'static [CliCommandVariantRow] {
    CLI_COMMAND_VARIANT_ROWS
}

struct SupportMatrixRow {
    surface: &'static str,
    command_surface_names: &'static [&'static str],
    support_policy_markers: &'static [&'static str],
    compatibility_markers: &'static [&'static str],
    source_markers: &'static [&'static str],
    test_markers: &'static [&'static str],
    exception_markers: &'static [&'static str],
}

const SUPPORT_MATRIX_ROWS: &[SupportMatrixRow] = &[
    SupportMatrixRow {
        surface: "assura CLI root",
        command_surface_names: &["assura"],
        support_policy_markers: &["This policy applies to Assura pre-1.0 releases."],
        compatibility_markers: &["# Compatibility And Public Surface"],
        source_markers: &["#[command(name = \"assura\")]"],
        test_markers: &[],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check",
        command_surface_names: &["assura check"],
        support_policy_markers: &["`assura check` structure validation"],
        compatibility_markers: &["| `assura check` | Supported |"],
        source_markers: &["Commands::Check"],
        test_markers: &["tests/cli_check_tests.rs", "run_check", "--format"],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format json",
        command_surface_names: &[],
        support_policy_markers: &["`assura check --format json` and `--format yaml`"],
        compatibility_markers: &["| `assura check --format json` | Supported |"],
        source_markers: &["CheckOutputFormat::Json"],
        test_markers: &["tests/cli_command_surface_tests.rs", "\"json\""],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format yaml",
        command_surface_names: &[],
        support_policy_markers: &["`assura check --format json` and `--format yaml`"],
        compatibility_markers: &["| `assura check --format yaml` | Supported |"],
        source_markers: &["CheckOutputFormat::Yaml"],
        test_markers: &["tests/cli_command_surface_tests.rs", "\"yaml\""],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format advice",
        command_surface_names: &[],
        support_policy_markers: &["`assura check --format advice` and `--format status`"],
        compatibility_markers: &["| `assura check --format advice` | Supported |"],
        source_markers: &["CheckOutputFormat::Advice"],
        test_markers: &[
            "tests/real_project_agentic_feedback_tests.rs",
            "check_advice_format_renders_guided_output_in_one_command",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format status",
        command_surface_names: &[],
        support_policy_markers: &["`assura check --format advice` and `--format status`"],
        compatibility_markers: &["| `assura check --format status` | Supported |"],
        source_markers: &["CheckOutputFormat::Status"],
        test_markers: &[
            "tests/real_project_agentic_feedback_tests.rs",
            "check_status_format_supports_general_display_limits",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format agent",
        command_surface_names: &[],
        support_policy_markers: &["`assura check --format agent`"],
        compatibility_markers: &["| `assura check --format agent` | Supported |"],
        source_markers: &["CheckOutputFormat::Agent"],
        test_markers: &["tests/cli_command_surface_tests.rs", "--format", "agent"],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura check --format agent --agent codex",
        command_surface_names: &[],
        support_policy_markers: &["`--agent codex` delivery"],
        compatibility_markers: &[
            "| `assura check --format agent --agent codex` | Supported adapter |",
        ],
        source_markers: &["AgentTarget::Codex"],
        test_markers: &["tests/cli_command_surface_tests.rs", "--agent", "codex"],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura init",
        command_surface_names: &["assura init"],
        support_policy_markers: &["`assura init`"],
        compatibility_markers: &["| `assura init` | Supported |"],
        source_markers: &["Commands::Init"],
        test_markers: &[
            "tests/cli_command_surface_tests.rs",
            ".arg(\"init\")",
            "--no-git-hooks",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura config add-recipe",
        command_surface_names: &["assura config", "assura config add-recipe"],
        support_policy_markers: &["`assura config add-recipe`"],
        compatibility_markers: &[
            "| `assura config add-recipe` | Supported project-owned policy authoring |",
        ],
        source_markers: &["Commands::Config", "ConfigCommands::AddRecipe"],
        test_markers: &[
            "tests/cli_command_surface_tests.rs",
            "config_add_recipe_dry_run_materializes_project_owned_policy",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura status --format json",
        command_surface_names: &["assura status"],
        support_policy_markers: &["`assura status --format json`"],
        compatibility_markers: &["| `assura status --format json` | Supported |"],
        source_markers: &["Commands::Status"],
        test_markers: &[
            "tests/real_project_agentic_feedback_tests.rs",
            ".arg(\"status\")",
            "\"json\"",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura review",
        command_surface_names: &["assura review"],
        support_policy_markers: &["`assura review`"],
        compatibility_markers: &["| `assura review` | Supported compact project review |"],
        source_markers: &["Commands::Review", "project_review_command"],
        test_markers: &[
            "tests/project_review_cli.rs",
            "review_clean_repo_reports_inactive_guidance_without_blocking",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura cache",
        command_surface_names: &["assura cache", "assura cache status", "assura cache clean"],
        support_policy_markers: &["`assura check --cache`, `assura cache status`, and `assura cache clean`"],
        compatibility_markers: &["| `assura cache status|clean` | Experimental correctness-checked cache |"],
        source_markers: &["Commands::Cache", "cache_command"],
        test_markers: &[
            "tests/cli_command_surface_tests.rs",
            "cache_status_and_clean_report_observable_namespaces",
            "cache_clean_refuses_an_unrecognized_or_project_root",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura doctor",
        command_surface_names: &["assura doctor"],
        support_policy_markers: &["`assura doctor`"],
        compatibility_markers: &["| `assura doctor` | Experimental local project doctor |"],
        source_markers: &["Commands::Doctor", "doctor_command"],
        test_markers: &[
            "tests/doctor_explain_cli.rs",
            "doctor_reports_clean_check_with_inactive_and_unwired_model_gap",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura explain",
        command_surface_names: &["assura explain"],
        support_policy_markers: &["`assura explain`"],
        compatibility_markers: &[
            "| `assura explain` | Supported local path explanation |",
        ],
        source_markers: &["Commands::Explain", "explain_command"],
        test_markers: &[
            "tests/doctor_explain_cli.rs",
            "explain_reports_inherited_scope_and_source_markdown_skips",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura migrate",
        command_surface_names: &["assura migrate"],
        support_policy_markers: &["`assura migrate` for complete LS-Lint 2.3 config semantics"],
        compatibility_markers: &[
            "| `assura migrate` | Supported for complete LS-Lint 2.3 config semantics |",
        ],
        source_markers: &["Commands::Migrate"],
        test_markers: &["tests/ls_lint_rule_coverage_tests.rs", ".arg(\"migrate\")"],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura hooks",
        command_surface_names: &[
            "assura hooks",
            "assura hooks install",
            "assura hooks uninstall",
            "assura hooks status",
            "assura hooks verify",
        ],
        support_policy_markers: &["`assura hooks` for local git hooks"],
        compatibility_markers: &["| `assura hooks` | Supported for local git hooks |"],
        source_markers: &["HookCommands::Install", "GitHooksManager"],
        test_markers: &[
            "tests/cli_command_surface_tests.rs",
            "hooks_help_lists_local_hook_subcommands",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura quality plan",
        command_surface_names: &["assura quality plan"],
        support_policy_markers: &["`assura quality plan`"],
        compatibility_markers: &[
            "| `assura quality plan` | Supported for local quality planning |",
        ],
        source_markers: &["QualityCommands::Plan", "QualityPlanCommandOptions"],
        test_markers: &[
            "tests/cli_command_surface_tests.rs",
            "quality_plan_emits_config_backed_checks_for_changed_paths",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura performance-report",
        command_surface_names: &["assura performance-report"],
        support_policy_markers: &["`assura performance-report`"],
        compatibility_markers: &["| `assura performance-report` | Supported evidence command |"],
        source_markers: &[
            "Commands::PerformanceReport",
            "PerformanceReportCommandOptions",
        ],
        test_markers: &[
            "tests/performance_report_contract_tests.rs",
            "two_x_claim_status",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura fix markdown",
        command_surface_names: &["assura fix markdown"],
        support_policy_markers: &[
            "`assura fix markdown --dry-run --format json`",
            "`assura fix markdown --apply --format json`",
        ],
        compatibility_markers: &[
            "| `assura fix markdown --dry-run --format json` | Experimental safe-fix preview contract |",
            "| `assura fix markdown --apply --format json` | Experimental safe-fix apply/audit contract |",
        ],
        source_markers: &["FixCommands::Markdown", "fix_markdown_command"],
        test_markers: &[
            "tests/markdown_lint_fix_tests.rs",
            "fix_markdown_dry_run_reports_safe_fix_without_writing",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura agent",
        command_surface_names: &[
            "assura agent",
            "assura agent context",
            "assura agent diagnostics",
            "assura agent context-pack",
            "assura agent show",
            "assura agent search",
            "assura agent missing-relations",
            "assura agent expand",
            "assura agent safe-fixes",
            "assura agent onboard",
            "assura agent nudge",
            "assura agent integration",
            "assura agent integration install",
            "assura agent integration update",
            "assura agent integration remove",
            "assura agent integration status",
            "assura agent integration doctor",
            "assura agent session",
        ],
        support_policy_markers: &["`assura agent`"],
        compatibility_markers: &[
            "| `assura agent` | Supported local agent project-intelligence surface |",
            "| `assura agent onboard` | Supported local agent-ready onboarding surface |",
            "| `assura agent nudge` | Experimental local agent nudge payload |",
            "| `assura agent integration` | Experimental local agent integration lifecycle |",
            "| `assura agent session` | Supported local agent session alias |",
        ],
        source_markers: &[
            "Commands::Agent",
            "AgentCommands::Onboard",
            "AgentCommands::Context",
            "AgentCommands::Nudge",
            "AgentCommands::Integration",
        ],
        test_markers: &[
            "tests/agent_surface_cli.rs",
            "agent_surface_defaults_to_json_and_reuses_content_contracts",
            "agent_onboard_generates_broad_baseline_and_packet",
            "agent_nudge_after_tool_reports_bounded_changed_path_findings",
            "agent_integration_lifecycle_installs_reviewable_bundles_for_all_hosts",
            "agent_surface_session_alias_reuses_json_line_session_contract",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura editor",
        command_surface_names: &["assura editor", "assura editor session"],
        support_policy_markers: &["`assura editor session`"],
        compatibility_markers: &[
            "| `assura editor` | Supported local editor project-intelligence surface |",
            "| `assura editor session` | Supported local editor session |",
        ],
        source_markers: &["Commands::Editor", "EditorCommands::Session"],
        test_markers: &[
            "tests/editor_surface_cli.rs",
            "editor_surface_returns_lsp_shaped_diagnostics_for_file",
            "editor_surface_code_actions_preview_safe_fixes_without_writes",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura content",
        command_surface_names: &[
            "assura content",
            "assura content agent-context",
            "assura content agent-query",
            "assura content context-pack",
            "assura content session",
            "assura content collections",
            "assura content instances",
            "assura content show",
            "assura content search",
            "assura content semantic-search",
            "assura content symbols",
            "assura content symbol-refs",
            "assura content missing-relations",
            "assura content references",
            "assura content expand",
        ],
        support_policy_markers: &[
            "`assura content` collection validation and query commands",
            "`assura content session`",
        ],
        compatibility_markers: &[
            "| `assura content` | Supported first project-intelligence query surface |",
            "| `assura content session` | Supported local project-intelligence session |",
        ],
        source_markers: &["Commands::Content", "ContentCommands::AgentContext"],
        test_markers: &[
            "tests/content_query_cli.rs",
            "content_query_lists_collections_and_instances",
            "tests/project_intelligence_session.rs",
            "content_session_reuses_context_for_repeated_requests",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura daemon",
        command_surface_names: &[
            "assura daemon",
            "assura daemon status",
            "assura daemon start",
            "assura daemon stop",
            "assura daemon restart",
            "assura daemon doctor",
            "assura daemon logs",
            "assura daemon health",
            "assura daemon check-path",
            "assura daemon references",
        ],
        support_policy_markers: &["| `assura daemon` | Experimental local daemon process |"],
        compatibility_markers: &[
            "| `assura daemon` | Experimental local daemon process |",
            "| `assura daemon status` | Experimental local daemon status |",
            "| `assura daemon start` | Experimental local daemon lifecycle |",
            "| `assura daemon stop` | Experimental local daemon lifecycle |",
            "| `assura daemon restart` | Experimental local daemon lifecycle |",
            "| `assura daemon doctor` | Experimental local daemon doctor |",
            "| `assura daemon logs` | Experimental local daemon logs preview |",
        ],
        source_markers: &[
            "Commands::Daemon",
            "DaemonCommands::Status",
            "DaemonCommands::Start",
            "DaemonCommands::Stop",
            "DaemonCommands::Restart",
            "DaemonCommands::Doctor",
            "DaemonCommands::Logs",
            "DaemonCommands::Health",
            "daemon_command",
        ],
        test_markers: &[
            "tests/daemon_cli_tests.rs",
            "daemon_status_json_reports_management_contract",
            "daemon_start_stop_json_are_idempotent_and_status_reflects_runtime",
            "daemon_restart_and_logs_json_use_runtime_area",
            "daemon_doctor_json_reports_actionable_checks",
            "daemon_check_path_json_uses_running_ipc_process",
            "daemon_status_reports_crashed_process_without_fresh_running_state",
            "daemon_health_json_exposes_running_state_and_fallback",
            "daemon_references_source_json_matches_content_references",
            "daemon_references_target_json_matches_content_references",
        ],
        exception_markers: &[],
    },
    SupportMatrixRow {
        surface: "assura info",
        command_surface_names: &["assura info"],
        support_policy_markers: &["`assura info`"],
        compatibility_markers: &["| `assura info` | Experimental diagnostic |"],
        source_markers: &["Commands::Info"],
        test_markers: &[],
        exception_markers: &["Experimental diagnostic"],
    },
    SupportMatrixRow {
        surface: "assura watch",
        command_surface_names: &["assura watch"],
        support_policy_markers: &["`assura watch`"],
        compatibility_markers: &["| `assura watch` | Experimental |"],
        source_markers: &["Commands::Watch"],
        test_markers: &[],
        exception_markers: &["Experimental"],
    },
    SupportMatrixRow {
        surface: "internal Rust APIs",
        command_surface_names: &[],
        support_policy_markers: &["Internal Rust APIs"],
        compatibility_markers: &[
            "## Rust Library Surface",
            "Public module visibility in `src/lib.rs` does not imply release support",
        ],
        source_markers: &[
            "pub mod intelligence;",
            "pub mod maturity;",
            "pub mod validation;",
        ],
        test_markers: &[],
        exception_markers: &["unstable internal APIs"],
    },
];

fn support_matrix_rows() -> &'static [SupportMatrixRow] {
    SUPPORT_MATRIX_ROWS
}

fn check_manifest_semantics(checks: &mut Checks) {
    let Some(metadata) = cargo_metadata(checks) else {
        return;
    };
    let root_version = metadata_package(&metadata, "Cargo.toml")
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let root_rust_version = metadata_package(&metadata, "Cargo.toml")
        .and_then(|package| package.get("rust_version"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    for row in manifest_matrix_rows() {
        let Some(package) = metadata_package(&metadata, row.manifest) else {
            checks.add(format!(
                "{}: {} manifest is missing from cargo metadata",
                row.manifest, row.classification
            ));
            continue;
        };
        for field in row.required_package_fields {
            checks.require(
                metadata_string_field(package, field).is_some(),
                format!(
                    "{}: {} manifest missing package.{field}",
                    row.manifest, row.classification
                ),
            );
        }
        if row.semver_version {
            let version = metadata_string_field(package, "version").unwrap_or_default();
            checks.require(
                semver_like(&version),
                format!(
                    "{}: {} package.version must be SemVer-like",
                    row.manifest, row.classification
                ),
            );
        }
        if row.version_matches_root {
            checks.require(
                metadata_string_field(package, "version").as_deref() == Some(root_version.as_str()),
                format!(
                    "{}: {} crate version must match root package",
                    row.manifest, row.classification
                ),
            );
        }
        if row.rust_version_matches_root {
            checks.require(
                metadata_string_field(package, "rust_version").as_deref()
                    == Some(root_rust_version.as_str()),
                format!(
                    "{}: {} crate MSRV must match root package",
                    row.manifest, row.classification
                ),
            );
        }
        if let Some(expected_publish) = row.publish {
            checks.require(
                metadata_publish_value(package) == Some(expected_publish),
                format!(
                    "{}: {} crate publish policy must be publish={expected_publish}",
                    row.manifest, row.classification
                ),
            );
        }
        if let Some(expected_default_run) = row.default_run {
            checks.require(
                metadata_string_field(package, "default_run").as_deref()
                    == Some(expected_default_run),
                format!(
                    "{}: {} package default-run must be {expected_default_run}",
                    row.manifest, row.classification
                ),
            );
        }
    }

    let expected_members = manifest_matrix_rows()
        .iter()
        .filter_map(|row| {
            metadata_package(&metadata, row.manifest)
                .and_then(|package| metadata_string_field(package, "name"))
        })
        .collect::<BTreeSet<_>>();
    checks.require(
        workspace_member_names(&metadata, "workspace_members") == expected_members,
        "Cargo.toml: workspace members drifted",
    );
    if metadata.get("workspace_default_members").is_some() {
        checks.require(
            workspace_member_names(&metadata, "workspace_default_members") == expected_members,
            "Cargo.toml: workspace default-members must include all current members",
        );
    }
}

fn cargo_metadata(checks: &mut Checks) -> Option<Value> {
    let output = match command_stdout("cargo", ["metadata", "--no-deps", "--format-version", "1"]) {
        Ok(output) => output,
        Err(error) => {
            checks.add(format!("cargo metadata failed: {error}"));
            return None;
        }
    };
    match serde_json::from_str::<Value>(&output) {
        Ok(metadata) => Some(metadata),
        Err(error) => {
            checks.add(format!("cargo metadata output is invalid JSON: {error}"));
            None
        }
    }
}

fn metadata_package<'a>(metadata: &'a Value, manifest: &str) -> Option<&'a Value> {
    metadata
        .get("packages")
        .and_then(Value::as_array)?
        .iter()
        .find(|package| {
            package
                .get("manifest_path")
                .and_then(Value::as_str)
                .is_some_and(|path| metadata_manifest_rel(path) == manifest)
        })
}

fn metadata_manifest_rel(path: &str) -> String {
    let path = Path::new(path);
    if let Ok(cwd) = env::current_dir() {
        if let Ok(relative) = path.strip_prefix(cwd) {
            return rel(relative);
        }
    }
    rel(path)
}

fn metadata_string_field(package: &Value, field: &str) -> Option<String> {
    package
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn metadata_publish_value(package: &Value) -> Option<bool> {
    match package.get("publish") {
        Some(Value::Array(values)) if values.is_empty() => Some(false),
        Some(Value::Array(_)) => Some(true),
        Some(Value::Null) => None,
        _ => None,
    }
}

fn workspace_member_names(metadata: &Value, field: &str) -> BTreeSet<String> {
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|package| {
            let id = package.get("id").and_then(Value::as_str)?;
            let name = metadata_string_field(package, "name")?;
            Some((id.to_string(), name))
        })
        .collect::<BTreeMap<_, _>>();
    metadata
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|id| packages.get(id).cloned().unwrap_or_else(|| id.to_string()))
        .collect()
}

struct ManifestMatrixRow {
    manifest: &'static str,
    classification: &'static str,
    required_package_fields: &'static [&'static str],
    semver_version: bool,
    version_matches_root: bool,
    rust_version_matches_root: bool,
    publish: Option<bool>,
    default_run: Option<&'static str>,
}

const MANIFEST_MATRIX_ROWS: &[ManifestMatrixRow] = &[
    ManifestMatrixRow {
        manifest: "Cargo.toml",
        classification: "public root",
        required_package_fields: &[
            "name",
            "version",
            "edition",
            "default_run",
            "description",
            "license",
            "repository",
            "homepage",
            "documentation",
            "rust_version",
            "readme",
        ],
        semver_version: true,
        version_matches_root: false,
        rust_version_matches_root: false,
        publish: None,
        default_run: Some("assura"),
    },
    ManifestMatrixRow {
        manifest: "crates/assura-check-cli/Cargo.toml",
        classification: "internal support",
        required_package_fields: &[
            "name",
            "version",
            "edition",
            "description",
            "license",
            "rust_version",
        ],
        semver_version: true,
        version_matches_root: true,
        rust_version_matches_root: true,
        publish: Some(false),
        default_run: None,
    },
    ManifestMatrixRow {
        manifest: "crates/assura-stable-hash/Cargo.toml",
        classification: "internal support",
        required_package_fields: &[
            "name",
            "version",
            "edition",
            "description",
            "license",
            "rust_version",
        ],
        semver_version: true,
        version_matches_root: true,
        rust_version_matches_root: true,
        publish: Some(false),
        default_run: None,
    },
    ManifestMatrixRow {
        manifest: "xtask/Cargo.toml",
        classification: "internal maintenance",
        required_package_fields: &[
            "name",
            "version",
            "edition",
            "description",
            "license",
            "rust_version",
        ],
        semver_version: true,
        version_matches_root: true,
        rust_version_matches_root: true,
        publish: Some(false),
        default_run: None,
    },
];

fn manifest_matrix_rows() -> &'static [ManifestMatrixRow] {
    MANIFEST_MATRIX_ROWS
}

fn toml_string_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix(&format!("{key} = "))?;
        Some(rest.trim().trim_matches('"').to_string())
    })
}

fn semver_like(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() >= 3
        && parts[..3]
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

fn check_test_relationships(checks: &mut Checks) {
    let support_text = read("docs/support-policy.md");
    let compatibility_text = read("docs/compatibility-and-surface.md");
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

    for row in support_matrix_rows() {
        let has_exception = row.exception_markers.iter().any(|marker| {
            support_text.contains(marker)
                || compatibility_text.contains(marker)
                || source_text.contains(marker)
        });
        let missing_tests = row
            .test_markers
            .iter()
            .filter(|marker| !test_text.contains(**marker))
            .copied()
            .collect::<Vec<_>>();
        if row.test_markers.is_empty() {
            checks.require(
                has_exception || row.command_surface_names == ["assura"],
                format!(
                    "{}: support matrix row needs test markers or an explicit exception",
                    row.surface
                ),
            );
        }
        checks.require(
            missing_tests.is_empty(),
            format!(
                "{}: missing test coverage markers {missing_tests:?}",
                row.surface
            ),
        );
        let missing_source = row
            .source_markers
            .iter()
            .filter(|marker| !source_text.contains(**marker))
            .copied()
            .collect::<Vec<_>>();
        checks.require(
            missing_source.is_empty(),
            format!("{}: missing source markers {missing_source:?}", row.surface),
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
    let release_checklist_text = read("docs/release-candidate-checklist.md");
    let release_train_text = read("docs/release-train.md");
    let release_surfaces_text = read("docs/data/release-surfaces.json");
    let code_intelligence_text = read("website/src/content/docs/product/code-intelligence.md");
    let release_readiness_text = read("website/src/content/docs/reference/release-readiness.md");
    let installation_text = read("website/src/content/docs/guides/installation.md");
    let performance_text = read("website/src/content/docs/reference/performance.mdx");
    let performance_implementation_text =
        read("website/src/content/docs/reference/performance-implementation.mdx");
    let performance_cases_text =
        read("website/src/content/docs/reference/performance-test-cases.mdx");
    let why_assura_text = read("website/src/content/docs/why-assura.md");
    let performance_reassessment_text =
        read("docs/analysis/2026-07-02-ls-lint-performance-reassessment.md");
    let performance_review_text =
        read("docs/analysis/2026-06-19-goal-13-release-performance-review.md");
    let goal_13_text =
        read("docs/goals/assura-goal-13-performance-and-release-evidence-governance.md");
    let release_workflow = read(".github/workflows/release.yml");
    let ci_workflow = read(".github/workflows/ci.yml");
    let install_sh = read("website/public/install.sh");
    let install_ps1 = read("website/public/install.ps1");

    for artifact in release_artifacts() {
        let archive = artifact.archive;
        checks.require(
            compatibility_text.contains(archive),
            format!("docs/compatibility-and-surface.md: missing {archive}"),
        );
        checks.require(
            release_text.contains(archive),
            format!("docs/release-notes.md: missing {archive}"),
        );
        checks.require(
            release_checklist_text.contains(archive),
            format!("docs/release-candidate-checklist.md: missing {archive}"),
        );
        checks.require(
            release_readiness_text.contains(archive),
            format!("website release readiness docs: missing {archive}"),
        );
        checks.require(
            release_workflow.contains(&format!("archive_name: {archive}")),
            format!(".github/workflows/release.yml: missing release artifact {archive}"),
        );
        if let Some(installer) = artifact.installer {
            let install_text = if installer == "install.ps1" {
                &install_ps1
            } else {
                &install_sh
            };
            checks.require(
                install_text.contains(archive),
                format!("website/public/{installer}: missing installer archive {archive}"),
            );
            checks.require(
                installation_text.contains(archive),
                format!("website installation docs: missing installer archive {archive}"),
            );
        }
        if let Some(ci_label) = artifact.ci_smoke_label {
            checks.require(
                ci_workflow.contains(&format!("archive_name: {archive}"))
                    && ci_workflow.contains(ci_label),
                format!(
                    ".github/workflows/ci.yml: missing installable adoption smoke for {archive}"
                ),
            );
        }
    }
    checks.require(
        release_workflow.contains("target/${{ matrix.archive_name }}.sha256")
            && ci_workflow.contains("target/${{ matrix.archive_name }}.sha256"),
        "release workflows must upload checksum sidecars for every archive",
    );
    checks.require(
        release_text.contains("`.sha256` checksum file next to every archive")
            && compatibility_text.contains(".sha256"),
        "release docs must describe checksum sidecars for every archive",
    );
    for (path, text) in [
        ("docs/release-train.md", &release_train_text),
        (
            "docs/release-candidate-checklist.md",
            &release_checklist_text,
        ),
        (
            "website/src/content/docs/reference/release-readiness.md",
            &release_readiness_text,
        ),
    ] {
        checks.require(
            text.contains("cargo xtask release-readiness --format json"),
            format!("{path}: missing release-readiness command"),
        );
    }
    checks.require(
        release_train_text.contains("assura.release-readiness.v1")
            && release_readiness_text.contains("assura.release-readiness.v1"),
        "release train docs must name the release-readiness JSON schema",
    );
    checks.require(
        release_train_text.contains("docs/data/release-surfaces.json")
            && release_readiness_text.contains("docs/data/release-surfaces.json"),
        "release train docs must point to the release surfaces manifest",
    );
    match release_surfaces_report(
        "docs/data/release-surfaces.json",
        Some(&format!("v{version}")),
    ) {
        report if report.get("error").is_some() => checks.add(format!(
            "docs/data/release-surfaces.json: {}",
            report
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("invalid")
        )),
        report => {
            let has_unreleased_surfaces = report
                .get("unreleased_user_facing_changes")
                .and_then(Value::as_array)
                .map(|changes| !changes.is_empty())
                == Some(true);
            let release_candidate_manifest_ready = release_surfaces_text
                .contains(&format!("\"first_release\": \"v{version}\""))
                && !release_surfaces_text.contains("\"first_release\": \"next\"");
            checks.require(
                has_unreleased_surfaces || release_candidate_manifest_ready,
                "docs/data/release-surfaces.json: expected unreleased surfaces or concrete current-version release surfaces",
            );
        }
    }
    checks.require(
        release_surfaces_text.contains("\"project-intelligence-local-surfaces\""),
        "docs/data/release-surfaces.json: missing Project Intelligence release surface",
    );
    checks.require(
        release_surfaces_text.contains("\"content-collections-querying\""),
        "docs/data/release-surfaces.json: missing content collections/querying release surface",
    );
    checks.require(
        code_intelligence_text.contains("Experimental candidate enrichment"),
        "website code-intelligence docs: code-symbol surfaces must be candidate enrichment",
    );
    checks.require(
        !code_intelligence_text.contains("| Symbol queries | Supported |"),
        "website code-intelligence docs: symbol queries must not be marked supported",
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
    for field in [
        "commit_sha",
        "branch",
        "source_worktree_dirty",
        "environment",
        "command_line",
        "iterations",
        "ls_lint_status",
    ] {
        checks.require(
            bench_current.get(field).is_some(),
            format!("performance current.json: missing provenance field {field}"),
        );
    }
    checks.require(
        bench_current.get("schema_version").and_then(Value::as_str)
            == Some("assura.performance.v1"),
        "performance current.json: unexpected schema_version",
    );
    checks.require(
        bench_current
            .get("source_worktree_dirty")
            .and_then(Value::as_bool)
            == Some(false),
        "performance current.json: source_worktree_dirty must be false for the checked in-place report",
    );
    check_source_provenance_contract(checks, &bench_current, "performance current.json");
    checks.require(
        bench_current
            .pointer("/claim_summary/two_x_claim_verdict")
            .is_some()
            && bench_current
                .pointer("/warm_claim_summary/two_x_claim_verdict")
                .is_some(),
        "performance current.json: missing cold or warm claim verdict",
    );
    if let Some(rows) = bench_current.get("results").and_then(Value::as_array) {
        let mut accepted_count = 0usize;
        for row in rows {
            let fixture_id = row
                .get("fixture_id")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            let Some(acceptance) = row.get("fixture_acceptance").and_then(Value::as_str) else {
                checks.add(format!(
                    "performance current.json: row {fixture_id} missing fixture_acceptance"
                ));
                continue;
            };
            checks.require(
                matches!(
                    acceptance,
                    "accepted-ls-lint-equivalent"
                        | "diagnostic"
                        | "experimental"
                        | "retired"
                        | "assura-native-diagnostic"
                ),
                format!(
                    "performance current.json: row {fixture_id} has unknown fixture_acceptance {acceptance}"
                ),
            );
            if acceptance == "accepted-ls-lint-equivalent" {
                accepted_count += 1;
            }
        }
        checks.require(
            accepted_count > 0,
            "performance current.json: missing accepted-ls-lint-equivalent fixture rows",
        );
    }
    match performance_no_slower_failures(
        &bench_current,
        "realistic-equivalent",
        "assura-cli",
        "ls-lint-cli",
    ) {
        Ok(failures) => checks.require(
            failures.is_empty(),
            format!("performance current.json: no-slower gate failed: {failures:?}"),
        ),
        Err(error) => checks.add(format!(
            "performance current.json: no-slower gate could not be evaluated: {error}"
        )),
    }
    checks.require(
        performance_reassessment_text
            .contains("Post-Beta LS-Lint Performance Reassessment")
            && performance_reassessment_text.contains("every accepted LS-Lint-equivalent fixture row is no slower")
            && performance_reassessment_text.contains("Warm/session evidence remains separate")
            && performance_reassessment_text.contains("Phase Attribution")
            && performance_reassessment_text.contains("process/Rust CLI floors"),
        "performance reassessment analysis: missing post-beta no-slower, cold/warm split, or phase-attribution markers",
    );
    checks.require(
        performance_text.contains("Post-beta LS-Lint performance reassessment")
            && performance_text.contains("phase-level")
            && performance_text.contains("CLI-floor attribution")
            && performance_text
                .contains("docs/analysis/2026-07-02-ls-lint-performance-reassessment.md")
            && performance_implementation_text.contains("Post-Beta Reassessment")
            && performance_implementation_text.contains("accepted LS-Lint-equivalent cold rows")
            && performance_implementation_text
                .contains("docs/analysis/2026-07-02-ls-lint-performance-reassessment.md"),
        "performance docs: missing post-beta LS-Lint reassessment links and claim boundaries",
    );
    checks.require(
        text_contains_ordered(
            &ci_workflow,
            &[
                "performance:\n    name: Performance Report",
                "- name: Generate comparison report",
                "--output target/performance/ls-lint-comparison.json",
                "--iterations 16",
                "- name: Enforce no-slower gate",
                "run: cargo xtask performance-no-slower target/performance/ls-lint-comparison.json",
                "- name: Summarize performance",
                "if: always()",
                "- name: Upload performance artifact",
                "if: always()",
            ],
        ),
        ".github/workflows/ci.yml: Performance Report job must generate a 16-iteration report, enforce cargo xtask performance-no-slower on that report, and keep summary/artifact steps on failure",
    );
    checks.require(
        text_contains_ordered(
            &ci_workflow,
            &[
                "- name: Generate native performance report",
                "--suite native",
                "--output target/performance/native-current.json",
                "--history target/performance/native-history.jsonl",
                "--iterations 5",
                "- name: Enforce native performance gate",
                "run: cargo xtask native-performance-no-regression target/performance/native-current.json",
                "name: native-performance-report",
            ],
        ),
        ".github/workflows/ci.yml: Performance Report job must generate, gate, and upload the native performance report",
    );

    check_native_performance_artifacts(
        checks,
        &performance_text,
        &performance_implementation_text,
        &performance_cases_text,
    );

    let current_cohort = bench_current
        .pointer("/claim_summary/fixture_cohort")
        .and_then(Value::as_str)
        .unwrap_or_default();
    checks.require(
        !current_cohort.is_empty(),
        "performance current.json: missing claim_summary.fixture_cohort",
    );
    if !current_cohort.is_empty() {
        let cohort_marker = format!("`{current_cohort}`");
        checks.require(
            performance_text.contains(&cohort_marker),
            format!("performance docs: missing current checked cohort {cohort_marker}"),
        );
        checks.require(
            performance_cases_text.contains(&cohort_marker),
            format!("performance test cases docs: missing current checked cohort {cohort_marker}"),
        );
        checks.require(
            why_assura_text.contains(&cohort_marker),
            format!("why-assura docs: missing current checked cohort {cohort_marker}"),
        );
    }

    let current_command = bench_current
        .get("command_line")
        .and_then(Value::as_str)
        .unwrap_or_default();
    checks.require(
        !current_command.is_empty(),
        "performance current.json: missing command_line",
    );
    for (option, expected_value) in [
        ("--output", "benches/history/current.json"),
        (
            "--history",
            "benches/history/ls-lint-comparison-history.jsonl",
        ),
        ("--website-dir", "website/public/data/performance"),
    ] {
        let actual_value = command_option_value(current_command, option);
        checks.require(
            actual_value == Some(expected_value),
            format!("performance current.json command_line must set {option} to {expected_value}"),
        );
        checks.require(
            text_contains_option_value(&performance_text, option, expected_value),
            format!("performance docs baseline command missing {option} {expected_value}"),
        );
        checks.require(
            text_contains_option_value(&performance_cases_text, option, expected_value),
            format!("performance test cases command missing {option} {expected_value}"),
        );
    }
    let report_iterations = bench_current.get("iterations").and_then(Value::as_u64);
    let command_iterations =
        command_option_value(current_command, "--iterations").and_then(|value| value.parse().ok());
    checks.require(
        command_iterations.is_some() && command_iterations == report_iterations,
        "performance current.json command_line iterations must match iterations field",
    );
    if let Some(iterations) = report_iterations {
        let iterations = iterations.to_string();
        checks.require(
            text_contains_option_value(&performance_text, "--iterations", &iterations),
            format!("performance docs baseline command missing --iterations {iterations}"),
        );
        checks.require(
            text_contains_option_value(&performance_cases_text, "--iterations", &iterations),
            format!("performance test cases command missing --iterations {iterations}"),
        );
    }
    if !current_command.contains("--include-external-fixtures")
        && current_cohort != "real-repo-headline"
    {
        checks.require(
            !performance_text.contains("--include-external-fixtures"),
            "performance docs: baseline command must not include external fixtures when current report does not",
        );
        checks.require(
            !performance_text.contains("ten pinned open-source repositories"),
            "performance docs: current checked claim must not cite ten pinned repositories without real-repo-headline data",
        );
        checks.require(
            !why_assura_text.contains("ten pinned real"),
            "why-assura docs: current checked claim must not cite ten pinned real repositories without real-repo-headline data",
        );
    }

    let cold_verdict = bench_current
        .pointer("/claim_summary/two_x_claim_verdict")
        .and_then(Value::as_str)
        .unwrap_or_default();
    checks.require(
        !cold_verdict.is_empty(),
        "performance current.json: cold claim verdict must be a non-empty string",
    );
    if cold_verdict != "complete" {
        checks.require(
            performance_review_text.contains("Cold Gate Follow-Up Acceptance"),
            "performance review: non-complete cold verdict requires accepted bounded follow-up",
        );
        checks.require(
            performance_review_text.contains("accepted follow-up is bounded"),
            "performance review: cold follow-up must state a bounded accepted follow-up",
        );
        checks.require(
            performance_review_text.contains(cold_verdict),
            format!("performance review: missing cold verdict {cold_verdict}"),
        );
        checks.require(
            goal_13_text.contains("accepted bounded follow-up"),
            "Goal 13 progress log: missing accepted bounded follow-up record",
        );
    }
}

fn check_native_performance_artifacts(
    checks: &mut Checks,
    performance_text: &str,
    performance_implementation_text: &str,
    performance_cases_text: &str,
) {
    let Ok(bench_native) =
        serde_json::from_str::<Value>(&read("benches/history/native-current.json"))
    else {
        checks.add("benches/history/native-current.json: invalid JSON");
        return;
    };
    let Ok(website_native) =
        serde_json::from_str::<Value>(&read("website/public/data/performance/native-current.json"))
    else {
        checks.add("website/public/data/performance/native-current.json: invalid JSON");
        return;
    };
    checks.require(
        bench_native == website_native,
        "native performance current.json drift: benches/history and website/public data must match",
    );
    checks.require(
        read("benches/history/native-history.jsonl")
            == read("website/public/data/performance/native-history.jsonl"),
        "native performance history drift: benches/history and website/public data must match",
    );
    checks.require(
        bench_native.get("schema_version").and_then(Value::as_str) == Some("assura.performance.v1"),
        "native performance current.json: unexpected schema_version",
    );
    check_source_provenance_contract(checks, &bench_native, "native performance current.json");
    checks.require(
        bench_native
            .get("source_worktree_dirty")
            .and_then(Value::as_bool)
            == Some(true),
        "native performance current.json: source_worktree_dirty must describe the dirty source lane behind the materialized snapshot",
    );
    checks.require(
        bench_native.get("ls_lint_package").and_then(Value::as_str) == Some("not-applicable"),
        "native performance current.json: ls_lint_package must be not-applicable",
    );
    let native_command = bench_native
        .get("command_line")
        .and_then(Value::as_str)
        .unwrap_or_default();
    for (option, expected_value) in [
        ("--suite", "native"),
        ("--output", "benches/history/native-current.json"),
        ("--history", "benches/history/native-history.jsonl"),
        ("--website-dir", "website/public/data/performance"),
    ] {
        checks.require(
            command_option_value(native_command, option) == Some(expected_value),
            format!("native performance command_line must set {option} to {expected_value}"),
        );
        checks.require(
            text_contains_option_value(performance_text, option, expected_value)
                || text_contains_option_value(performance_cases_text, option, expected_value),
            format!("native performance docs command missing {option} {expected_value}"),
        );
    }
    match native_performance_failures(&bench_native) {
        Ok(failures) => checks.require(
            failures.is_empty(),
            format!("native performance current.json gate failed: {failures:?}"),
        ),
        Err(error) => checks.add(format!(
            "native performance current.json gate could not be evaluated: {error}"
        )),
    }
    for marker in [
        "native-current.json",
        "native-history.jsonl",
        "assura-native-diagnostic",
        "native:phase:*",
        "native:phase:incremental-replace-generation",
        "native-performance-no-regression",
    ] {
        checks.require(
            performance_text.contains(marker)
                || performance_implementation_text.contains(marker)
                || performance_cases_text.contains(marker),
            format!("performance docs: missing native performance marker {marker}"),
        );
    }
    for fixture in NATIVE_PERFORMANCE_FIXTURES {
        checks.require(
            performance_cases_text.contains(&format!("`{fixture}`")),
            format!("performance test cases docs: missing native fixture {fixture}"),
        );
    }
}

fn check_public_roadmap(checks: &mut Checks) {
    let roadmap_path = "docs/data/public-roadmap.json";
    let Ok(roadmap) = serde_json::from_str::<Value>(&read(roadmap_path)) else {
        checks.add(format!("{roadmap_path}: invalid JSON"));
        return;
    };
    checks.require(
        roadmap.get("schema_version").and_then(Value::as_str) == Some("assura.public-roadmap.v1"),
        format!("{roadmap_path}: unexpected schema_version"),
    );
    checks.require(
        roadmap.get("source").and_then(Value::as_str) == Some(".trellis/spec/assura/roadmap.md"),
        format!("{roadmap_path}: source must point at .trellis/spec/assura/roadmap.md"),
    );

    let Some(groups) = roadmap.get("groups").and_then(Value::as_array) else {
        checks.add(format!("{roadmap_path}: groups must be an array"));
        return;
    };

    let mut statuses = BTreeSet::new();
    let mut labels = BTreeSet::new();
    let internal_roadmap = read(".trellis/spec/assura/roadmap.md");
    let current_recommended_goal =
        backticked_value_after_marker(&internal_roadmap, "Current recommended goal:");
    checks.require(
        current_recommended_goal.is_some(),
        ".trellis/spec/assura/roadmap.md must identify a current recommended goal",
    );
    let mut has_current_recommended_goal = false;

    for group in groups {
        let status = group
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        statuses.insert(status.to_string());
        checks.require(
            matches!(status, "done" | "now" | "next"),
            format!("{roadmap_path}: invalid group status {status:?}"),
        );
        checks.require(
            group.get("title").and_then(Value::as_str).is_some(),
            format!("{roadmap_path}: {status} group missing title"),
        );

        let Some(items) = group.get("items").and_then(Value::as_array) else {
            checks.add(format!(
                "{roadmap_path}: {status} group items must be an array"
            ));
            continue;
        };
        checks.require(
            !items.is_empty(),
            format!("{roadmap_path}: {status} group must include at least one item"),
        );

        for item in items {
            let label = item
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            let detail_path = item
                .get("detail_path")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            let href = item
                .get("href")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            let word_count = label_word_count(label);
            checks.require(
                (2..=4).contains(&word_count),
                format!("{roadmap_path}: label {label:?} must be two to four words"),
            );
            checks.require(
                labels.insert(label.to_string()),
                format!("{roadmap_path}: duplicate label {label:?}"),
            );
            checks.require(
                exists(detail_path),
                format!("{roadmap_path}: detail_path {detail_path} does not exist"),
            );
            checks.require(
                roadmap_href_matches_detail(href, detail_path),
                format!("{roadmap_path}: href {href} does not map to detail_path {detail_path}"),
            );
            if status == "now" && current_recommended_goal == Some(detail_path) {
                has_current_recommended_goal = true;
            }
        }
    }

    for expected in ["done", "now", "next"] {
        checks.require(
            statuses.contains(expected),
            format!("{roadmap_path}: missing {expected} group"),
        );
    }
    checks.require(
        has_current_recommended_goal,
        format!(
            "{roadmap_path}: now group must include the current recommended goal from .trellis/spec/assura/roadmap.md"
        ),
    );

    let public_page = read("website/src/content/docs/roadmap.mdx");
    let public_component = read("website/src/components/public-roadmap.astro");
    let astro_config = read("website/astro.config.mjs");
    checks.require(
        public_page.contains("PublicRoadmap"),
        "website roadmap page must render the PublicRoadmap component",
    );
    checks.require(
        public_component.contains("docs/data/public-roadmap.json"),
        "public roadmap component must import docs/data/public-roadmap.json",
    );
    checks.require(
        astro_config.contains("{ label: 'Roadmap', slug: 'roadmap' }"),
        "website sidebar must include the public Roadmap page",
    );
    checks.require(
        internal_roadmap.contains("docs/data/public-roadmap.json"),
        ".trellis/spec/assura/roadmap.md must point to the public roadmap artifact",
    );
}

fn check_agent_onboarding_website(checks: &mut Checks) {
    let guide_path = "website/src/content/docs/guides/agent-ready-onboarding.md";
    checks.require(
        exists(guide_path),
        format!("{guide_path}: dedicated agent-ready onboarding guide is missing"),
    );
    if !exists(guide_path) {
        return;
    }

    let guide = read(guide_path);
    let astro_config = read("website/astro.config.mjs");
    let home = read("website/src/content/docs/index.mdx");
    let getting_started = read("website/src/content/docs/guides/getting-started.md");

    checks.require(
        astro_config
            .contains("{ label: 'Agent-Ready Onboarding', slug: 'guides/agent-ready-onboarding' }"),
        "website sidebar must include the Agent-Ready Onboarding guide",
    );
    checks.require(
        home.contains("/guides/agent-ready-onboarding/")
            || getting_started.contains("/guides/agent-ready-onboarding/"),
        "website entry points must link to the Agent-Ready Onboarding guide",
    );

    for marker in [
        "## First-Run Phases",
        "## Report Shape",
        "## Generated Packet",
        "## Project-Local Skills",
        "## Agent Prompt",
        "## Agent-Next Questions",
        "## Checked Versus Unchecked",
        "## Content And Project Packs",
        "## Lifecycle Profiles",
        "## Specialization Flow",
        "\"content\"",
        "\"template\": \"none\"",
        "\"status\": \"inactive\"",
        "\"lifecycle_profiles\"",
        "\"mode\": \"nudge\"",
        "\"mode\": \"warn\"",
        "\"mode\": \"gate\"",
        "\"blocking\": true",
        "\"action\": \"Ask remaining specialization questions\"",
        "\"affected_paths\": [\".assura/onboarding/questions.md\"]",
        "The local command is:",
        "reviewable local integration bundle",
        "\"rule_recommendations\"",
        "\"local_rule\": \"$agent-entrypoint\"",
        "`not-applied` when the selected config lacks them",
        "assura agent onboard . --agent auto --format json",
        "assura agent onboard . --content-template agent-project --format json",
        "assura agent onboard . --content-template document-project --format json",
        ".agents/skills/",
        "assura-structure-fit",
        "STRUCTURE_FIT_CHECK",
        "does not silently mutate host-agent or global skill",
        "source-documents/",
        "library/topics/",
        "docs/drafts/",
        "docs/final/",
        "research-authoring project",
        "literature reviews",
        "papers, theses",
        "Treat inactive entries as unchecked",
        "assura doctor . --format json",
        "assura explain AGENTS.md --format json",
        "Roadmap note",
    ] {
        checks.require(
            guide.contains(marker),
            format!("{guide_path}: missing agent onboarding marker {marker:?}"),
        );
    }

    for forbidden in [
        "assura bootstrap",
        "assura agent specialize",
        "assura agent onboard --remote",
        "assura agent onboard . --remote",
        "assura remote bootstrap",
        "assura check --format codex-hook",
    ] {
        checks.require(
            !guide.contains(forbidden),
            format!("{guide_path}: contains unsupported onboarding command {forbidden:?}"),
        );
    }
}

fn label_word_count(label: &str) -> usize {
    label.split_whitespace().count()
}

fn backticked_value_after_marker<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let after_marker = text.split_once(marker)?.1;
    let after_open_tick = after_marker.split_once('`')?.1;
    let (value, _) = after_open_tick.split_once('`')?;
    if value.is_empty() {
        return None;
    }
    Some(value)
}

fn roadmap_href_matches_detail(href: &str, detail_path: &str) -> bool {
    const GITHUB_PREFIX: &str = "https://github.com/rothnic/assura/blob/master/";
    if let Some(repo_path) = href.strip_prefix(GITHUB_PREFIX) {
        return repo_path == detail_path && exists(repo_path);
    }
    if !href.starts_with('/') || href.contains("..") {
        return false;
    }
    let slug = href.trim_matches('/');
    if slug.is_empty() {
        return false;
    }
    let md = format!("website/src/content/docs/{slug}.md");
    let mdx = format!("website/src/content/docs/{slug}.mdx");
    (detail_path == md && exists(&md)) || (detail_path == mdx && exists(&mdx))
}

fn command_option_value<'a>(command_line: &'a str, option: &str) -> Option<&'a str> {
    let equals_prefix = format!("{option}=");
    let mut tokens = command_line.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == option {
            return tokens.next();
        }
        if let Some(value) = token.strip_prefix(&equals_prefix) {
            return Some(value);
        }
    }
    None
}

fn check_source_provenance_contract(checks: &mut Checks, report: &Value, label: &str) {
    let source_commit_sha = report.get("source_commit_sha").and_then(Value::as_str);
    let source_branch = report.get("source_branch").and_then(Value::as_str);
    let source_patch_id = report.get("source_patch_id").and_then(Value::as_str);
    let present_fields = [source_commit_sha, source_branch, source_patch_id]
        .into_iter()
        .filter(Option::is_some)
        .count();

    checks.require(
        present_fields == 0 || present_fields == 3,
        format!("{label}: source provenance fields must be all present or all absent"),
    );
    if present_fields == 3 {
        checks.require(
            source_commit_sha.is_some_and(is_full_hex_sha),
            format!("{label}: source_commit_sha must be a full hex SHA"),
        );
        checks.require(
            source_branch.is_some_and(|value| !value.is_empty()),
            format!("{label}: source_branch must be non-empty"),
        );
        checks.require(
            source_patch_id.is_some_and(is_full_hex_sha),
            format!("{label}: source_patch_id must be a full hex SHA"),
        );
        checks.require(
            report
                .get("source_worktree_dirty")
                .and_then(Value::as_bool)
                .is_some(),
            format!(
                "{label}: source_worktree_dirty must be a bool when source provenance is present"
            ),
        );
    }
}

fn is_full_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn text_contains_option_value(text: &str, option: &str, value: &str) -> bool {
    text.contains(&format!("{option} {value}")) || text.contains(&format!("{option}={value}"))
}

fn text_contains_ordered(text: &str, needles: &[&str]) -> bool {
    let mut remaining = text;
    for needle in needles {
        let Some(index) = remaining.find(needle) else {
            return false;
        };
        remaining = &remaining[index + needle.len()..];
    }
    true
}

struct ReleaseArtifact {
    archive: &'static str,
    installer: Option<&'static str>,
    ci_smoke_label: Option<&'static str>,
}

const RELEASE_ARTIFACTS: &[ReleaseArtifact] = &[
    ReleaseArtifact {
        archive: "assura-linux-amd64.tar.gz",
        installer: Some("install.sh"),
        ci_smoke_label: Some("ubuntu-x86_64"),
    },
    ReleaseArtifact {
        archive: "assura-linux-musl-amd64.tar.gz",
        installer: None,
        ci_smoke_label: None,
    },
    ReleaseArtifact {
        archive: "assura-macos-arm64.tar.gz",
        installer: Some("install.sh"),
        ci_smoke_label: Some("macos-arm64"),
    },
    ReleaseArtifact {
        archive: "assura-macos-amd64.tar.gz",
        installer: Some("install.sh"),
        ci_smoke_label: Some("macos-x86_64"),
    },
    ReleaseArtifact {
        archive: "assura-windows-amd64.zip",
        installer: Some("install.ps1"),
        ci_smoke_label: Some("windows-x86_64"),
    },
];

fn release_artifacts() -> &'static [ReleaseArtifact] {
    RELEASE_ARTIFACTS
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

fn check_goal_revalidation_route(checks: &mut Checks) {
    let skill = ".agents/skills/assura-goal-validation/SKILL.md";
    checks.require(
        exists(skill),
        format!("{skill}: stale-goal validation skill is missing"),
    );
    if exists(skill) {
        let skill_text = read(skill);
        for marker in [
            "selecting the next goal",
            "already achieved, superseded, duplicated",
            "record the revalidation result",
        ] {
            checks.require(
                skill_text.contains(marker),
                format!("{skill}: missing stale-goal validation marker {marker:?}"),
            );
        }
    }

    let agents_text = read("AGENTS.md");
    checks.require(
        agents_text.contains("assura-goal-validation"),
        "AGENTS.md: missing assura-goal-validation routing entry",
    );
    let iteration_text =
        read("docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md");
    checks.require(
        iteration_text.contains("assura-goal-validation"),
        "Iteration 02 roadmap: missing stale-goal validation direction lock",
    );
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
        "scripts/ci-scope-github.sh".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn performance_no_slower_passes_when_assura_is_not_slower() {
        let report = json!({
            "results": [
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "faster",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 4.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "faster",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 5.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "equal",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 6.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "equal",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 6.0
                }
            ]
        });

        let failures = performance_no_slower_failures(
            &report,
            "realistic-equivalent",
            "assura-cli",
            "ls-lint-cli",
        )
        .expect("report is valid");

        assert!(failures.is_empty());
    }

    #[test]
    fn performance_no_slower_reports_slower_and_missing_pairs() {
        let report = json!({
            "results": [
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "slower",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 7.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "slower",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 5.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "missing-ls-lint",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "missing-assura",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                }
            ]
        });

        let failures = performance_no_slower_failures(
            &report,
            "realistic-equivalent",
            "assura-cli",
            "ls-lint-cli",
        )
        .expect("report is valid");

        assert_eq!(
            failures,
            vec![
                NoSlowerFailure::MissingAssura {
                    fixture_id: "missing-assura".to_string()
                },
                NoSlowerFailure::MissingLsLint {
                    fixture_id: "missing-ls-lint".to_string()
                },
                NoSlowerFailure::Slower {
                    fixture_id: "slower".to_string(),
                    assura_ms: 7.0,
                    ls_lint_ms: 5.0
                }
            ]
        );
    }

    #[test]
    fn performance_no_slower_rejects_non_native_or_non_passing_rows() {
        let report = json!({
            "results": [
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "non-native-ls-lint",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "non-native-ls-lint",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "node-wrapper",
                    "ls_lint_execution_mode": "node-wrapper",
                    "status": "pass",
                    "median_runtime_ms": 2.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "skipped-assura",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura-cli",
                    "status": "skipped"
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "skipped-assura",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "no-target-rows",
                    "fixture_acceptance": "diagnostic",
                    "row_family": "assura:phase:walk-and-validate",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                }
            ]
        });

        let failures = performance_no_slower_failures(
            &report,
            "realistic-equivalent",
            "assura-cli",
            "ls-lint-cli",
        )
        .expect("report is valid");

        assert_eq!(
            failures,
            vec![
                NoSlowerFailure::InvalidLsLint {
                    fixture_id: "non-native-ls-lint".to_string(),
                    reason: "tool_name \"node-wrapper\" is not native LS-Lint".to_string()
                },
                NoSlowerFailure::InvalidAssura {
                    fixture_id: "skipped-assura".to_string(),
                    reason: "status \"skipped\"".to_string()
                }
            ]
        );
    }

    #[test]
    fn performance_no_slower_requires_accepted_fixture_metadata() {
        let report = json!({
            "results": [
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "missing-acceptance",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "diagnostic-row",
                    "fixture_acceptance": "diagnostic",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 100.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "diagnostic-row",
                    "fixture_acceptance": "diagnostic",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                }
            ]
        });

        let failures = performance_no_slower_failures(
            &report,
            "realistic-equivalent",
            "assura-cli",
            "ls-lint-cli",
        )
        .expect("report is valid");

        assert_eq!(
            failures,
            vec![NoSlowerFailure::InvalidAcceptance {
                fixture_id: "missing-acceptance".to_string(),
                reason: "missing fixture_acceptance".to_string()
            }]
        );
    }

    #[test]
    fn performance_no_slower_requires_target_rows_for_accepted_fixtures() {
        let report = json!({
            "results": [
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "accepted-without-targets",
                    "fixture_acceptance": "accepted-ls-lint-equivalent",
                    "row_family": "assura:phase:walk-and-validate",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                }
            ]
        });

        let failures = performance_no_slower_failures(
            &report,
            "realistic-equivalent",
            "assura-cli",
            "ls-lint-cli",
        )
        .expect("report is valid");

        assert_eq!(
            failures,
            vec![
                NoSlowerFailure::MissingAssura {
                    fixture_id: "accepted-without-targets".to_string()
                },
                NoSlowerFailure::MissingLsLint {
                    fixture_id: "accepted-without-targets".to_string()
                }
            ]
        );
    }

    #[test]
    fn native_performance_gate_accepts_complete_passing_matrix() {
        let mut rows = Vec::new();
        for fixture_id in NATIVE_PERFORMANCE_FIXTURES {
            for row_family in NATIVE_PERFORMANCE_ROW_FAMILIES {
                let expected_assura_exit_status =
                    expected_native_assura_exit_status(fixture_id, row_family);
                rows.push(json!({
                    "fixture_cohort": "assura-native",
                    "fixture_id": fixture_id,
                    "fixture_acceptance": "assura-native-diagnostic",
                    "row_family": row_family,
                    "expected_assura_exit_status": expected_assura_exit_status,
                    "status": "pass",
                    "median_runtime_ms": 1.0,
                    "native_regression_baseline_median_ms": 1.0,
                    "native_regression_threshold_ms": 1.0,
                    "native_regression_delta_ms": 0.0,
                    "native_regression_status": "within-calibrated-baseline",
                    "distribution": {
                        "samples": 1
                    }
                }));
            }
        }
        let report = json!({
            "schema_version": "assura.performance.v1",
            "ls_lint_package": "not-applicable",
            "command_line": "assura performance-report --suite native",
            "results": rows
        });

        let failures = native_performance_failures(&report).expect("report is valid");

        assert!(failures.is_empty());
    }

    #[test]
    fn native_performance_gate_rejects_inconsistent_regression_numbers() {
        let report = json!({
            "schema_version": "assura.performance.v1",
            "ls_lint_package": "not-applicable",
            "command_line": "assura performance-report --suite native",
            "results": [
                {
                    "fixture_cohort": "assura-native",
                    "fixture_id": "native_small",
                    "fixture_acceptance": "assura-native-diagnostic",
                    "row_family": "native:content-check-cli",
                    "expected_assura_exit_status": 0,
                    "status": "pass",
                    "median_runtime_ms": 2.0,
                    "native_regression_baseline_median_ms": 1.0,
                    "native_regression_threshold_ms": 1.5,
                    "native_regression_delta_ms": 1.0,
                    "native_regression_status": "within-calibrated-baseline",
                    "distribution": {
                        "samples": 1
                    }
                }
            ]
        });

        let failures = native_performance_failures(&report).expect("report is valid");

        assert!(failures.iter().any(|failure| failure
            .contains("native_regression_status within-calibrated-baseline disagrees")));
    }

    #[test]
    fn native_performance_gate_accepts_provisional_baseline_status() {
        let mut rows = Vec::new();
        for fixture_id in NATIVE_PERFORMANCE_FIXTURES {
            for row_family in NATIVE_PERFORMANCE_ROW_FAMILIES {
                let expected_assura_exit_status =
                    expected_native_assura_exit_status(fixture_id, row_family);
                rows.push(json!({
                    "fixture_cohort": "assura-native",
                    "fixture_id": fixture_id,
                    "fixture_acceptance": "assura-native-diagnostic",
                    "row_family": row_family,
                    "expected_assura_exit_status": expected_assura_exit_status,
                    "status": "pass",
                    "median_runtime_ms": 1.0,
                    "native_regression_baseline_median_ms": 1.0,
                    "native_regression_baseline_report_count": 2,
                    "native_regression_baseline_sample_count": 10,
                    "native_regression_threshold_ms": 1.0,
                    "native_regression_delta_ms": 0.0,
                    "native_regression_status": "within-calibrated-baseline",
                    "distribution": {
                        "samples": 1
                    }
                }));
            }
        }
        rows[0]["median_runtime_ms"] = json!(1.2);
        rows[0]["native_regression_baseline_report_count"] = json!(1);
        rows[0]["native_regression_baseline_sample_count"] = json!(5);
        rows[0]["native_regression_threshold_ms"] = json!(1.25);
        rows[0]["native_regression_delta_ms"] = json!(0.2);
        rows[0]["native_regression_status"] = json!("within-provisional-baseline");
        let report = json!({
            "schema_version": "assura.performance.v1",
            "ls_lint_package": "not-applicable",
            "command_line": "assura performance-report --suite native",
            "results": rows
        });

        let failures = native_performance_failures(&report).expect("report is valid");

        assert!(failures.is_empty());
    }

    #[test]
    fn native_performance_gate_reports_skipped_and_missing_rows() {
        let report = json!({
            "schema_version": "assura.performance.v1",
            "ls_lint_package": "not-applicable",
            "command_line": "assura performance-report --suite native",
            "results": [
                {
                    "fixture_cohort": "assura-native",
                    "fixture_id": "native_small",
                    "fixture_acceptance": "assura-native-diagnostic",
                    "row_family": "native:content-check-cli",
                    "expected_assura_exit_status": 1,
                    "status": "skipped",
                    "details": "expected exit 0, got Some(2)",
                    "distribution": {
                        "samples": 0
                    }
                }
            ]
        });

        let failures = native_performance_failures(&report).expect("report is valid");

        assert!(failures.iter().any(
            |failure| failure.contains("native_small native:content-check-cli: status skipped")
        ));
        assert!(failures.iter().any(|failure| failure.contains(
            "native_adapter_mix native:agent-query-keyword-search-cli: missing native matrix row"
        )));
    }

    #[test]
    fn markdown_engine_probe_options_parse_measurement_flags() {
        let options = parse_markdown_engine_probe_options(&[
            "--candidate".to_string(),
            "rumdl".to_string(),
            "--fixture".to_string(),
            "frontmatter-link-heavy".to_string(),
            "--run-external".to_string(),
            "--measure".to_string(),
            "--iterations".to_string(),
            "7".to_string(),
        ])
        .expect("options parse");

        assert_eq!(options.candidate.as_deref(), Some("rumdl"));
        assert_eq!(options.fixture.as_deref(), Some("frontmatter-link-heavy"));
        assert!(options.run_external);
        assert!(options.measure);
        assert_eq!(markdown_engine_probe_iterations(&options), 7);
    }

    #[test]
    fn markdown_engine_probe_fixture_rejects_scope_outside_fixture_root() {
        let root = std::env::temp_dir().join(format!("assura-xtask-scope-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let fixture_root = root.join("fixtures");
        std::fs::create_dir_all(fixture_root.join("selected/docs")).unwrap();
        std::fs::create_dir_all(root.join("outside/docs")).unwrap();
        let matrix = json!({
            "probe_profiles": {
                "bad": {
                    "path": "selected",
                    "markdown_scope": "../outside/docs"
                }
            }
        });
        let options = MarkdownEngineProbeOptions {
            fixture: Some("bad".to_string()),
            ..MarkdownEngineProbeOptions::default()
        };

        let error = markdown_engine_probe_fixture(&root, &fixture_root, &matrix, &options)
            .expect_err("scope outside fixture root must fail")
            .to_string();
        assert!(error.contains("markdown_scope must stay under fixture path"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn markdown_engine_timing_report_sorts_and_summarizes_samples() {
        let report = timing_report(4, vec![10.0, 1.0, 20.0, 5.0], vec!["boom".to_string()]);

        assert_eq!(report["iterations"], 4);
        assert_eq!(report["successful_runs"], 4);
        assert_eq!(report["failed_runs"], 1);
        assert_eq!(report["median_ms"], 5.0);
        assert_eq!(report["p95_ms"], 20.0);
        assert_eq!(report["min_ms"], 1.0);
        assert_eq!(report["max_ms"], 20.0);
        assert_eq!(report["samples_ms"], json!([1.0, 5.0, 10.0, 20.0]));
    }

    #[test]
    fn markdown_engine_probe_status_distinguishes_findings_from_failures() {
        assert_eq!(markdown_probe_status(Some(0)), "ran");
        assert_eq!(markdown_probe_status(Some(1)), "ran_with_findings");
        assert_eq!(markdown_probe_status(Some(2)), "probe_failed");
        assert_eq!(markdown_probe_status(None), "probe_failed");
    }

    #[test]
    #[cfg(unix)]
    fn markdown_engine_fix_output_detects_mdlint_failed_fixes() {
        use std::os::unix::process::ExitStatusExt;

        let candidate = MarkdownEngineCandidate {
            name: "mdlint",
            binary: Some("mdlint"),
            probe_args: &[],
            fix_args: Some(&[]),
        };
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(1 << 8),
            stdout: Vec::new(),
            stderr: b"Failed to apply fixes: Cannot apply fixes".to_vec(),
        };

        assert!(!expected_markdown_fix_output(&candidate, &output));
    }

    #[test]
    fn markdown_engine_fix_validation_helpers_track_preservation() {
        let before = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\n".to_string(),
            frontmatter: leading_frontmatter_block("---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\n"),
            line_endings: line_ending_style("---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\n"),
        }];
        let after = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\nBody\r\n".to_string(),
            frontmatter: leading_frontmatter_block(
                "---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\nBody\r\n",
            ),
            line_endings: line_ending_style("---\r\ntitle: Note\r\n---\r\n\r\n# Note\r\nBody\r\n"),
        }];

        assert_eq!(
            before[0].frontmatter.as_deref(),
            Some("---\r\ntitle: Note\r\n---\r\n")
        );
        assert_eq!(before[0].line_endings, "crlf");
        assert_eq!(
            changed_markdown_files(&before, &after),
            vec!["docs/note.md".to_string()]
        );
        assert!(markdown_frontmatter_preserved(&before, &after));
        assert!(markdown_line_endings_preserved(&before, &after));
    }

    #[test]
    fn markdown_engine_fix_validation_helpers_detect_frontmatter_loss() {
        let before = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "---\ntitle: Note\n---\n\n# Note\n".to_string(),
            frontmatter: leading_frontmatter_block("---\ntitle: Note\n---\n\n# Note\n"),
            line_endings: line_ending_style("---\ntitle: Note\n---\n\n# Note\n"),
        }];
        let after = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "title: Note\n---\n\n# Note\n".to_string(),
            frontmatter: leading_frontmatter_block("title: Note\n---\n\n# Note\n"),
            line_endings: line_ending_style("title: Note\n---\n\n# Note\n"),
        }];

        assert!(!markdown_frontmatter_preserved(&before, &after));
        assert!(markdown_line_endings_preserved(&before, &after));
    }

    #[test]
    fn markdown_engine_fix_validation_helpers_detect_empty_frontmatter_loss() {
        let before = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "---\n---\n\n# Note\n".to_string(),
            frontmatter: leading_frontmatter_block("---\n---\n\n# Note\n"),
            line_endings: line_ending_style("---\n---\n\n# Note\n"),
        }];
        let after = vec![MarkdownFileSnapshot {
            path: "docs/note.md".to_string(),
            content: "# Note\n".to_string(),
            frontmatter: leading_frontmatter_block("# Note\n"),
            line_endings: line_ending_style("# Note\n"),
        }];

        assert_eq!(before[0].frontmatter.as_deref(), Some("---\n---\n"));
        assert!(!markdown_frontmatter_preserved(&before, &after));
    }

    #[test]
    fn roadmap_href_mapping_accepts_local_and_github_details() {
        assert_eq!(label_word_count("Structure Validation"), 2);
        assert_eq!(label_word_count("Content Collections Querying"), 3);
        assert_eq!(
            backticked_value_after_marker(
                "Current recommended goal:\n`docs/goals/example.md`.\n",
                "Current recommended goal:"
            ),
            Some("docs/goals/example.md")
        );
        assert_eq!(
            backticked_value_after_marker(
                "Current recommended goal:\nmissing\n",
                "Current recommended goal:"
            ),
            None
        );

        assert!(roadmap_href_matches_detail(
            "/product/structure-validation/",
            "website/src/content/docs/product/structure-validation.md"
        ));
        assert!(roadmap_href_matches_detail(
            "https://github.com/rothnic/assura/blob/master/docs/goals/assura-beta-code-agnostic-capabilities-program.md",
            "docs/goals/assura-beta-code-agnostic-capabilities-program.md"
        ));
        assert!(!roadmap_href_matches_detail(
            "/product/structure-validation/",
            "docs/goals/assura-beta-code-agnostic-capabilities-program.md"
        ));
        assert!(!roadmap_href_matches_detail(
            "https://example.com/docs/goals/assura-beta-code-agnostic-capabilities-program.md",
            "docs/goals/assura-beta-code-agnostic-capabilities-program.md"
        ));
    }

    #[test]
    fn release_readiness_helpers_extract_version_and_surface_report() {
        assert_eq!(
            release_notes_version("# Assura v0.2.0 Current Branch Release Notes"),
            Some("0.2.0".to_string())
        );
        assert_eq!(release_notes_version("# No version here"), None);

        let report = release_surfaces_report("docs/data/release-surfaces.json", Some("v0.2.0"));
        assert_eq!(
            report.get("schema_version").and_then(Value::as_str),
            Some("assura.release-surfaces.v1")
        );
        let unreleased = report
            .get("unreleased_user_facing_changes")
            .and_then(Value::as_array)
            .expect("unreleased surfaces array");
        if !unreleased.is_empty() {
            assert!(unreleased
                .iter()
                .any(|surface| surface.get("id").and_then(Value::as_str)
                    == Some("agent-ready-doctor-explain")));
            assert!(unreleased.iter().all(|surface| {
                surface.get("id").and_then(Value::as_str)
                    != Some("project-intelligence-local-surfaces")
            }));
            let surfaces = serde_json::from_str::<Value>(&read("docs/data/release-surfaces.json"))
                .expect("release surfaces json");
            assert!(surfaces
                .get("surfaces")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|surface| surface.get("id").and_then(Value::as_str)
                    == Some("project-intelligence-local-surfaces")
                    && surface.get("first_release").and_then(Value::as_str) == Some("v0.2.0")));
        } else {
            let surfaces = serde_json::from_str::<Value>(&read("docs/data/release-surfaces.json"))
                .expect("release surfaces json");
            assert!(surfaces
                .get("surfaces")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|surface| surface.get("id").and_then(Value::as_str)
                    == Some("project-intelligence-local-surfaces")
                    && surface.get("first_release").and_then(Value::as_str) == Some("v0.2.0")));
        }
    }

    #[test]
    fn release_readiness_report_passes_for_next_release_candidate() {
        let report = release_readiness_report_from_inputs(
            "0.2.0",
            "0.2.0",
            "A release PR cannot close if",
            release_checklist_fixture(),
            "Compatibility And Public Surface",
            serde_json::json!({
                "schema_version": "assura.release-surfaces.v1",
                "path": "docs/data/release-surfaces.json",
                "surface_count": 2,
                "unreleased_user_facing_changes": []
            }),
            serde_json::json!({ "tagName": "v0.1.0" }),
        );
        assert_eq!(
            report.get("schema_version").and_then(Value::as_str),
            Some("assura.release-readiness.v1")
        );
        assert_eq!(report.get("ready").and_then(Value::as_bool), Some(true));
        assert_eq!(report.get("verdict").and_then(Value::as_str), Some("pass"));
        assert!(report
            .get("reasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| reasons.is_empty()));
    }

    #[test]
    fn release_readiness_report_fails_when_latest_tag_has_unreleased_surfaces() {
        let report = release_readiness_report_from_inputs(
            "0.2.0",
            "0.2.0",
            "A release PR cannot close if",
            release_checklist_fixture(),
            "Compatibility And Public Surface",
            serde_json::json!({
                "schema_version": "assura.release-surfaces.v1",
                "path": "docs/data/release-surfaces.json",
                "surface_count": 1,
                "unreleased_user_facing_changes": [{
                    "id": "project-intelligence-local-surfaces",
                    "status": "supported",
                    "first_release": "unreleased",
                    "detail_path": "docs/release-notes.md"
                }]
            }),
            serde_json::json!({ "tagName": "v0.2.0" }),
        );
        assert_eq!(report.get("ready").and_then(Value::as_bool), Some(false));
        assert_eq!(report.get("verdict").and_then(Value::as_str), Some("fail"));
        assert!(report
            .get("reasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| reasons.iter().any(|reason| reason
                .as_str()
                .is_some_and(|reason| reason.contains("already the latest GitHub release")))));
    }

    #[test]
    fn release_surfaces_report_rejects_placeholder_supported_releases() {
        let path = env::temp_dir().join(format!(
            "assura-release-surfaces-invalid-{}.json",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{
              "schema_version": "assura.release-surfaces.v1",
              "surfaces": [
                {
                  "id": "daemon-management-cli-preview",
                  "status": "experimental",
                  "first_release": "future",
                  "detail_path": "docs/release-notes.md"
                }
              ]
            }"#,
        )
        .expect("write release surface fixture");
        let report =
            release_surfaces_report(path.to_str().expect("utf-8 temp path"), Some("v0.2.0"));
        let error = report
            .get("error")
            .and_then(Value::as_str)
            .expect("surface validation error");
        assert!(error.contains("invalid first_release"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn release_surfaces_report_rejects_later_supported_release() {
        let path = env::temp_dir().join(format!(
            "assura-release-surfaces-later-{}.json",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{
              "schema_version": "assura.release-surfaces.v1",
              "surfaces": [
                {
                  "id": "future-supported-surface",
                  "status": "supported",
                  "first_release": "v0.3.0",
                  "detail_path": "docs/release-notes.md"
                }
              ]
            }"#,
        )
        .expect("write release surface fixture");
        let report =
            release_surfaces_report(path.to_str().expect("utf-8 temp path"), Some("v0.2.0"));
        let error = report
            .get("error")
            .and_then(Value::as_str)
            .expect("surface validation error");
        assert!(error.contains("after local release tag"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn warm_loop_gate_checks_every_budget_row() {
        let budgets = BTreeMap::from([
            ("fast".to_string(), 10.0),
            ("slow".to_string(), 20.0),
            ("missing".to_string(), 30.0),
        ]);
        let report = json!({
            "schema_version": "assura.warm-loop-performance.v1",
            "rows": [
                {"id": "fast", "p95_ms": 9.5, "median_ms": 9.5, "iterations": WARM_LOOP_MIN_ITERATIONS, "samples_ms": vec![9.5; WARM_LOOP_MIN_ITERATIONS]},
                {"id": "slow", "p95_ms": 20.5, "median_ms": 20.5, "iterations": WARM_LOOP_MIN_ITERATIONS, "samples_ms": vec![20.5; WARM_LOOP_MIN_ITERATIONS]}
            ]
        });

        let failures = warm_loop_regression_failures(&report, &budgets).unwrap();

        assert_eq!(failures.len(), 3);
        assert!(failures.iter().any(|failure| failure.contains("row count")));
        assert!(failures.iter().any(|failure| failure.contains("slow: p95")));
        assert!(failures
            .iter()
            .any(|failure| failure == "missing: missing measured row"));
    }

    #[test]
    fn warm_loop_gate_rejects_missing_or_inconsistent_samples() {
        let budgets = BTreeMap::from([("no-change-warm-review".to_string(), 10.0)]);
        let report = json!({
            "schema_version": "assura.warm-loop-performance.v1",
            "rows": [{
                "id": "no-change-warm-review",
                "command": warm_loop_display_command(WarmLoopScenarioKind::NoChangeReview),
                "iterations": WARM_LOOP_MIN_ITERATIONS,
                "median_ms": 1.0,
                "p95_ms": 1.0,
                "samples_ms": []
            }]
        });

        let failures = warm_loop_regression_failures(&report, &budgets).unwrap();

        assert!(failures
            .iter()
            .any(|failure| failure.contains("sample count 0")));
    }

    #[test]
    fn warm_loop_scenarios_match_the_versioned_product_budget_lane() {
        let ids = warm_loop_scenarios()
            .iter()
            .map(|scenario| scenario.id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "no-change-warm-review",
                "one-file-change",
                "directory-create-delete",
                "config-change",
                "agent-nudge"
            ]
        );
    }

    fn release_checklist_fixture() -> &'static str {
        "cargo fmt --all -- --check\n\
         cargo test --all-targets --quiet\n\
         cargo clippy --all-targets --all-features -- -D warnings\n\
         cargo xtask release-readiness --format json\n\
         cargo xtask release-smoke\n\
         cargo xtask release-live"
    }
}
