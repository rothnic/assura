//! Internal companion binary for the complete multi-command CLI.

fn main() {
    std::process::exit(assura::cli::full_entry::run_full_cli_from_env());
}
