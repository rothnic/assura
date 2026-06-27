use jsonschema::Validator;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const FIXTURE_ROOT: &str = "tests/fixtures/artifact_modeling_options/authoring_paths";

#[test]
fn linkml_profile_and_typespec_decorators_share_runtime_contract() {
    let linkml = load_schema("linkml_profile");
    let typespec = load_schema("typespec_decorators");

    assert_eq!(
        runtime_contract_without_source(&linkml),
        runtime_contract_without_source(&typespec),
        "authoring paths should produce the same normalized runtime contract"
    );
    assert!(linkml["x-assura-source"] != typespec["x-assura-source"]);
}

#[test]
fn large_model_validates_markdown_json_yaml_jsonl_and_references() {
    let validators = RuntimeValidators::from_schema(&load_schema("linkml_profile"));
    let records = load_fixture_records(fixture_root("fixtures/pass"));

    validators.assert_valid("Goal", &records.goals[0]);
    validators.assert_valid("Spec", &records.specs[0]);
    validators.assert_valid("Task", &records.tasks[0]);
    for decision in &records.decisions {
        validators.assert_valid("Decision", decision);
    }

    assert_relations(&records).expect("fixture references should resolve");
}

#[test]
fn round_trip_writes_preserve_formatting_for_markdown_yaml_json_jsonl() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(&fixture_root("fixtures/pass"), temp.path());

    let goal_path = temp.path().join("docs/goals/goal_model_runtime.md");
    let yaml_path = temp.path().join("tasks/task_profile_review.yaml");
    let json_path = temp.path().join("specs/spec_artifact_runtime.json");
    let jsonl_path = temp.path().join("decisions/decisions.jsonl");

    let original_goal = fs::read_to_string(&goal_path).expect("goal markdown");
    update_markdown_frontmatter_scalar(&goal_path, "title", "Updated native runtime model");
    let updated_goal = fs::read_to_string(&goal_path).expect("updated goal markdown");
    assert!(updated_goal.contains("\nThe body stays as normal Markdown"));
    assert!(updated_goal.ends_with(
        "Agents should be able to update the frontmatter without rewriting this prose.\n"
    ));
    assert_eq!(
        markdown_body(&original_goal),
        markdown_body(&updated_goal),
        "frontmatter updates must leave Markdown body bytes unchanged"
    );

    update_yaml_scalar(&yaml_path, "priority", "p0");
    let updated_yaml = fs::read_to_string(&yaml_path).expect("updated yaml");
    assert!(updated_yaml.contains("priority: p0\n"));
    assert!(updated_yaml.contains("evidence:\n  - kind: file\n"));

    update_json_scalar(&json_path, "title", "Updated artifact runtime contract");
    let updated_json = fs::read_to_string(&json_path).expect("updated json");
    assert!(updated_json.starts_with("{\n  "));
    assert!(updated_json.ends_with('\n'));

    let original_jsonl = fs::read_to_string(&jsonl_path).expect("jsonl");
    update_jsonl_record_title(
        &jsonl_path,
        "decision_authoring_profile",
        "Updated authoring profile decision",
    );
    let updated_jsonl = fs::read_to_string(&jsonl_path).expect("updated jsonl");
    assert_eq!(
        original_jsonl.lines().count(),
        updated_jsonl.lines().count()
    );
    assert!(updated_jsonl.contains("Updated authoring profile decision"));

    let validators = RuntimeValidators::from_schema(&load_schema("typespec_decorators"));
    let records = load_fixture_records(temp.path().to_path_buf());
    validators.assert_valid("Goal", &records.goals[0]);
    validators.assert_valid("Spec", &records.specs[0]);
    validators.assert_valid("Task", &records.tasks[0]);
    assert_relations(&records).expect("round-tripped records should still resolve");
}

#[test]
fn agent_typed_operations_create_update_records_against_contract() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(&fixture_root("fixtures/pass"), temp.path());
    let validators = RuntimeValidators::from_schema(&load_schema("linkml_profile"));
    let command_validator =
        jsonschema::validator_for(&load_operation_contract()).expect("operation schema compiles");

    let create_task_value = json!({
        "op": "create_task",
        "task": {
            "id": "task_agent_contract",
            "title": "Create records through typed operation",
            "status": "active",
            "goal": "goal_model_runtime",
            "spec": "spec_artifact_runtime",
            "assignee": "agent",
            "priority": "p2",
            "evidence": [{"kind": "file", "value": "tests/artifact_authoring_paths_proof.rs"}],
            "metadata": {
                "summary": "Prove agents write through the contract instead of editing arbitrary files.",
                "risk": "medium",
                "tags": ["agent", "contract"]
            }
        }
    });
    assert!(command_validator.is_valid(&create_task_value));
    let create_task = AgentCommand::CreateTask {
        task: create_task_value["task"].clone(),
    };
    apply_agent_command(temp.path(), &validators, create_task).expect("valid task create");

    let append_task_value = json!({
        "op": "append_goal_task",
        "goal_id": "goal_model_runtime",
        "task_id": "task_agent_contract"
    });
    assert!(command_validator.is_valid(&append_task_value));
    apply_agent_command(
        temp.path(),
        &validators,
        AgentCommand::AppendGoalTask {
            goal_id: "goal_model_runtime".to_string(),
            task_id: "task_agent_contract".to_string(),
        },
    )
    .expect("valid goal update");

    let records = load_fixture_records(temp.path().to_path_buf());
    assert!(records
        .tasks
        .iter()
        .any(|task| task["id"] == "task_agent_contract"));
    assert_relations(&records).expect("agent-written records should resolve");

    assert!(!command_validator.is_valid(&json!({
        "op": "append_goal_task",
        "goal_id": "goal_model_runtime"
    })));

    let bad_task = AgentCommand::CreateTask {
        task: json!({
            "id": "task_bad_reference",
            "title": "Bad reference should be rejected",
            "status": "active",
            "goal": "missing_goal",
            "priority": "p1",
            "metadata": {"summary": "Bad ref", "risk": "low"}
        }),
    };
    let err = apply_agent_command(temp.path(), &validators, bad_task)
        .expect_err("bad reference should reject before writing");
    assert!(err.contains("missing_goal"));
    assert!(!temp.path().join("tasks/task_bad_reference.yaml").exists());
}

#[test]
fn native_runtime_performance_uses_cached_json_schema_validators() {
    let temp = tempfile::tempdir().expect("tempdir");
    let validators = RuntimeValidators::from_schema(&load_schema("typespec_decorators"));
    seed_large_repo(temp.path(), 200);

    let start = Instant::now();
    let records = load_fixture_records(temp.path().to_path_buf());
    for goal in &records.goals {
        validators.assert_valid("Goal", goal);
    }
    for spec in &records.specs {
        validators.assert_valid("Spec", spec);
    }
    for task in &records.tasks {
        validators.assert_valid("Task", task);
    }
    for decision in &records.decisions {
        validators.assert_valid("Decision", decision);
    }
    assert_relations(&records).expect("generated references should resolve");
    let elapsed = start.elapsed();
    eprintln!("cached_json_schema_800_file_backed_records={elapsed:?}");

    assert_eq!(records.total(), 800);
    assert!(
        elapsed < Duration::from_secs(5),
        "expected cached Rust JSON Schema validation of 800 file-backed records under 5s, got {elapsed:?}"
    );
}

struct RuntimeValidators {
    goal: Validator,
    spec: Validator,
    task: Validator,
    decision: Validator,
}

impl RuntimeValidators {
    fn from_schema(schema: &Value) -> Self {
        Self {
            goal: compile_definition(schema, "Goal"),
            spec: compile_definition(schema, "Spec"),
            task: compile_definition(schema, "Task"),
            decision: compile_definition(schema, "Decision"),
        }
    }

    fn assert_valid(&self, class: &str, record: &Value) {
        let validator = match class {
            "Goal" => &self.goal,
            "Spec" => &self.spec,
            "Task" => &self.task,
            "Decision" => &self.decision,
            _ => panic!("unknown class {class}"),
        };
        if let Err(error) = validator.validate(record) {
            panic!("{class} failed validation: {error}; record={record}");
        }
    }
}

#[derive(Default)]
struct Records {
    goals: Vec<Value>,
    specs: Vec<Value>,
    tasks: Vec<Value>,
    decisions: Vec<Value>,
}

impl Records {
    fn total(&self) -> usize {
        self.goals.len() + self.specs.len() + self.tasks.len() + self.decisions.len()
    }
}

enum AgentCommand {
    CreateTask { task: Value },
    AppendGoalTask { goal_id: String, task_id: String },
}

fn fixture_root(relative: &str) -> PathBuf {
    Path::new(FIXTURE_ROOT).join(relative)
}

fn schema_path(source: &str) -> PathBuf {
    fixture_root(&format!("generated_outputs/{source}.runtime.schema.json"))
}

fn operation_contract_path() -> PathBuf {
    fixture_root("contracts/agent_operations.schema.json")
}

fn load_schema(source: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(schema_path(source)).expect("schema file"))
        .expect("valid schema json")
}

fn load_operation_contract() -> Value {
    serde_json::from_str(
        &fs::read_to_string(operation_contract_path()).expect("operation contract file"),
    )
    .expect("valid operation contract json")
}

fn runtime_contract_without_source(schema: &Value) -> Value {
    let mut normalized = schema.clone();
    normalized
        .as_object_mut()
        .expect("schema object")
        .remove("x-assura-source");
    normalized
}

fn compile_definition(schema: &Value, class: &str) -> Validator {
    let compiled_schema = json!({
        "$schema": schema["$schema"].clone(),
        "$defs": schema["$defs"].clone(),
        "$ref": format!("#/$defs/{class}")
    });
    jsonschema::validator_for(&compiled_schema).expect("schema should compile")
}

fn load_fixture_records(root: PathBuf) -> Records {
    let mut records = Records::default();
    for path in sorted_files(&root.join("docs/goals"), "md") {
        records.goals.push(read_markdown_frontmatter(&path));
    }
    for path in sorted_files(&root.join("specs"), "json") {
        records.specs.push(read_json(&path));
    }
    for path in sorted_files(&root.join("tasks"), "yaml") {
        records.tasks.push(read_yaml(&path));
    }
    for path in sorted_files(&root.join("decisions"), "jsonl") {
        records.decisions.extend(read_jsonl(&path));
    }
    records
}

fn sorted_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some(extension))
        .collect();
    files.sort();
    files
}

fn read_markdown_frontmatter(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("markdown file");
    let (frontmatter, _) = split_markdown_frontmatter(&contents);
    let yaml: serde_yaml::Value = serde_yaml::from_str(frontmatter).expect("frontmatter yaml");
    serde_json::to_value(yaml).expect("yaml to json")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("json file")).expect("valid json")
}

fn read_yaml(path: &Path) -> Value {
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(path).expect("yaml file")).expect("valid yaml");
    serde_json::to_value(yaml).expect("yaml to json")
}

fn read_jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .expect("jsonl file")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("valid jsonl record"))
        .collect()
}

fn split_markdown_frontmatter(contents: &str) -> (&str, &str) {
    let body_start = contents
        .strip_prefix("---\n")
        .expect("markdown starts with frontmatter");
    let end = body_start
        .find("\n---\n")
        .expect("markdown closes frontmatter");
    let frontmatter = &body_start[..end];
    let body = &body_start[end + "\n---\n".len()..];
    (frontmatter, body)
}

fn markdown_body(contents: &str) -> &str {
    split_markdown_frontmatter(contents).1
}

fn assert_relations(records: &Records) -> Result<(), String> {
    let goals = ids(&records.goals);
    let specs = ids(&records.specs);
    let tasks = ids(&records.tasks);
    let decisions = ids(&records.decisions);

    for goal in &records.goals {
        assert_array_refs(goal, "specs", &specs)?;
        assert_array_refs(goal, "tasks", &tasks)?;
        assert_array_refs(goal, "decisions", &decisions)?;
    }
    for task in &records.tasks {
        assert_scalar_ref(task, "goal", &goals)?;
        if task.get("spec").is_some() {
            assert_scalar_ref(task, "spec", &specs)?;
        }
    }
    for decision in &records.decisions {
        if decision.get("supersedes").is_some() {
            assert_scalar_ref(decision, "supersedes", &decisions)?;
        }
        assert_array_refs(decision, "affects_specs", &specs)?;
        assert_array_refs(decision, "affects_tasks", &tasks)?;
    }
    Ok(())
}

fn ids(records: &[Value]) -> HashSet<String> {
    records
        .iter()
        .map(|record| record["id"].as_str().expect("id").to_string())
        .collect()
}

fn assert_scalar_ref(record: &Value, field: &str, ids: &HashSet<String>) -> Result<(), String> {
    let id = record[field].as_str().ok_or_else(|| {
        format!(
            "{}.{field} must be a string",
            record["id"].as_str().unwrap_or("<unknown>")
        )
    })?;
    if ids.contains(id) {
        Ok(())
    } else {
        Err(format!(
            "{}.{field} references missing id {id}",
            record["id"].as_str().unwrap_or("<unknown>")
        ))
    }
}

fn assert_array_refs(record: &Value, field: &str, ids: &HashSet<String>) -> Result<(), String> {
    let Some(values) = record.get(field).and_then(Value::as_array) else {
        return Ok(());
    };
    for value in values {
        let id = value.as_str().ok_or_else(|| {
            format!(
                "{}.{field} must contain strings",
                record["id"].as_str().unwrap_or("<unknown>")
            )
        })?;
        if !ids.contains(id) {
            return Err(format!(
                "{}.{field} references missing id {id}",
                record["id"].as_str().unwrap_or("<unknown>")
            ));
        }
    }
    Ok(())
}

fn update_markdown_frontmatter_scalar(path: &Path, key: &str, value: &str) {
    let contents = fs::read_to_string(path).expect("markdown file");
    let (frontmatter, body) = split_markdown_frontmatter(&contents);
    let updated = replace_scalar_line(frontmatter, key, value);
    fs::write(path, format!("---\n{updated}\n---\n{body}")).expect("write markdown");
}

fn update_yaml_scalar(path: &Path, key: &str, value: &str) {
    let contents = fs::read_to_string(path).expect("yaml file");
    fs::write(path, replace_scalar_line(&contents, key, value)).expect("write yaml");
}

fn replace_scalar_line(contents: &str, key: &str, value: &str) -> String {
    contents
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{key}: ")) {
                format!("{key}: {value}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn update_json_scalar(path: &Path, key: &str, value: &str) {
    let mut json = read_json(path);
    json[key] = Value::String(value.to_string());
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json).expect("pretty json")
        ),
    )
    .expect("write json");
}

fn update_jsonl_record_title(path: &Path, id: &str, title: &str) {
    let contents = fs::read_to_string(path).expect("jsonl file");
    let updated = contents
        .lines()
        .map(|line| {
            let mut record: Value = serde_json::from_str(line).expect("jsonl record");
            if record["id"] == id {
                record["title"] = Value::String(title.to_string());
            }
            serde_json::to_string(&record).expect("compact jsonl")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{updated}\n")).expect("write jsonl");
}

fn apply_agent_command(
    root: &Path,
    validators: &RuntimeValidators,
    command: AgentCommand,
) -> Result<(), String> {
    match command {
        AgentCommand::CreateTask { task } => {
            validators.assert_valid("Task", &task);
            let mut records = load_fixture_records(root.to_path_buf());
            records.tasks.push(task.clone());
            assert_relations(&records)?;
            let id = task["id"].as_str().expect("task id");
            let path = root.join(format!("tasks/{id}.yaml"));
            fs::write(
                path,
                serde_yaml::to_string(&task).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())
        }
        AgentCommand::AppendGoalTask { goal_id, task_id } => {
            let path = root.join(format!("docs/goals/{goal_id}.md"));
            append_frontmatter_list_item(&path, "tasks", &task_id);
            let records = load_fixture_records(root.to_path_buf());
            assert_relations(&records)
        }
    }
}

fn append_frontmatter_list_item(path: &Path, key: &str, item: &str) {
    let contents = fs::read_to_string(path).expect("markdown file");
    let (frontmatter, body) = split_markdown_frontmatter(&contents);
    let mut lines: Vec<String> = frontmatter.lines().map(String::from).collect();
    let start = lines
        .iter()
        .position(|line| line == &format!("{key}:"))
        .expect("frontmatter list");
    let mut insert_at = start + 1;
    while insert_at < lines.len() && lines[insert_at].starts_with("  - ") {
        insert_at += 1;
    }
    lines.insert(insert_at, format!("  - {item}"));
    fs::write(path, format!("---\n{}\n---\n{body}", lines.join("\n"))).expect("write markdown");
}

fn seed_large_repo(root: &Path, count: usize) {
    fs::create_dir_all(root.join("docs/goals")).expect("goals dir");
    fs::create_dir_all(root.join("specs")).expect("specs dir");
    fs::create_dir_all(root.join("tasks")).expect("tasks dir");
    fs::create_dir_all(root.join("decisions")).expect("decisions dir");

    let mut decisions = String::new();
    for index in 0..count {
        let id = format!("{index:04}");
        fs::write(
            root.join(format!("docs/goals/goal_{id}.md")),
            format!(
                "---\nid: goal_{id}\ntitle: Goal {id}\nstatus: active\nowners:\n  - platform\nspecs:\n  - spec_{id}\ntasks:\n  - task_{id}\ndecisions:\n  - decision_{id}\nmetadata:\n  summary: Generated goal {id}\n  risk: low\n---\n\nGenerated benchmark body.\n"
            ),
        )
        .expect("write goal");
        fs::write(
            root.join(format!("specs/spec_{id}.json")),
            serde_json::to_string(&json!({
                "id": format!("spec_{id}"),
                "title": format!("Spec {id}"),
                "status": "active",
                "owner": "platform",
                "metadata": {"summary": format!("Generated spec {id}"), "risk": "low"},
                "decisions": [format!("decision_{id}")]
            }))
            .expect("json spec"),
        )
        .expect("write spec");
        fs::write(
            root.join(format!("tasks/task_{id}.yaml")),
            format!(
                "id: task_{id}\ntitle: Task {id}\nstatus: active\ngoal: goal_{id}\nspec: spec_{id}\npriority: p1\nmetadata:\n  summary: Generated task {id}\n  risk: low\n"
            ),
        )
        .expect("write task");
        decisions.push_str(
            &serde_json::to_string(&json!({
                "id": format!("decision_{id}"),
                "title": format!("Decision {id}"),
                "status": "active",
                "affects_specs": [format!("spec_{id}")],
                "affects_tasks": [format!("task_{id}")],
                "metadata": {"summary": format!("Generated decision {id}"), "risk": "low"}
            }))
            .expect("json decision"),
        );
        decisions.push('\n');
    }
    fs::write(root.join("decisions/decisions.jsonl"), decisions).expect("write decisions");
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).expect("read source dir") {
        let entry = entry.expect("source entry");
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            fs::create_dir_all(&target).expect("create target dir");
            copy_dir(&source, &target);
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).expect("create parent dir");
            }
            fs::copy(&source, &target).expect("copy file");
        }
    }
}
