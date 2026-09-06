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

    match run_companion(&args) {
        Ok(Some(exit_code)) => process::exit(exit_code),
        Ok(None) => {}
        Err(error) => {
            eprintln!("Error: {error}");
            process::exit(1);
        }
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

fn run_companion(args: &[OsString]) -> std::io::Result<Option<i32>> {
    let Some(companion) = companion_path() else {
        return Ok(None);
    };
    if !companion.is_file() {
        return Ok(None);
    }

    let mut command = Command::new(&companion);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.arg0("assura");
    }
    command.env("ASSURA_CLI_BIN_NAME", "assura");
    let status = command
        .args(args.iter().skip(1))
        .status()
        .map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!(
                    "failed to launch companion '{}': {error}",
                    companion.display()
                ),
            )
        })?;
    Ok(Some(status.code().unwrap_or(1)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    static COMPANION_PATH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn present_unlaunchable_companion_is_not_treated_as_absent() {
        let _guard = COMPANION_PATH_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("companion test lock");
        let companion = companion_path().expect("test binary path");
        assert!(
            !companion.exists(),
            "test requires no bundled companion next to its harness: {}",
            companion.display()
        );
        fs::write(&companion, b"not an executable companion").expect("write invalid companion");

        let args = vec![OsString::from("assura"), OsString::from("--help")];
        let result = run_companion(&args);

        fs::remove_file(&companion).expect("remove invalid companion");
        assert!(
            result.is_err(),
            "a present companion that cannot launch must not fall back as absent"
        );
    }
}
