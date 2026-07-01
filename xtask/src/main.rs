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
        "release-readiness" => run_release_readiness(&rest),
        "performance-no-slower" => run_performance_no_slower(&rest),
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
        "Usage: cargo xtask <fast|check|test|evidence|target-state|hygiene|docs|release-size|release-smoke|release-live|release-readiness|performance-no-slower|changed|pr|full>"
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
    check_manifest_semantics(&mut checks);
    check_test_relationships(&mut checks);
    check_docs_release_performance(&mut checks);
    check_public_roadmap(&mut checks);
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
            "Performance no-slower gate passed for cohort {} ({} <= {}).",
            options.cohort, options.assura_row, options.ls_lint_row
        );
        return Ok(());
    }

    eprintln!(
        "Performance no-slower gate failed for cohort {} ({} must be <= {}).",
        options.cohort, options.assura_row, options.ls_lint_row
    );
    for failure in failures {
        match failure {
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
    let mut timings = BTreeMap::<String, FixtureTiming>::new();

    for row in rows {
        if row.get("fixture_cohort").and_then(Value::as_str) != Some(cohort) {
            continue;
        }
        let fixture_id = row
            .get("fixture_id")
            .and_then(Value::as_str)
            .ok_or("performance row missing fixture_id")?;
        let timing = timings.entry(fixture_id.to_string()).or_default();

        let Some(row_family) = row.get("row_family").and_then(Value::as_str) else {
            continue;
        };
        if row_family == assura_row {
            timing.assura = Some(timing_from_row(row));
        } else if row_family == ls_lint_row {
            timing.ls_lint = Some(native_ls_lint_timing_from_row(row));
        }
    }

    if timings.is_empty() {
        return Err(format!("performance report has no fixture rows for cohort {cohort}").into());
    }

    let mut failures = Vec::new();
    for (fixture_id, timing) in timings {
        match (timing.assura, timing.ls_lint) {
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
        variant_name: "Init",
        command_surface_names: &["assura init"],
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
            "assura agent nudge",
            "assura agent session",
        ],
        support_policy_markers: &["`assura agent`"],
        compatibility_markers: &[
            "| `assura agent` | Supported local agent project-intelligence surface |",
            "| `assura agent nudge` | Experimental local agent nudge payload |",
            "| `assura agent session` | Supported local agent session alias |",
        ],
        source_markers: &["Commands::Agent", "AgentCommands::Context", "AgentCommands::Nudge"],
        test_markers: &[
            "tests/agent_surface_cli.rs",
            "agent_surface_defaults_to_json_and_reuses_content_contracts",
            "agent_nudge_after_tool_reports_bounded_changed_path_findings",
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
        support_policy_markers: &["| `assura daemon` | Experimental local daemon management preview |"],
        compatibility_markers: &[
            "| `assura daemon` | Experimental local daemon management preview |",
            "| `assura daemon status` | Experimental local daemon status preview |",
            "| `assura daemon start` | Experimental local daemon lifecycle preview |",
            "| `assura daemon stop` | Experimental local daemon lifecycle preview |",
            "| `assura daemon restart` | Experimental local daemon lifecycle preview |",
            "| `assura daemon doctor` | Experimental local daemon doctor preview |",
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
    let performance_cases_text =
        read("website/src/content/docs/reference/performance-test-cases.mdx");
    let why_assura_text = read("website/src/content/docs/why-assura.md");
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
        "performance current.json: source_worktree_dirty must be false",
    );
    checks.require(
        bench_current
            .pointer("/claim_summary/two_x_claim_verdict")
            .is_some()
            && bench_current
                .pointer("/warm_claim_summary/two_x_claim_verdict")
                .is_some(),
        "performance current.json: missing cold or warm claim verdict",
    );
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
        text_contains_ordered(
            &ci_workflow,
            &[
                "performance:\n    name: Performance Report",
                "- name: Generate comparison report",
                "--output target/performance/ls-lint-comparison.json",
                "--iterations 5",
                "- name: Enforce no-slower gate",
                "run: cargo xtask performance-no-slower target/performance/ls-lint-comparison.json",
                "- name: Summarize performance",
                "if: always()",
                "- name: Upload performance artifact",
                "if: always()",
            ],
        ),
        ".github/workflows/ci.yml: Performance Report job must generate a 5-iteration report, enforce cargo xtask performance-no-slower on that report, and keep summary/artifact steps on failure",
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
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 4.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "faster",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 5.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "equal",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 6.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "equal",
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
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 7.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "slower",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 5.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "missing-ls-lint",
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "missing-assura",
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
                    "row_family": "assura-cli",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "non-native-ls-lint",
                    "row_family": "ls-lint-cli",
                    "tool_name": "node-wrapper",
                    "ls_lint_execution_mode": "node-wrapper",
                    "status": "pass",
                    "median_runtime_ms": 2.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "skipped-assura",
                    "row_family": "assura-cli",
                    "status": "skipped"
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "skipped-assura",
                    "row_family": "ls-lint-cli",
                    "tool_name": "ls-lint-native-cli",
                    "ls_lint_execution_mode": "native-binary-from-pinned-npm-package",
                    "status": "pass",
                    "median_runtime_ms": 1.0
                },
                {
                    "fixture_cohort": "realistic-equivalent",
                    "fixture_id": "no-target-rows",
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
                    fixture_id: "no-target-rows".to_string()
                },
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
                    == Some("project-intelligence-local-surfaces")));
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

    fn release_checklist_fixture() -> &'static str {
        "cargo fmt --all -- --check\n\
         cargo test --all-targets --quiet\n\
         cargo clippy --all-targets --all-features -- -D warnings\n\
         cargo xtask release-readiness --format json\n\
         cargo xtask release-smoke\n\
         cargo xtask release-live"
    }
}
