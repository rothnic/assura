use assura::maturity::{CiExecutionState, ProjectObservations};
use std::fs;

#[test]
fn local_files_are_observations_not_quality_proof() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workflow_dir = temp_dir.path().join(".github/workflows");
    fs::create_dir_all(&workflow_dir).unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = 'example'\n",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = 'example'\n",
    )
    .unwrap();

    let observations = ProjectObservations::collect(temp_dir.path()).unwrap();

    assert!(!observations.ci_config_present);
    assert_eq!(
        observations.ci_execution_verified,
        CiExecutionState::Unverified
    );
    assert!(!observations.black_config_present);
    assert_eq!(
        observations.package_manifests,
        vec!["Cargo.toml".to_string(), "pyproject.toml".to_string()]
    );
}

#[test]
fn configured_ci_and_black_remain_distinct_from_execution_evidence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workflow_dir = temp_dir.path().join(".github/workflows");
    fs::create_dir_all(&workflow_dir).unwrap();
    fs::write(workflow_dir.join("ci.yml"), "name: CI\n").unwrap();
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[tool.black]\nline-length = 88\n",
    )
    .unwrap();

    let observations = ProjectObservations::collect(temp_dir.path()).unwrap();

    assert!(observations.ci_config_present);
    assert_eq!(
        observations.ci_execution_verified,
        CiExecutionState::Unverified
    );
    assert!(observations.black_config_present);
    assert!(observations
        .package_manifests
        .contains(&"pyproject.toml".to_string()));
}
