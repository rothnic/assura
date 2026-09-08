//! Release-readiness command and report construction.

use serde_json::Value;

use crate::{latest_github_release, read, release_notes_version, release_surfaces_report, Result};

/// Run the release-readiness command with its existing argument contract.
pub(crate) fn run(args: &[String]) -> Result<()> {
    let mut format = "text";
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --format".into());
                };
                format = value;
                index += 2;
            }
            other => return Err(format!("unknown release-readiness option: {other}").into()),
        }
    }
    if !matches!(format, "json" | "text") {
        return Err(format!("unsupported release-readiness format: {format}").into());
    }

    let report = report();
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let verdict = report
            .get("verdict")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("Release readiness: {verdict}");
        if let Some(reasons) = report.get("reasons").and_then(Value::as_array) {
            for reason in reasons {
                if let Some(reason) = reason.as_str() {
                    println!("- {reason}");
                }
            }
        }
    }

    if report.get("ready").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err("release readiness failed".into())
    }
}

fn report() -> Value {
    let local_version =
        crate::toml_string_value(&read("Cargo.toml"), "version").unwrap_or_default();
    let release_notes_text = read("docs/release-notes.md");
    let release_notes_version = release_notes_version(&release_notes_text).unwrap_or_default();
    report_from_inputs(
        &local_version,
        &release_notes_version,
        &read("docs/support-policy.md"),
        &read("docs/release-candidate-checklist.md"),
        &read("docs/compatibility-and-surface.md"),
        release_surfaces_report(
            "docs/data/release-surfaces.json",
            Some(&format!("v{local_version}")),
        ),
        latest_github_release(),
    )
}

/// Build the stable release-readiness report from captured inputs.
fn report_from_inputs(
    local_version: &str,
    release_notes_version: &str,
    support_policy_text: &str,
    release_checklist_text: &str,
    compatibility_text: &str,
    release_surfaces: Value,
    latest_release: Value,
) -> Value {
    let local_tag = format!("v{local_version}");
    let mut reasons = Vec::new();
    let mut missing_checklist_items = Vec::new();
    for required in [
        "cargo fmt --all -- --check",
        "cargo test --all-targets --quiet",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo xtask website-demo-data --check --released",
        "cargo xtask release-readiness --format json",
        "cargo xtask release-smoke",
        "cargo xtask release-live",
    ] {
        if !release_checklist_text.contains(required) {
            missing_checklist_items.push(required.to_string());
        }
    }
    if !missing_checklist_items.is_empty() {
        reasons.push("release checklist is missing required gates".to_string());
    }

    if release_notes_version != local_version {
        reasons.push(format!(
            "release notes version {release_notes_version:?} does not match Cargo.toml version {local_version:?}"
        ));
    }
    if !support_policy_text.contains("A release PR cannot close if") {
        reasons.push("support policy is missing release PR blocking criteria".to_string());
    }
    if !compatibility_text.contains("Compatibility And Public Surface") {
        reasons.push("compatibility matrix is missing release surface source of truth".to_string());
    }

    let unreleased_user_facing_changes = release_surfaces
        .get("unreleased_user_facing_changes")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    if let Some(error) = release_surfaces.get("error").and_then(Value::as_str) {
        reasons.push(format!(
            "release surface manifest could not be checked: {error}"
        ));
    }
    if let Some(error) = latest_release.get("error").and_then(Value::as_str) {
        reasons.push(format!(
            "latest GitHub release could not be checked: {error}"
        ));
    }
    if unreleased_user_facing_changes
        .as_array()
        .map(|changes| !changes.is_empty())
        .unwrap_or(false)
    {
        reasons.push(format!(
            "{local_tag} cannot publish while supported or experimental user-facing surfaces remain unreleased"
        ));
    }

    let ready = reasons.is_empty();
    serde_json::json!({
        "schema_version": "assura.release-readiness.v1",
        "latest_github_release": latest_release,
        "local_package_version": local_version,
        "local_tag": local_tag,
        "release_notes_version": release_notes_version,
        "unreleased_user_facing_changes": unreleased_user_facing_changes,
        "release_surfaces": release_surfaces,
        "missing_checklist_items": missing_checklist_items,
        "ready": ready,
        "verdict": if ready { "pass" } else { "fail" },
        "reasons": reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn release_readiness_report_passes_for_next_release_candidate() {
        let report = report_from_inputs(
            "0.2.0",
            "0.2.0",
            "A release PR cannot close if",
            release_checklist_fixture(),
            "Compatibility And Public Surface",
            json!({
                "schema_version": "assura.release-surfaces.v1",
                "path": "docs/data/release-surfaces.json",
                "surface_count": 2,
                "unreleased_user_facing_changes": []
            }),
            json!({ "tagName": "v0.1.0" }),
        );
        assert_eq!(
            report.get("schema_version").and_then(Value::as_str),
            Some("assura.release-readiness.v1")
        );
        assert_eq!(report.get("ready").and_then(Value::as_bool), Some(true));
        assert_eq!(report.get("verdict").and_then(Value::as_str), Some("pass"));
        assert!(report
            .get("reasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| reasons.is_empty()));
    }

    #[test]
    fn release_readiness_report_fails_for_any_candidate_with_unreleased_surfaces() {
        let report = report_from_inputs(
            "0.2.0",
            "0.2.0",
            "A release PR cannot close if",
            release_checklist_fixture(),
            "Compatibility And Public Surface",
            json!({
                "schema_version": "assura.release-surfaces.v1",
                "path": "docs/data/release-surfaces.json",
                "surface_count": 1,
                "unreleased_user_facing_changes": [{
                    "id": "project-intelligence-local-surfaces",
                    "status": "supported",
                    "first_release": "unreleased",
                    "detail_path": "docs/release-notes.md"
                }]
            }),
            json!({ "tagName": "v0.1.0" }),
        );
        assert_eq!(report.get("ready").and_then(Value::as_bool), Some(false));
        assert_eq!(report.get("verdict").and_then(Value::as_str), Some("fail"));
        assert!(report
            .get("reasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| reasons.iter().any(|reason| reason
                .as_str()
                .is_some_and(|reason| reason.contains("cannot publish")))));
    }

    #[test]
    fn release_readiness_module_preserves_the_report_schema_for_fixed_inputs() {
        let report = report_from_inputs(
            "0.4.0-rc.1",
            "0.4.0-rc.1",
            "A release PR cannot close if",
            release_checklist_fixture(),
            "Compatibility And Public Surface",
            json!({
                "schema_version": "assura.release-surfaces.v1",
                "path": "docs/data/release-surfaces.json",
                "surface_count": 2,
                "unreleased_user_facing_changes": []
            }),
            json!({ "tagName": "v0.3.0", "isPrerelease": false }),
        );

        // A complete independent value catches dropped/renamed fields and
        // changes to captured inputs, not just agreement on the schema label.
        assert_eq!(
            report,
            json!({
                "schema_version": "assura.release-readiness.v1",
                "latest_github_release": { "tagName": "v0.3.0", "isPrerelease": false },
                "local_package_version": "0.4.0-rc.1",
                "local_tag": "v0.4.0-rc.1",
                "release_notes_version": "0.4.0-rc.1",
                "unreleased_user_facing_changes": [],
                "release_surfaces": {
                    "schema_version": "assura.release-surfaces.v1",
                    "path": "docs/data/release-surfaces.json",
                    "surface_count": 2,
                    "unreleased_user_facing_changes": []
                },
                "missing_checklist_items": [],
                "ready": true,
                "verdict": "pass",
                "reasons": []
            })
        );
    }

    #[test]
    fn release_readiness_rejects_invalid_arguments_before_loading_report() {
        for (args, expected) in [
            (vec!["--format"], "missing value for --format"),
            (
                vec!["--unknown"],
                "unknown release-readiness option: --unknown",
            ),
            (
                vec!["--format", "yaml"],
                "unsupported release-readiness format: yaml",
            ),
            (
                vec!["--format", "json", "--unknown"],
                "unknown release-readiness option: --unknown",
            ),
        ] {
            let args = args.into_iter().map(String::from).collect::<Vec<_>>();
            assert_eq!(run(&args).unwrap_err().to_string(), expected, "{args:?}");
        }
    }

    #[test]
    fn release_readiness_report_rejects_missing_release_documents() {
        let report = report_from_inputs(
            "0.2.0",
            "",
            "",
            "",
            "",
            json!({ "unreleased_user_facing_changes": [] }),
            json!({ "tagName": "v0.1.0" }),
        );

        assert_eq!(report["ready"], false);
        assert_eq!(report["verdict"], "fail");
        assert_eq!(
            report["missing_checklist_items"],
            json!([
                "cargo fmt --all -- --check",
                "cargo test --all-targets --quiet",
                "cargo clippy --all-targets --all-features -- -D warnings",
                "cargo xtask website-demo-data --check --released",
                "cargo xtask release-readiness --format json",
                "cargo xtask release-smoke",
                "cargo xtask release-live"
            ])
        );
        assert_eq!(
            report["reasons"],
            json!([
                "release checklist is missing required gates",
                "release notes version \"\" does not match Cargo.toml version \"0.2.0\"",
                "support policy is missing release PR blocking criteria",
                "compatibility matrix is missing release surface source of truth"
            ])
        );
    }

    #[test]
    fn release_readiness_report_rejects_unavailable_external_inputs() {
        for (surfaces, latest_release, expected) in [
            (
                json!({ "error": "fixture manifest is missing" }),
                json!({ "tagName": "v0.1.0" }),
                "release surface manifest could not be checked: fixture manifest is missing",
            ),
            (
                json!({ "unreleased_user_facing_changes": [] }),
                json!({ "error": "fixture release query is unavailable" }),
                "latest GitHub release could not be checked: fixture release query is unavailable",
            ),
        ] {
            let report = report_from_inputs(
                "0.2.0",
                "0.2.0",
                "A release PR cannot close if",
                release_checklist_fixture(),
                "Compatibility And Public Surface",
                surfaces,
                latest_release,
            );
            assert_eq!(report["ready"], false);
            assert_eq!(report["verdict"], "fail");
            assert_eq!(report["reasons"], json!([expected]));
        }
    }

    fn release_checklist_fixture() -> &'static str {
        "cargo fmt --all -- --check\n\
         cargo test --all-targets --quiet\n\
         cargo clippy --all-targets --all-features -- -D warnings\n\
         cargo xtask website-demo-data --check --released\n\
         cargo xtask release-readiness --format json\n\
         cargo xtask release-smoke\n\
         cargo xtask release-live"
    }
}
