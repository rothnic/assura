//! Report file rendering and persistence helpers.

use super::{PerformanceReport, PerformanceResultRow};
use std::fs;
use std::path::Path;

pub(super) fn render_jsonl(rows: &[PerformanceResultRow]) -> String {
    let mut rendered = String::new();
    for row in rows {
        if let Ok(line) = serde_json::to_string(row) {
            rendered.push_str(&line);
            rendered.push('\n');
        }
    }
    rendered
}

pub(super) fn append_history(path: &Path, rows: &[PerformanceResultRow]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    existing.push_str(&render_jsonl(rows));
    fs::write(path, existing)
}

pub(super) fn write_website_data(path: &Path, report: &PerformanceReport) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    fs::write(
        path.join("current.json"),
        serde_json::to_string_pretty(report).unwrap_or_default(),
    )?;
    fs::write(
        path.join("ls-lint-comparison-history.jsonl"),
        render_jsonl(&report.results),
    )
}

pub(super) fn write_text(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}
