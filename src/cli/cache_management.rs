//! User-facing inspection and cleanup for the conservative check cache.

use super::{ExitCode, OutputFormat};
use crate::cli::check::cache::{clean_check_cache, inspect_check_cache, CheckCacheStatus};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct CacheCommandReport {
    action: &'static str,
    removed_entries: usize,
    removed_bytes: u64,
    status: CheckCacheStatus,
}

/// Inspect or clean the cache rooted for one project/worktree.
pub fn cache_command(
    path: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    format: OutputFormat,
    clean: bool,
) -> ExitCode {
    let path = match path {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("Error: failed to read current directory: {error}");
                return ExitCode::RuntimeError;
            }
        },
    };
    let before = inspect_check_cache(&path, cache_dir.as_deref());
    let removed_entries = before.entries;
    let removed_bytes = before.bytes;
    let status = if clean {
        match clean_check_cache(&path, cache_dir.as_deref()) {
            Ok(_) => inspect_check_cache(&path, cache_dir.as_deref()),
            Err(error) => {
                eprintln!("Error: failed to clean check cache: {error}");
                return ExitCode::RuntimeError;
            }
        }
    } else {
        before
    };
    let report = CacheCommandReport {
        action: if clean { "clean" } else { "status" },
        removed_entries: if clean { removed_entries } else { 0 },
        removed_bytes: if clean { removed_bytes } else { 0 },
        status,
    };
    println!("{}", render(&report, format));
    ExitCode::Success
}

fn render(report: &CacheCommandReport, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => format!(
            "Assura cache {}\nroot: {}\nworktree: {}\nshared: {}\nfallback: {}\nentries: {}\nbytes: {}\nremoved: {} entries / {} bytes",
            report.action,
            report.status.cache_root,
            report.status.worktree_namespace,
            report.status.shared_namespace.as_deref().unwrap_or("unavailable"),
            report.status.fallback_reason.as_deref().unwrap_or("none"),
            report.status.entries,
            report.status.bytes,
            report.removed_entries,
            report.removed_bytes,
        ),
    }
}
