//! Binary-safe compiled structure config artifacts.

use super::compiled_config::CompiledStructureConfig;
use super::compiled_fingerprint::SourceConfigFingerprint;
use super::compiled_plan_artifact::PortableCompiledPlan;
use crate::config::config::{
    Config, DirectoryBundle, DirectoryNode, ExistsValidation, FileBundle, MarkdownBundle,
};
use crate::config::ls_compat::LsLintCompatibility;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const COMPILED_CONFIG_SCHEMA_VERSION: u32 = 7;
const ASSURA_VERSION_HASH: u64 = stable_hash(env!("CARGO_PKG_VERSION").as_bytes());

/// Portable artifact containing a parsed Assura structure config.
#[derive(Debug, Deserialize, Serialize)]
pub struct CompiledStructureConfigArtifact {
    /// Binary artifact schema version.
    pub schema_version: u32,
    /// Stable hash of the Assura version that produced the artifact.
    assura_version_hash: u64,
    /// Hash of the YAML config bytes that produced this artifact.
    source_config_hash: Option<u64>,
    /// Cheap filesystem fingerprint for the source config at compile time.
    #[serde(default)]
    source_config_fingerprint: Option<SourceConfigFingerprint>,
    /// Canonical project root that produced this artifact.
    #[serde(default)]
    source_project_root: Option<String>,
    /// Canonical source config path that produced this artifact.
    #[serde(default)]
    source_config_path: Option<String>,
    /// Parsed structure config in a binary-safe portable representation.
    config: Option<PortableConfig>,
    /// Normalized validation plan compiled from the parsed config.
    plan: PortableCompiledPlan,
}

impl CompiledStructureConfigArtifact {
    /// Create an artifact for the current Assura binary version.
    pub fn new(config: Config) -> Self {
        let plan = PortableCompiledPlan::from_config(&config);
        let config = if plan.is_fast_only() {
            None
        } else {
            Some(config.into())
        };
        Self {
            schema_version: COMPILED_CONFIG_SCHEMA_VERSION,
            assura_version_hash: ASSURA_VERSION_HASH,
            source_config_hash: None,
            source_config_fingerprint: None,
            source_project_root: None,
            source_config_path: None,
            config,
            plan,
        }
    }

    /// Create an artifact tied to a specific source config file.
    pub fn new_with_source(
        config: Config,
        config_path: &Path,
        source_bytes: &[u8],
    ) -> std::io::Result<Self> {
        let plan = PortableCompiledPlan::from_config(&config);
        let config = if plan.is_fast_only() {
            None
        } else {
            Some(config.into())
        };
        let canonical_config_path = config_path.canonicalize()?;
        let canonical_project_root = infer_project_root(&canonical_config_path)?;
        Ok(Self {
            schema_version: COMPILED_CONFIG_SCHEMA_VERSION,
            assura_version_hash: ASSURA_VERSION_HASH,
            source_config_hash: Some(stable_hash(source_bytes)),
            source_config_fingerprint: SourceConfigFingerprint::from_path(config_path).ok(),
            source_project_root: Some(path_to_portable(canonical_project_root)),
            source_config_path: Some(path_to_portable(canonical_config_path)),
            config,
            plan,
        })
    }

    /// Return true when the artifact can be consumed by this binary.
    pub fn is_compatible(&self) -> bool {
        self.schema_version == COMPILED_CONFIG_SCHEMA_VERSION
            && self.assura_version_hash == ASSURA_VERSION_HASH
    }

    /// Return true when the artifact was compiled from the current config bytes.
    pub fn matches_source_config(&self, config_path: &Path) -> std::io::Result<bool> {
        self.matches_source_config_with_path_requirement(config_path, true)
    }

    /// Return true when the artifact matches the current default source config.
    ///
    /// This is for convention-bound default artifacts after the caller has
    /// already verified the project root. It still requires either a matching
    /// strong source fingerprint or exact source-byte hash, but it does not
    /// reject same-content configs solely because the artifact was produced
    /// through another path to equivalent config bytes.
    pub fn matches_default_source_config(&self, config_path: &Path) -> std::io::Result<bool> {
        self.matches_source_config_with_path_requirement(config_path, false)
    }

    fn matches_source_config_with_path_requirement(
        &self,
        config_path: &Path,
        require_path_match: bool,
    ) -> std::io::Result<bool> {
        let Some(expected_hash) = self.source_config_hash else {
            return Ok(false);
        };
        if require_path_match {
            let Some(expected_path) = &self.source_config_path else {
                return Ok(false);
            };
            if !portable_path_matches(config_path, expected_path)? {
                return Ok(false);
            }
        }
        if self
            .source_config_fingerprint
            .as_ref()
            .is_some_and(|expected| expected.differs_from_path(config_path))
        {
            return Ok(false);
        }
        let source_bytes = std::fs::read(config_path)?;
        Ok(stable_hash(&source_bytes) == expected_hash)
    }

    /// Return true when the artifact was compiled for the current project root.
    pub fn matches_project_root(&self, project_root: &Path) -> std::io::Result<bool> {
        let Some(expected_root) = &self.source_project_root else {
            return Ok(false);
        };
        portable_path_matches(project_root, expected_root)
    }

    /// Convert the artifact back into the runtime structure config.
    pub fn into_config(self) -> Result<Config, crate::cli::config::ConfigError> {
        self.config.map(Into::into).ok_or_else(|| {
            crate::cli::config::ConfigError::Invalid(
                "compiled config artifact does not contain a fallback config".to_string(),
            )
        })
    }

    pub(in crate::cli::check) fn into_fast_compiled_config(
        self,
        fail_fast: bool,
    ) -> Result<CompiledStructureConfig, Box<Self>> {
        if !self.plan.can_run_without_config(fail_fast) {
            return Err(Box::new(self));
        }
        Ok(self.plan.into_compiled_config(Config::new(), fail_fast))
    }

    pub(in crate::cli::check) fn into_compiled_config(
        self,
        fail_fast: bool,
    ) -> Result<CompiledStructureConfig, crate::cli::config::ConfigError> {
        let Some(config) = self.config else {
            return Err(crate::cli::config::ConfigError::Invalid(
                "compiled config artifact does not contain a fallback config".to_string(),
            ));
        };
        Ok(self.plan.into_compiled_config(config.into(), fail_fast))
    }
}

const fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }
    hash
}

fn infer_project_root(config_path: &Path) -> std::io::Result<PathBuf> {
    let config_dir = config_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "config path has no parent directory",
        )
    })?;
    if config_dir.file_name().and_then(|name| name.to_str()) == Some(".assura") {
        return config_dir.parent().map(Path::to_path_buf).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "config path is not inside a project root",
            )
        });
    }
    Ok(config_dir.to_path_buf())
}

fn path_to_portable(path: PathBuf) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn portable_path_matches(path: &Path, expected: &str) -> std::io::Result<bool> {
    if path.is_absolute() && path_to_portable(path.to_path_buf()) == expected {
        return Ok(true);
    }

    Ok(path_to_portable(path.canonicalize()?) == expected)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableConfig {
    patterns: HashMap<String, PortableFileBundle>,
    structure: HashMap<String, PortableDirectoryNode>,
    ls: Option<LsLintCompatibility>,
    exclude: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PortableDirectoryNode {
    files: Option<PortableFileBundle>,
    directories: Option<PortableDirectoryBundle>,
    markdown: Option<PortableMarkdownBundle>,
    exists: Option<PortableExistsValidation>,
    children: Option<HashMap<String, PortableDirectoryNode>>,
    inherit: bool,
    required: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PortableFileBundle {
    naming: Option<String>,
    naming_patterns: Option<HashMap<String, String>>,
    max_lines: Option<usize>,
    max_size: Option<String>,
    require_docs: Option<bool>,
    extensions: Option<Vec<String>>,
    severity: Option<String>,
    required: Option<Vec<String>>,
    allowed_names: Option<Vec<String>>,
    allowed_patterns: Option<Vec<String>>,
    forbidden_patterns: Option<Vec<String>>,
    allow_extra: Option<bool>,
    exists: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PortableDirectoryBundle {
    naming: Option<String>,
    required: Option<Vec<String>>,
    allowed_names: Option<Vec<String>>,
    allowed_patterns: Option<Vec<String>>,
    forbidden_patterns: Option<Vec<String>>,
    allow_extra: Option<bool>,
    severity: Option<String>,
    exists: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PortableMarkdownBundle {
    require_frontmatter: Option<bool>,
    required_fields: Option<Vec<String>>,
    max_heading_depth: Option<u8>,
    check_links: Option<bool>,
    required_sections: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableExistsValidation {
    files: Option<Vec<String>>,
    directories: Option<Vec<String>>,
}

impl From<Config> for PortableConfig {
    fn from(config: Config) -> Self {
        Self {
            patterns: config
                .patterns
                .into_iter()
                .map(|(path, bundle)| (path, bundle.into()))
                .collect(),
            structure: config
                .structure
                .into_iter()
                .map(|(path, node)| (path, node.into()))
                .collect(),
            ls: config.ls,
            exclude: config.exclude,
        }
    }
}

impl From<PortableConfig> for Config {
    fn from(config: PortableConfig) -> Self {
        Self {
            patterns: config
                .patterns
                .into_iter()
                .map(|(path, bundle)| (path, bundle.into()))
                .collect(),
            structure: config
                .structure
                .into_iter()
                .map(|(path, node)| (path, node.into()))
                .collect(),
            ls: config.ls,
            exclude: config.exclude,
        }
    }
}

impl From<DirectoryNode> for PortableDirectoryNode {
    fn from(node: DirectoryNode) -> Self {
        Self {
            files: node.files.map(Into::into),
            directories: node.directories.map(Into::into),
            markdown: node.markdown.map(Into::into),
            exists: node.exists.map(Into::into),
            children: node.children.map(|children| {
                children
                    .into_iter()
                    .map(|(name, child)| (name, child.into()))
                    .collect()
            }),
            inherit: node.inherit,
            required: node.required,
        }
    }
}

impl From<PortableDirectoryNode> for DirectoryNode {
    fn from(node: PortableDirectoryNode) -> Self {
        Self {
            files: node.files.map(Into::into),
            directories: node.directories.map(Into::into),
            markdown: node.markdown.map(Into::into),
            exists: node.exists.map(Into::into),
            children: node.children.map(|children| {
                children
                    .into_iter()
                    .map(|(name, child)| (name, child.into()))
                    .collect()
            }),
            inherit: node.inherit,
            required: node.required,
        }
    }
}

impl From<FileBundle> for PortableFileBundle {
    fn from(bundle: FileBundle) -> Self {
        Self {
            naming: bundle.naming,
            naming_patterns: bundle.naming_patterns,
            max_lines: bundle.max_lines,
            max_size: bundle.max_size,
            require_docs: bundle.require_docs,
            extensions: bundle.extensions,
            severity: bundle.severity,
            required: bundle.required,
            allowed_names: bundle.allowed_names,
            allowed_patterns: bundle.allowed_patterns,
            forbidden_patterns: bundle.forbidden_patterns,
            allow_extra: bundle.allow_extra,
            exists: bundle.exists,
        }
    }
}

impl From<PortableFileBundle> for FileBundle {
    fn from(bundle: PortableFileBundle) -> Self {
        Self {
            naming: bundle.naming,
            naming_patterns: bundle.naming_patterns,
            max_lines: bundle.max_lines,
            max_size: bundle.max_size,
            require_docs: bundle.require_docs,
            extensions: bundle.extensions,
            severity: bundle.severity,
            required: bundle.required,
            allowed_names: bundle.allowed_names,
            allowed_patterns: bundle.allowed_patterns,
            forbidden_patterns: bundle.forbidden_patterns,
            allow_extra: bundle.allow_extra,
            exists: bundle.exists,
        }
    }
}

impl From<DirectoryBundle> for PortableDirectoryBundle {
    fn from(bundle: DirectoryBundle) -> Self {
        Self {
            naming: bundle.naming,
            required: bundle.required,
            allowed_names: bundle.allowed_names,
            allowed_patterns: bundle.allowed_patterns,
            forbidden_patterns: bundle.forbidden_patterns,
            allow_extra: bundle.allow_extra,
            severity: bundle.severity,
            exists: bundle.exists,
        }
    }
}

impl From<PortableDirectoryBundle> for DirectoryBundle {
    fn from(bundle: PortableDirectoryBundle) -> Self {
        Self {
            naming: bundle.naming,
            required: bundle.required,
            allowed_names: bundle.allowed_names,
            allowed_patterns: bundle.allowed_patterns,
            forbidden_patterns: bundle.forbidden_patterns,
            allow_extra: bundle.allow_extra,
            severity: bundle.severity,
            exists: bundle.exists,
        }
    }
}

impl From<MarkdownBundle> for PortableMarkdownBundle {
    fn from(bundle: MarkdownBundle) -> Self {
        Self {
            require_frontmatter: bundle.require_frontmatter,
            required_fields: bundle.required_fields,
            max_heading_depth: bundle.max_heading_depth,
            check_links: bundle.check_links,
            required_sections: bundle.required_sections,
        }
    }
}

impl From<PortableMarkdownBundle> for MarkdownBundle {
    fn from(bundle: PortableMarkdownBundle) -> Self {
        Self {
            require_frontmatter: bundle.require_frontmatter,
            required_fields: bundle.required_fields,
            max_heading_depth: bundle.max_heading_depth,
            check_links: bundle.check_links,
            required_sections: bundle.required_sections,
        }
    }
}

impl From<ExistsValidation> for PortableExistsValidation {
    fn from(exists: ExistsValidation) -> Self {
        Self {
            files: exists.files,
            directories: exists.directories,
        }
    }
}

impl From<PortableExistsValidation> for ExistsValidation {
    fn from(exists: PortableExistsValidation) -> Self {
        Self {
            files: exists.files,
            directories: exists.directories,
        }
    }
}
