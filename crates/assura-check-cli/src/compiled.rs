//! Structure-check CLI that consumes a precompiled binary config artifact.

use assura::cli::{
    config::ConfigError, run_structure_check_with_artifact, CheckError,
    CompiledStructureConfigArtifact, ConfigDiscovery, StructureCheckReport,
};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Options {
    path: Option<PathBuf>,
    compiled_config: Option<PathBuf>,
    config: Option<PathBuf>,
    quiet: bool,
}

fn main() {
    let options = match parse_options() {
        Ok(ParseOutcome::Run(options)) => options,
        Ok(ParseOutcome::ExitSuccess) => process::exit(0),
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Try 'assura-check-compiled --help' for usage.");
            process::exit(2);
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

    process::exit(exit_code);
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
        println!("assura-check-compiled {VERSION}");
        return Ok(ParseOutcome::ExitSuccess);
    }

    let quiet = args.contains(["-q", "--quiet"]);
    let compiled_config = args
        .opt_value_from_os_str("--compiled-config", path_from_os_str)
        .map_err(|error| error.to_string())?;
    let config = args
        .opt_value_from_os_str("--config", path_from_os_str)
        .map_err(|error| error.to_string())?;
    let path = args
        .opt_free_from_os_str(path_from_os_str)
        .map_err(|error| error.to_string())?;
    let remaining = args.finish();
    if !remaining.is_empty() {
        return Err(format!("unexpected argument {:?}", remaining[0]));
    }

    Ok(ParseOutcome::Run(Options {
        path,
        compiled_config,
        config,
        quiet,
    }))
}

fn path_from_os_str(value: &OsStr) -> Result<PathBuf, &'static str> {
    Ok(PathBuf::from(value))
}

fn run(options: Options) -> Result<bool, CheckError> {
    if options.path.is_none() && options.compiled_config.is_none() {
        let checked_path = std::env::current_dir()?;
        let compiled_config = checked_path.join(".assura/check-config.bin");
        if let Ok(bytes) = std::fs::read(&compiled_config) {
            let explicit_source_config = options.config.is_some();
            let source_config = options
                .config
                .or_else(|| Some(checked_path.join(".assura/config.yml")));
            return run_with_artifact_bytes(
                checked_path.clone(),
                checked_path,
                compiled_config,
                bytes,
                source_config,
                explicit_source_config,
                options.quiet,
            );
        }
    }

    let (checked_path, project_root) = resolve_checked_path_and_project_root(options.path)?;
    let explicit_compiled_config = options.compiled_config.is_some();
    let compiled_config = options
        .compiled_config
        .unwrap_or_else(|| project_root.join(".assura/check-config.bin"));
    let explicit_source_config = options.config.is_some();
    let source_config = options
        .config
        .or_else(|| (!explicit_compiled_config).then(|| project_root.join(".assura/config.yml")));
    let bytes = std::fs::read(&compiled_config)?;

    run_with_artifact_bytes(
        checked_path,
        project_root,
        compiled_config,
        bytes,
        source_config,
        explicit_source_config,
        options.quiet,
    )
}

fn run_with_artifact_bytes(
    checked_path: PathBuf,
    project_root: PathBuf,
    compiled_config: PathBuf,
    bytes: Vec<u8>,
    config: Option<PathBuf>,
    require_source_config_path_match: bool,
    quiet: bool,
) -> Result<bool, CheckError> {
    let artifact: CompiledStructureConfigArtifact =
        postcard::from_bytes(&bytes).map_err(|error| {
            CheckError::Config(ConfigError::Invalid(format!(
                "invalid compiled config: {error}"
            )))
        })?;
    if !artifact.is_compatible() {
        return Err(CheckError::Config(ConfigError::Invalid(
            "compiled config was produced by an incompatible Assura version".to_string(),
        )));
    }
    if !artifact.matches_project_root(&project_root)? {
        return Err(CheckError::Config(ConfigError::Invalid(format!(
            "compiled config is stale for project root {}; recompile it before running",
            project_root.display()
        ))));
    }
    if let Some(config_path) = &config {
        let source_config_matches = if require_source_config_path_match {
            artifact.matches_source_config(config_path)?
        } else {
            artifact.matches_default_source_config(config_path)?
        };
        if !source_config_matches {
            return Err(CheckError::Config(ConfigError::Invalid(format!(
                "compiled config is stale for {}; recompile it before running with --config",
                config_path.display()
            ))));
        }
    }

    let report = run_structure_check_with_artifact(
        project_root,
        compiled_config,
        checked_path,
        artifact,
        false,
    )?;
    if !quiet || !report.success {
        println!("{}", format_text_report(&report));
    }
    Ok(report.success)
}

fn resolve_checked_path_and_project_root(
    path: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), CheckError> {
    let Some(requested_path) = path else {
        let checked_path = std::env::current_dir()?;
        if checked_path.join(".assura/config.yml").exists() {
            return Ok((checked_path.clone(), checked_path));
        }

        let project_root = ConfigDiscovery::find_project_root(&checked_path)
            .ok_or_else(|| CheckError::NoConfig(checked_path.clone()))?
            .canonicalize()?;
        return Ok((checked_path, project_root));
    };

    if !requested_path.exists() {
        return Err(CheckError::MissingPath(requested_path));
    }
    let checked_path = requested_path.canonicalize()?;
    let project_root = if checked_path.is_dir() && checked_path.join(".assura/config.yml").exists()
    {
        checked_path.clone()
    } else {
        ConfigDiscovery::find_project_root(&checked_path)
            .ok_or_else(|| CheckError::NoConfig(checked_path.clone()))?
            .canonicalize()?
    };

    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    Ok((checked_path, project_root))
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

// allow-reason: keep the catch-all arm when feature-gated CheckError variants
// differ across companion binary builds.
#[allow(unreachable_patterns)]
fn exit_code_for_check_error(error: &CheckError) -> i32 {
    match error {
        CheckError::MissingPath(_)
        | CheckError::NoConfig(_)
        | CheckError::InvalidConfigLocation(_)
        | CheckError::OutsideProject { .. }
        | CheckError::Config(_) => 2,
        CheckError::Io(_) | CheckError::Walkdir(_) => 3,
        _ => 3,
    }
}

fn print_help() {
    println!(
        "assura-check-compiled {VERSION}

Usage: assura-check-compiled [OPTIONS] [PATH]

Options:
  --compiled-config <PATH>  Binary config artifact to use for validation
                            (default: .assura/check-config.bin under project root)
  --config <PATH>           Verify the artifact matches this source config
  -q, --quiet               Suppress output when validation succeeds
  -h, --help                Print help
  -V, --version             Print version"
    );
}
