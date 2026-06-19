//! Release artifact contract validation.

use super::{CheckError, StructureCheckReport, StructureChecker};
use crate::cli::config::ConfigError;
use crate::config::config::ReleaseContractConfig;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_release_contracts(
        &self,
        contracts: &[ReleaseContractConfig],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for contract in contracts {
            self.validate_release_contract(contract, report)?;
        }
        Ok(())
    }

    fn validate_release_contract(
        &self,
        contract: &ReleaseContractConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        validate_release_contract_shape(contract)?;

        let workflow_files = self.read_release_contract_files(
            contract,
            "workflow",
            &contract.workflow_files,
            report,
        )?;
        let docs_files =
            self.read_release_contract_files(contract, "docs", &contract.docs_files, report)?;
        let installer_files = self.read_release_contract_files(
            contract,
            "installer",
            &contract.installer_files,
            report,
        )?;

        let artifact_names = contract
            .artifacts
            .iter()
            .map(|artifact| artifact.name.as_str())
            .collect::<HashSet<_>>();
        let checksum_sidecars = contract
            .artifacts
            .iter()
            .filter(|artifact| artifact.checksum_sidecar)
            .map(|artifact| format!("{}.sha256", artifact.name))
            .collect::<HashSet<_>>();

        self.validate_workflow_artifacts(contract, &workflow_files, report);
        self.validate_documented_artifacts(
            contract,
            &artifact_names,
            &checksum_sidecars,
            docs_files.iter().chain(installer_files.iter()),
            report,
        );
        self.validate_documented_checksum_sidecars(
            contract,
            &checksum_sidecars,
            &workflow_files,
            docs_files.iter().chain(installer_files.iter()),
            report,
        );
        self.validate_install_urls(contract, &artifact_names, &installer_files, report);

        Ok(())
    }

    fn read_release_contract_files(
        &self,
        contract: &ReleaseContractConfig,
        kind: &str,
        configured_paths: &[String],
        report: &mut StructureCheckReport,
    ) -> Result<Vec<ReleaseContractFile>, CheckError> {
        let mut files = Vec::new();
        for configured_path in configured_paths {
            let rel = safe_release_contract_path(contract, configured_path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_release_contract_violation(
                    report,
                    contract,
                    rel,
                    format!(
                        "Release contract `{}` configured {kind} file `{configured_path}` does not exist",
                        contract.id
                    ),
                );
                continue;
            }
            let content = fs::read_to_string(&path)?;
            files.push(ReleaseContractFile { rel, content });
        }
        Ok(files)
    }

    fn validate_workflow_artifacts(
        &self,
        contract: &ReleaseContractConfig,
        workflow_files: &[ReleaseContractFile],
        report: &mut StructureCheckReport,
    ) {
        let fallback_path = first_file_path(workflow_files);
        for artifact in &contract.artifacts {
            if !artifact.name.is_empty()
                && !workflow_files
                    .iter()
                    .any(|file| file.content.contains(&artifact.name))
            {
                self.push_release_contract_violation(
                    report,
                    contract,
                    fallback_path.clone(),
                    format!(
                        "Release contract `{}` expects workflow files to publish artifact `{}`, but no configured workflow file mentions it",
                        contract.id, artifact.name
                    ),
                );
            }
            if artifact.checksum_sidecar {
                let sidecar = format!("{}.sha256", artifact.name);
                if !sidecar.is_empty()
                    && !workflow_files
                        .iter()
                        .any(|file| file.content.contains(&sidecar))
                {
                    self.push_release_contract_violation(
                        report,
                        contract,
                        fallback_path.clone(),
                        format!(
                            "Release contract `{}` expects workflow files to publish checksum sidecar `{sidecar}`",
                            contract.id
                        ),
                    );
                }
            }
        }
    }

    fn validate_documented_artifacts<'a>(
        &self,
        contract: &ReleaseContractConfig,
        artifact_names: &HashSet<&str>,
        checksum_sidecars: &HashSet<String>,
        files: impl Iterator<Item = &'a ReleaseContractFile>,
        report: &mut StructureCheckReport,
    ) {
        for file in files {
            let mut seen = HashSet::new();
            for token in release_asset_tokens(&file.content) {
                if !seen.insert(token.clone()) {
                    continue;
                }
                if artifact_names.contains(token.as_str()) || checksum_sidecars.contains(&token) {
                    continue;
                }
                if let Some(base) = token.strip_suffix(".sha256") {
                    if artifact_names.contains(base) {
                        continue;
                    }
                }
                self.push_release_contract_violation(
                    report,
                    contract,
                    file.rel.clone(),
                    format!(
                        "Release contract `{}` does not declare documented artifact `{token}` in `{}`",
                        contract.id,
                        display_rel_path(&file.rel)
                    ),
                );
            }
        }
    }

    fn validate_documented_checksum_sidecars<'a>(
        &self,
        contract: &ReleaseContractConfig,
        checksum_sidecars: &HashSet<String>,
        workflow_files: &[ReleaseContractFile],
        docs_and_installers: impl Iterator<Item = &'a ReleaseContractFile>,
        report: &mut StructureCheckReport,
    ) {
        if checksum_sidecars.is_empty() {
            return;
        }

        let docs_and_installers = docs_and_installers.collect::<Vec<_>>();
        let docs_fallback = docs_and_installers
            .first()
            .map(|file| file.rel.clone())
            .unwrap_or_else(|| PathBuf::from(".assura/config.yml"));
        let workflow_fallback = first_file_path(workflow_files);

        for sidecar in checksum_sidecars {
            if !sidecar.is_empty()
                && !workflow_files
                    .iter()
                    .any(|file| file.content.contains(sidecar))
            {
                self.push_release_contract_violation(
                    report,
                    contract,
                    workflow_fallback.clone(),
                    format!(
                        "Release contract `{}` requires checksum sidecar `{sidecar}`, but configured workflow files do not mention it",
                        contract.id
                    ),
                );
            }
            if !sidecar.is_empty()
                && !docs_and_installers
                    .iter()
                    .any(|file| file.content.contains(sidecar))
            {
                self.push_release_contract_violation(
                    report,
                    contract,
                    docs_fallback.clone(),
                    format!(
                        "Release contract `{}` requires checksum sidecar `{sidecar}`, but configured docs/installers do not mention it",
                        contract.id
                    ),
                );
            }
        }
    }

    fn validate_install_urls(
        &self,
        contract: &ReleaseContractConfig,
        artifact_names: &HashSet<&str>,
        installer_files: &[ReleaseContractFile],
        report: &mut StructureCheckReport,
    ) {
        let allowed_branches = contract
            .allowed_url_branches
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        for file in installer_files {
            for url in url_tokens(&file.content) {
                if let Some(branch) = raw_or_blob_url_branch(&url) {
                    if !allowed_branches.is_empty() && !allowed_branches.contains(branch) {
                        self.push_release_contract_violation(
                            report,
                            contract,
                            file.rel.clone(),
                            format!(
                                "Release contract `{}` does not allow install URL branch `{branch}` in `{}`",
                                contract.id,
                                display_rel_path(&file.rel)
                            ),
                        );
                    }
                }
                for token in release_asset_tokens(&url) {
                    let asset = token.strip_suffix(".sha256").unwrap_or(&token);
                    if !artifact_names.contains(asset) {
                        self.push_release_contract_violation(
                            report,
                            contract,
                            file.rel.clone(),
                            format!(
                                "Release contract `{}` does not declare install URL artifact `{token}` in `{}`",
                                contract.id,
                                display_rel_path(&file.rel)
                            ),
                        );
                    }
                }
            }
        }
    }

    fn push_release_contract_violation(
        &self,
        report: &mut StructureCheckReport,
        contract: &ReleaseContractConfig,
        path: PathBuf,
        message: String,
    ) {
        self.push_violation(
            report,
            path,
            format!("release_contract:{}", contract.id),
            message,
            contract.severity.as_deref().unwrap_or("medium"),
        );
    }
}

#[derive(Debug)]
struct ReleaseContractFile {
    rel: PathBuf,
    content: String,
}

fn validate_release_contract_shape(contract: &ReleaseContractConfig) -> Result<(), CheckError> {
    if contract.id.trim().is_empty() {
        return invalid_release_contract("release contract id must not be empty");
    }
    if contract.artifacts.is_empty() {
        return invalid_release_contract(&format!(
            "release contract `{}` must declare at least one artifact",
            contract.id
        ));
    }
    if contract.workflow_files.is_empty() {
        return invalid_release_contract(&format!(
            "release contract `{}` must declare at least one workflow file",
            contract.id
        ));
    }
    if contract.docs_files.is_empty() && contract.installer_files.is_empty() {
        return invalid_release_contract(&format!(
            "release contract `{}` must declare at least one docs or installer file",
            contract.id
        ));
    }

    let mut artifacts = HashSet::new();
    for artifact in &contract.artifacts {
        if artifact.name.trim().is_empty() {
            return invalid_release_contract(&format!(
                "release contract `{}` artifact name must not be empty",
                contract.id
            ));
        }
        if !artifacts.insert(artifact.name.as_str()) {
            return invalid_release_contract(&format!(
                "release contract `{}` declares duplicate artifact `{}`",
                contract.id, artifact.name
            ));
        }
    }
    Ok(())
}

fn safe_release_contract_path(
    contract: &ReleaseContractConfig,
    configured_path: &str,
) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return invalid_release_contract(&format!(
            "release contract `{}` path `{configured_path}` must be project-relative and must not use parent traversal",
            contract.id
        ));
    }
    Ok(rel)
}

fn invalid_release_contract<T>(message: &str) -> Result<T, CheckError> {
    Err(CheckError::Config(ConfigError::Invalid(
        message.to_string(),
    )))
}

fn first_file_path(files: &[ReleaseContractFile]) -> PathBuf {
    files
        .first()
        .map(|file| file.rel.clone())
        .unwrap_or_else(|| PathBuf::from(".assura/config.yml"))
}

fn release_asset_tokens(content: &str) -> Vec<String> {
    content
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '+')))
        .map(|token| token.trim_matches('.'))
        .filter(|token| is_release_asset_token(token))
        .map(str::to_string)
        .collect()
}

fn is_release_asset_token(token: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        ".tar.gz",
        ".tar.gz.sha256",
        ".tar.xz",
        ".tar.xz.sha256",
        ".tgz",
        ".tgz.sha256",
        ".zip",
        ".zip.sha256",
    ];
    SUFFIXES.iter().any(|suffix| token.ends_with(suffix))
}

fn url_tokens(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut remaining = content;
    while let Some(index) = next_url_start(remaining) {
        let start_slice = &remaining[index..];
        let end_index = start_slice
            .find(|ch: char| {
                ch.is_whitespace()
                    || matches!(
                        ch,
                        '"' | '\''
                            | '`'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '<'
                            | '>'
                            | ';'
                            | ','
                            | '\\'
                    )
            })
            .unwrap_or(start_slice.len());
        if end_index > 0 {
            urls.push(start_slice[..end_index].to_string());
        }
        remaining = &start_slice[end_index..];
    }
    urls
}

fn next_url_start(content: &str) -> Option<usize> {
    match (content.find("http://"), content.find("https://")) {
        (Some(http), Some(https)) => Some(http.min(https)),
        (Some(index), None) | (None, Some(index)) => Some(index),
        (None, None) => None,
    }
}

fn raw_or_blob_url_branch(url: &str) -> Option<&str> {
    if let Some(raw) = url.strip_prefix("https://raw.githubusercontent.com/") {
        let mut parts = raw.split('/');
        let _owner = parts.next()?;
        let _repo = parts.next()?;
        let branch = parts.next()?;
        return (!branch.is_empty()).then_some(branch);
    }
    github_blob_url_branch(url)
}

fn github_blob_url_branch(url: &str) -> Option<&str> {
    let github = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let mut parts = github.split('/');
    let _owner = parts.next()?;
    let _repo = parts.next()?;
    match parts.next()? {
        "blob" | "raw" => {
            let branch = parts.next()?;
            (!branch.is_empty()).then_some(branch)
        }
        _ => None,
    }
}

fn display_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
