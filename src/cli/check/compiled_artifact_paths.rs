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
