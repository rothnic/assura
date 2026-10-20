//! Regression tests for performance-report fixture enumeration and metadata.

use super::counterexample_fixtures::create_many_configured_scopes_regression_project;
use super::external_fixture_catalog::external_fixture_specs;
use super::fixtures::{materialize_fixture, scenarios};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[test]
fn default_scenarios_include_rich_monorepo_policy_without_external_repos() {
    let scenarios = scenarios(false);
    assert!(scenarios
        .iter()
        .any(|scenario| scenario.id == "monorepo_policy"));
    assert!(!scenarios.iter().any(|scenario| matches!(
        scenario.kind,
        kind if kind.is_external_pinned()
    )));

    let fixture = materialize_fixture(
        scenarios
            .into_iter()
            .find(|scenario| scenario.id == "monorepo_policy")
            .unwrap(),
    )
    .unwrap();

    assert_eq!(fixture.metadata.source_type, "generated");
    assert_eq!(fixture.metadata.source_revision, "generated-fixtures-v2");
    assert_eq!(fixture.metadata.cohort, "realistic-equivalent");
    assert_eq!(fixture.metadata.rule_count, 38);
    assert!(fixture.metadata.checked_file_count > 30);
    assert!(fixture.metadata.ignored_file_count > 0);
    assert_eq!(
        fixture.metadata.config_generation_method,
        "hand-authored-equivalent-pair"
    );
    let ls_lint_config = fs::read_to_string(fixture.root.join(".ls-lint.yml")).unwrap();
    assert!(ls_lint_config.contains("regex:^$"));
    assert!(ls_lint_config
        .contains(r".dir: regex:^(\.github|apps|packages|docs|scripts|infra|config|tests|node_modules|dist|coverage)$"));
    assert!(ls_lint_config.contains(
        r".*: regex:^(README|AGENTS|CONTRIBUTING|LICENSE|package|pnpm-lock|tsconfig|turbo)$"
    ));
    let _ = fs::remove_dir_all(fixture.root);
}

#[test]
fn external_scenarios_are_opt_in_and_pinned() {
    let scenarios = scenarios(true);
    let external = scenarios
        .iter()
        .filter(|scenario| scenario.kind.is_external_pinned())
        .collect::<Vec<_>>();
    let specs = external_fixture_specs();
    let spec_ids = specs
        .iter()
        .map(|spec| spec.fixture_id)
        .collect::<HashSet<_>>();
    let spec_revisions = specs
        .iter()
        .map(|spec| (spec.fixture_id, spec.revision))
        .collect::<std::collections::HashMap<_, _>>();
    let config_bodies = specs
        .iter()
        .map(|spec| spec.ls_lint_config)
        .collect::<HashSet<_>>();

    assert_eq!(external.len(), 10);
    assert_eq!(specs.len(), 10);
    assert_eq!(
        config_bodies.len(),
        10,
        "each real repo needs a unique policy"
    );

    for scenario in external {
        assert!(
            spec_ids.contains(scenario.id),
            "missing external spec for {}",
            scenario.id
        );
        assert_eq!(
            Some(&scenario.source_revision),
            spec_revisions.get(scenario.id),
            "scenario revision must match external spec for {}",
            scenario.id
        );
    }
}

#[test]
#[ignore = "manual performance fixture export; run only in the isolated NU11 workflow"]
fn export_nu11_accepted_many_scope_fixture() {
    let source_commit = std::env::var("ASSURA_NU11_COMMIT")
        .expect("ASSURA_NU11_COMMIT must bind the exporter to the workflow source");
    assert!(
        source_commit.len() == 40
            && source_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "ASSURA_NU11_COMMIT must be a lowercase full commit SHA"
    );

    let output = PathBuf::from(
        std::env::var_os("ASSURA_NU11_FIXTURE_ROOT")
            .expect("ASSURA_NU11_FIXTURE_ROOT must be an explicit absolute path"),
    );
    assert!(output.is_absolute(), "fixture root must be absolute");
    assert!(
        !output.exists(),
        "refusing to overwrite fixture root {}",
        output.display()
    );
    fs::create_dir(&output).expect("create fresh fixture root");

    create_many_configured_scopes_regression_project(&output)
        .unwrap_or_else(|error| panic!("generate accepted many-scope fixture: {error}"));

    let ls_path = output.join(".ls-lint.yml");
    let assura_path = output.join(".assura/config.yml");
    let ls_config: serde_yaml::Value =
        serde_yaml::from_slice(&fs::read(&ls_path).expect("read generated LS-Lint config"))
            .expect("parse generated LS-Lint config");
    let ls_rules = ls_config
        .get("ls")
        .and_then(serde_yaml::Value::as_mapping)
        .expect("generated LS-Lint config must contain an ls mapping");
    assert_eq!(
        ls_rules.len(),
        801,
        "fixture must contain 801 LS-Lint rules"
    );

    let assura_config: serde_yaml::Value =
        serde_yaml::from_slice(&fs::read(&assura_path).expect("read generated Assura config"))
            .expect("parse generated Assura config");
    let policy = assura_config
        .get("structure")
        .and_then(serde_yaml::Value::as_mapping)
        .expect("generated Assura config must contain a structure policy");
    assert!(
        !policy.is_empty(),
        "generated Assura structure policy is empty"
    );

    let mut package_directories = 0usize;
    let mut package_files = 0usize;
    for index in 0..800 {
        let package = output.join(format!("pkg-{index:04}"));
        assert!(
            package.is_dir(),
            "missing package directory {}",
            package.display()
        );
        assert!(package.join("file-name.js").is_file());
        package_directories += 1;
        package_files += 1;
    }
    assert_eq!(package_directories, 800);
    assert_eq!(package_files, 800);

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let constructor_source =
        project_root.join("src/cli/performance_report/counterexample_fixtures.rs");
    let exporter_source = project_root.join("src/cli/performance_report/fixture_tests.rs");
    let source_identity = serde_json::json!({
        "commit": source_commit,
        "constructor": {
            "path": "src/cli/performance_report/counterexample_fixtures.rs",
            "sha256": sha256_file(&constructor_source),
        },
        "exporter": {
            "path": "src/cli/performance_report/fixture_tests.rs",
            "sha256": sha256_file(&exporter_source),
        },
    });
    let files = fixture_file_manifest(&output);
    let manifest = serde_json::json!({
        "schema": "assura.nu11.fixture.v1",
        "source": source_identity,
        "root": output,
        "package_directory_count": package_directories,
        "package_file_count": package_files,
        "rule_entry_count": ls_rules.len(),
        "assura_policy_nonempty": !policy.is_empty(),
        "configs": {
            "ls_lint": {"path": ".ls-lint.yml", "sha256": sha256_file(&ls_path)},
            "assura": {"path": ".assura/config.yml", "sha256": sha256_file(&assura_path)},
        },
        "files": files,
    });
    let manifest_path = output.join("nu11-fixture-manifest.json");
    let mut manifest_file = File::create(&manifest_path).expect("create fixture manifest");
    serde_json::to_writer_pretty(&mut manifest_file, &manifest).expect("write fixture manifest");
    manifest_file
        .write_all(b"\n")
        .expect("finish fixture manifest");
    println!(
        "NU11_FIXTURE_EXPORT {}",
        serde_json::to_string(&manifest).unwrap()
    );
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture_file_manifest(root: &Path) -> Vec<serde_json::Value> {
    fn visit(root: &Path, current: &Path, files: &mut Vec<serde_json::Value>) {
        let mut entries = fs::read_dir(current)
            .unwrap_or_else(|error| panic!("read {}: {error}", current.display()))
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            let metadata = fs::symlink_metadata(&path).expect("inspect fixture entry");
            assert!(
                !metadata.file_type().is_symlink(),
                "fixture contains a symlink"
            );
            if metadata.is_dir() {
                visit(root, &path, files);
            } else {
                assert!(metadata.is_file(), "fixture contains a non-regular file");
                let relative = path.strip_prefix(root).expect("relative fixture path");
                files.push(serde_json::json!({
                    "path": relative.to_string_lossy().replace('\\', "/"),
                    "sha256": sha256_file(&path),
                    "size_bytes": metadata.len(),
                }));
            }
        }
    }

    let mut files = Vec::new();
    visit(root, root, &mut files);
    files
}
