//! Compile an Assura YAML config into a binary structure-check artifact.

use assura::cli::CompiledStructureConfigArtifact;
use assura::config::loader::ConfigLoader;
use lexopt::prelude::*;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct Options {
    config: PathBuf,
    output: PathBuf,
}

fn main() {
    let options = match parse_options() {
        Ok(ParseOutcome::Run(options)) => options,
        Ok(ParseOutcome::ExitSuccess) => process::exit(0),
        Err(error) => {
            eprintln!("Error: {error}");
            eprintln!("Try 'assura-check-compile-config --help' for usage.");
            process::exit(2);
        }
    };

    if let Err(error) = run(options) {
        eprintln!("Error: {error}");
        process::exit(3);
    }
}

enum ParseOutcome {
    Run(Options),
    ExitSuccess,
}

fn parse_options() -> Result<ParseOutcome, lexopt::Error> {
    let mut config = None;
    let mut output = None;
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("config") => config = Some(path_value(parser.value()?)),
            Short('o') | Long("output") => output = Some(path_value(parser.value()?)),
            Short('h') | Long("help") => {
                print_help();
                return Ok(ParseOutcome::ExitSuccess);
            }
            Short('V') | Long("version") => {
                println!("assura-check-compile-config {VERSION}");
                return Ok(ParseOutcome::ExitSuccess);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(ParseOutcome::Run(Options {
        config: config.ok_or("missing --config <PATH>")?,
        output: output.ok_or("missing --output <PATH>")?,
    }))
}

fn path_value(value: OsString) -> PathBuf {
    PathBuf::from(value)
}

fn run(options: Options) -> Result<(), String> {
    let source_bytes = std::fs::read(&options.config)
        .map_err(|error| format!("read {}: {error}", options.config.display()))?;
    let source = std::str::from_utf8(&source_bytes)
        .map_err(|error| format!("read {} as UTF-8: {error}", options.config.display()))?;
    let config = ConfigLoader::parse_validated(source)
        .map_err(|error| format!("load {}: {error}", options.config.display()))?;
    let artifact =
        CompiledStructureConfigArtifact::new_with_source(config, &options.config, &source_bytes)
            .map_err(|error| {
                format!(
                    "fingerprint source config {}: {error}",
                    options.config.display()
                )
            })?;
    let content =
        postcard::to_allocvec(&artifact).map_err(|error| format!("serialize config: {error}"))?;
    if let Some(parent) = options.output.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    std::fs::write(&options.output, content)
        .map_err(|error| format!("write {}: {error}", options.output.display()))
}

fn print_help() {
    println!(
        "assura-check-compile-config {VERSION}

Usage: assura-check-compile-config --config <PATH> --output <PATH>

Options:
  --config <PATH>   YAML config to compile
  -o, --output      Binary artifact path
  -h, --help        Print help
  -V, --version     Print version"
    );
}
