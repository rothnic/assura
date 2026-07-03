//! Bounded raw repository text search for day-one discovery.

use super::context::{ignored_reference_scan_path, QueryContext};
use super::keyword::lexical_match_score;
use super::output::{SearchMatchOutput, SearchOutput};
use crate::intelligence::FactId;
use std::fs;
use std::path::{Path, PathBuf};

const RAW_SEARCH_FILE_LIMIT: usize = 2048;
const RAW_SEARCH_FILE_SIZE_LIMIT: u64 = 512 * 1024;

pub(super) fn raw_search(context: &QueryContext, query: &str, limit: usize) -> SearchOutput {
    let mut matches = raw_search_matches(&context.project_root, query)
        .into_iter()
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
    });
    matches.truncate(limit);

    SearchOutput {
        query: query.to_string(),
        mode: "raw",
        fallback_used: false,
        matches,
    }
}

pub(super) fn modeled_with_raw_fallback(
    context: &QueryContext,
    query: &str,
    limit: usize,
) -> SearchOutput {
    let mut modeled = super::keyword::search(context, query);
    if !modeled.matches.is_empty() {
        return modeled;
    }

    let mut raw = raw_search(context, query, limit);
    raw.mode = "raw_fallback";
    raw.fallback_used = true;
    modeled.matches.clear();
    raw
}

fn raw_search_matches(project_root: &Path, query: &str) -> Vec<SearchMatchOutput> {
    let terms = query_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }

    raw_search_files(project_root)
        .into_iter()
        .filter_map(|path| {
            let rel_path = path.strip_prefix(project_root).ok()?.to_path_buf();
            let content = fs::read_to_string(&path).ok()?;
            Some(raw_file_matches(query, &terms, &rel_path, &content))
        })
        .flatten()
        .collect()
}

fn raw_file_matches(
    query: &str,
    terms: &[String],
    rel_path: &Path,
    content: &str,
) -> Vec<SearchMatchOutput> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let lower = line.to_ascii_lowercase();
            if !terms.iter().all(|term| lower.contains(term)) {
                return None;
            }
            let line_number = index + 1;
            let column = terms
                .iter()
                .filter_map(|term| lower.find(term).map(|column| column + 1))
                .min();
            Some(SearchMatchOutput {
                source_id: raw_source_id(rel_path, line_number).to_string(),
                source_kind: "raw_text".to_string(),
                score: lexical_match_score(query, line),
                collection: None,
                instance_id: None,
                path: Some(rel_path.to_path_buf()),
                line: Some(line_number),
                column,
                text: line.trim().to_string(),
            })
        })
        .collect()
}

fn raw_search_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_raw_search_files(project_root, project_root, &mut files);
    files.sort();
    files.truncate(RAW_SEARCH_FILE_LIMIT);
    files
}

fn collect_raw_search_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };
        if ignored_reference_scan_path(rel_path) {
            continue;
        }
        if file_type.is_dir() {
            collect_raw_search_files(root, &path, files);
        } else if file_type.is_file() && is_raw_search_file(&path) {
            files.push(path);
        }
    }
}

fn is_raw_search_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.len() <= RAW_SEARCH_FILE_SIZE_LIMIT)
        .unwrap_or(false)
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "md" | "mdx"
                        | "txt"
                        | "rs"
                        | "py"
                        | "js"
                        | "jsx"
                        | "ts"
                        | "tsx"
                        | "go"
                        | "java"
                        | "kt"
                        | "swift"
                        | "c"
                        | "h"
                        | "hpp"
                        | "cpp"
                        | "cs"
                        | "rb"
                        | "php"
                        | "sh"
                        | "bash"
                        | "zsh"
                        | "fish"
                        | "toml"
                        | "yaml"
                        | "yml"
                        | "json"
                        | "jsonl"
                )
            })
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|term| term.to_ascii_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

fn raw_source_id(path: &Path, line_number: usize) -> FactId {
    FactId::from_parts(
        "raw_text",
        &format!(
            "{}:{line_number}",
            path.to_string_lossy().replace('\\', "/")
        ),
    )
}
