//! Discover policy files in the checked target's Git and filesystem context.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceOrigin {
    UserDefault,
    Explicit,
    Git(String),
    Discovered,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Selection {
    pub(crate) path: PathBuf,
    pub(crate) origin: SourceOrigin,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Checkout {
    pub(crate) root: PathBuf,
    pub(crate) common_dir: PathBuf,
    pub(crate) git_dir: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PolicySources {
    pub(crate) checkout: Option<Checkout>,
    pub(crate) project_root: PathBuf,
    pub(crate) user_baseline: Option<Selection>,
    pub(crate) scoped_owner: Option<Selection>,
    pub(crate) project: Option<Selection>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SelectionError {
    #[error("git {operation} in {directory:?} failed ({status}): {diagnostic}")]
    Git {
        operation: String,
        directory: PathBuf,
        status: std::process::ExitStatus,
        diagnostic: String,
    },
    #[error(
        "git {operation} in {directory:?} returned invalid framing or unsupported path encoding"
    )]
    GitOutput {
        operation: String,
        directory: PathBuf,
    },
    #[error("{operation} at {path:?}: {source}")]
    Io {
        operation: String,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{layer} from {origin:?} must select a readable policy file at an absolute or home-relative path: {path:?}")]
    InvalidPath {
        layer: &'static str,
        origin: SourceOrigin,
        path: PathBuf,
    },
}

/// Choose the three policy inputs without loading their contents.
#[allow(
    dead_code,
    reason = "Working-tree consumers are wired in a later slice"
)]
pub(crate) fn select_sources(
    target: &Path,
    explicit_baseline: Option<&Path>,
    explicit_project: Option<&Path>,
) -> Result<PolicySources, SelectionError> {
    select_policy_sources(target, explicit_baseline, explicit_project, false)
}

pub(crate) fn select_snapshot_sources(
    target: &Path,
    explicit_baseline: Option<&Path>,
    explicit_project: Option<&Path>,
) -> Result<PolicySources, SelectionError> {
    select_policy_sources(target, explicit_baseline, explicit_project, true)
}

fn select_policy_sources(
    target: &Path,
    explicit_baseline: Option<&Path>,
    explicit_project: Option<&Path>,
    snapshot: bool,
) -> Result<PolicySources, SelectionError> {
    let home = env::home_dir();
    select(
        target,
        explicit_baseline,
        explicit_project,
        default_user_baseline(home.as_deref()).as_deref(),
        home.as_deref(),
        snapshot,
        || Command::new("git"),
    )
}

fn default_user_baseline(home: Option<&Path>) -> Option<PathBuf> {
    let config = if cfg!(windows) {
        PathBuf::from(env::var_os("APPDATA")?)
    } else if cfg!(target_os = "macos") {
        home?.join("Library/Application Support")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| home.map(|path| path.join(".config")))?
    };
    config
        .is_absolute()
        .then(|| config.join("assura/policy.yml"))
}

fn select(
    target: &Path,
    explicit_baseline: Option<&Path>,
    explicit_project: Option<&Path>,
    default_baseline: Option<&Path>,
    home: Option<&Path>,
    snapshot: bool,
    git: impl Fn() -> Command,
) -> Result<PolicySources, SelectionError> {
    let metadata = fs::metadata(target).map_err(|err| io_error("inspect target", target, err))?;
    let start = if metadata.is_dir() {
        target
    } else {
        target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
    };
    let start = start
        .canonicalize()
        .map_err(|err| io_error("resolve target directory", start, err))?;
    let run = |args: &[&str]| {
        git()
            .current_dir(&start)
            .env("LC_ALL", "C")
            .args(args)
            .output()
            .map_err(|err| io_error(format!("run git {}", args.join(" ")), &start, err))
    };

    let checkout = match git_path(&start, &run, "--show-toplevel", true)? {
        Some(root) => Some(Checkout {
            root,
            common_dir: required_git_path(&start, &run, "--git-common-dir")?,
            git_dir: required_git_path(&start, &run, "--git-dir")?,
        }),
        None => None,
    };
    let user_baseline = match explicit_baseline {
        Some(path) => Some(selected(
            "user baseline",
            path,
            SourceOrigin::Explicit,
            home,
        )?),
        None => default_baseline
            .map(|path| optional_file("user baseline", path, SourceOrigin::UserDefault))
            .transpose()?
            .flatten(),
    };

    let scalar = |key| match checkout {
        Some(_) => git_scalar(
            &start,
            &run,
            key,
            home,
            snapshot && key == "assura.projectConfig",
        ),
        None => Ok(None),
    };
    let scoped_owner = scalar("assura.ownerPolicy")?;
    let mut project_root = checkout
        .as_ref()
        .map_or_else(|| start.clone(), |git| git.root.clone());
    let project = match explicit_project {
        Some(path) if snapshot => Some(selection(
            "project config",
            path,
            SourceOrigin::Explicit,
            home,
        )?),
        Some(path) => Some(selected(
            "project config",
            path,
            SourceOrigin::Explicit,
            home,
        )?),
        None => match scalar("assura.projectConfig")? {
            Some(source) => Some(source),
            None if snapshot => None,
            None => {
                let mut discovered = None;
                for dir in start.ancestors().take_while(|dir| {
                    checkout
                        .as_ref()
                        .is_none_or(|git| dir.starts_with(&git.root))
                }) {
                    let path = dir.join(".assura/config.yml");
                    if let Some(source) =
                        optional_file("project config", &path, SourceOrigin::Discovered)?
                    {
                        project_root = dir.to_path_buf();
                        discovered = Some(source);
                        break;
                    }
                }
                discovered
            }
        },
    };
    Ok(PolicySources {
        checkout,
        project_root,
        user_baseline,
        scoped_owner,
        project,
    })
}

fn required_git_path(
    directory: &Path,
    run: &impl Fn(&[&str]) -> Result<Output, SelectionError>,
    option: &str,
) -> Result<PathBuf, SelectionError> {
    git_path(directory, run, option, false)?.ok_or_else(|| invalid_output(directory, option))
}

fn git_path(
    directory: &Path,
    run: &impl Fn(&[&str]) -> Result<Output, SelectionError>,
    option: &str,
    allow_non_repo: bool,
) -> Result<Option<PathBuf>, SelectionError> {
    let args = ["rev-parse", "--path-format=absolute", option];
    let output = run(&args)?;
    // Git uses 128 for both ordinary non-repositories and fatal errors.
    if allow_non_repo
        && output.status.code() == Some(128)
        && output.stdout.is_empty()
        && output
            .stderr
            .starts_with(b"fatal: not a git repository (or any of the parent directories): .git")
    {
        return Ok(None);
    }
    check_git(directory, &args, &output)?;
    let path = output
        .stdout
        .strip_suffix(b"\n")
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| invalid_output(directory, option))?;
    path.canonicalize()
        .map(Some)
        .map_err(|err| io_error(format!("resolve Git path from {option}"), &path, err))
}

fn git_scalar(
    directory: &Path,
    run: &impl Fn(&[&str]) -> Result<Output, SelectionError>,
    key: &'static str,
    home: Option<&Path>,
    defer_read: bool,
) -> Result<Option<Selection>, SelectionError> {
    let args = ["config", "--null", "--show-origin", "--get", key];
    let output = run(&args)?;
    if output.status.code() == Some(1) && output.stderr.is_empty() && output.stdout.is_empty() {
        return Ok(None);
    }
    check_git(directory, &args, &output)?;
    let decoded =
        std::str::from_utf8(&output.stdout).map_err(|_| invalid_output(directory, key))?;
    let mut fields = decoded.split('\0');
    match (fields.next(), fields.next(), fields.next(), fields.next()) {
        (Some(origin), Some(value), Some(""), None) if !origin.is_empty() => {
            let resolve = if defer_read { selection } else { selected };
            resolve(
                key,
                Path::new(value),
                SourceOrigin::Git(origin.to_owned()),
                home,
            )
            .map(Some)
        }
        _ => Err(invalid_output(directory, key)),
    }
}

fn check_git(directory: &Path, args: &[&str], output: &Output) -> Result<(), SelectionError> {
    if output.status.success() && output.stderr.is_empty() {
        return Ok(());
    }
    Err(SelectionError::Git {
        operation: args.join(" "),
        directory: directory.to_path_buf(),
        status: output.status,
        diagnostic: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn invalid_output(directory: &Path, operation: &str) -> SelectionError {
    SelectionError::GitOutput {
        operation: operation.to_owned(),
        directory: directory.to_path_buf(),
    }
}

fn io_error(operation: impl Into<String>, path: &Path, source: io::Error) -> SelectionError {
    SelectionError::Io {
        operation: operation.into(),
        path: path.to_path_buf(),
        source,
    }
}

fn optional_file(
    layer: &'static str,
    path: &Path,
    origin: SourceOrigin,
) -> Result<Option<Selection>, SelectionError> {
    match fs::symlink_metadata(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(io_error(
            format!("inspect {layer} from {origin:?}"),
            path,
            err,
        )),
        Ok(_) => selected(layer, path, origin, None).map(Some),
    }
}

fn selected(
    layer: &'static str,
    path: &Path,
    origin: SourceOrigin,
    home: Option<&Path>,
) -> Result<Selection, SelectionError> {
    let selection = selection(layer, path, origin, home)?;
    let Selection { path, origin } = &selection;
    let metadata = fs::metadata(path)
        .map_err(|err| io_error(format!("inspect {layer} from {origin:?}"), path, err))?;
    if !metadata.is_file() {
        return Err(SelectionError::InvalidPath {
            layer,
            origin: selection.origin,
            path: selection.path,
        });
    }
    fs::File::open(path)
        .map_err(|err| io_error(format!("open {layer} from {origin:?}"), path, err))?;
    Ok(selection)
}

fn selection(
    layer: &'static str,
    path: &Path,
    origin: SourceOrigin,
    home: Option<&Path>,
) -> Result<Selection, SelectionError> {
    let resolved = if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        home.zip(path.strip_prefix("~").ok())
            .map(|(home, rest)| home.join(rest))
            .filter(|path| path.is_absolute())
    };
    let Some(resolved) = resolved else {
        return Err(SelectionError::InvalidPath {
            layer,
            origin,
            path: path.to_path_buf(),
        });
    };
    Ok(Selection {
        path: resolved,
        origin,
    })
}

#[cfg(test)]
#[path = "source_selection_tests.rs"]
mod tests;
