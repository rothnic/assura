//! Source freshness helpers for check-only sibling binaries.

use std::path::Path;

pub(super) fn latest_check_source_modified(binary_name: &str) -> Option<std::time::SystemTime> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (bin_source, extra_bin_sources): (&str, &[&str]) =
        if binary_name.starts_with("assura-check-unix-client") {
            ("crates/assura-check-cli/src/unix_client.rs", &[])
        } else if binary_name.starts_with("assura-check-session") {
            ("crates/assura-check-cli/src/session.rs", &[])
        } else if binary_name.starts_with("assura-check-client") {
            ("crates/assura-check-cli/src/client.rs", &[])
        } else if binary_name.starts_with("assura-check-status") {
            (
                "crates/assura-check-cli/src/status.rs",
                &["crates/assura-check-cli/src/status_file.rs"],
            )
        } else if binary_name.starts_with("assura-check-noop") {
            ("crates/assura-check-cli/src/noop.rs", &[])
        } else if binary_name.starts_with("assura-check-compiled") {
            ("crates/assura-check-cli/src/compiled.rs", &[])
        } else if binary_name.starts_with("assura-check-compile-config") {
            ("crates/assura-check-cli/src/compile_config.rs", &[])
        } else if binary_name.starts_with("assura-checkd") {
            (
                "crates/assura-check-cli/src/server.rs",
                &[
                    "crates/assura-check-cli/src/server_dirty.rs",
                    "crates/assura-check-cli/src/server_io.rs",
                    "crates/assura-check-cli/src/status_file.rs",
                ],
            )
        } else {
            ("crates/assura-check-cli/src/main.rs", &[])
        };

    let mut paths = vec![
        manifest_dir.join("Cargo.toml"),
        manifest_dir.join("src/cli/check.rs"),
        manifest_dir.join("src/cli/check"),
        manifest_dir.join("src/cli/config.rs"),
        manifest_dir.join("src/cli/mod.rs"),
        manifest_dir.join("src/config"),
        manifest_dir.join("src/constraints"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join(bin_source),
    ];
    paths.extend(
        extra_bin_sources
            .iter()
            .map(|source| manifest_dir.join(source)),
    );

    paths
        .iter()
        .filter_map(|path| latest_modified_under(path))
        .max()
}

fn latest_modified_under(path: &Path) -> Option<std::time::SystemTime> {
    let metadata = path.metadata().ok()?;
    if metadata.is_file() {
        return metadata.modified().ok();
    }

    let mut latest = metadata.modified().ok();
    let entries = path.read_dir().ok()?;
    for entry in entries.flatten() {
        if let Some(modified) = latest_modified_under(&entry.path()) {
            latest = Some(
                latest
                    .map(|current| current.max(modified))
                    .unwrap_or(modified),
            );
        }
    }
    latest
}
