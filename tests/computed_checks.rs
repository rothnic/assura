use serde_json::Value;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "command emits JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            stdout(output),
            stderr(output)
        )
    })
}

#[test]
fn computed_check_passes_when_script_emits_no_findings() {
    let project = computed_project("pass.sh", pass_script(), 5_000);
    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(json_output(&output)["success"], true);
}

#[test]
fn computed_check_finding_flows_through_check_report() {
    let project = computed_project("finding.sh", finding_script(), 5_000);
    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let report = json_output(&output);
    let violation = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| {
            item["rule"] == "computed_check:rollup_score:score_low"
                && item["path"] == "docs/source.md"
        });
    let violation = violation.unwrap_or_else(|| panic!("{report:#}"));
    assert_eq!(violation["metadata"]["score"], 42);
    assert_eq!(violation["metadata"]["source"], "fixture");
}

#[test]
fn computed_check_reports_missing_invalid_nonzero_and_timeout_scripts() {
    let missing = computed_project_without_script("missing.sh", 5_000);
    let missing_output = run_assura(&[
        "check",
        missing.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_rule(
        &missing_output,
        "computed_check:rollup_score:script_missing",
    );

    let invalid = computed_project("invalid.sh", invalid_script(), 5_000);
    let invalid_output = run_assura(&[
        "check",
        invalid.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_rule(
        &invalid_output,
        "computed_check:rollup_score:invalid_output",
    );

    let nonzero = computed_project("nonzero.sh", nonzero_script(), 5_000);
    let nonzero_output = run_assura(&[
        "check",
        nonzero.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_rule(&nonzero_output, "computed_check:rollup_score:nonzero_exit");

    let timeout = computed_project("timeout.sh", timeout_script(), 50);
    let timeout_output = run_assura(&[
        "check",
        timeout.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_rule(&timeout_output, "computed_check:rollup_score:timeout");

    let invalid_severity =
        computed_project("invalid-severity.sh", invalid_severity_script(), 5_000);
    let invalid_severity_output = run_assura(&[
        "check",
        invalid_severity.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_rule(
        &invalid_severity_output,
        "computed_check:rollup_score:invalid_output",
    );
}

#[test]
fn computed_check_rejects_unsafe_windows_script_path() {
    let project = computed_project_without_script("missing.sh", 5_000);
    let config_path = project.path().join(".assura/config.yml");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        &config_path,
        config.replace(
            "      windows_script: scripts/missing.cmd\n",
            "      windows_script: ../outside.cmd\n",
        ),
    )
    .unwrap();

    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    assert!(stderr(&output).contains("windows_script"));
}

#[test]
fn computed_check_rejects_script_symlink_that_escapes_project() {
    #[cfg(unix)]
    {
        let project = computed_project_without_script("outside.sh", 5_000);
        let outside = TempDir::new().unwrap();
        fs::create_dir_all(outside.path().join("scripts")).unwrap();
        write_script(outside.path(), "outside.sh", pass_script());
        std::os::unix::fs::symlink(
            outside.path().join("scripts/outside.sh"),
            project.path().join("scripts/outside.sh"),
        )
        .unwrap();

        let output = run_assura(&[
            "check",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ]);
        assert_rule(
            &output,
            "computed_check:rollup_score:script_outside_project",
        );
    }
}

#[test]
fn computed_check_passes_args_literally_and_sends_versioned_stdin() {
    let project = computed_project_with_args(
        "contract.sh",
        contract_script(),
        5_000,
        &["literal;echo-no-shell", "two words"],
    );
    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
}

#[test]
fn computed_checks_are_visible_to_doctor_and_agent_query() {
    let project = computed_project("finding.sh", finding_script(), 5_000);
    let path = project.path().to_str().unwrap();

    let diagnostics = run_assura(&[
        "content",
        "agent-query",
        "diagnostics",
        path,
        "--format",
        "json",
    ]);
    assert!(
        diagnostics.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&diagnostics),
        stderr(&diagnostics)
    );
    let diagnostics_json = json_output(&diagnostics);
    assert!(diagnostics_json["response"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["rule"] == "computed_check:rollup_score:score_low"
                && item["metadata"]["score"] == 42
                && item["metadata"]["source"] == "fixture"
        }));

    let gaps = run_assura(&["content", "agent-query", "gaps", path, "--format", "json"]);
    let gaps_json = json_output(&gaps);
    assert_eq!(gaps_json["response"]["computed_checks"], 1);

    let doctor_project = computed_project_without_script("missing.sh", 5_000);
    let doctor = run_assura(&[
        "doctor",
        doctor_project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(doctor.status.code(), Some(1));
    let doctor_json = json_output(&doctor);
    assert!(doctor_json["gaps"].as_array().unwrap().iter().any(|gap| {
        gap["name"] == "computed_check_script_missing:rollup_score" && gap["status"] == "gap"
    }));
}

fn assert_rule(output: &Output, rule: &str) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
    let report = json_output(output);
    assert!(
        report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["rule"] == rule),
        "{report:#}"
    );
}

fn computed_project(script: &str, script_body: &str, timeout_ms: u64) -> TempDir {
    computed_project_with_args(script, script_body, timeout_ms, &[])
}

fn computed_project_with_args(
    script: &str,
    script_body: &str,
    timeout_ms: u64,
    args: &[&str],
) -> TempDir {
    let project = computed_project_without_script(script, timeout_ms);
    let args_yaml = if args.is_empty() {
        String::new()
    } else {
        format!(
            "      args:\n{}",
            args.iter()
                .map(|arg| format!("        - {arg:?}\n"))
                .collect::<String>()
        )
    };
    let config_path = project.path().join(".assura/config.yml");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        &config_path,
        config.replace(
            &format!("      timeout_ms: {timeout_ms}\n"),
            &format!("{args_yaml}      timeout_ms: {timeout_ms}\n"),
        ),
    )
    .unwrap();
    write_script(project.path(), script, script_body);
    project
}

fn computed_project_without_script(script: &str, timeout_ms: u64) -> TempDir {
    let project = TempDir::new().unwrap();
    for dir in [".assura", "scripts", "docs", "schemas", "records"] {
        fs::create_dir_all(project.path().join(dir)).unwrap();
    }
    let windows_script = windows_script_name(script);
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"extensions:
  computed_checks:
    - id: rollup_score
      severity: high
      script: scripts/{script}
      windows_script: scripts/{windows_script}
      timeout_ms: {timeout_ms}
models:
  validation_artifact: schemas/content.schema.json
collections:
  docs:
    class: Doc
    path: records/doc.json
    adapter: json_record
    id: id
structure:
  ./:
    required: false
exclude:
  - ".assura/**"
"#
        ),
    )
    .unwrap();
    fs::write(project.path().join("docs/source.md"), "# Source\n").unwrap();
    fs::write(
        project.path().join("records/doc.json"),
        r#"{"id":"doc-source","title":"Source"}"#,
    )
    .unwrap();
    fs::write(project.path().join("schemas/content.schema.json"), schema()).unwrap();
    project
}

fn windows_script_name(script: &str) -> String {
    Path::new(script)
        .with_extension("cmd")
        .to_string_lossy()
        .replace('\\', "/")
}

fn write_script(project_root: &Path, name: &str, body: &str) {
    let path = project_root.join("scripts").join(name);
    fs::write(&path, body).unwrap();
    fs::write(
        project_root.join("scripts").join(windows_script_name(name)),
        windows_script_body(name),
    )
    .unwrap();
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
}

fn windows_script_body(name: &str) -> &'static str {
    match name {
        "pass.sh" => pass_cmd_script(),
        "finding.sh" => finding_cmd_script(),
        "invalid.sh" => invalid_cmd_script(),
        "nonzero.sh" => nonzero_cmd_script(),
        "timeout.sh" => timeout_cmd_script(),
        "invalid-severity.sh" => invalid_severity_cmd_script(),
        "contract.sh" => contract_cmd_script(),
        _ => pass_cmd_script(),
    }
}

fn pass_script() -> &'static str {
    "#!/bin/sh\nread _request || true\nprintf '%s\\n' '{\"schema\":\"assura.computed-check.output.v1\",\"findings\":[]}'\n"
}

fn finding_script() -> &'static str {
    "#!/bin/sh\nread _request || true\nprintf '%s\\n' '{\"schema\":\"assura.computed-check.output.v1\",\"findings\":[{\"code\":\"score_low\",\"message\":\"Rollup score is below threshold\",\"path\":\"docs/source.md\",\"severity\":\"high\",\"metadata\":{\"score\":42,\"source\":\"fixture\"}}]}'\n"
}

fn invalid_script() -> &'static str {
    "#!/bin/sh\nread _request || true\nprintf 'not-json\\n'\n"
}

fn nonzero_script() -> &'static str {
    "#!/bin/sh\nread _request || true\necho boom >&2\nexit 7\n"
}

fn timeout_script() -> &'static str {
    "#!/bin/sh\nread _request || true\nsleep 2\nprintf '%s\\n' '{\"schema\":\"assura.computed-check.output.v1\",\"findings\":[]}'\n"
}

fn invalid_severity_script() -> &'static str {
    "#!/bin/sh\nread _request || true\nprintf '%s\\n' '{\"schema\":\"assura.computed-check.output.v1\",\"findings\":[{\"code\":\"bad_severity\",\"message\":\"Bad severity\",\"severity\":\"urgent\"}]}'\n"
}

fn contract_script() -> &'static str {
    r#"#!/bin/sh
read request || exit 9
case "$request" in
  *'"schema":"assura.computed-check.input.v1"'*) ;;
  *) echo "bad schema request: $request" >&2; exit 10 ;;
esac
case "$request" in
  *'"id":"rollup_score"'*) ;;
  *) echo "bad id request: $request" >&2; exit 10 ;;
esac
case "$request" in
  *'"checked_path":'*'"config_path":'*) ;;
  *) echo "bad path request: $request" >&2; exit 10 ;;
esac
if [ "$#" -ne 2 ] || [ "$1" != "literal;echo-no-shell" ] || [ "$2" != "two words" ]; then
  echo "bad args: $*" >&2
  exit 11
fi
printf '%s\n' '{"schema":"assura.computed-check.output.v1","findings":[]}'
"#
}

fn pass_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo {"schema":"assura.computed-check.output.v1","findings":[]}
"#
}

fn finding_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo {"schema":"assura.computed-check.output.v1","findings":[{"code":"score_low","message":"Rollup score is below threshold","path":"docs/source.md","severity":"high","metadata":{"score":42,"source":"fixture"}}]}
"#
}

fn invalid_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo not-json
"#
}

fn nonzero_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo boom 1>&2
exit /b 7
"#
}

fn timeout_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
ping -n 3 127.0.0.1 >nul
echo {"schema":"assura.computed-check.output.v1","findings":[]}
"#
}

fn invalid_severity_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo {"schema":"assura.computed-check.output.v1","findings":[{"code":"bad_severity","message":"Bad severity","severity":"urgent"}]}
"#
}

fn contract_cmd_script() -> &'static str {
    r#"@echo off
set /p request=
echo %request% | findstr /C:"\"schema\":\"assura.computed-check.input.v1\"" >nul || exit /b 10
echo %request% | findstr /C:"\"id\":\"rollup_score\"" >nul || exit /b 10
echo %request% | findstr /C:"\"checked_path\":" >nul || exit /b 10
echo %request% | findstr /C:"\"config_path\":" >nul || exit /b 10
if not "%~1"=="literal;echo-no-shell" exit /b 11
if not "%~2"=="two words" exit /b 11
echo {"schema":"assura.computed-check.output.v1","findings":[]}
"#
}

fn schema() -> &'static str {
    r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Doc": {
      "type": "object",
      "required": ["id", "title"],
      "properties": {
        "id": { "type": "string" },
        "title": { "type": "string" }
      },
      "additionalProperties": true
    }
  }
}
"##
}
