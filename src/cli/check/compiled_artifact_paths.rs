impl CompiledStructureConfigArtifact {
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

    pub(super) fn restore_legacy_directory_counts(
        &mut self,
        source_config: Option<&Path>,
    ) -> Result<(), ConfigError> {
        if self.config.is_some() || !self.plan.has_self_directory_counts() {
            return Ok(());
        }

        // Older fast artifacts dropped the authored nodes. Effective scopes lose
        // the distinction between an inherited count and a required local one.
        #[cfg(feature = "yaml-config")]
        if let Some(path) = source_config {
            let bytes = std::fs::read(path).map_err(ConfigError::Io)?;
            if self.source_config_hash != Some(stable_hash(&bytes)) {
                return Err(ConfigError::Invalid(format!(
                    "compiled config is stale for {}; recompile it before checking directory counts",
                    path.display()
                )));
            }
            let content = std::str::from_utf8(&bytes).map_err(|error| {
                ConfigError::Invalid(format!("read {}: {error}", path.display()))
            })?;
            self.config =
                Some(crate::config::loader::ConfigLoader::parse_validated(content)?.into());
            self.plan.has_direct_count_constraints = true;
            return Ok(());
        }
        #[cfg(not(feature = "yaml-config"))]
        let _ = source_config;

        Err(ConfigError::Invalid(
            "compiled config is missing directory count requirements; supply the original config or recompile the artifact".into(),
        ))
    }
}

/// Infer the project root from a source config path stored in an artifact.
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

pub(super) fn path_to_portable(path: PathBuf) -> String {
    let portable = path.to_string_lossy().replace('\\', "/");
    portable
        .strip_prefix("//?/")
        .or_else(|| portable.strip_prefix("//./"))
        .unwrap_or(&portable)
        .to_string()
}

fn portable_path_matches(path: &Path, expected: &str) -> std::io::Result<bool> {
    if path.is_absolute() && path_to_portable(path.to_path_buf()) == expected {
        return Ok(true);
    }

    Ok(path_to_portable(path.canonicalize()?) == expected)
}
