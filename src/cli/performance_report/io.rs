//! Report file rendering and persistence helpers.

use super::{PerformanceReport, PerformanceResultRow};
use crate::cli::args::PerformanceReportSuite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const TRACKED_HISTORY_ROW_LIMIT: usize = 1000;

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
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(render_jsonl(rows).as_bytes())?;
    drop(file);
    truncate_history_rows(path, TRACKED_HISTORY_ROW_LIMIT)
}

fn truncate_history_rows(path: &Path, row_limit: usize) -> std::io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let lines = contents.lines().collect::<Vec<_>>();
    if lines.len() <= row_limit {
        return Ok(());
    }

    let start = lines.len() - row_limit;
    let mut truncated = lines[start..].join("\n");
    truncated.push('\n');
    fs::write(path, truncated)
}

pub(super) fn write_website_data(
    path: &Path,
    report: &PerformanceReport,
    history_source: Option<&Path>,
    suite: PerformanceReportSuite,
) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    let current_target = path.join(match suite {
        PerformanceReportSuite::LsLint => "current.json",
        PerformanceReportSuite::Native => "native-current.json",
    });
    fs::write(
        current_target,
        serde_json::to_string(report).unwrap_or_default(),
    )?;
    let history_target = path.join(match suite {
        PerformanceReportSuite::LsLint => "ls-lint-comparison-history.jsonl",
        PerformanceReportSuite::Native => "native-history.jsonl",
    });
    if let Some(history_source) = history_source {
        fs::copy(history_source, history_target)?;
    } else {
        append_history(&history_target, &report.results)?;
    }
    Ok(())
}

pub(super) fn write_text(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::truncate_history_rows;
    use std::fs;

    #[test]
    fn truncates_jsonl_history_to_most_recent_rows() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("history.jsonl");
        fs::write(&path, "old\nmiddle\nnew\n").unwrap();

        truncate_history_rows(&path, 2).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "middle\nnew\n");
    }
}
