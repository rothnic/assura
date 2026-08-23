//! Deterministic product evidence for the public marketing site.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const FIXTURE_SOURCE: &str = "tests/fixtures/real-project-agentic-feedback/valid";
const INTELLIGENCE_FIXTURE_SOURCE: &str = "tests/fixtures/content_runtime/valid";
const OUTPUT_DIR: &str = "website/src/data";
const RELEASE_SURFACES_PATH: &str = "docs/data/release-surfaces.json";
const CONFIG_EXAMPLES_DIR: &str = "website/src/data/config-examples";
const MARKETING_DATA_PATH: &str = "website/src/data/marketing.ts";
const PINNED_SOURCE_PREVIEW_FILES: &[&str] = &[
    "website/src/data/review-demo.json",
    "website/src/data/check-demo.json",
    "website/src/data/onboarding-demo.json",
    "website/src/data/intelligence-demo.json",
    "src/cli/agent_onboarding_templates.rs",
    "src/cli/agent_onboarding_handoff_templates.rs",
];
const FORBIDDEN_WEBSITE_COMMANDS: &[&str] = &["assura review --path"];

#[derive(Deserialize)]
struct ReleaseSurfaceManifest {
    surfaces: Vec<ReleaseSurface>,
}

#[derive(Deserialize)]
struct ReleaseSurface {
    id: String,
    status: String,
    first_release: String,
    #[serde(default)]
    marketing_claim: bool,
    evidence_status: Option<String>,
    evidence_refs: Option<Vec<String>>,
    command: Option<String>,
    smoke_args: Option<Vec<String>>,
    expected_exit: Option<i32>,
}

pub(crate) fn run(args: &[String]) -> Result<()> {
    let (check_only, require_released) = execution_mode(args)?;

    let root = std::env::current_dir()?.canonicalize()?;
    if !root.join("Cargo.toml").is_file() || !root.join("website").is_dir() {
        return Err("website-demo-data must run from the repository root".into());
    }
    build_assura(&root)?;
    let binary = root.join("target/debug/assura-full");
    validate_lslint_config_example(&root, &binary)?;
    let policy_demo = validate_homepage_policy_example(&root, &binary)?;
    validate_agentic_monorepo_example(&root, &binary)?;
    println!("Website Assura config examples are valid.");
    let claims_fixture = prepare_fixture(&root, "claims")?;
    let (onboarding_work, onboarding, onboarding_text) =
        prepare_onboarding_evidence(&root, &binary)?;
    validate_claims(&root, &binary, &claims_fixture, require_released)?;
    validate_setup_delivery(&root, require_released)?;
    validate_website_commands(&root)?;
    validate_docs_config_examples(&root, &binary)?;

    let review_json_fixture = prepare_fixture(&root, "review-json")?;
    let review_text_fixture = prepare_fixture(&root, "review-text")?;
    let check_json_fixture = prepare_fixture(&root, "check-json")?;
    let check_text_fixture = prepare_fixture(&root, "check-text")?;
    let review = run_json_at(
        &binary,
        &["review", "--format", "json", "."],
        &review_json_fixture,
    )?;
    let review_text = normalize_project_path(
        &run_text_at(
            &binary,
            &["review", "--format", "text", "."],
            &review_text_fixture,
        )?,
        &review_text_fixture,
    );
    let check = run_json_at(
        &binary,
        &["check", "--format", "json", "."],
        &check_json_fixture,
    )?;
    let check_text = normalize_project_path(
        &run_text_at(
            &binary,
            &["check", "--format", "text", "."],
            &check_text_fixture,
        )?,
        &check_text_fixture,
    );
    let intelligence_fixture = root.join(INTELLIGENCE_FIXTURE_SOURCE);
    let intelligence_args = [
        "content",
        "context-pack",
        ".",
        "--collection",
        "goals",
        "--id",
        "goal-portable-structure",
        "--format",
    ];
    let mut intelligence_json_args = intelligence_args.to_vec();
    intelligence_json_args.push("json");
    let intelligence = run_json_at(&binary, &intelligence_json_args, &intelligence_fixture)?;
    let mut intelligence_text_args = intelligence_args.to_vec();
    intelligence_text_args.push("text");
    let intelligence_text = run_text_at(&binary, &intelligence_text_args, &intelligence_fixture)?;
    let performance = read_json(root.join("website/public/data/performance/current.json"))?;

    let outputs = [
        ("review-demo.json", compact_review(&review, &review_text)?),
        ("check-demo.json", compact_check(&check, &check_text)?),
        (
            "onboarding-demo.json",
            compact_onboarding(&onboarding, &onboarding_text)?,
        ),
        (
            "intelligence-demo.json",
            compact_intelligence(&intelligence, &intelligence_text)?,
        ),
        ("policy-demo.json", policy_demo),
        (
            "performance-summary.json",
            compact_performance(&root, &performance)?,
        ),
    ];

    let output_dir = root.join(OUTPUT_DIR);
    fs::create_dir_all(&output_dir)?;
    let mut stale = Vec::new();
    for (name, value) in outputs {
        let path = output_dir.join(name);
        let content = format!("{}\n", serde_json::to_string_pretty(&value)?);
        if check_only {
            if fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
                stale.push(path);
            }
        } else {
            fs::write(&path, content)?;
            println!("generated {}", path.display());
        }
    }

    for fixture in [
        claims_fixture,
        review_json_fixture,
        review_text_fixture,
        check_json_fixture,
        check_text_fixture,
    ] {
        let _ = fs::remove_dir_all(fixture.parent().unwrap_or(&fixture));
    }
    let _ = fs::remove_dir_all(onboarding_work);
    if stale.is_empty() {
        if check_only {
            println!("Website demo data is current.");
        }
        Ok(())
    } else {
        let paths = stale
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(
            format!("website demo data is stale: {paths}; run `cargo xtask website-demo-data`")
                .into(),
        )
    }
}

fn execution_mode(args: &[String]) -> Result<(bool, bool)> {
    let require_released = args.iter().any(|arg| arg == "--released");
    if args
        .iter()
        .any(|arg| !matches!(arg.as_str(), "--check" | "--released"))
    {
        return Err("Usage: cargo xtask website-demo-data [--check] [--released]".into());
    }
    Ok((
        require_released || args.iter().any(|arg| arg == "--check"),
        require_released,
    ))
}

pub(crate) fn validate_config_examples(args: &[String]) -> Result<()> {
    if !args.is_empty() {
        return Err("Usage: cargo xtask website-config-examples".into());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    if !root.join("Cargo.toml").is_file() || !root.join("website").is_dir() {
        return Err("website-config-examples must run from the repository root".into());
    }
    build_assura(&root)?;
    validate_lslint_config_example(&root, &root.join("target/debug/assura-full"))?;
    validate_homepage_policy_example(&root, &root.join("target/debug/assura-full"))?;
    validate_agentic_monorepo_example(&root, &root.join("target/debug/assura-full"))?;
    validate_docs_config_examples(&root, &root.join("target/debug/assura-full"))?;
    println!("Website Assura config examples are valid.");
    Ok(())
}

fn validate_docs_config_examples(root: &Path, binary: &Path) -> Result<()> {
    let docs_root = root.join("website/src/content/docs");
    let mut files = Vec::new();
    collect_files(&docs_root, &mut files)?;
    let mut checked = 0usize;

    for path in files {
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("md" | "mdx")
        ) {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        let mut info = None::<String>;
        let mut body = String::new();
        let mut fence_index = 0usize;
        for event in Parser::new_ext(&source, Options::all()) {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(value))) => {
                    info = Some(value.to_string());
                    body.clear();
                }
                Event::Text(value) if info.is_some() => body.push_str(&value),
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    fence_index += 1;
                    let metadata = info.take().unwrap_or_default();
                    if is_yaml_fence(&metadata) {
                        validate_yaml_fence(root, binary, &path, fence_index, &metadata, &body)?;
                        checked += 1;
                    }
                    body.clear();
                }
                _ => {}
            }
        }
    }

    println!("Validated {checked} documentation YAML examples.");
    Ok(())
}

fn is_yaml_fence(metadata: &str) -> bool {
    matches!(metadata.split_whitespace().next(), Some("yaml" | "yml"))
}

fn validate_yaml_fence(
    root: &Path,
    binary: &Path,
    source_path: &Path,
    fence_index: usize,
    metadata: &str,
    body: &str,
) -> Result<()> {
    let parsed: serde_yaml::Value = serde_yaml::from_str(body).map_err(|error| {
        format!(
            "{} YAML fence #{fence_index} is invalid: {error}",
            source_path.display()
        )
    })?;
    if metadata.contains("config-fragment")
        || metadata.contains("data-yaml")
        || metadata.contains("ls-lint-config")
    {
        return Ok(());
    }

    let Some(mapping) = parsed.as_mapping() else {
        return Ok(());
    };
    let looks_like_config = [
        "structure",
        "rules",
        "exclude",
        "extensions",
        "models",
        "collections",
        "relations",
        "code_symbols",
        "quality",
    ]
    .iter()
    .any(|key| mapping.contains_key(serde_yaml::Value::String((*key).to_string())));
    if !looks_like_config {
        return Ok(());
    }

    let work = root.join("target").join(format!(
        "website-doc-example-{}-{fence_index}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(work.join(".assura"))?;
    fs::write(work.join(".assura/config.yml"), body)?;
    let output = Command::new(binary)
        .current_dir(&work)
        .args(["status", ".", "--format", "json"])
        .output()?;
    let expected_invalid = metadata.contains("assura-config-invalid");
    let valid = output.status.success();
    let _ = fs::remove_dir_all(&work);

    if valid == expected_invalid {
        let expectation = if expected_invalid { "fail" } else { "load" };
        return Err(format!(
            "{} YAML fence #{fence_index} was expected to {expectation} as an Assura config: {}",
            source_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(())
}

fn validate_agentic_monorepo_example(root: &Path, binary: &Path) -> Result<()> {
    let project = example_project(root, "agentic-monorepo")?;
    install_example_config(root, &project, "agentic-monorepo.yml")?;
    fs::create_dir_all(project.join("docs"))?;
    fs::write(
        project.join("docs/agent-guidance.md"),
        "# Agent Guidance\n\n## Skills\n\n## Layout\n",
    )?;
    fs::write(project.join("docs/structure.md"), "# Project Structure\n")?;
    validate_repair_document_references(&project)?;
    fs::write(project.join("AGENTS.md"), "# Root guidance\n")?;
    fs::write(project.join("package.json"), "{}\n")?;
    fs::write(project.join("pnpm-lock.yaml"), "lockfileVersion: 9\n")?;
    fs::write(
        project.join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*\n",
    )?;
    fs::write(project.join("README.md"), "# Project\n")?;
    fs::write(project.join("turbo.json"), "{}\n")?;
    for package in ["core", "ui-kit"] {
        let package_root = project.join("packages").join(package);
        fs::create_dir_all(package_root.join("src"))?;
        fs::write(package_root.join("AGENTS.md"), "# Package guidance\n")?;
        fs::write(package_root.join("package.json"), "{}\n")?;
        fs::write(
            package_root.join("src").join(format!("{package}.ts")),
            "export const value = true;\n",
        )?;
        fs::write(
            package_root.join("src").join(format!("{package}-view.tsx")),
            "export const value = true;\n",
        )?;
    }
    fs::create_dir_all(project.join("packages/ui-kit/stories"))?;
    fs::write(
        project.join("packages/core/README.md"),
        "Package documentation.\n".repeat(300),
    )?;
    run_example_check(binary, &project, true)?;

    fs::write(
        project.join("packages/core/src/too-long.ts"),
        "export const value = true;\n".repeat(501),
    )?;
    fs::write(
        project.join("packages/ui-kit/src/BadName.tsx"),
        "export const value = true;\n",
    )?;
    fs::create_dir_all(project.join("packages/core/tests"))?;
    fs::write(
        project.join("packages/core/tests/BadName.test.ts"),
        "export const value = true;\n",
    )?;
    let report = run_example_check(binary, &project, false)?;
    require_example_violation("agentic-monorepo.yml", &report, "too-long.ts", "max_lines")?;
    require_example_violation(
        "agentic-monorepo.yml",
        &report,
        "BadName.tsx",
        "file_naming",
    )?;
    require_example_violation(
        "agentic-monorepo.yml",
        &report,
        "BadName.test.ts",
        "file_naming",
    )?;
    fs::remove_dir_all(project.parent().unwrap_or(&project))?;
    Ok(())
}

fn validate_homepage_policy_example(root: &Path, binary: &Path) -> Result<Value> {
    const SOURCE_LINE_LIMIT: usize = 500;
    let project = example_project(root, "homepage-agentic-project")?;
    install_example_config(root, &project, "homepage-agentic-project.yml")?;
    fs::create_dir_all(project.join("docs"))?;
    fs::write(project.join("docs/agents.md"), "# Agent Guidance\n")?;
    fs::write(project.join("AGENTS.md"), "# Root guidance\n")?;
    fs::write(project.join("README.md"), "# Project\n")?;
    let package = project.join("packages/core");
    fs::create_dir_all(package.join("src"))?;
    fs::write(package.join("AGENTS.md"), "# Package guidance\n")?;
    fs::write(package.join("package.json"), "{}\n")?;
    fs::write(
        package.join("src/user-menu.ts"),
        "export const userMenuPart = true;\n".repeat(184),
    )?;
    let baseline_report = run_example_check(binary, &project, true)?;

    fs::remove_file(package.join("AGENTS.md"))?;
    fs::remove_file(package.join("package.json"))?;
    let report = run_example_check(binary, &project, false)?;
    require_example_violation_message(
        "homepage-agentic-project.yml",
        &report,
        "packages/core",
        "exists_count",
        "AGENTS.md",
    )?;
    require_example_violation_message(
        "homepage-agentic-project.yml",
        &report,
        "packages/core",
        "exists_count",
        "package.json",
    )?;
    fs::write(package.join("AGENTS.md"), "# Package guidance\n")?;
    fs::write(package.join("package.json"), "{}\n")?;

    fs::write(
        package.join("src/BadName.ts"),
        "export const badName = true;\n",
    )?;
    fs::write(
        package.join("src/checkout-flow.ts"),
        "export const checkoutFlowPart = true;\n".repeat(537),
    )?;
    let report = run_example_check(binary, &project, false)?;
    require_example_violation(
        "homepage-agentic-project.yml",
        &report,
        "BadName.ts",
        "file_naming",
    )?;
    require_example_violation(
        "homepage-agentic-project.yml",
        &report,
        "checkout-flow.ts",
        "max_lines",
    )?;
    let naming_violation = find_example_violation(&report, "BadName.ts", "file_naming")?;
    let line_violation = find_example_violation(&report, "checkout-flow.ts", "max_lines")?;
    let config = fs::read_to_string(project.join(".assura/config.yml"))?;
    if !config.contains(&format!("max_lines:{SOURCE_LINE_LIMIT}")) {
        return Err("homepage policy source line limit is not represented in its config".into());
    }
    let user_menu_lines = file_line_count(&package.join("src/user-menu.ts"))?;
    let checkout_lines = file_line_count(&package.join("src/checkout-flow.ts"))?;
    let artifact = json!({
        "schema": "assura.website-policy-demo.v1",
        "generated_from": "assura check --format json .",
        "presentation": "fixture_explanation",
        "command": "assura check --format json .",
        "config": config.trim_end(),
        "markers": [
            {"id": "1", "match": "AGENTS.md: exists:1 | $agent-doc", "label": "required agent guidance"},
            {"id": "2", "match": "./*/: $workspace", "label": "reused workspace contract"},
            {"id": "3", "match": ".ts: kebab-case | max_lines:500", "label": "source naming and line limit"}
        ],
        "tree": [
            {"path": "project/", "depth": 0, "status": "context", "detail": "top-down policy"},
            {"path": "AGENTS.md", "depth": 1, "status": "observed", "detail": "present in passing fixture", "marker": "1"},
            {"path": "packages/core/", "depth": 1, "status": "context", "detail": "workspace contract", "marker": "2"},
            {"path": "AGENTS.md", "full_path": "packages/core/AGENTS.md", "depth": 2, "status": "observed", "detail": "present in passing fixture", "marker": "1"},
            {"path": "package.json", "full_path": "packages/core/package.json", "depth": 2, "status": "observed", "detail": "present in passing fixture"},
            {"path": "src/", "full_path": "packages/core/src/", "depth": 2, "status": "context", "detail": "recursive defaults"},
            {"path": "user-menu.ts", "full_path": "packages/core/src/user-menu.ts", "depth": 3, "status": "observed", "detail": format!("{user_menu_lines} / {SOURCE_LINE_LIMIT} lines in passing fixture"), "marker": "3"},
            {"path": "BadName.ts", "full_path": "packages/core/src/BadName.ts", "depth": 3, "status": "violation", "detail": "expected kebab-case", "message": naming_violation["message"], "marker": "3"},
            {"path": "checkout-flow.ts", "full_path": "packages/core/src/checkout-flow.ts", "depth": 3, "status": "violation", "detail": format!("{checkout_lines} / {SOURCE_LINE_LIMIT} lines"), "message": line_violation["message"], "marker": "3"}
        ],
        "evidence": {
            "baseline_success": baseline_report["success"],
            "baseline_files_checked": baseline_report["files_checked"],
            "baseline_dirs_checked": baseline_report["dirs_checked"],
            "violation_success": report["success"]
        },
        "summary": {
            "files_checked": report["files_checked"],
            "dirs_checked": report["dirs_checked"],
            "violations": report["violations"].as_array().map_or(0, Vec::len)
        }
    });
    fs::remove_dir_all(project.parent().unwrap_or(&project))?;
    Ok(artifact)
}

fn find_example_violation<'a>(report: &'a Value, path: &str, rule: &str) -> Result<&'a Value> {
    report["violations"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|violation| {
            violation["path"]
                .as_str()
                .is_some_and(|candidate| candidate.contains(path))
                && violation["rule"].as_str() == Some(rule)
        })
        .ok_or_else(|| format!("missing `{rule}` website example violation for `{path}`").into())
}

fn file_line_count(path: &Path) -> Result<usize> {
    Ok(fs::read_to_string(path)?.lines().count())
}

fn validate_repair_document_references(project: &Path) -> Result<()> {
    let config_path = project.join(".assura/config.yml");
    let config: serde_yaml::Value = serde_yaml::from_str(&fs::read_to_string(&config_path)?)?;
    let mut references = Vec::new();
    collect_repair_document_references(&config, &mut references);
    for reference in references {
        let (path, fragment) = reference
            .split_once('#')
            .map_or((reference.as_str(), None), |(path, fragment)| {
                (path, Some(fragment))
            });
        let target = project.join(path);
        if !target.is_file() {
            return Err(format!(
                "{} repair message references missing project document {path}",
                config_path.display()
            )
            .into());
        }
        if let Some(fragment) = fragment {
            let expected_heading = fragment
                .split('-')
                .map(|word| {
                    let mut characters = word.chars();
                    characters.next().map_or_else(String::new, |first| {
                        first.to_uppercase().collect::<String>() + characters.as_str()
                    })
                })
                .collect::<Vec<_>>()
                .join(" ");
            let contents = fs::read_to_string(&target)?;
            if !contents
                .lines()
                .any(|line| line.trim_start_matches('#').trim() == expected_heading)
            {
                return Err(format!(
                    "{} repair message references missing heading #{fragment} in {path}",
                    config_path.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

fn collect_repair_document_references(value: &serde_yaml::Value, references: &mut Vec<String>) {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (key, value) in mapping {
                if key.as_str() == Some("message") {
                    if let Some(reference) = value
                        .as_str()
                        .and_then(|message| message.strip_prefix("See "))
                        .map(|reference| reference.trim_end_matches('.'))
                    {
                        references.push(reference.to_string());
                    }
                }
                collect_repair_document_references(value, references);
            }
        }
        serde_yaml::Value::Sequence(values) => {
            for value in values {
                collect_repair_document_references(value, references);
            }
        }
        _ => {}
    }
}

fn validate_lslint_config_example(root: &Path, binary: &Path) -> Result<()> {
    let input = root
        .join(CONFIG_EXAMPLES_DIR)
        .join("agentic-monorepo.ls-lint.yml");
    let output = root
        .join("target")
        .join("website-agentic-monorepo-ls-lint-migration.yml");
    let command = Command::new(binary)
        .current_dir(root)
        .arg("migrate")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()?;
    if !command.status.success() {
        return Err(command_error("validate website LS-Lint example", &command).into());
    }
    fs::remove_file(output)?;
    Ok(())
}

fn example_project(root: &Path, name: &str) -> Result<PathBuf> {
    let work = root.join("target").join(format!(
        "website-config-example-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&work);
    let project = work.join("project");
    fs::create_dir_all(&project)?;
    Ok(project)
}

fn install_example_config(root: &Path, project: &Path, name: &str) -> Result<()> {
    let config_dir = project.join(".assura");
    fs::create_dir_all(&config_dir)?;
    fs::copy(
        root.join(CONFIG_EXAMPLES_DIR).join(name),
        config_dir.join("config.yml"),
    )?;
    Ok(())
}

fn run_example_check(binary: &Path, project: &Path, expect_success: bool) -> Result<Value> {
    let output = Command::new(binary)
        .current_dir(project)
        .args(["check", "--format", "json", "."])
        .output()?;
    if output.status.success() != expect_success {
        return Err(command_error("validate website config example", &output).into());
    }
    serde_json::from_slice(&output.stdout).map_err(Into::into)
}

fn require_example_violation(name: &str, report: &Value, path: &str, rule: &str) -> Result<()> {
    let found = report["violations"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|violation| {
            violation["path"]
                .as_str()
                .is_some_and(|candidate| candidate.contains(path))
                && violation["rule"].as_str() == Some(rule)
        });
    if !found {
        return Err(
            format!("website config example {name} did not report `{rule}` for `{path}`").into(),
        );
    }
    Ok(())
}

fn require_example_violation_message(
    name: &str,
    report: &Value,
    path: &str,
    rule: &str,
    message: &str,
) -> Result<()> {
    let found = report["violations"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|violation| {
            violation["path"]
                .as_str()
                .is_some_and(|candidate| candidate.contains(path))
                && violation["rule"].as_str() == Some(rule)
                && violation["message"]
                    .as_str()
                    .is_some_and(|candidate| candidate.contains(message))
        });
    if !found {
        return Err(format!(
            "website config example {name} did not report `{rule}` for `{path}` with `{message}`"
        )
        .into());
    }
    Ok(())
}

fn build_assura(root: &Path) -> Result<()> {
    run_status(
        Command::new("cargo")
            .current_dir(root)
            .args(["build", "--quiet", "--bin", "assura-full"]),
        "build assura-full for website evidence",
    )
}

fn validate_claims(
    root: &Path,
    binary: &Path,
    fixture: &Path,
    require_released: bool,
) -> Result<()> {
    let manifest: ReleaseSurfaceManifest =
        serde_json::from_str(&fs::read_to_string(root.join(RELEASE_SURFACES_PATH))?)?;
    let rendered_claim_ids = website_release_surface_ids(root)?;
    let manifest_ids = manifest
        .surfaces
        .iter()
        .map(|claim| claim.id.clone())
        .collect::<BTreeSet<_>>();
    let marketing_claim_ids = manifest
        .surfaces
        .iter()
        .filter(|claim| claim.marketing_claim)
        .map(|claim| claim.id.clone())
        .collect::<BTreeSet<_>>();
    validate_claim_id_mapping(&rendered_claim_ids, &manifest_ids, &marketing_claim_ids)?;
    let marketing_claim_count = marketing_claim_ids.len();
    let package_version = package_version(&root.join("Cargo.toml"))?;
    for claim in manifest.surfaces {
        if !claim.marketing_claim {
            continue;
        }
        if !rendered_claim_ids.contains(&claim.id) {
            return Err(format!(
                "marketing claim `{}` is not mapped from a website data-release-surface marker",
                claim.id
            )
            .into());
        }
        validate_marketing_claim_release(&claim, &package_version, require_released)?;
        let evidence_status = claim
            .evidence_status
            .as_deref()
            .ok_or_else(|| format!("marketing claim `{}` is missing evidence_status", claim.id))?;
        if !matches!(evidence_status, "verified" | "measured") {
            return Err(format!(
                "marketing claim `{}` has invalid evidence_status `{evidence_status}`",
                claim.id
            )
            .into());
        }
        let evidence_refs = claim
            .evidence_refs
            .as_ref()
            .ok_or_else(|| format!("marketing claim `{}` is missing evidence_refs", claim.id))?;
        if evidence_refs.is_empty() {
            return Err(format!("marketing claim `{}` has no evidence_refs", claim.id).into());
        }
        for path in evidence_refs {
            if !root.join(path).exists() {
                return Err(format!(
                    "marketing claim `{}` references missing evidence `{path}`",
                    claim.id
                )
                .into());
            }
        }
        let Some(command) = claim.command.as_deref() else {
            if evidence_status != "measured" {
                return Err(format!(
                    "marketing claim `{}` needs a public command or measured evidence",
                    claim.id
                )
                .into());
            }
            continue;
        };
        let smoke_args = claim
            .smoke_args
            .ok_or_else(|| format!("claim `{}` is missing smoke_args", claim.id))?;
        let output = Command::new(binary)
            .current_dir(fixture)
            .args(&smoke_args)
            .output()?;
        let expected_exit = claim.expected_exit.unwrap_or(0);
        if output.status.code() != Some(expected_exit) {
            return Err(format!(
                "public command claim `{}` returned {} instead of {expected_exit} for `{command}`: {}",
                claim.id,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
    }
    println!("Validated {marketing_claim_count} marketed release-surface mappings.");
    Ok(())
}

fn validate_marketing_claim_release(
    claim: &ReleaseSurface,
    package_version: &str,
    require_released: bool,
) -> Result<()> {
    if claim.status != "supported" {
        return Err(format!(
            "marketing claim `{}` must be supported, found `{}`",
            claim.id, claim.status
        )
        .into());
    }
    if !require_released {
        return Ok(());
    }
    if claim.first_release == "unreleased" {
        return Err(format!(
            "marketing claim `{}` is not available in release v{package_version}",
            claim.id
        )
        .into());
    }
    let first_release = super::release_tag_version(&claim.first_release).ok_or_else(|| {
        format!(
            "marketing claim `{}` has invalid first_release `{}`",
            claim.id, claim.first_release
        )
    })?;
    let package_release = semver::Version::parse(package_version)
        .map_err(|_| format!("Cargo package version `{package_version}` is not valid SemVer"))?;
    if first_release > package_release {
        return Err(format!(
            "marketing claim `{}` first ships in {} after local release v{package_version}",
            claim.id, claim.first_release
        )
        .into());
    }
    Ok(())
}

fn validate_setup_delivery(root: &Path, require_released: bool) -> Result<()> {
    let manifest: ReleaseSurfaceManifest =
        serde_json::from_str(&fs::read_to_string(root.join(RELEASE_SURFACES_PATH))?)?;
    let package_version = package_version(&root.join("Cargo.toml"))?;
    let package_release = semver::Version::parse(&package_version)
        .map_err(|_| format!("Cargo package version `{package_version}` is not valid SemVer"))?;
    let needs_source_preview = manifest
        .surfaces
        .iter()
        .filter(|surface| surface.marketing_claim)
        .try_fold(false, |needs_source_preview, surface| {
            if surface.first_release == "unreleased" {
                return Ok::<bool, Box<dyn std::error::Error>>(true);
            }
            let first_release =
                super::release_tag_version(&surface.first_release).ok_or_else(|| {
                    format!(
                        "marketing claim `{}` has invalid first_release `{}`",
                        surface.id, surface.first_release
                    )
                })?;
            Ok(needs_source_preview || first_release > package_release)
        })?;
    let source = fs::read_to_string(root.join(MARKETING_DATA_PATH))?;
    validate_setup_delivery_source(&source, needs_source_preview, require_released)?;
    if needs_source_preview && !require_released {
        validate_pinned_artifact_provenance(root, source_preview_revision(&source)?)?;
    }
    println!("Validated marketing setup delivery boundary.");
    Ok(())
}

fn source_preview_revision(source: &str) -> Result<&str> {
    const REVISION_PREFIX: &str = "export const sourcePreviewRevision = '";
    source
        .split_once(REVISION_PREFIX)
        .and_then(|(_, suffix)| suffix.split_once("';"))
        .map(|(revision, _)| revision)
        .ok_or_else(|| "marketing setup must declare a pinned sourcePreviewRevision".into())
}

fn validate_pinned_artifact_provenance(root: &Path, revision: &str) -> Result<()> {
    run_status(
        Command::new("git").current_dir(root).args([
            "merge-base",
            "--is-ancestor",
            revision,
            "HEAD",
        ]),
        "verify the marketing implementation revision is in repository history",
    )?;

    for path in PINNED_SOURCE_PREVIEW_FILES {
        let pinned = Command::new("git")
            .current_dir(root)
            .args(["show", &format!("{revision}:{path}")])
            .output()?;
        if !pinned.status.success() {
            return Err(format!(
                "cannot read `{path}` from pinned revision `{revision}`; website validation requires full Git history"
            )
            .into());
        }
        if pinned.stdout != fs::read(root.join(path))? {
            return Err(format!(
                "source-preview file `{path}` differs from pinned implementation revision `{revision}`"
            )
            .into());
        }
    }
    Ok(())
}

fn validate_setup_delivery_source(
    source: &str,
    needs_source_preview: bool,
    require_released: bool,
) -> Result<()> {
    const PINNED_INSTALL: &str = "cargo install --git ${sourceRepositoryUrl} --rev ${sourcePreviewRevision} --locked --bin assura assura";

    if require_released && needs_source_preview {
        return Err("released marketing cannot include capabilities newer than the package".into());
    }

    if needs_source_preview {
        let revision = source_preview_revision(source)?;
        if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(
                "marketing sourcePreviewRevision must be a full 40-character Git SHA".into(),
            );
        }
        if !source.contains(PINNED_INSTALL) || !source.contains("${installCommand}") {
            return Err(
                "marketing agent and manual setup must share the pinned source install command"
                    .into(),
            );
        }
        if source.contains("assura.dev/install.sh") {
            return Err(
                "marketing setup cannot use the public installer for unreleased capabilities"
                    .into(),
            );
        }
        return Ok(());
    }

    if source.contains("sourcePreviewRevision") || source.contains("cargo install --git") {
        return Err("released marketing setup must not remain pinned to a source revision".into());
    }
    if !source.contains("assura.dev/install.sh") {
        return Err("released marketing setup must use the public installer".into());
    }
    Ok(())
}

fn validate_claim_id_mapping(
    rendered_claim_ids: &BTreeSet<String>,
    manifest_ids: &BTreeSet<String>,
    marketing_claim_ids: &BTreeSet<String>,
) -> Result<()> {
    for id in rendered_claim_ids {
        if !manifest_ids.contains(id) {
            return Err(format!(
                "website data-release-surface marker references unknown claim `{id}`"
            )
            .into());
        }
        if !marketing_claim_ids.contains(id) {
            return Err(format!(
                "website data-release-surface marker references `{id}`, but its manifest row is not a marketing claim"
            )
            .into());
        }
    }
    for id in marketing_claim_ids {
        if !rendered_claim_ids.contains(id) {
            return Err(format!(
                "marketing claim `{id}` is not mapped from a website data-release-surface marker"
            )
            .into());
        }
    }
    Ok(())
}

fn website_release_surface_ids(root: &Path) -> Result<BTreeSet<String>> {
    let mut files = Vec::new();
    collect_files(&root.join("website/src"), &mut files)?;
    let mut ids = BTreeSet::new();
    for path in files {
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("astro" | "md" | "mdx")
        ) {
            continue;
        }
        let text = fs::read_to_string(path)?;
        for marker in ["data-release-surface=\"", "releaseSurface=\""] {
            for suffix in text.split(marker).skip(1) {
                let Some(value) = suffix.split('"').next() else {
                    continue;
                };
                ids.extend(value.split_whitespace().map(str::to_string));
            }
        }
    }
    Ok(ids)
}

fn validate_website_commands(root: &Path) -> Result<()> {
    let website_source = root.join("website/src");
    let mut files = Vec::new();
    collect_files(&website_source, &mut files)?;
    for path in files {
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("astro" | "md" | "mdx" | "yml")
        ) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        for forbidden in FORBIDDEN_WEBSITE_COMMANDS {
            if text.contains(forbidden) {
                return Err(format!(
                    "{} contains unsupported public command pattern `{forbidden}`",
                    path.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

fn prepare_fixture(root: &Path, name: &str) -> Result<PathBuf> {
    let work = root
        .join("target")
        .join(format!("website-demo-{name}-{}", std::process::id()));
    let project = work.join("project");
    let remote = work.join("remote.git");
    let _ = fs::remove_dir_all(&work);
    copy_dir(&root.join(FIXTURE_SOURCE), &project)?;
    let config = project.join(".assura/config.yml");
    let mut config_text = fs::read_to_string(&config)?;
    config_text.push_str("  - .git/**\n");
    fs::write(config, config_text)?;
    git(&project, &["init"])?;
    fs::write(project.join(".git/info/exclude"), ".assura/cache/\n")?;
    git(
        &project,
        &["config", "user.email", "website-evidence@assura.dev"],
    )?;
    git(
        &project,
        &["config", "user.name", "Assura Website Evidence"],
    )?;
    git(&project, &["branch", "-M", "main"])?;
    git(&project, &["add", "."])?;
    git(&project, &["commit", "-m", "baseline"])?;

    run_status(
        Command::new("git").args(["init", "--bare", path_text(&remote)?]),
        "initialize website evidence remote",
    )?;
    git(&project, &["remote", "add", "origin", path_text(&remote)?])?;
    git(&project, &["push", "-u", "origin", "main"])?;
    git(&project, &["checkout", "-b", "feature/agent-review"])?;

    let branch_doc = project.join("apps/web/docs/branch-review.md");
    fs::write(
        &branch_doc,
        "# Branch review\n\nThis change belongs to the active feature branch.\n",
    )?;
    git(&project, &["add", "apps/web/docs/branch-review.md"])?;
    git(&project, &["commit", "-m", "add branch review"])?;
    git(&project, &["push", "-u", "origin", "feature/agent-review"])?;

    let home = project.join("apps/web/src/home-page.tsx");
    let mut home_text = fs::read_to_string(&home)?;
    home_text.push_str("\nexport const changedByAgent = true;\n");
    fs::write(home, home_text)?;
    fs::write(
        project.join("apps/web/src/BadName.tsx"),
        "export const rushedHelper = true;\n",
    )?;
    Ok(project)
}

fn prepare_onboarding_evidence(root: &Path, binary: &Path) -> Result<(PathBuf, Value, String)> {
    let work = root
        .join("target")
        .join(format!("website-onboarding-demo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let json_project = work.join("json-project");
    let text_project = work.join("text-project");
    prepare_onboarding_project(&json_project)?;
    prepare_onboarding_project(&text_project)?;
    let args = [
        "agent",
        "onboard",
        ".",
        "--agent",
        "codex",
        "--activate",
        "--format",
    ];
    let mut json_args = args.to_vec();
    json_args.push("json");
    let onboarding = run_json_at(binary, &json_args, &json_project)?;
    let mut text_args = args.to_vec();
    text_args.push("text");
    let text = run_text_at(binary, &text_args, &text_project)?;
    Ok((
        work,
        onboarding,
        normalize_project_path(&text, &text_project),
    ))
}

fn prepare_onboarding_project(project: &Path) -> Result<()> {
    fs::create_dir_all(project.join("src"))?;
    fs::write(
        project.join("package.json"),
        "{\n  \"name\": \"agent-ready-project\"\n}\n",
    )?;
    fs::write(project.join("src/index.ts"), "export const ready = true;\n")?;
    Ok(())
}

fn compact_review(report: &Value, rendered_text: &str) -> Result<Value> {
    let rendered_text = rendered_text.trim_end();
    let display_lines = terminal_lines(rendered_text, review_tone);
    let hero_lines = select_terminal_lines(
        rendered_text,
        &[
            "Assura review",
            "Status",
            "Scope",
            "Findings",
            "Branch",
            "Worktree",
            "Watch",
            "Hot path",
            "Fix first",
            "Run",
        ],
        review_tone,
    )?;
    let artifact_lines = select_terminal_lines(
        rendered_text,
        &[
            "Assura review",
            "Status",
            "Scope",
            "Findings",
            "Branch",
            "Worktree",
            "Watch",
            "Hot path",
            "Fix first",
            "Next",
            "Run",
        ],
        review_tone,
    )?;
    let totals = &report["heatmap"]["totals"];
    let signals = report["heatmap"]["hot_dirs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|directory| {
            matches!(
                directory["path"].as_str(),
                Some("apps/web/src" | "apps/web/docs")
            )
        })
        .take(3)
        .map(|directory| {
            json!({
                "path": directory["path"],
                "violations": directory["validation_violations"],
                "naming_violations": directory["naming_violations"],
                "modified_files": directory["modified_files"],
                "untracked_files": directory["untracked_files"],
                "branch_changed_files": directory["branch_changed_files"],
                "branch_lines": {
                    "added": directory["branch_line_additions"],
                    "deleted": directory["branch_line_deletions"]
                },
                "worktree_lines": {
                    "added": directory["worktree_line_additions"],
                    "deleted": directory["worktree_line_deletions"]
                }
            })
        })
        .collect::<Vec<_>>();
    let findings = report["findings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|finding| finding["severity"] == "blocking")
        .take(3)
        .map(|finding| {
            json!({
                "id": finding["id"],
                "title": finding["title"],
                "detail": finding["detail"],
                "command": finding["command"]
            })
        })
        .collect::<Vec<_>>();
    let next_command = report["next_actions"]
        .as_array()
        .and_then(|actions| actions.first())
        .and_then(|action| action["command"].as_str())
        .unwrap_or("assura explain apps/web/src");
    let metrics = json!([
        {"label": "Policy", "value": report["summary"]["blocking"].to_string(), "detail": "blocking violation"},
        {"label": "Branch", "value": format!("{} file", totals["branch_changed_files"].as_u64().unwrap_or_default()), "detail": format!("+{} / -{} lines", totals["branch_line_additions"].as_u64().unwrap_or_default(), totals["branch_line_deletions"].as_u64().unwrap_or_default())},
        {"label": "Worktree", "value": format!("{} modified", totals["modified_files"].as_u64().unwrap_or_default()), "detail": format!("{} untracked", totals["untracked_files"].as_u64().unwrap_or_default())},
        {"label": "Inactive", "value": report["summary"]["inactive"].to_string(), "detail": "reported, not passed"}
    ]);

    Ok(json!({
        "schema": "assura.website-review-demo.v1",
        "generated_from": report["schema"],
        "command": "assura review",
        "status": report["status"],
        "structure": report["structure"],
        "summary": report["summary"],
        "branch": {
            "name": report["heatmap"]["branch"]["name"],
            "base": report["heatmap"]["branch"]["base"],
            "commits": report["heatmap"]["branch"]["commits_on_branch"],
            "files": totals["branch_changed_files"],
            "lines_added": totals["branch_line_additions"],
            "lines_deleted": totals["branch_line_deletions"]
        },
        "worktree": {
            "modified": totals["modified_files"],
            "untracked": totals["untracked_files"],
            "lines_added": totals["worktree_line_additions"],
            "lines_deleted": totals["worktree_line_deletions"]
        },
        "signals": signals,
        "findings": findings,
        "next_command": next_command,
        "rendered_text": rendered_text,
        "display_lines": display_lines,
        "hero_lines": hero_lines,
        "artifact_lines": artifact_lines,
        "metrics": metrics
    }))
}

fn compact_check(report: &Value, rendered_text: &str) -> Result<Value> {
    let rendered_text = rendered_text.trim_end();
    let display_lines = terminal_lines(rendered_text, check_tone);
    let first_fix = rendered_text
        .lines()
        .position(|line| line.starts_with("  Fix:"))
        .ok_or("check text renderer did not include corrective context")?;
    let artifact_text = rendered_text
        .lines()
        .take(first_fix + 1)
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "Assura structure check",
        "Violations",
        "  Blocking: true",
        "  Fix:",
    ] {
        if !artifact_text.contains(required) {
            return Err(format!("check text renderer did not include `{required}`").into());
        }
    }
    let artifact_lines = terminal_lines(&artifact_text, check_tone);
    let violations = report["violations"]
        .as_array()
        .into_iter()
        .flatten()
        .take(4)
        .map(|violation| {
            json!({
                "path": violation["path"],
                "rule": violation["rule"],
                "message": violation["message"],
                "severity": violation["severity"],
                "blocking": violation["blocking"]
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schema": "assura.website-check-demo.v1",
        "generated_from": "assura check --format json",
        "command": "assura check",
        "success": report["success"],
        "files_checked": report["files_checked"],
        "dirs_checked": report["dirs_checked"],
        "violation_count": report["violations"].as_array().map_or(0, Vec::len),
        "violations": violations,
        "rendered_text": rendered_text,
        "display_lines": display_lines,
        "artifact_lines": artifact_lines
    }))
}

fn compact_onboarding(report: &Value, rendered_text: &str) -> Result<Value> {
    let rendered_text = rendered_text.trim_end();
    for required in [
        "Assura agent onboarding",
        "Host",
        "Lifecycle",
        "Review",
        "Deferred",
        "Packet",
    ] {
        if !rendered_text.lines().any(|line| line.starts_with(required)) {
            return Err(format!("onboarding text renderer did not include `{required}`").into());
        }
    }
    Ok(json!({
        "schema": "assura.website-onboarding-demo.v1",
        "generated_from": report["schema"],
        "command": "assura agent onboard . --agent codex --activate --format text",
        "installed": report["installed"],
        "detected": report["detected"],
        "integration": report["integration"],
        "content": report["content"],
        "files": report["files"],
        "verified": report["verified"],
        "inactive": report["inactive"],
        "lifecycle_profiles": report["lifecycle_profiles"].as_array().into_iter().flatten().map(|profile| json!({
            "name": profile["name"],
            "mode": profile["mode"],
            "trigger": profile["trigger"],
            "blocking": profile["blocking"]
        })).collect::<Vec<_>>(),
        "next_actions": report["next_actions"],
        "rendered_text": rendered_text,
        "display_lines": terminal_lines(rendered_text, onboarding_tone),
        "artifact_lines": select_terminal_lines(
            rendered_text,
            &[
                "Assura agent onboarding",
                "Project",
                "Agent",
                "Policy",
                "Host",
                "Lifecycle",
                "Verified",
                "Review",
                "Deferred",
                "Next",
                "Packet",
            ],
            onboarding_tone,
        )?
    }))
}

fn compact_intelligence(report: &Value, rendered_text: &str) -> Result<Value> {
    let rendered_text = rendered_text.trim_end();
    for required in [
        "Context pack: object",
        "diagnostics:",
        "instance:",
        "omissions:",
    ] {
        if !rendered_text.lines().any(|line| line.starts_with(required)) {
            return Err(format!("context-pack text renderer did not include `{required}`").into());
        }
    }
    Ok(json!({
        "schema": "assura.website-intelligence-demo.v1",
        "generated_from": report["schema"],
        "command": "assura content context-pack . --collection goals --id goal-portable-structure --format text",
        "request": report["request"],
        "bounds": report["bounds"],
        "diagnostics": report["diagnostics"],
        "instance": report["instance"],
        "related": report["related"],
        "repository_references": report["repository_references"],
        "rendered_text": rendered_text,
        "artifact_lines": terminal_lines(rendered_text, intelligence_tone)
    }))
}

fn terminal_lines(text: &str, tone: fn(&str) -> &'static str) -> Vec<Value> {
    text.lines()
        .map(|line| json!({"text": line, "tone": tone(line)}))
        .collect()
}

fn select_terminal_lines(
    rendered_text: &str,
    prefixes: &[&str],
    tone: fn(&str) -> &'static str,
) -> Result<Vec<Value>> {
    let lines = rendered_text.lines().collect::<Vec<_>>();
    prefixes
        .iter()
        .map(|prefix| {
            let line = lines
                .iter()
                .find(|line| line.starts_with(prefix))
                .ok_or_else(|| format!("text renderer did not include `{prefix}`"))?;
            Ok(json!({"text": line, "tone": tone(line)}))
        })
        .collect()
}

fn review_tone(line: &str) -> &'static str {
    if line == "Assura review" {
        "prompt"
    } else if line.starts_with("Status") {
        if line.contains("needs attention") {
            "warn"
        } else {
            "pass"
        }
    } else if line.starts_with("Fix first") {
        "fail"
    } else if line.starts_with("Watch") {
        "warn"
    } else if line.starts_with("Run") {
        "pass"
    } else if ["Scope", "Findings", "Branch", "Worktree", "Hot path"]
        .iter()
        .any(|prefix| line.starts_with(prefix))
    {
        "info"
    } else if line.starts_with("Next") {
        "muted"
    } else {
        "plain"
    }
}

fn check_tone(line: &str) -> &'static str {
    if line == "Assura structure check" {
        "prompt"
    } else if line.starts_with("Violations: ") {
        if line.ends_with(" 0") {
            "pass"
        } else {
            "fail"
        }
    } else if line.starts_with("  Blocking: true") || line.contains(" [") {
        "fail"
    } else if line.starts_with("  Fix:") {
        "warn"
    } else if line.starts_with("Project root:")
        || line.starts_with("Config:")
        || line.starts_with("Checked path:")
    {
        "muted"
    } else if line.starts_with("Files checked:") || line.starts_with("Directories checked:") {
        "info"
    } else {
        "plain"
    }
}

fn onboarding_tone(line: &str) -> &'static str {
    if line == "Assura agent onboarding" {
        "prompt"
    } else if line.starts_with("Host") {
        if line.contains("activated=true")
            && line.contains("verified=true")
            && line.contains("conflicted=false")
        {
            "pass"
        } else {
            "warn"
        }
    } else if line.starts_with("Verified") {
        "pass"
    } else if line.starts_with("Review") || line.starts_with("Deferred") {
        "warn"
    } else if line.starts_with("Next") || line.starts_with("Packet") {
        "pass"
    } else if line.starts_with("Version") || line.starts_with("Content") {
        "muted"
    } else {
        "info"
    }
}

fn intelligence_tone(line: &str) -> &'static str {
    if line.starts_with("Context pack:") {
        "prompt"
    } else if line.starts_with("diagnostics:") {
        if line.starts_with("diagnostics: 0;") {
            "pass"
        } else {
            "warn"
        }
    } else if line.starts_with("instance:") {
        "info"
    } else if line.starts_with("omissions:") {
        "muted"
    } else {
        "plain"
    }
}

fn compact_performance(root: &Path, report: &Value) -> Result<Value> {
    let version = report["results"]
        .as_array()
        .and_then(|rows| rows.iter().find_map(|row| row["assura_version"].as_str()))
        .ok_or("performance report has no Assura version")?;
    let package_version = package_version(&root.join("Cargo.toml"))?;
    if version != package_version {
        return Err(format!(
            "performance report version {version} does not match Cargo package version {package_version}"
        )
        .into());
    }
    Ok(json!({
        "schema": "assura.website-performance-summary.v1",
        "source_schema": report["schema_version"],
        "timestamp": report["timestamp"],
        "assura_version": version,
        "environment": report["environment"],
        "cold": report["claim_summary"],
        "warm": report["warm_claim_summary"]
    }))
}

fn run_json_at(binary: &Path, args: &[&str], current_dir: &Path) -> Result<Value> {
    let output = Command::new(binary)
        .current_dir(current_dir)
        .args(args)
        .output()?;
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(command_error("run Assura website evidence command", &output).into());
    }
    serde_json::from_slice(&output.stdout).map_err(Into::into)
}

fn run_text_at(binary: &Path, args: &[&str], current_dir: &Path) -> Result<String> {
    let output = Command::new(binary)
        .current_dir(current_dir)
        .env("NO_COLOR", "1")
        .args(args)
        .output()?;
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(command_error("run Assura website text evidence command", &output).into());
    }
    String::from_utf8(output.stdout).map_err(Into::into)
}

fn normalize_project_path(text: &str, project: &Path) -> String {
    let mut normalized = text.replace(&project.display().to_string(), ".");
    if let Ok(canonical) = project.canonicalize() {
        normalized = normalized.replace(&canonical.display().to_string(), ".");
    }
    normalized
}

fn package_version(path: &Path) -> Result<String> {
    fs::read_to_string(path)?
        .lines()
        .find_map(|line| {
            line.strip_prefix("version = \"")
                .and_then(|line| line.strip_suffix('"'))
        })
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Cargo.toml package version was not found".into())
}

fn read_json(path: PathBuf) -> Result<Value> {
    serde_json::from_str(&fs::read_to_string(path)?).map_err(Into::into)
}

fn git(project: &Path, args: &[&str]) -> Result<()> {
    run_status(
        Command::new("git").current_dir(project).args(args),
        "prepare website evidence Git fixture",
    )
}

fn run_status(command: &mut Command, context: &str) -> Result<()> {
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error(context, &output).into())
    }
}

fn command_error(context: &str, output: &Output) -> String {
    format!(
        "{context} failed (status {}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    )
}

fn path_text(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_files(&entry.path(), files)?;
        } else {
            files.push(entry.path());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_summary_keeps_cold_and_warm_claims_separate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let version = package_version(&root.join("Cargo.toml")).expect("package version");
        let report = json!({
            "schema_version": "assura.performance.v1",
            "timestamp": "2026-07-10T00:00:00Z",
            "environment": {"os": "linux"},
            "claim_summary": {"aggregate_speedup_ratio": 1.5},
            "warm_claim_summary": {"aggregate_speedup_ratio": 20.0},
            "results": [{"assura_version": version}]
        });
        let summary = compact_performance(root, &report).expect("summary");
        assert_eq!(summary["assura_version"], version);
        assert_eq!(summary["cold"]["aggregate_speedup_ratio"], 1.5);
        assert_eq!(summary["warm"]["aggregate_speedup_ratio"], 20.0);
    }

    #[test]
    fn release_surface_mapping_is_bidirectional() {
        let rendered = BTreeSet::from(["verified".to_string()]);
        let manifest = BTreeSet::from(["verified".to_string(), "internal".to_string()]);
        let marketing = BTreeSet::from(["verified".to_string()]);
        validate_claim_id_mapping(&rendered, &manifest, &marketing).unwrap();

        let unknown = BTreeSet::from(["unknown".to_string()]);
        assert!(validate_claim_id_mapping(&unknown, &manifest, &marketing)
            .unwrap_err()
            .to_string()
            .contains("unknown claim"));

        let non_marketed = BTreeSet::from(["internal".to_string()]);
        assert!(
            validate_claim_id_mapping(&non_marketed, &manifest, &marketing)
                .unwrap_err()
                .to_string()
                .contains("not a marketing claim")
        );

        let missing = BTreeSet::new();
        assert!(validate_claim_id_mapping(&missing, &manifest, &marketing)
            .unwrap_err()
            .to_string()
            .contains("is not mapped"));
    }

    fn release_surface(id: &str, status: &str, first_release: &str) -> ReleaseSurface {
        ReleaseSurface {
            id: id.to_string(),
            status: status.to_string(),
            first_release: first_release.to_string(),
            marketing_claim: true,
            evidence_status: Some("verified".to_string()),
            evidence_refs: Some(vec!["tests/example.rs".to_string()]),
            command: None,
            smoke_args: None,
            expected_exit: None,
        }
    }

    #[test]
    fn marketing_claims_require_supported_status() {
        for (id, status) in [
            ("experimental-watch", "experimental"),
            ("planned-search", "planned"),
        ] {
            let claim = release_surface(id, status, "v0.3.0");
            let error = validate_marketing_claim_release(&claim, "0.3.0", false)
                .expect_err("non-supported claim must fail")
                .to_string();
            assert!(error.contains(id), "error did not name {id}: {error}");
            assert!(error.contains("must be supported"));
        }
    }

    #[test]
    fn released_mode_is_always_read_only() {
        assert_eq!(execution_mode(&[]).unwrap(), (false, false));
        assert_eq!(
            execution_mode(&["--check".to_string()]).unwrap(),
            (true, false)
        );
        assert_eq!(
            execution_mode(&["--released".to_string()]).unwrap(),
            (true, true)
        );
        assert!(execution_mode(&["--unknown".to_string()]).is_err());
    }

    #[test]
    fn released_marketing_claims_must_ship_by_the_local_version() {
        validate_marketing_claim_release(
            &release_surface("check", "supported", "v0.1.0"),
            "0.3.0",
            true,
        )
        .expect("older supported surface");

        validate_marketing_claim_release(
            &release_surface("check-rc", "supported", "v0.4.0-rc.1"),
            "0.4.0-rc.1",
            true,
        )
        .expect("matching release candidate");

        let unreleased = validate_marketing_claim_release(
            &release_surface("review", "supported", "unreleased"),
            "0.3.0",
            true,
        )
        .expect_err("unreleased claim must fail strict release validation");
        assert!(unreleased.to_string().contains("not available"));

        let future = validate_marketing_claim_release(
            &release_surface("review", "supported", "v0.4.0"),
            "0.3.0",
            true,
        )
        .expect_err("future claim must fail strict release validation");
        assert!(future.to_string().contains("after local release"));
    }

    #[test]
    fn unreleased_marketing_requires_one_shared_pinned_source_install() {
        assert!(
            PINNED_SOURCE_PREVIEW_FILES.contains(&"src/cli/agent_onboarding_handoff_templates.rs")
        );
        let source = r#"
export const sourceRepositoryUrl = 'https://github.com/rothnic/assura';
export const sourcePreviewRevision = 'b8f8375835095ce4f83b872c33b9d4e163ab283a';
export const installCommand = `cargo install --git ${sourceRepositoryUrl} --rev ${sourcePreviewRevision} --locked --bin assura assura`;
export const agentSetupPrompt = `${installCommand}`;
"#;
        validate_setup_delivery_source(source, true, false).expect("pinned source setup");

        let public_installer = source.replace(
            "export const sourceRepositoryUrl = 'https://github.com/rothnic/assura';",
            "export const installScriptUrl = 'https://assura.dev/install.sh';",
        );
        assert!(validate_setup_delivery_source(&public_installer, true, false).is_err());
    }

    #[test]
    fn released_marketing_rejects_source_preview_delivery() {
        let pinned = "export const sourcePreviewRevision = 'b8f8375835095ce4f83b872c33b9d4e163ab283a';\nexport const installCommand = `cargo install --git repo --rev sha`;";
        assert!(validate_setup_delivery_source(pinned, false, true).is_err());

        let released = "export const installScriptUrl = 'https://assura.dev/install.sh';";
        validate_setup_delivery_source(released, false, true).expect("public release installer");
    }
}
