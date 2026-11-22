//! Resolve Git comparisons before applying policy. Paths are repository-relative.

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

#[derive(Clone, Copy)]
pub(crate) enum Scope<'a> {
    Staged,
    Direct { base: &'a str, head: &'a str },
    MergeBase { target: &'a str, head: &'a str },
}

#[derive(Clone, Copy)]
pub(crate) enum Side {
    Baseline,
    Selected,
}

#[derive(Debug)]
pub(crate) struct Tree {
    pub(crate) oid: String,
    pub(crate) commit: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Change {
    Added(PathBuf),
    Deleted(PathBuf),
    Modified(PathBuf),
    TypeChanged(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

#[derive(Debug)]
pub(crate) struct Snapshot {
    git_cwd: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) baseline: Tree,
    pub(crate) selected: Tree,
    pub(crate) changes: Vec<Change>,
}

impl Snapshot {
    pub(crate) fn resolve(cwd: &Path, scope: Scope<'_>) -> Result<Self, String> {
        // Keep Git's invocation directory stable, including for relative index overrides.
        let git_cwd = std::path::absolute(cwd)
            .map_err(|error| format!("resolve Git directory {cwd:?}: {error}"))?;
        let root = git(&git_cwd, ["rev-parse", "--show-toplevel"])?;
        let root = git_path(
            root.strip_suffix(b"\n")
                .ok_or("missing Git root terminator")?,
        )?;
        let (baseline, selected) = match scope {
            Scope::Staged => (
                head_or_empty(&git_cwd)?,
                Tree {
                    oid: oid(&git(&git_cwd, ["write-tree"])?)?,
                    commit: None,
                },
            ),
            Scope::Direct { base, head } => {
                (resolve_tree(&git_cwd, base)?, resolve_tree(&git_cwd, head)?)
            }
            Scope::MergeBase { target, head } => {
                let target = revision(&git_cwd, target, "commit")?;
                let head = revision(&git_cwd, head, "commit")?;
                let result = run_git(&git_cwd, ["merge-base", "--all", &target, &head])?;
                if result.status.code() == Some(1) {
                    return Err(format!(
                        "no merge base between {target} and {head}; choose a direct comparison"
                    ));
                }
                let bases = checked("merge-base", result)?;
                if bases.iter().filter(|&&byte| byte == b'\n').count() != 1 {
                    return Err(format!(
                        "multiple merge bases between {target} and {head}; choose a direct comparison"
                    ));
                }
                (
                    commit_tree(&git_cwd, oid(&bases)?)?,
                    commit_tree(&git_cwd, head)?,
                )
            }
        };
        let diff = git(
            &git_cwd,
            [
                "diff",
                "--name-status",
                "-z",
                "--no-relative",
                "--no-color",
                "--no-ext-diff",
                "--no-textconv",
                "--ignore-submodules=none",
                "--no-renames",
                "--find-renames=50%",
                "-l1000",
                &baseline.oid,
                &selected.oid,
                "--",
            ],
        )?;
        Ok(Self {
            git_cwd,
            root,
            baseline,
            selected,
            changes: parse_changes(&diff)?,
        })
    }

    /// Read any regular file from the chosen tree, including unchanged policy.
    /// A path absent from that tree yields None. Other Git entry types are errors.
    pub(crate) fn read_file(&self, side: Side, path: &Path) -> Result<Option<Vec<u8>>, String> {
        if path.as_os_str().is_empty()
            || !path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(format!("unsupported repository-relative Git path {path:?}"));
        }
        let tree = match side {
            Side::Baseline => &self.baseline,
            Side::Selected => &self.selected,
        };
        let mut literal = OsString::new();
        for component in path.components() {
            if !literal.is_empty() {
                literal.push("/");
            }
            literal.push(component.as_os_str());
        }
        let args = [
            OsStr::new("--literal-pathspecs"),
            OsStr::new("ls-tree"),
            OsStr::new("--full-tree"),
            OsStr::new("-z"),
            OsStr::new(&tree.oid),
            OsStr::new("--"),
            literal.as_os_str(),
        ];
        let entry = git(&self.git_cwd, args)?;
        if entry.is_empty() {
            return Ok(None);
        }
        let tab = entry
            .iter()
            .position(|&byte| byte == b'\t')
            .ok_or("invalid Git tree entry")?;
        let metadata = &entry[..tab];
        let blob = [b"100644 blob ".as_slice(), b"100755 blob ".as_slice()]
            .into_iter()
            .find_map(|prefix| metadata.strip_prefix(prefix))
            .ok_or_else(|| format!("unsupported Git entry at {path:?} in {}", tree.oid))?;
        Ok(Some(git(&self.git_cwd, ["cat-file", "blob", &oid(blob)?])?))
    }
}

fn head_or_empty(cwd: &Path) -> Result<Tree, String> {
    let head = run_git(cwd, ["rev-parse", "--verify", "--quiet", "HEAD"])?;
    if head.status.code() == Some(1) {
        let reference = git(cwd, ["symbolic-ref", "HEAD"])?;
        let reference = git_path(
            reference
                .strip_suffix(b"\n")
                .ok_or("missing Git HEAD terminator")?,
        )?;
        let exists = run_git(
            cwd,
            [
                OsStr::new("show-ref"),
                OsStr::new("--exists"),
                reference.as_os_str(),
            ],
        )?;
        if exists.status.code() == Some(2) {
            return Ok(Tree {
                oid: oid(&git(cwd, ["mktree"])?)?,
                commit: None,
            });
        }
        checked("show-ref", exists)?;
    }
    let head = oid(&checked("rev-parse HEAD", head)?)?;
    commit_tree(cwd, revision(cwd, &head, "commit")?)
}

fn resolve_tree(cwd: &Path, name: &str) -> Result<Tree, String> {
    match revision(cwd, name, "commit") {
        Ok(commit) => commit_tree(cwd, commit),
        Err(_) => Ok(Tree {
            oid: revision(cwd, name, "tree")?,
            commit: None,
        }),
    }
}

fn commit_tree(cwd: &Path, commit: String) -> Result<Tree, String> {
    Ok(Tree {
        oid: revision(cwd, &commit, "tree")?,
        commit: Some(commit),
    })
}

fn revision(cwd: &Path, name: &str, kind: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err(format!("expected an explicit Git {kind} revision"));
    }
    let expression = format!("{name}^{{{kind}}}");
    oid(&git(
        cwd,
        ["rev-parse", "--verify", "--end-of-options", &expression],
    )
    .map_err(|error| format!("resolve Git {kind} {name:?}: {error}"))?)
}

fn oid(bytes: &[u8]) -> Result<String, String> {
    let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    if matches!(bytes.len(), 40 | 64) && bytes.iter().all(u8::is_ascii_hexdigit) {
        return Ok(bytes.iter().map(|&byte| char::from(byte)).collect());
    }
    Err("invalid Git object ID".into())
}

fn parse_changes(diff: &[u8]) -> Result<Vec<Change>, String> {
    if diff.is_empty() {
        return Ok(Vec::new());
    }
    let mut fields = diff
        .strip_suffix(b"\0")
        .ok_or("missing Git diff record terminator")?
        .split(|&byte| byte == 0);
    let mut changes = Vec::new();
    let mut added = 0;
    let mut deleted = 0;
    while let Some(status) = fields.next() {
        let path = change_path(&mut fields)?;
        let change = match status {
            b"A" => {
                added += 1;
                Change::Added(path)
            }
            b"D" => {
                deleted += 1;
                Change::Deleted(path)
            }
            b"M" => Change::Modified(path),
            b"T" => Change::TypeChanged(path),
            [b'R', ..] => {
                added += 1;
                deleted += 1;
                Change::Renamed {
                    from: path,
                    to: change_path(&mut fields)?,
                }
            }
            _ => {
                return Err(format!(
                    "unsupported Git change status {status:?} at {path:?}"
                ))
            }
        };
        changes.push(change);
    }

    // Git chooses one pairing even when candidates compete. A single add/delete
    // pair is the only rename we can currently certify without that tie-break.
    if added > 1 || deleted > 1 {
        let mut unpaired = Vec::with_capacity(changes.len());
        for change in changes {
            match change {
                Change::Renamed { from, to } => {
                    unpaired.push(Change::Deleted(from));
                    unpaired.push(Change::Added(to));
                }
                change => unpaired.push(change),
            }
        }
        return Ok(unpaired);
    }
    Ok(changes)
}

fn change_path<'a>(fields: &mut impl Iterator<Item = &'a [u8]>) -> Result<PathBuf, String> {
    let bytes = fields
        .next()
        .filter(|path| !path.is_empty())
        .ok_or("missing Git change path")?;
    if cfg!(windows) && bytes.contains(&b'\\') {
        return Err(format!("unsupported Git path on Windows {bytes:?}"));
    }
    git_path(bytes)
}

fn git_path(bytes: &[u8]) -> Result<PathBuf, String> {
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(bytes.to_vec())
    };
    #[cfg(not(unix))]
    let name = OsString::from(
        std::str::from_utf8(bytes)
            .map_err(|error| format!("unsupported non-UTF-8 Git path {bytes:?}: {error}"))?,
    );
    Ok(PathBuf::from(name))
}

fn run_git(
    cwd: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<Output, String> {
    Command::new("git")
        .arg("--no-replace-objects")
        .current_dir(cwd)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .args(args)
        .output()
        .map_err(|error| format!("failed to run Git in {cwd:?}: {error}"))
}

fn git(cwd: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<Vec<u8>, String> {
    let args: Vec<_> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect();
    checked(&format!("{args:?}"), run_git(cwd, &args)?)
}

fn checked(operation: &str, output: Output) -> Result<Vec<u8>, String> {
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(format!(
        "Git {operation} failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim_end()
    ))
}
