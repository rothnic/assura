//! Lightweight `assura check` entrypoint optimized for validation latency.
use super::{
    run_structure_check_cached, run_structure_check_with_target_mode, run_structure_checks,
    CheckError, CheckTargetMode, StructureCheckReport,
};
use crate::cli::check_feedback::{
    render_agent_feedback, render_check_feedback, render_codex_agent_feedback, CheckFeedbackFormat,
    FeedbackOptions,
};
use serde::Serialize;
use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::path::PathBuf;
#[path = "fast_cli_options.rs"]
mod option_helpers;
use option_helpers::reject_unknown_option;
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug)]
struct Options {
    paths: Vec<PathBuf>,
    config: Option<PathBuf>,
    format: OutputFormat,
    agent: AgentTarget,
    min_severity: Option<String>,
    max_issues: Option<usize>,
    output: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    cache: bool,
    fail_fast: bool,
    warn: bool,
    ls_lint_target_semantics: bool,
    quiet: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
    Advice,
    Status,
    Agent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentTarget {
    Generic,
    Codex,
}
/// Run the lightweight check parser from process arguments.
pub fn run_check_cli_from_env(command_name: &str) -> i32 {
    run_check_cli(command_name, std::env::args_os().skip(1))
}
/// Run the lightweight check parser from already-normalized arguments.
pub fn run_check_cli<I>(command_name: &str, args: I) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let options = match parse_options(command_name, args) {
        Ok(ParseOutcome::Run(options)) => options,
        Ok(ParseOutcome::ExitSuccess) => return 0,
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Try '{command_name} --help' for usage.");
            return 2;
        }
    };

    match run(options) {
        Ok(success) => i32::from(!success),
        Err(error) => {
            eprintln!("Error: {error}");
            exit_code_for_check_error(&error)
        }
    }
}

/// Try to handle `assura check` before the full CLI stack starts.
/// Returns `None` when the invocation needs the complete Clap/Tokio command
/// path, preserving the rest of the product CLI while keeping common checks on
/// the low-overhead path.
pub fn try_run_primary_check_cli<I>(args: I) -> Option<i32>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let _program = args.next();
    let args: Vec<OsString> = args.collect();
    let command_index = args.iter().position(|arg| arg == "check")?;
    let mut normalized = Vec::with_capacity(args.len().saturating_sub(1));

    let mut before = args[..command_index].iter();
    while let Some(arg) = before.next() {
        match arg.to_string_lossy().as_ref() {
            "-c" | "--config" => {
                normalized.push(arg.clone());
                normalized.push(before.next()?.clone());
            }
            "--quiet" | "-q" => normalized.push(arg.clone()),
            _ => return None,
        }
    }

    let after = &args[command_index + 1..];
    if after.iter().any(requires_full_check_path) {
        return None;
    }
    normalized.extend(after.iter().cloned());
    Some(run_check_cli("assura check", normalized))
}

fn requires_full_check_path(arg: &OsString) -> bool {
    matches!(
        arg.to_string_lossy().as_ref(),
        "--watch" | "--verbose" | "-v"
    )
}

enum ParseOutcome {
    Run(Options),
    ExitSuccess,
}

fn parse_options<I>(command_name: &str, args: I) -> Result<ParseOutcome, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = pico_args::Arguments::from_vec(args.into_iter().collect());
    if args.contains(["-h", "--help"]) {
        print_help(command_name);
        return Ok(ParseOutcome::ExitSuccess);
    }
    if args.contains(["-V", "--version"]) {
        println!("{command_name} {VERSION}");
        return Ok(ParseOutcome::ExitSuccess);
    }

    let config = args
        .opt_value_from_os_str(["-c", "--config"], path_from_os_str)
        .map_err(|error| error.to_string())?;
    let format = args
        .opt_value_from_fn(["-f", "--format"], parse_format)
        .map_err(|error| error.to_string())?
        .unwrap_or(OutputFormat::Text);
    let agent = args
        .opt_value_from_fn("--agent", parse_agent_target)
        .map_err(|error| error.to_string())?
        .unwrap_or(AgentTarget::Generic);
    if agent != AgentTarget::Generic && format != OutputFormat::Agent {
        return Err("--agent requires --format agent".to_string());
    }
    let min_severity = args
        .opt_value_from_fn("--min-severity", parse_min_severity)
        .map_err(|error| error.to_string())?;
    let max_issues = args
        .opt_value_from_str("--max-issues")
        .map_err(|error| error.to_string())?;
    let output = args
        .opt_value_from_os_str(["-o", "--output"], path_from_os_str)
        .map_err(|error| error.to_string())?;
    let cache_dir = args
        .opt_value_from_os_str("--cache-dir", path_from_os_str)
        .map_err(|error| error.to_string())?;
    let cache = args.contains("--cache");
    let fail_fast = args.contains("--fail-fast");
    let warn = args.contains("--warn");
    let _no_parallel = args.contains("--no-parallel");
    let ls_lint_target_semantics = args.contains("--ls-lint-target-semantics");
    let quiet = args.contains(["-q", "--quiet"]);
    let remaining = args.finish();
    let mut paths = Vec::with_capacity(remaining.len());
    for value in remaining {
        reject_unknown_option(&value)?;
        paths.push(PathBuf::from(value));
    }

    Ok(ParseOutcome::Run(Options {
        paths,
        config,
        format,
        agent,
        min_severity,
        max_issues,
        output,
        cache_dir,
        cache,
        fail_fast,
        warn,
        ls_lint_target_semantics,
        quiet,
    }))
}

fn path_from_os_str(value: &OsStr) -> Result<PathBuf, &'static str> {
    Ok(PathBuf::from(value))
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        "yaml" => Ok(OutputFormat::Yaml),
        "advice" => Ok(OutputFormat::Advice),
        "status" => Ok(OutputFormat::Status),
        "agent" => Ok(OutputFormat::Agent),
        other => Err(format!("unsupported format '{other}'")),
    }
}

fn parse_agent_target(value: &str) -> Result<AgentTarget, String> {
    match value {
        "generic" => Ok(AgentTarget::Generic),
        "codex" => Ok(AgentTarget::Codex),
        other => Err(format!(
            "unsupported agent target '{other}'; expected generic or codex"
        )),
    }
}

fn parse_min_severity(value: &str) -> Result<String, String> {
    match value {
        "low" | "medium" | "high" | "critical" => Ok(value.to_string()),
        other => Err(format!(
            "unsupported minimum severity '{other}'; expected low, medium, high, or critical"
        )),
    }
}

fn run(options: Options) -> Result<bool, CheckError> {
    let cache_dir = if options.cache {
        let path = options
            .paths
            .first()
            .cloned()
            .unwrap_or(std::env::current_dir()?);
        Some(super::cache::default_check_cache_dir(&path))
    } else {
        options.cache_dir.clone()
    };
    if cache_dir.is_none() && options.paths.len() <= 1 {
        let path = options.paths.first().cloned();
        let target_mode = if options.ls_lint_target_semantics {
            CheckTargetMode::LsLint
        } else {
            CheckTargetMode::Recursive
        };
        let report = run_structure_check_with_target_mode(
            path,
            options.config.clone(),
            options.fail_fast,
            target_mode,
        )?;
        let success = report.success;
        if !options.quiet || !success || options.output.is_some() {
            let rendered = format_report(&report, options.format, &options);
            if let Some(output) = options.output {
                std::fs::write(output, rendered)?;
            } else {
                println!("{rendered}");
            }
        }
        return Ok(success || options.warn);
    }

    let paths = if options.paths.is_empty() {
        vec![None]
    } else {
        options.paths.iter().cloned().map(Some).collect()
    };
    if options.ls_lint_target_semantics {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(
                "--ls-lint-target-semantics only supports a single explicit path".to_string(),
            ),
        ));
    }
    let reports = if let Some(cache_dir) = &cache_dir {
        let mut reports = Vec::with_capacity(paths.len());
        for path in paths {
            reports.push(run_structure_check_cached(
                path,
                options.config.clone(),
                options.fail_fast,
                cache_dir.clone(),
            )?);
        }
        reports.sort_by(|left, right| left.checked_path.cmp(&right.checked_path));
        reports
    } else {
        run_structure_checks(paths, options.config.clone(), options.fail_fast)?
    };
    let success = reports.iter().all(|report| report.success);

    if !options.quiet || !success || options.output.is_some() {
        let rendered = format_reports(&reports, options.format, &options);
        if let Some(output) = options.output {
            std::fs::write(output, rendered)?;
        } else {
            println!("{rendered}");
        }
    }
    Ok(success || options.warn)
}

fn format_reports(
    reports: &[StructureCheckReport],
    format: OutputFormat,
    options: &Options,
) -> String {
    if reports.len() == 1 {
        return format_report(&reports[0], format, options);
    }

    let success = reports.iter().all(|report| report.success);
    let batch = BatchReport { success, reports };
    match format {
        OutputFormat::Text => format_batch_text_report(reports),
        OutputFormat::Json => serde_json::to_string_pretty(&batch).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(&batch).unwrap_or_default(),
        OutputFormat::Advice => {
            format_feedback_reports(reports, options, CheckFeedbackFormat::Advice)
        }
        OutputFormat::Status => {
            format_feedback_reports(reports, options, CheckFeedbackFormat::Status)
        }
        OutputFormat::Agent if options.agent == AgentTarget::Codex => {
            render_codex_agent_feedback(reports.iter(), &feedback_options(options))
        }
        OutputFormat::Agent => render_agent_feedback(reports.iter(), &feedback_options(options)),
    }
}

#[derive(Debug, Serialize)]
struct BatchReport<'a> {
    success: bool,
    reports: &'a [StructureCheckReport],
}

fn format_report(report: &StructureCheckReport, format: OutputFormat, options: &Options) -> String {
    match format {
        OutputFormat::Text => format_text_report(report),
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        OutputFormat::Advice => format_feedback(report, options, CheckFeedbackFormat::Advice),
        OutputFormat::Status => format_feedback(report, options, CheckFeedbackFormat::Status),
        OutputFormat::Agent if options.agent == AgentTarget::Codex => {
            render_codex_agent_feedback([report], &feedback_options(options))
        }
        OutputFormat::Agent => render_agent_feedback([report], &feedback_options(options)),
    }
}

fn format_feedback(
    report: &StructureCheckReport,
    options: &Options,
    format: CheckFeedbackFormat,
) -> String {
    render_check_feedback(report, format, &feedback_options(options))
}

fn feedback_options(options: &Options) -> FeedbackOptions {
    FeedbackOptions {
        minimum_severity: options.min_severity.clone(),
        max_issues: options.max_issues,
        warn: options.warn,
    }
}

fn format_feedback_reports(
    reports: &[StructureCheckReport],
    options: &Options,
    format: CheckFeedbackFormat,
) -> String {
    if reports.len() == 1 {
        return format_feedback(&reports[0], options, format);
    }

    let mut rendered = String::new();
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            rendered.push_str("\n\n");
        }
        rendered.push_str(&format_feedback(report, options, format));
    }
    rendered
}

fn format_batch_text_report(reports: &[StructureCheckReport]) -> String {
    let files_checked: usize = reports.iter().map(|report| report.files_checked).sum();
    let dirs_checked: usize = reports.iter().map(|report| report.dirs_checked).sum();
    let violations: usize = reports
        .iter()
        .map(StructureCheckReport::violation_count)
        .sum();
    let mut output = String::new();
    output.push_str("Assura structure check batch\n");
    output.push_str("============================\n");
    output.push_str(&format!("Projects checked: {}\n", reports.len()));
    output.push_str(&format!("Files checked: {files_checked}\n"));
    output.push_str(&format!("Directories checked: {dirs_checked}\n"));
    output.push_str(&format!("Violations: {violations}\n"));

    if violations == 0 {
        output.push_str("\nAll configured structure checks passed.\n");
        return output;
    }

    output.push_str("\nViolations\n----------\n");
    for report in reports {
        for violation in &report.violations {
            let _ = write!(
                output,
                "{}: {} [{}:{}] {}\n  Fix: {}\n  Blocking: {}\n",
                report.checked_path.display(),
                violation.path.display(),
                violation.severity,
                violation.rule,
                violation.message,
                violation.corrective_context,
                violation.blocking,
            );
        }
    }
    output
}

fn format_text_report(report: &StructureCheckReport) -> String {
    let mut output = String::new();
    output.push_str("Assura structure check\n");
    output.push_str("======================\n");
    output.push_str(&format!(
        "Project root: {}\n",
        report.project_root.display()
    ));
    output.push_str(&format!("Config: {}\n", report.config_path.display()));
    output.push_str(&format!(
        "Checked path: {}\n",
        report.checked_path.display()
    ));
    output.push_str(&format!("Files checked: {}\n", report.files_checked));
    output.push_str(&format!("Directories checked: {}\n", report.dirs_checked));
    output.push_str(&format!("Violations: {}\n", report.violation_count()));

    if report.violations.is_empty() {
        output.push_str("\nAll configured structure checks passed.\n");
        return output;
    }

    output.push_str("\nViolations\n----------\n");
    for violation in &report.violations {
        let _ = write!(
            output,
            "{} [{}:{}] {}\n  Fix: {}\n  Blocking: {}\n",
            violation.path.display(),
            violation.severity,
            violation.rule,
            violation.message,
            violation.corrective_context,
            violation.blocking,
        );
    }
    output
}

fn print_help(command_name: &str) {
    println!(
        "\
{command_name} {VERSION}
Fast structure validation entrypoint.

Usage:
  {command_name} [OPTIONS] [PATH...]

Options:
  -c, --config <PATH>    Assura config path
  -f, --format <FORMAT>  Output format: text, json, yaml, advice, status, agent [default: text]
      --agent <TARGET>   Delivery adapter for --format agent: generic, codex [default: generic]
      --min-severity <SEVERITY>
                          Only show feedback items for this severity or higher
      --max-issues <COUNT>
                          Maximum feedback items to show
  -o, --output <PATH>    Write report to a file
      --cache-dir <PATH> Reuse hot check results from this cache directory
      --cache            Reuse correctness-checked results in the default Git-aware cache
      --fail-fast        Stop after the first violation
      --warn             Report violations but exit successfully
      --ls-lint-target-semantics
                          Validate only the explicit target path
  -q, --quiet            Suppress success output
  -h, --help             Show this help
  -V, --version          Show version
"
    );
}

fn exit_code_for_check_error(error: &CheckError) -> i32 {
    match error {
        CheckError::NoConfig(_) => 4,
        CheckError::Config(_) => 2,
        _ => 3,
    }
}
