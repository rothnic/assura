use std::path::Path;

use serde::{Deserialize, Serialize};

use super::MaturityResult;

/// Whether Assura has evidence that configured CI has actually executed.
///
/// Local repository files can establish configuration presence only. They
/// cannot establish execution, so locally collected observations are always
/// `Unverified` until a future evidence source supplies a verified result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiExecutionState {
    Unverified,
}

/// Bounded facts observed from local repository files.
///
/// These fields deliberately do not calculate a project-quality or maturity
/// score. File presence is evidence of configuration, not evidence that the
/// configuration runs or that a tool is effective.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectObservations {
    pub ci_config_present: bool,
    pub ci_execution_verified: CiExecutionState,
    pub black_config_present: bool,
    pub package_manifests: Vec<String>,
}

impl ProjectObservations {
    /// Collect local, non-evaluative repository observations.
    pub fn collect(path: impl AsRef<Path>) -> MaturityResult<Self> {
        let root = path.as_ref();
        let workflow_dir = root.join(".github/workflows");
        let ci_config_present = workflow_dir.is_dir()
            && workflow_dir
                .read_dir()?
                .filter_map(Result::ok)
                .any(|entry| {
                    entry
                        .file_type()
                        .map(|kind| kind.is_file())
                        .unwrap_or(false)
                });

        let pyproject = root.join("pyproject.toml");
        let black_config_present = pyproject.is_file()
            && std::fs::read_to_string(&pyproject)?
                .lines()
                .any(|line| line.trim() == "[tool.black]");

        let package_manifests = [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "requirements.txt",
        ]
        .into_iter()
        .filter(|name| root.join(name).is_file())
        .map(str::to_string)
        .collect();

        Ok(Self {
            ci_config_present,
            ci_execution_verified: CiExecutionState::Unverified,
            black_config_present,
            package_manifests,
        })
    }
}
