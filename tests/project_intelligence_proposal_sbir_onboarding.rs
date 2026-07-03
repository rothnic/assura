use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

#[test]
fn proposal_sbir_pack_generates_valid_domain_project() {
    let project = TempDir::new().unwrap();
    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--content-template",
        "proposal-sbir",
        "--format",
        "json",
    ]));

    assert_eq!(output["content"]["template"], "proposal-sbir");
    assert_eq!(output["content"]["status"], "active");
    assert!(!output["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "domain_pack"));
    assert!(output["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(|item| item["action"] == "Fill missing proposal evidence"
            && item["affected_paths"][0] == "proposals/evidence/"));
    assert!(output["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(|item| item["action"] == "Resolve proposal review findings"
            && item["affected_paths"][0] == "proposals/review-findings/"));

    for path in [
        ".assura/scripts/proposal-sbir-readiness.sh",
        ".assura/scripts/proposal-sbir-readiness.cmd",
        "proposals/requirements/requirement-sbir-technical-merit.md",
        "proposals/evidence/evidence-sbir-technical-merit.md",
        "proposals/claims/claim-sbir-technical-merit.md",
        "proposals/scorecards/scorecard-sbir-baseline.md",
        "proposals/review-findings/finding-sbir-technical-review.md",
        "proposals/package/package-manifest-sbir-baseline.md",
        "proposals/submission/submission-checklist-sbir-baseline.md",
        "source-documents/manifest.md",
        "docs/final/final-document-project-baseline.md",
    ] {
        assert!(project.path().join(path).is_file(), "missing {path}");
    }

    let config = fs::read_to_string(project.path().join(".assura/config.yml")).unwrap();
    assert!(config.contains("proposal_sbir_traceability"));
    assert!(config.contains("proposal_sbir_readiness"));
    assert!(config.contains("windows_script: \".assura/scripts/proposal-sbir-readiness.cmd\""));
    assert!(config.contains("proposal_scorecards"));
    assert!(config.contains("proposal_package_manifests.final_docs"));

    let agent_next =
        fs::read_to_string(project.path().join(".assura/onboarding/agent-next.md")).unwrap();
    assert!(agent_next.contains("Proposal SBIR Workflow"));
    assert!(agent_next.contains("Scores are planning signals for humans"));

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);
    assert_eq!(check["violations"].as_array().expect("violations").len(), 0);

    let scorecard_search = json_from_success(run_assura(&[
        "content",
        "search",
        "SBIR Baseline Scorecard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(scorecard_search["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .any(|item| item["collection"] == "proposal_scorecards"
            && item["instance_id"] == "scorecard-sbir-baseline"));
}

#[test]
fn proposal_sbir_pack_reports_traceability_and_readiness_gaps() {
    let project = TempDir::new().unwrap();
    json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--content-template",
        "proposal-sbir",
        "--format",
        "json",
    ]));

    let evidence_path = project
        .path()
        .join("proposals/evidence/evidence-sbir-technical-merit.md");
    let evidence = fs::read_to_string(&evidence_path)
        .unwrap()
        .replace("source_documents:\n  - source-documents-manifest\n", "");
    fs::write(&evidence_path, evidence).unwrap();

    let review_path = project
        .path()
        .join("proposals/review-findings/finding-sbir-technical-review.md");
    let review = fs::read_to_string(&review_path)
        .unwrap()
        .replace("status: resolved", "status: open");
    fs::write(&review_path, review).unwrap();

    let package_path = project
        .path()
        .join("proposals/package/package-manifest-sbir-baseline.md");
    let package = fs::read_to_string(&package_path).unwrap().replace(
        "final_package_path: docs/final/final-document-project-baseline.md",
        "final_package_path: docs/final/missing.md",
    );
    fs::write(&package_path, package).unwrap();

    let check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(check.status.code(), Some(1));
    let check_json: Value = serde_json::from_slice(&check.stdout).unwrap();
    let violations = check_json["violations"].as_array().expect("violations");
    assert!(violations.iter().any(|item| {
        item["rule"] == "requirements_traceability:proposal_sbir_traceability"
            && item["path"] == "proposals/evidence/evidence-sbir-technical-merit.md"
    }));
    assert!(violations.iter().any(|item| {
        item["rule"] == "computed_check:proposal_sbir_readiness:review_not_resolved"
            && item["path"] == "proposals/review-findings/finding-sbir-technical-review.md"
    }));
    assert!(violations.iter().any(|item| {
        item["rule"] == "computed_check:proposal_sbir_readiness:package_missing_final_path"
            && item["path"] == "proposals/package/package-manifest-sbir-baseline.md"
    }));

    let next_actions = json_from_success(run_assura(&[
        "content",
        "agent-query",
        "next-actions",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(next_actions["response"]["actions"]
        .as_array()
        .expect("actions")
        .iter()
        .any(|item| item["reason"] == "requirements traceability gaps exist"));
    assert!(next_actions["response"]["actions"]
        .as_array()
        .expect("actions")
        .iter()
        .any(|item| item["reason"] == "computed check findings exist"));
}
