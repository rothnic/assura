//! Executable proof for the artifact modeling options comparison goal.

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const ROOT: &str = "tests/fixtures/artifact_modeling_options";
const CANDIDATES: &[&str] = &[
    "typespec",
    "linkml",
    "cue",
    "json_schema",
    "zod",
    "assura_control",
];

#[test]
fn every_candidate_schema_validates_the_same_passing_fixture() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load(candidate);
        let result = validate_fixture("pass", &schemas);
        assert!(
            result.is_empty(),
            "{candidate} should pass, got findings: {result:?}"
        );
    }
}

#[test]
fn every_candidate_schema_rejects_missing_markdown_frontmatter_field() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load(candidate);
        let result = validate_fixture("fail_missing_field", &schemas);
        assert!(
            result.iter().any(|finding| finding.contains("Goal schema")),
            "{candidate} should reject missing Goal title, got {result:?}"
        );
    }
}

#[test]
fn every_candidate_schema_rejects_bad_json_record_enum() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load(candidate);
        let result = validate_fixture("fail_json_record", &schemas);
        assert!(
            result.iter().any(|finding| finding.contains("Spec schema")),
            "{candidate} should reject invalid Spec status, got {result:?}"
        );
    }
}

#[test]
fn every_candidate_proof_rejects_missing_cross_collection_reference() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load(candidate);
        let result = validate_fixture("fail_bad_reference", &schemas);
        assert!(
            result
                .iter()
                .any(|finding| finding.contains("missing Spec reference")),
            "{candidate} should reject missing referenced Spec, got {result:?}"
        );
    }
}

#[test]
fn generated_or_compiled_artifacts_validate_the_same_fixture_outcomes() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load_generated_or_compiled(candidate);

        let pass = validate_fixture("pass", &schemas);
        assert!(
            pass.is_empty(),
            "{candidate} generated/compiled artifact should pass, got {pass:?}"
        );

        let missing = validate_fixture("fail_missing_field", &schemas);
        assert!(
            missing
                .iter()
                .any(|finding| finding.contains("Goal schema")),
            "{candidate} generated/compiled artifact should reject missing Goal title, got {missing:?}"
        );

        let bad_json = validate_fixture("fail_json_record", &schemas);
        assert!(
            bad_json
                .iter()
                .any(|finding| finding.contains("Spec schema")),
            "{candidate} generated/compiled artifact should reject invalid Spec status, got {bad_json:?}"
        );

        let bad_reference = validate_fixture("fail_bad_reference", &schemas);
        assert!(
            bad_reference
                .iter()
                .any(|finding| finding.contains("missing Spec reference")),
            "{candidate} generated/compiled artifact should reject missing reference, got {bad_reference:?}"
        );
    }
}

#[test]
fn native_runtime_reuses_loaded_schema_artifacts_without_subprocesses() {
    let schemas = CANDIDATES
        .iter()
        .map(|candidate| {
            (
                *candidate,
                CandidateSchemas::load_generated_or_compiled(candidate),
            )
        })
        .collect::<Vec<_>>();

    let started = Instant::now();
    for _ in 0..200 {
        for (candidate, schemas) in &schemas {
            let findings = validate_fixture("pass", schemas);
            assert!(
                findings.is_empty(),
                "{candidate} cached native validation should pass, got {findings:?}"
            );
        }
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "cached native validation loop should stay comfortably below 5s, got {elapsed:?}"
    );
}

#[test]
fn native_safe_update_preserves_identity_and_revalidates_frontmatter() {
    for candidate in CANDIDATES {
        let schemas = CandidateSchemas::load(candidate);
        let temp = tempfile::tempdir().expect("create temporary fixture directory");
        let target = temp.path().join("goal_artifact_models.md");
        fs::copy(
            Path::new(ROOT).join("fixtures/pass/docs/goals/goal_artifact_models.md"),
            &target,
        )
        .expect("copy markdown fixture");

        safe_update_markdown_frontmatter(&target, &schemas.goal, |frontmatter| {
            frontmatter["title"] = Value::String("Updated artifact modeling comparison".into());
        })
        .unwrap_or_else(|error| panic!("{candidate} safe update failed: {error}"));

        let updated = read_markdown_frontmatter(&target);
        assert_eq!(
            updated.get("id").and_then(Value::as_str),
            Some("goal-artifact-models"),
            "{candidate} safe update should preserve artifact id"
        );
        assert_eq!(
            updated.get("title").and_then(Value::as_str),
            Some("Updated artifact modeling comparison"),
            "{candidate} safe update should persist title"
        );

        let rejected = safe_update_markdown_frontmatter(&target, &schemas.goal, |frontmatter| {
            frontmatter["id"] = Value::String("changed-artifact-id".into());
        })
        .expect_err("safe update should reject identity changes");
        assert!(
            rejected.contains("cannot change id"),
            "{candidate} should explain identity rejection, got {rejected}"
        );
    }
}

struct CandidateSchemas {
    goal: Value,
    spec: Value,
    decision: Value,
}

impl CandidateSchemas {
    fn load(candidate: &str) -> Self {
        let path = Path::new(ROOT)
            .join("schemas")
            .join(format!("{candidate}.artifacts.schema.json"));
        let schema = read_json(&path);
        Self {
            goal: schema_def(&schema, "Goal").clone(),
            spec: schema_def(&schema, "Spec").clone(),
            decision: schema_def(&schema, "Decision").clone(),
        }
    }

    fn load_generated_or_compiled(candidate: &str) -> Self {
        match candidate {
            "typespec" => {
                let root = read_yaml(
                    &Path::new(ROOT).join("generated_outputs/typespec_artifacts.schema.yaml"),
                );
                Self::from_defs(&root)
            }
            "linkml" => {
                let root = read_json(
                    &Path::new(ROOT).join("generated_outputs/linkml_artifacts.schema.json"),
                );
                Self::from_defs(&root)
            }
            "cue" => Self {
                goal: read_json(&Path::new(ROOT).join("generated_outputs/cue_goal.schema.json")),
                spec: read_json(&Path::new(ROOT).join("generated_outputs/cue_spec.schema.json")),
                decision: read_json(
                    &Path::new(ROOT).join("generated_outputs/cue_decision.schema.json"),
                ),
            },
            "json_schema" => {
                let root =
                    read_json(&Path::new(ROOT).join("models/json_schema/artifacts.schema.json"));
                Self::from_defs(&root)
            }
            "zod" => {
                let root =
                    read_json(&Path::new(ROOT).join("generated_outputs/zod_artifacts.schema.json"));
                Self {
                    goal: normalize_generated_schema(&root, schema_key(&root, "Goal")),
                    spec: normalize_generated_schema(&root, schema_key(&root, "Spec")),
                    decision: normalize_generated_schema(&root, schema_key(&root, "Decision")),
                }
            }
            "assura_control" => Self::load(candidate),
            other => panic!("unknown generated candidate {other}"),
        }
    }

    fn from_defs(root: &Value) -> Self {
        Self {
            goal: normalize_generated_schema(root, schema_def(root, "Goal")),
            spec: normalize_generated_schema(root, schema_def(root, "Spec")),
            decision: normalize_generated_schema(root, schema_def(root, "Decision")),
        }
    }
}

fn schema_def<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .pointer(&format!("/$defs/{name}"))
        .unwrap_or_else(|| panic!("schema missing $defs/{name}"))
}

fn schema_key<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .get(name)
        .unwrap_or_else(|| panic!("schema missing {name}"))
}

fn normalize_generated_schema(root: &Value, schema: &Value) -> Value {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        if let Some(name) = reference.strip_prefix("#/$defs/") {
            return normalize_generated_schema(root, schema_def(root, name));
        }
    }

    let mut normalized = schema.clone();

    if let Some(constants) = schema
        .get("anyOf")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("const").cloned())
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
    {
        normalized["enum"] = Value::Array(constants);
        normalized["type"] = Value::String("string".to_string());
    }

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        let mut normalized_properties = serde_json::Map::new();
        for (field, field_schema) in properties {
            normalized_properties.insert(
                field.clone(),
                normalize_generated_schema(root, field_schema),
            );
        }
        normalized["properties"] = Value::Object(normalized_properties);
    }

    if let Some(items) = schema.get("items") {
        normalized["items"] = normalize_generated_schema(root, items);
    }

    normalized
}

fn validate_fixture(name: &str, schemas: &CandidateSchemas) -> Vec<String> {
    let root = Path::new(ROOT).join("fixtures").join(name);
    let goal = read_markdown_frontmatter(&root.join("docs/goals/goal_artifact_models.md"));
    let spec = read_json(&root.join("specs/spec_artifact_models.json"));
    let decision = read_yaml(&root.join("decisions/decision_model_source.yaml"));

    let mut findings = Vec::new();
    validate_object("Goal", &schemas.goal, &goal, &mut findings);
    validate_object("Spec", &schemas.spec, &spec, &mut findings);
    validate_object("Decision", &schemas.decision, &decision, &mut findings);
    validate_goal_spec_reference(&goal, &spec, &mut findings);
    findings
}

fn validate_object(label: &str, schema: &Value, value: &Value, findings: &mut Vec<String>) {
    validate_schema_object(label, schema, value, findings);
}

fn validate_schema_object(label: &str, schema: &Value, value: &Value, findings: &mut Vec<String>) {
    let Some(object) = value.as_object() else {
        findings.push(format!("{label} schema violation: value must be object"));
        return;
    };

    for required in schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !object.contains_key(required) {
            findings.push(format!(
                "{label} schema violation: missing required field '{required}'"
            ));
        }
    }

    if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
        let allowed = schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|properties| {
                properties
                    .keys()
                    .map(String::as_str)
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        for field in object.keys() {
            if !allowed.contains(field.as_str()) {
                findings.push(format!(
                    "{label} schema violation: additional field '{field}'"
                ));
            }
        }
    }

    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    for (field, field_schema) in properties {
        if let Some(field_value) = object.get(field) {
            validate_field(label, field, field_schema, field_value, findings);
        }
    }
}

fn validate_field(
    label: &str,
    field: &str,
    schema: &Value,
    value: &Value,
    findings: &mut Vec<String>,
) {
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.iter().any(|allowed| allowed == value) {
            findings.push(format!(
                "{label} schema violation: field '{field}' has disallowed value"
            ));
        }
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("string") => validate_string(label, field, schema, value, findings),
        Some("array") => validate_array(label, field, schema, value, findings),
        Some("object") if !value.is_object() => findings.push(format!(
            "{label} schema violation: field '{field}' must be object"
        )),
        _ => {}
    }

    if schema.get("type").is_none()
        && (schema.get("pattern").is_some() || schema.get("not").is_some())
    {
        validate_string(label, field, schema, value, findings);
    }
}

fn validate_string(
    label: &str,
    field: &str,
    schema: &Value,
    value: &Value,
    findings: &mut Vec<String>,
) {
    let Some(text) = value.as_str() else {
        findings.push(format!(
            "{label} schema violation: field '{field}' must be string"
        ));
        return;
    };
    let min_length = schema.get("minLength").and_then(Value::as_u64).unwrap_or(0);
    if text.len() < min_length as usize {
        findings.push(format!(
            "{label} schema violation: field '{field}' is shorter than {min_length}"
        ));
    }
    if schema
        .pointer("/not/const")
        .and_then(Value::as_str)
        .is_some_and(|disallowed| text == disallowed)
    {
        findings.push(format!(
            "{label} schema violation: field '{field}' has disallowed value"
        ));
    }
    if schema
        .get("pattern")
        .and_then(Value::as_str)
        .is_some_and(|pattern| pattern == "^.+$" && text.is_empty())
    {
        findings.push(format!(
            "{label} schema violation: field '{field}' is shorter than 1"
        ));
    }
}

fn validate_array(
    label: &str,
    field: &str,
    schema: &Value,
    value: &Value,
    findings: &mut Vec<String>,
) {
    let Some(items) = value.as_array() else {
        findings.push(format!(
            "{label} schema violation: field '{field}' must be array"
        ));
        return;
    };

    let min_items = schema
        .get("minItems")
        .or_else(|| schema.get("minLength"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if items.len() < min_items as usize {
        findings.push(format!(
            "{label} schema violation: field '{field}' needs at least {min_items} items"
        ));
    }

    let item_schema = schema.get("items").unwrap_or(&Value::Null);
    for item in items {
        validate_field(label, field, item_schema, item, findings);
    }
}

fn validate_goal_spec_reference(goal: &Value, spec: &Value, findings: &mut Vec<String>) {
    let spec_ids = spec
        .get("id")
        .and_then(Value::as_str)
        .into_iter()
        .collect::<HashSet<_>>();
    let Some(references) = goal.get("specs").and_then(Value::as_array) else {
        return;
    };

    for reference in references {
        let Some(reference) = reference.as_str() else {
            continue;
        };
        if !spec_ids.contains(reference) {
            findings.push(format!("Goal has missing Spec reference '{reference}'"));
        }
    }
}

fn safe_update_markdown_frontmatter(
    path: &Path,
    schema: &Value,
    update: impl FnOnce(&mut Value),
) -> Result<(), String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("failed to read fixture: {error}"))?;
    let (frontmatter, body) = split_frontmatter_parts(&content)
        .ok_or_else(|| format!("missing frontmatter in {}", path.display()))?;
    let mut value = yaml_frontmatter_to_json_result(frontmatter, path)?;
    let original_id = value
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "frontmatter missing id before update".to_string())?
        .to_owned();

    update(&mut value);

    let new_id = value
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "frontmatter missing id after update".to_string())?;
    if new_id != original_id {
        return Err(format!(
            "cannot change id from '{original_id}' to '{new_id}'"
        ));
    }

    let mut findings = Vec::new();
    validate_object("Goal", schema, &value, &mut findings);
    if !findings.is_empty() {
        return Err(format!("updated frontmatter is invalid: {findings:?}"));
    }

    let encoded = serde_yaml::to_string(&value)
        .map_err(|error| format!("failed to encode updated frontmatter: {error}"))?;
    fs::write(path, format!("---\n{encoded}---{body}"))
        .map_err(|error| format!("failed to write updated markdown: {error}"))
}

fn read_json(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    serde_json::from_str(&content).unwrap_or_else(|error| {
        panic!("failed to parse JSON {}: {error}", path.display());
    })
}

fn read_yaml(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&content).unwrap_or_else(|error| {
        panic!("failed to parse YAML {}: {error}", path.display());
    });
    serde_json::to_value(yaml).unwrap_or_else(|error| {
        panic!(
            "failed to normalize YAML {} to JSON: {error}",
            path.display()
        );
    })
}

fn read_markdown_frontmatter(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    let frontmatter = split_frontmatter(&content).unwrap_or_else(|| {
        panic!("missing frontmatter in {}", path.display());
    });
    yaml_frontmatter_to_json(frontmatter, path)
}

fn yaml_frontmatter_to_json(frontmatter: &str, path: &Path) -> Value {
    yaml_frontmatter_to_json_result(frontmatter, path).unwrap_or_else(|error| panic!("{error}"))
}

fn yaml_frontmatter_to_json_result(frontmatter: &str, path: &Path) -> Result<Value, String> {
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(frontmatter)
        .map_err(|error| format!("failed to parse frontmatter {}: {error}", path.display()))?;
    serde_json::to_value(yaml).map_err(|error| {
        format!(
            "failed to normalize frontmatter {} to JSON: {error}",
            path.display()
        )
    })
}

fn split_frontmatter(content: &str) -> Option<&str> {
    split_frontmatter_parts(content).map(|(frontmatter, _)| frontmatter)
}

fn split_frontmatter_parts(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix("---")?;
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let (frontmatter, body) = rest.split_once("\n---")?;
    Some((frontmatter, body))
}
