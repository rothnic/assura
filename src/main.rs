//! Primary Assura launcher.
//!
//! The common `assura check` path runs in this process through the lightweight
//! checker. Less frequent multi-command surfaces are dispatched to the
//! `assura-full` companion binary in release bundles.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    if let Some(exit_code) =
        assura::cli::check::fast_cli::try_run_primary_check_cli(args.iter().cloned())
    {
        process::exit(exit_code);
    }

    if let Some(exit_code) = run_companion(&args) {
        process::exit(exit_code);
    }

    #[cfg(feature = "full-cli")]
    process::exit(assura::cli::full_entry::run_full_cli_from_env());

    #[cfg(not(feature = "full-cli"))]
    {
        eprintln!("Error: assura-full companion binary was not found next to assura.");
        eprintln!("Install the complete Assura release bundle or use `assura check`.");
        process::exit(127);
    }
}

fn run_companion(args: &[OsString]) -> Option<i32> {
    let companion = companion_path()?;
    if !companion.is_file() {
        return None;
    }

    let mut command = Command::new(companion);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.arg0("assura");
    }
    command.env("ASSURA_CLI_BIN_NAME", "assura");
    let status = command.args(args.iter().skip(1)).status().ok()?;
    Some(status.code().unwrap_or(1))
}

fn companion_path() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.set_file_name(companion_file_name());
    Some(path)
}

fn companion_file_name() -> &'static str {
    if cfg!(windows) {
        "assura-full.exe"
    } else {
        "assura-full"
    }
}
