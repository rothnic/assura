//! Atomic file I/O helpers for content repository mutations.

use super::model::ContentFinding;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(super) fn read_record(root: &Path, rel_path: &Path) -> Result<String, Box<ContentFinding>> {
    fs::read_to_string(root.join(rel_path)).map_err(|error| {
        Box::new(ContentFinding::new(
            "read_error",
            Some(rel_path.to_path_buf()),
            format!("Failed to read '{}': {error}", rel_path.display()),
        ))
    })
}

pub(super) fn replace_or_write_record(
    root: &Path,
    rel_path: &Path,
    content: &str,
) -> Result<(), Box<ContentFinding>> {
    if root.join(rel_path).exists() {
        replace_record(root, rel_path, content)
    } else {
        write_record(root, rel_path, content)
    }
}

pub(super) fn write_record(
    root: &Path,
    rel_path: &Path,
    content: &str,
) -> Result<(), Box<ContentFinding>> {
    let destination = root.join(rel_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            Box::new(ContentFinding::new(
                "write_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Failed to create parent directory for '{}': {error}",
                    rel_path.display()
                ),
            ))
        })?;
    }

    let parent = destination.parent().unwrap_or(root);
    let temp_path = write_temp_file(parent, rel_path, content, "assura-create")?;
    if let Err(error) = fs::hard_link(&temp_path, &destination) {
        let _ = fs::remove_file(&temp_path);
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            return Err(Box::new(ContentFinding::new(
                "content_create_path_exists",
                Some(rel_path.to_path_buf()),
                format!("Destination '{}' already exists", rel_path.display()),
            )));
        }
        return Err(Box::new(ContentFinding::new(
            "write_error",
            Some(rel_path.to_path_buf()),
            format!("Failed to finalize '{}': {error}", rel_path.display()),
        )));
    }
    let _ = fs::remove_file(&temp_path);
    Ok(())
}

pub(super) fn replace_record(
    root: &Path,
    rel_path: &Path,
    content: &str,
) -> Result<(), Box<ContentFinding>> {
    let destination = root.join(rel_path);
    let Some(parent) = destination.parent() else {
        return Err(Box::new(ContentFinding::new(
            "write_error",
            Some(rel_path.to_path_buf()),
            format!(
                "Failed to resolve parent directory for '{}'",
                rel_path.display()
            ),
        )));
    };
    let temp_path = write_temp_file(parent, rel_path, content, "assura-update")?;
    if let Err(error) = fs::rename(&temp_path, &destination) {
        let _ = fs::remove_file(&temp_path);
        return Err(Box::new(ContentFinding::new(
            "write_error",
            Some(rel_path.to_path_buf()),
            format!("Failed to replace '{}': {error}", rel_path.display()),
        )));
    }
    Ok(())
}

fn write_temp_file(
    parent: &Path,
    rel_path: &Path,
    content: &str,
    label: &str,
) -> Result<PathBuf, Box<ContentFinding>> {
    let file_name = rel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("record");
    let mut last_error = None;
    for attempt in 0..1000 {
        let temp_path = parent.join(format!(
            ".{file_name}.{label}-{}-{attempt}.tmp",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(content.as_bytes())
                    .and_then(|_| file.sync_all())
                {
                    let _ = fs::remove_file(&temp_path);
                    return Err(Box::new(ContentFinding::new(
                        "write_error",
                        Some(rel_path.to_path_buf()),
                        format!("Failed to write '{}': {error}", rel_path.display()),
                    )));
                }
                return Ok(temp_path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
            }
            Err(error) => {
                return Err(Box::new(ContentFinding::new(
                    "write_error",
                    Some(rel_path.to_path_buf()),
                    format!(
                        "Failed to create temporary file for '{}': {error}",
                        rel_path.display()
                    ),
                )));
            }
        }
    }

    Err(Box::new(ContentFinding::new(
        "write_error",
        Some(rel_path.to_path_buf()),
        format!(
            "Failed to create temporary file for '{}': {}",
            rel_path.display(),
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "temporary name exhausted".to_string())
        ),
    )))
}
