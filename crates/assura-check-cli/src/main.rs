//! Lightweight structure-check entrypoint optimized for validation latency.

fn main() {
    std::process::exit(assura::cli::check::fast_cli::run_check_cli_from_env(
        "assura-check",
    ));
}
