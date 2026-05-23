//! Tests for Assura CLI subprocess discovery helpers.

use super::{depinfo_candidates, latest_sibling_build_modified, primary_assura_binary_path};
use std::fs;
use std::time::Duration;

#[test]
fn depinfo_candidates_include_cargo_output_shapes() {
    let path = std::path::Path::new("target/release/assura-check.exe");
    let candidates = depinfo_candidates(path);

    assert!(candidates
        .iter()
        .any(|candidate| candidate.ends_with("assura-check.d")));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.ends_with("assura-check.exe.d")));
}

#[test]
fn depinfo_mtime_counts_as_latest_build_check() {
    let temp = tempfile::tempdir().unwrap();
    let binary = temp.path().join("assura-check");
    let depinfo = temp.path().join("assura-check.d");

    fs::write(&binary, b"binary").unwrap();
    std::thread::sleep(Duration::from_millis(10));
    fs::write(&depinfo, b"depinfo").unwrap();

    let binary_modified = fs::metadata(&binary).unwrap().modified().unwrap();
    let latest = latest_sibling_build_modified(&binary).unwrap();

    assert!(latest >= fs::metadata(&depinfo).unwrap().modified().unwrap());
    assert!(latest >= binary_modified);
}

#[test]
fn full_companion_maps_to_primary_assura_binary() {
    let current_exe = if cfg!(windows) {
        std::path::Path::new("target/release/assura-full.exe")
    } else {
        std::path::Path::new("target/release/assura-full")
    };

    let primary = primary_assura_binary_path(current_exe);

    assert!(primary.ends_with(if cfg!(windows) {
        "target/release/assura.exe"
    } else {
        "target/release/assura"
    }));
}
