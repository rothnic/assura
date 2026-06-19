//! Deterministic docs support-claim extraction for support matrices.

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DocsClaimSurface {
    pub(super) surface: String,
    pub(super) status: String,
}

pub(super) fn docs_claim_surfaces(content: &str) -> Vec<DocsClaimSurface> {
    let mut surfaces = Vec::new();
    let mut table_columns: Option<DocsClaimColumns> = None;

    for line in content.lines() {
        let Some(cells) = markdown_table_cells(line) else {
            table_columns = None;
            continue;
        };
        if is_markdown_separator_row(&cells) {
            continue;
        }
        if let Some(columns) = table_columns {
            let Some(surface_cell) = cells.get(columns.surface) else {
                continue;
            };
            let Some(status_cell) = cells.get(columns.status) else {
                continue;
            };
            let Some(status) = normalize_support_status(status_cell) else {
                continue;
            };
            let Some(surface) = normalize_docs_claim_surface(surface_cell, columns.surface_kind)
            else {
                continue;
            };
            surfaces.push(DocsClaimSurface { surface, status });
            continue;
        }
        table_columns = docs_claim_columns(&cells);
    }

    surfaces.sort_by(|left, right| left.surface.cmp(&right.surface));
    surfaces.dedup();
    surfaces
}

#[derive(Clone, Copy)]
struct DocsClaimColumns {
    surface: usize,
    status: usize,
    surface_kind: DocsClaimSurfaceKind,
}

#[derive(Clone, Copy)]
enum DocsClaimSurfaceKind {
    Surface,
    Command,
}

fn markdown_table_cells(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect::<Vec<_>>();
    (cells.len() >= 2).then_some(cells)
}

fn is_markdown_separator_row(cells: &[String]) -> bool {
    cells.iter().all(|cell| {
        let trimmed = cell.trim();
        !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|ch| matches!(ch, '-' | ':' | ' ' | '\t'))
    })
}

fn docs_claim_columns(cells: &[String]) -> Option<DocsClaimColumns> {
    let mut surface = None;
    let mut surface_kind = DocsClaimSurfaceKind::Surface;
    let mut status = None;
    for (index, cell) in cells.iter().enumerate() {
        let header = normalize_table_header(cell);
        match header.as_str() {
            "surface" => {
                surface = Some(index);
                surface_kind = DocsClaimSurfaceKind::Surface;
            }
            "command" => {
                surface = Some(index);
                surface_kind = DocsClaimSurfaceKind::Command;
            }
            "status" | "level" | "support" => {
                status = Some(index);
            }
            _ => {}
        }
    }
    Some(DocsClaimColumns {
        surface: surface?,
        status: status?,
        surface_kind,
    })
}

fn normalize_table_header(cell: &str) -> String {
    strip_markdown_inline(cell).to_ascii_lowercase()
}

fn normalize_support_status(cell: &str) -> Option<String> {
    let normalized = strip_markdown_inline(cell).to_ascii_lowercase();
    let first = normalized.split_whitespace().next()?;
    matches!(
        first,
        "supported" | "experimental" | "internal" | "roadmap" | "unsupported"
    )
    .then(|| first.to_string())
}

fn normalize_docs_claim_surface(cell: &str, surface_kind: DocsClaimSurfaceKind) -> Option<String> {
    let surface = strip_markdown_inline(cell);
    if surface.is_empty() || surface.contains(" and ") {
        return None;
    }
    if matches!(surface_kind, DocsClaimSurfaceKind::Command) {
        return Some(format!("command:{surface}"));
    }
    if surface.starts_with("command:")
        || surface.starts_with("rust:")
        || surface.starts_with("package:")
        || surface.starts_with("binary:")
    {
        return Some(surface);
    }
    if surface == "assura" || surface.starts_with("assura ") {
        return Some(format!("command:{surface}"));
    }
    None
}

fn strip_markdown_inline(cell: &str) -> String {
    let without_links = if let Some((_, rest)) = cell.split_once("](") {
        cell.split_once('[')
            .map(|(_, label)| label.split("](").next().unwrap_or(label))
            .unwrap_or(rest)
    } else {
        cell
    };
    without_links
        .trim()
        .trim_matches('`')
        .trim_matches('*')
        .trim()
        .to_string()
}
