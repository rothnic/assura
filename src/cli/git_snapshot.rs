//! Resolve Git comparisons before applying policy. Paths are repository-relative.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

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
    pub(crate) fn resolve(cwd: &Path, scope: Scope<'_>, root: PathBuf) -> Result<Self, String> {
        // Keep Git's invocation directory stable, including for relative index overrides.
        let git_cwd = std::path::absolute(cwd)
            .map_err(|error| format!("resolve Git directory {cwd:?}: {error}"))?;
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
        Self::from_trees(git_cwd, root, baseline, selected)
    }

    /// Pin a working-tree comparison against an explicit commit.
    pub(crate) fn resolve_working_tree(
        cwd: &Path,
        base: &str,
        root: PathBuf,
    ) -> Result<Self, String> {
        let git_cwd = std::path::absolute(cwd)
            .map_err(|error| format!("resolve Git directory {cwd:?}: {error}"))?;
        let baseline = commit_tree(&git_cwd, revision(&git_cwd, base, "commit")?)?;
        let selected = working_tree(&git_cwd)?;
        Self::from_trees(git_cwd, root, baseline, selected)
    }

    fn from_trees(
        git_cwd: PathBuf,
        root: PathBuf,
        baseline: Tree,
        selected: Tree,
    ) -> Result<Self, String> {
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
        self.read_tree_file(side, path, true)
    }

    /// Non-regular changed entries carry no file line budget.
    pub(crate) fn read_regular_file(
        &self,
        side: Side,
        path: &Path,
    ) -> Result<Option<Vec<u8>>, String> {
        self.read_tree_file(side, path, false)
    }

    fn read_tree_file(
        &self,
        side: Side,
        path: &Path,
        require_regular: bool,
    ) -> Result<Option<Vec<u8>>, String> {
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
            .find_map(|prefix| metadata.strip_prefix(prefix));
        let Some(blob) = blob else {
            return if require_regular {
                Err(format!("unsupported Git entry at {path:?} in {}", tree.oid))
            } else {
                Ok(None)
            };
        };
        Ok(Some(git(&self.git_cwd, ["cat-file", "blob", &oid(blob)?])?))
    }
}

fn working_tree(cwd: &Path) -> Result<Tree, String> {
    let index = git_path_line(cwd, ["rev-parse", "--git-path", "index"])?;
    let git_dir = git_path_line(cwd, ["rev-parse", "--absolute-git-dir"])?;
    let scratch = private_index(&git_dir)?;
    let scratch_index = scratch.0.join("index");
    copy_index(&cwd.join(index), &scratch_index)?;
    let git = |args: &[&str]| {
        let output = git_command(cwd)
            .env("GIT_INDEX_FILE", &scratch_index)
            .args(["-c", "core.fsmonitor=false"])
            .args(["-c", "core.splitIndex=false"])
            .args(args)
            .output()
            .map_err(|error| format!("failed to run Git in {cwd:?}: {error}"))?;
        checked(&format!("{args:?}"), output)
    };
    // Reject unmerged entries before add could resolve them in the private index.
    git(&["write-tree"])?;
    git(&["add", "--all"])?;
    Ok(Tree {
        oid: oid(&git(&["write-tree"])?)?,
        commit: None,
    })
}

fn git_path_line(
    cwd: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>> + std::fmt::Debug,
) -> Result<PathBuf, String> {
    let bytes = git(cwd, args)?;
    let line = bytes
        .strip_suffix(b"\n")
        .ok_or("missing Git path terminator")?;
    git_path(line)
}

fn copy_index(source: &Path, target: &Path) -> Result<(), String> {
    let mut index = match fs::File::open(source) {
        Ok(index) => index,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("open Git index {source:?}: {error}")),
    };
    let mut copy = || -> std::io::Result<()> {
        let modified = index.metadata()?.modified()?;
        let mut scratch = fs::File::create(target)?;
        std::io::copy(&mut index, &mut scratch)?;
        // Keep Git's ability to detect racily clean files from the index timestamp.
        scratch.set_modified(modified)
    };
    copy().map_err(|error| format!("copy Git index {source:?} to {target:?}: {error}"))
}

struct SnapshotIndex(PathBuf);

fn private_index(git_dir: &Path) -> Result<SnapshotIndex, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = git_dir.join(format!(
        "assura-snapshot-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path)
        .map_err(|error| format!("create private Git index at {path:?}: {error}"))?;
    Ok(SnapshotIndex(path))
}

impl Drop for SnapshotIndex {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn head_or_empty(cwd: &Path) -> Result<Tree, String> {
    let head = run_git(cwd, ["rev-parse", "--verify", "--quiet", "HEAD"])?;
    if head.status.code() == Some(1) {
        let reference = git_path_line(cwd, ["symbolic-ref", "HEAD"])?;
        let exists = run_git(
            cwd,
            [
                OsStr::new("show-ref"),
                OsStr::new("--exists"),
                reference.as_os_str(),
            ],
        )?;
        if exists.status.code() == Some(2)
            || (exists.status.code() == Some(129) && legacy_ref_missing(cwd, &reference)?)
        {
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

// Older Git lacks show-ref --exists. Its --verify error is also
// used for malformed loose refs, so check the ref path before treating it as absent.
fn legacy_ref_missing(cwd: &Path, reference: &Path) -> Result<bool, String> {
    let exists = run_git(
        cwd,
        [
            OsStr::new("show-ref"),
            OsStr::new("--verify"),
            reference.as_os_str(),
        ],
    )?;
    if exists.status.success() {
        return Ok(false);
    }
    let expected = [
        b"fatal: '".as_slice(),
        reference.as_os_str().as_encoded_bytes(),
        b"' - not a valid ref\n",
    ]
    .concat();
    if exists.status.code() != Some(128) || exists.stderr != expected {
        return checked("show-ref --verify", exists).map(|_| false);
    }
    let path = git_path_line(
        cwd,
        [
            OsStr::new("rev-parse"),
            OsStr::new("--git-path"),
            reference.as_os_str(),
        ],
    )?;
    match std::fs::symlink_metadata(cwd.join(path)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!("inspect Git ref {reference:?}: {error}")),
        Ok(_) => Err(format!(
            "cannot resolve existing Git HEAD ref {reference:?}"
        )),
    }
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
    git_command(cwd)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run Git in {cwd:?}: {error}"))
}

fn git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("--no-replace-objects")
        .current_dir(cwd)
        .env("LC_ALL", "C")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1");
    command
}

fn git(
    cwd: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>> + std::fmt::Debug,
) -> Result<Vec<u8>, String> {
    checked(&format!("{args:?}"), run_git(cwd, args)?)
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
