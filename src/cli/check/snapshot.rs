//! File line budgets for an explicit Git tree comparison.

use super::file_limits::{pattern_severity, repair_message};
use super::report::FindingSeverity;
use super::StructureChecker;
use crate::cli::args::{ChangeCheckFormat, ExitCode};
use crate::cli::git_snapshot::{Change, Scope, Side, Snapshot, Tree};
use crate::config::loader::ConfigLoader;
use crate::policy::owner_budget::OwnerBudget;
use crate::policy::source_selection::{select_snapshot_sources, Selection, SourceOrigin};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};

const POLICY: &str = ".assura/config.yml";

#[derive(Serialize)]
struct Report {
    success: bool,
    scope: &'static str,
    baseline: TreeIdentity,
    selected: TreeIdentity,
    policy_path: Option<PathBuf>,
    policies: Vec<Provenance>,
    exemptions: Vec<Exemption>,
    files_checked: usize,
    findings: Vec<Finding>,
}

#[derive(Serialize)]
struct TreeIdentity {
    tree: String,
    commit: Option<String>,
}

impl From<Tree> for TreeIdentity {
    fn from(tree: Tree) -> Self {
        Self {
            tree: tree.oid,
            commit: tree.commit,
        }
    }
}

#[derive(Serialize)]
struct Provenance {
    layer: &'static str,
    path: PathBuf,
    origin: SourceOrigin,
    tree: Option<String>,
}

#[derive(Serialize)]
struct Exemption {
    path: String,
    layer: &'static str,
    pattern: String,
}

enum Policy {
    Owner(OwnerBudget),
    Project {
        prefix: PathBuf,
        checker: Box<StructureChecker>,
    },
}

struct Budget {
    limit: usize,
    severity: FindingSeverity,
    repair_message: Option<String>,
}

impl Policy {
    fn budget(&mut self, path: &Path, display_path: &str) -> Option<Budget> {
        match self {
            Self::Owner(policy) => policy.exemption(display_path).is_none().then_some(Budget {
                limit: policy.max_lines,
                severity: FindingSeverity::High,
                repair_message: None,
            }),
            Self::Project { prefix, checker } => {
                let rel = path.strip_prefix(prefix).ok()?;
                let (limit, files, pattern) = selected_budget(checker, rel)?;
                let filename = rel.file_name()?.to_str()?;
                let severity = pattern_severity(
                    &files,
                    pattern.as_deref(),
                    filename,
                    rel,
                    &checker.glob_patterns,
                );
                let message = repair_message(
                    &files,
                    pattern.as_deref(),
                    filename,
                    rel,
                    &checker.glob_patterns,
                );
                Some(Budget {
                    limit,
                    severity: FindingSeverity::parse_or_default(&severity),
                    repair_message: message.map(str::to_owned),
                })
            }
        }
    }
}

#[derive(Serialize)]
struct Finding {
    path: String,
    rule: &'static str,
    layer: &'static str,
    severity: &'static str,
    blocking: bool,
    actual: usize,
    limit: usize,
    baseline_lines: Option<usize>,
    allowed_lines: usize,
    repair_message: Option<String>,
}

pub(in crate::cli) fn command(
    scope: Scope<'_>,
    format: ChangeCheckFormat,
    owner_baseline: Option<&Path>,
    project_config: Option<&Path>,
) -> ExitCode {
    match run(scope, owner_baseline, project_config).and_then(|report| {
        match format {
            ChangeCheckFormat::Json => {
                let output = serde_json::to_string_pretty(&report)
                    .map_err(|error| (ExitCode::RuntimeError, error.to_string()))?;
                println!("{output}");
            }
            ChangeCheckFormat::Text => report.print_text(),
        }
        Ok(report.success)
    }) {
        Ok(true) => ExitCode::Success,
        Ok(false) => ExitCode::ValidationFailed,
        Err((code, error)) => {
            eprintln!("Error: {}", error.escape_debug());
            code
        }
    }
}

impl Report {
    fn print_text(&self) {
        println!(
            "{} max_lines check: {} → {} ({}, {} files checked)",
            self.scope,
            self.baseline.tree,
            self.selected.tree,
            if self.success { "passed" } else { "failed" },
            self.files_checked,
        );
        for source in &self.policies {
            println!(
                "  {} policy: {:?} ({:?}, tree {:?})",
                source.layer, source.path, source.origin, source.tree
            );
        }
        for exemption in &self.exemptions {
            println!(
                "  {} exemption {:?} applies to {:?}",
                exemption.layer, exemption.pattern, exemption.path
            );
        }
        for finding in &self.findings {
            println!(
                "[{}/{}] {:?}: {} lines exceeds the allowed {} (configured limit {}). Lines to cut or move: {}.",
                finding.layer, finding.severity, finding.path, finding.actual, finding.allowed_lines, finding.limit,
                finding.actual - finding.allowed_lines,
            );
            if let Some(message) = &finding.repair_message {
                println!("  Policy message: {}", message.escape_debug());
            }
        }
    }
}

fn run(
    scope: Scope<'_>,
    owner_baseline: Option<&Path>,
    project_config: Option<&Path>,
) -> Result<Report, (ExitCode, String)> {
    let runtime_error = |error| (ExitCode::RuntimeError, error);
    let config_error = |error| (ExitCode::ConfigurationError, error);
    let cwd = std::env::current_dir().map_err(|error| runtime_error(error.to_string()))?;
    let sources = select_snapshot_sources(&cwd, owner_baseline, project_config)
        .map_err(|error| config_error(error.to_string()))?;
    let root = &sources
        .checkout
        .as_ref()
        .ok_or_else(|| runtime_error(format!("check-change requires a Git checkout at {cwd:?}")))?
        .root;
    let snapshot = Snapshot::resolve(&cwd, scope, root.clone()).map_err(runtime_error)?;
    let mut policies = Vec::new();
    for (layer, source) in [
        ("user_baseline", sources.user_baseline),
        ("scoped_owner", sources.scoped_owner),
    ] {
        if let Some(source) = source {
            let result = read_external(&source.path).and_then(|bytes| OwnerBudget::parse(&bytes));
            let budget = result.map_err(|error| {
                config_error(format!(
                    "{layer} policy {:?} from {:?}: {error}",
                    source.path, source.origin,
                ))
            })?;
            policies.push((provenance(layer, source, None), Policy::Owner(budget)));
        }
    }
    let project = load_project(&snapshot, &cwd, sources.project).map_err(config_error)?;
    let policy_path = project.as_ref().map(|(source, _)| source.path.clone());
    policies.extend(project);
    if policies.is_empty() {
        return Err((
            ExitCode::NoConfigFound,
            format!(
                "{POLICY} is absent from selected tree {} and no owner policy was selected",
                snapshot.selected.oid,
            ),
        ));
    }

    let mut findings = Vec::new();
    let mut exemptions = Vec::new();
    let mut files_checked = 0;
    for change in &snapshot.changes {
        let (path, previous) = match change {
            Change::Added(path) | Change::TypeChanged(path) => (path, None),
            Change::Modified(path) => (path, Some(path)),
            Change::Renamed { from, to } => (to, Some(from)),
            Change::Deleted(_) => continue,
        };
        let display_path = git_path_string(path).map_err(runtime_error)?;
        let budgets: Vec<_> = policies
            .iter_mut()
            .filter_map(|(source, policy)| {
                if let Policy::Owner(owner) = policy {
                    if let Some(pattern) = owner.exemption(&display_path) {
                        exemptions.push(Exemption {
                            path: display_path.clone(),
                            layer: source.layer,
                            pattern: pattern.to_owned(),
                        });
                        return None;
                    }
                }
                let budget = policy.budget(path, &display_path)?;
                Some((source.layer, policy, budget))
            })
            .collect();
        if budgets.is_empty() {
            continue;
        }
        let Some(actual) = count_lines(&snapshot, Side::Selected, path).map_err(runtime_error)?
        else {
            continue;
        };
        files_checked += 1;
        let mut baseline = None;
        for (layer, policy, budget) in budgets {
            if actual <= budget.limit {
                continue;
            }
            let baseline_lines = match previous {
                Some(previous)
                    if previous == path
                        || policy
                            .budget(previous, &git_path_string(previous).map_err(runtime_error)?)
                            .is_some_and(|old| old.limit == budget.limit) =>
                {
                    *match &mut baseline {
                        Some(lines) => lines,
                        None => baseline.insert(
                            count_lines(&snapshot, Side::Baseline, previous)
                                .map_err(runtime_error)?,
                        ),
                    }
                }
                _ => None,
            };
            let allowed_lines = budget.limit.max(baseline_lines.unwrap_or(0));
            if actual > allowed_lines {
                findings.push(Finding {
                    path: display_path.clone(),
                    rule: "max_lines",
                    layer,
                    severity: budget.severity.as_str(),
                    blocking: budget.severity.is_blocking(),
                    actual,
                    limit: budget.limit,
                    baseline_lines,
                    allowed_lines,
                    repair_message: budget.repair_message,
                });
            }
        }
    }
    findings.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Report {
        success: !findings.iter().any(|finding| finding.blocking),
        scope: match scope {
            Scope::Staged => "staged",
            Scope::Direct { .. } => "direct",
            Scope::MergeBase { .. } => "merge-base",
        },
        baseline: snapshot.baseline.into(),
        selected: snapshot.selected.into(),
        policy_path,
        policies: policies.into_iter().map(|(source, _)| source).collect(),
        exemptions,
        files_checked,
        findings,
    })
}

fn provenance(layer: &'static str, source: Selection, tree: Option<String>) -> Provenance {
    Provenance {
        layer,
        path: source.path,
        origin: source.origin,
        tree,
    }
}

fn load_project(
    snapshot: &Snapshot,
    cwd: &Path,
    selection: Option<Selection>,
) -> Result<Option<(Provenance, Policy)>, String> {
    let root = &snapshot.root;
    let (source, prefix, tree, bytes) = if let Some(mut source) = selection {
        let path = resolve_project_path(&source.path, root).map_err(|error| {
            format!(
                "project config {:?} from {:?}: {error}",
                source.path, source.origin
            )
        })?;
        if let Ok(path) = path.strip_prefix(root) {
            let bytes = snapshot.read_file(Side::Selected, path)?.ok_or_else(|| {
                format!(
                    "project config {:?} from {:?} is absent from selected tree {}",
                    source.path, source.origin, snapshot.selected.oid,
                )
            })?;
            let prefix = path
                .parent()
                .filter(|parent| parent.ends_with(".assura"))
                .and_then(Path::parent)
                .unwrap_or(Path::new(""))
                .to_path_buf();
            (source, prefix, Some(snapshot.selected.oid.clone()), bytes)
        } else {
            source.path = path;
            let bytes = read_external(&source.path).map_err(|error| {
                format!(
                    "project config {:?} from {:?}: {error}",
                    source.path, source.origin,
                )
            })?;
            (source, PathBuf::new(), None, bytes)
        }
    } else {
        let cwd = cwd
            .canonicalize()
            .map_err(|error| format!("resolve invocation directory {cwd:?}: {error}"))?;
        let mut found = None;
        for dir in cwd.ancestors().take_while(|dir| dir.starts_with(root)) {
            let prefix = dir
                .strip_prefix(root)
                .expect("ancestor checked against root");
            let path = prefix.join(POLICY);
            if let Some(bytes) = snapshot.read_file(Side::Selected, &path)? {
                found = Some((
                    Selection {
                        path,
                        origin: SourceOrigin::Discovered,
                    },
                    prefix.to_path_buf(),
                    Some(snapshot.selected.oid.clone()),
                    bytes,
                ));
                break;
            }
        }
        let Some(found) = found else { return Ok(None) };
        found
    };
    let location = || {
        format!(
            "project config {:?} from {:?} (tree {tree:?})",
            source.path, source.origin
        )
    };
    let yaml = std::str::from_utf8(&bytes).map_err(|error| format!("{}: {error}", location()))?;
    let config = ConfigLoader::parse(yaml).map_err(|error| format!("{}: {error}", location()))?;
    let checker = Box::new(StructureChecker::new(root.join(&prefix), config, false));
    Ok(Some((
        provenance("project", source, tree),
        Policy::Project { prefix, checker },
    )))
}

fn resolve_project_path(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let mut candidate = path.to_path_buf();
    for _ in 0..40 {
        if candidate
            .components()
            .any(|part| part == Component::ParentDir)
        {
            return Err(format!("unsupported parent component in {candidate:?}"));
        }
        if candidate.starts_with(root) {
            return Ok(candidate);
        }
        // Resolve aliases from the outside in. Once an ancestor reaches the
        // checkout, its remaining components are Git paths, even if absent on disk.
        let ancestors: Vec<_> = candidate.ancestors().skip(1).collect();
        let mut resolved = candidate.clone();
        for ancestor in ancestors.into_iter().rev() {
            let directory = ancestor
                .canonicalize()
                .map_err(|error| format!("resolve {ancestor:?}: {error}"))?;
            let suffix = candidate
                .strip_prefix(ancestor)
                .expect("candidate ancestor");
            resolved = directory.join(suffix);
            if directory.starts_with(root) {
                return Ok(resolved);
            }
        }
        let metadata = resolved
            .symlink_metadata()
            .map_err(|error| format!("inspect {resolved:?}: {error}"))?;
        if !metadata.is_symlink() {
            return Ok(resolved);
        }
        let target = resolved
            .read_link()
            .map_err(|error| format!("read link {resolved:?}: {error}"))?;
        resolved.pop();
        candidate = resolved.join(target);
    }
    Err(format!("too many symlinks at {path:?}"))
}

fn read_external(path: &Path) -> Result<Vec<u8>, String> {
    if !std::fs::metadata(path)
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("expected a regular policy file".into());
    }
    std::fs::read(path).map_err(|error| error.to_string())
}

fn selected_budget(
    checker: &mut StructureChecker,
    path: &Path,
) -> Option<(
    usize,
    std::sync::Arc<crate::config::config::FileBundle>,
    Option<String>,
)> {
    if path.ancestors().any(|part| checker.is_excluded_rel(part)) {
        return None;
    }
    let filename = path.file_name()?.to_str()?;
    let rules = checker.resolve_rules(path.parent()?);
    let files = rules.files?;
    let (limit, pattern) = checker.file_line_budget(path, &files, filename)?;
    let pattern = pattern.map(str::to_owned);
    Some((limit, files, pattern))
}

fn count_lines(snapshot: &Snapshot, side: Side, path: &Path) -> Result<Option<usize>, String> {
    let Some(bytes) = snapshot.read_regular_file(side, path)? else {
        return Ok(None);
    };
    let content = std::str::from_utf8(&bytes)
        .map_err(|error| format!("cannot count lines in Git blob {path:?}: {error}"))?;
    Ok(Some(content.lines().count()))
}

fn git_path_string(path: &Path) -> Result<String, String> {
    let path_string = path
        .to_str()
        .ok_or_else(|| format!("unsupported non-UTF-8 Git path {path:?}"))?;
    #[cfg(windows)]
    return Ok(path_string.replace('\\', "/"));
    #[cfg(not(windows))]
    Ok(path_string.to_owned())
}
