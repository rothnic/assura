//! Lightweight scan of top-level Rust public module and re-export families.

pub(super) fn rust_public_export_families(content: &str) -> Vec<String> {
    let mut families = Vec::new();
    let mut brace_depth = 0usize;
    let mut grouped_use_prefix: Option<String> = None;
    let mut in_block_comment = false;
    for line in content.lines() {
        let active_line = rust_code_without_comments(line, &mut in_block_comment);
        if let Some(prefix) = grouped_use_prefix.as_deref() {
            let (grouped, closed) = grouped_use_families(prefix, &active_line);
            families.extend(grouped);
            if closed {
                grouped_use_prefix = None;
            }
        } else if brace_depth == 0 {
            let (line_families, grouped_prefix) = rust_export_families(&active_line);
            families.extend(line_families);
            grouped_use_prefix = grouped_prefix;
        }
        brace_depth = update_brace_depth(brace_depth, &active_line);
    }
    families.sort();
    families.dedup();
    families
}

fn rust_export_families(line: &str) -> (Vec<String>, Option<String>) {
    let trimmed = line.trim_start();
    if let Some(family) = ident_after_prefix(trimmed, "pub mod ") {
        return (vec![family], None);
    }

    let Some(rest) = trimmed.strip_prefix("pub use ") else {
        return (Vec::new(), None);
    };
    if let Some((prefix, after_open)) = rest.split_once('{') {
        let (families, closed) = grouped_use_families(prefix, after_open);
        let grouped_prefix = (!closed).then(|| prefix.trim().to_string());
        return (families, grouped_prefix);
    }

    (
        use_export_family(rest).into_iter().collect::<Vec<_>>(),
        None,
    )
}

fn ident_after_prefix(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    let ident = rest
        .trim_start()
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()?;
    (!ident.is_empty()).then(|| ident.to_string())
}

fn rust_code_without_comments(line: &str, in_block_comment: &mut bool) -> String {
    let mut output = String::new();
    let mut rest = line;
    loop {
        if *in_block_comment {
            let Some(end) = rest.find("*/") else {
                break;
            };
            *in_block_comment = false;
            rest = &rest[end + 2..];
        }

        let line_comment = rest.find("//");
        let block_comment = rest.find("/*");
        match (line_comment, block_comment) {
            (Some(line_index), Some(block_index)) if line_index < block_index => {
                output.push_str(&rest[..line_index]);
                break;
            }
            (_, Some(block_index)) => {
                output.push_str(&rest[..block_index]);
                rest = &rest[block_index + 2..];
                *in_block_comment = true;
            }
            (Some(line_index), None) => {
                output.push_str(&rest[..line_index]);
                break;
            }
            (None, None) => {
                output.push_str(rest);
                break;
            }
        }
    }
    output
}

fn grouped_use_families(prefix: &str, fragment: &str) -> (Vec<String>, bool) {
    let content = fragment.split("//").next().unwrap_or(fragment);
    let closed = content.contains('}');
    let content = content
        .split('}')
        .next()
        .unwrap_or(content)
        .trim()
        .trim_end_matches(';');
    let prefix_family = use_export_family(prefix);
    let mut families = Vec::new();
    for entry in content.split(',') {
        let entry = entry.trim().trim_end_matches(';');
        if entry.is_empty() {
            continue;
        }
        if let Some(family) = prefix_family.clone().or_else(|| {
            let path = format!("{prefix}{entry}");
            use_export_family(&path)
        }) {
            families.push(family);
        }
    }
    (families, closed)
}

fn use_export_family(path: &str) -> Option<String> {
    path.trim()
        .trim_end_matches(';')
        .split("::")
        .map(clean_use_segment)
        .filter(|segment| !segment.is_empty())
        .find(|segment| !matches!(segment.as_str(), "crate" | "self" | "super"))
}

fn clean_use_segment(segment: &str) -> String {
    segment
        .trim()
        .trim_start_matches('{')
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()
        .unwrap_or_default()
        .to_string()
}

fn update_brace_depth(depth: usize, line: &str) -> usize {
    line.chars().fold(depth, |depth, ch| match ch {
        '{' => depth.saturating_add(1),
        '}' => depth.saturating_sub(1),
        _ => depth,
    })
}

#[cfg(test)]
mod tests {
    use super::rust_public_export_families;

    #[test]
    fn rust_public_export_families_extracts_module_and_reexport_families() {
        let content = r#"
            pub mod cli;
            pub mod stable_hash {
                pub use assura_stable_hash::*;
            }
            pub mod hidden {
                // } should not close this module.
                /* { should not change depth either. */
                pub use hidden_api::*;
            }
            pub use intelligence::{GraphBuilder, GraphResult};
            pub use crate::public_api::Widget;
            pub use {
                self::grouped::Thing,
                crate::policy::Rule,
            };
            pub use crate::{
                config::Config,
                validation::Validator,
            };
            pub mod comments_safe;
            pub use cli::Command;
            // pub mod ignored;
        "#;

        assert_eq!(
            rust_public_export_families(content),
            vec![
                "cli",
                "comments_safe",
                "config",
                "grouped",
                "hidden",
                "intelligence",
                "policy",
                "public_api",
                "stable_hash",
                "validation"
            ]
        );
    }
}
