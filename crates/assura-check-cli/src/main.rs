//! Lightweight structure-check entrypoint optimized for validation latency.

use assura::cli::run_structure_check_cached;
use assura::cli::{run_structure_check, run_structure_checks, CheckError, StructureCheckReport};
use serde::Serialize;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Options {
    paths: Vec<PathBuf>,
    config: Option<PathBuf>,
    format: OutputFormat,
    output: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    fail_fast: bool,
    quiet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

fn main() {
    let options = match parse_options() {
        Ok(ParseOutcome::Run(options)) => options,
        Ok(ParseOutcome::ExitSuccess) => std::process::exit(0),
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Try 'assura-check --help' for usage.");
            std::process::exit(2);
        }
    };

    let exit_code = match run(options) {
        Ok(success) => {
            if success {
                0
            } else {
                1
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            exit_code_for_check_error(&error)
        }
    };

    std::process::exit(exit_code);
}

enum ParseOutcome {
    Run(Options),
    ExitSuccess,
}

fn parse_options() -> Result<ParseOutcome, String> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        print_help();
        return Ok(ParseOutcome::ExitSuccess);
    }
    if args.contains(["-V", "--version"]) {
        println!("assura-check {VERSION}");
        return Ok(ParseOutcome::ExitSuccess);
    }

    let config = args
        .opt_value_from_os_str(["-c", "--config"], path_from_os_str)
        .map_err(|error| error.to_string())?;
    let format = args
        .opt_value_from_fn(["-f", "--format"], parse_format)
        .map_err(|error| error.to_string())?
        .unwrap_or(OutputFormat::Text);
    let output = args
        .opt_value_from_os_str(["-o", "--output"], path_from_os_str)
        .map_err(|error| error.to_string())?;
    let cache_dir = args
        .opt_value_from_os_str("--cache-dir", path_from_os_str)
        .map_err(|error| error.to_string())?;
    let fail_fast = args.contains("--fail-fast");
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
        output,
        cache_dir,
        fail_fast,
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
        other => Err(format!("unsupported format '{other}'")),
    }
}

fn reject_unknown_option(value: &OsString) -> Result<(), String> {
    let Some(value) = value.to_str() else {
        return Ok(());
    };
    if value.starts_with('-') {
        return Err(format!("unexpected argument {value:?}"));
    }
    Ok(())
}

fn run(options: Options) -> Result<bool, CheckError> {
    if options.cache_dir.is_none() && options.paths.len() <= 1 {
        let path = options.paths.first().cloned();
        let report = run_structure_check(path, options.config, options.fail_fast)?;
        let success = report.success;
        if !options.quiet || !success || options.output.is_some() {
            let rendered = format_report(&report, options.format);
            if let Some(output) = options.output {
                std::fs::write(output, rendered)?;
            } else {
                println!("{rendered}");
            }
        }
        return Ok(success);
    }

    let paths = if options.paths.is_empty() {
        vec![None]
    } else {
        options.paths.into_iter().map(Some).collect()
    };
    let reports = if let Some(cache_dir) = options.cache_dir {
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
        run_structure_checks(paths, options.config, options.fail_fast)?
    };
    let success = reports.iter().all(|report| report.success);

    if !options.quiet || !success || options.output.is_some() {
        let rendered = format_reports(&reports, options.format);
        if let Some(output) = options.output {
            std::fs::write(output, rendered)?;
        } else {
            println!("{rendered}");
        }
    }
    Ok(success)
}

fn format_reports(reports: &[StructureCheckReport], format: OutputFormat) -> String {
    if reports.len() == 1 {
        return format_report(&reports[0], format);
    }

    let success = reports.iter().all(|report| report.success);
    let batch = BatchReport { success, reports };
    match format {
        OutputFormat::Text => format_batch_text_report(reports),
        OutputFormat::Json => serde_json::to_string_pretty(&batch).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(&batch).unwrap_or_default(),
    }
}

#[derive(Debug, Serialize)]
struct BatchReport<'a> {
    success: bool,
    reports: &'a [StructureCheckReport],
}

fn format_report(report: &StructureCheckReport, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_text_report(report),
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
    }
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

    output.push_str("\nViolations\n");
    output.push_str("----------\n");
    for report in reports {
        for violation in &report.violations {
            output.push_str(&format!(
                "{}: {} [{}:{}] {}\n",
                report.checked_path.display(),
                violation.path.display(),
                violation.severity,
                violation.rule,
                violation.message
            ));
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

    output.push_str("\nViolations\n");
    output.push_str("----------\n");
    for violation in &report.violations {
        output.push_str(&format!(
            "{} [{}:{}] {}\n",
            violation.path.display(),
            violation.severity,
            violation.rule,
            violation.message
        ));
    }

    output
}

fn print_help() {
    println!(
        "\
assura-check {VERSION}
Fast structure validation entrypoint.

Usage:
  assura-check [OPTIONS] [PATH...]

Options:
  -c, --config <PATH>    Assura config path
  -f, --format <FORMAT>  Output format: text, json, yaml [default: text]
  -o, --output <PATH>    Write report to a file
      --cache-dir <PATH> Reuse hot check results from this cache directory
      --fail-fast        Stop after the first violation
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
