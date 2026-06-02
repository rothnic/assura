//! Checked real-project fixture materialization for agent feedback measurements.

use std::fs;
use std::path::{Path, PathBuf};

pub(in crate::cli::performance_report) fn create_real_project_agentic_feedback(
    root: &Path,
) -> Result<(), String> {
    let source = std::env::current_dir()
        .map(|cwd| cwd.join("tests/fixtures/real-project-agentic-feedback/valid"))
        .ok()
        .filter(|path| path.exists())
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/real-project-agentic-feedback/valid")
        });

    copy_dir_all(&source, root)
        .map_err(|error| format!("copy real-project feedback fixture: {error}"))
}

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
