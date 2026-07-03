//! Generated Assura-native performance fixtures.

use super::fixture_io::write_file;
use super::fixtures::{FixtureKind, FixtureScenario};
use std::fs;
use std::path::Path;

pub(in crate::cli::performance_report) fn native_scenarios() -> Vec<FixtureScenario> {
    vec![
        FixtureScenario {
            id: "native_small",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-small-content-authoring",
            dirs: 25,
            files_per_dir: 25,
            kind: FixtureKind::NativeSmall,
        },
        FixtureScenario {
            id: "native_medium",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-medium-content-authoring",
            dirs: 250,
            files_per_dir: 250,
            kind: FixtureKind::NativeMedium,
        },
        FixtureScenario {
            id: "native_large",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-large-content-authoring",
            dirs: 2500,
            files_per_dir: 2500,
            kind: FixtureKind::NativeLarge,
        },
        FixtureScenario {
            id: "native_reference_heavy",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-reference-heavy-content-authoring",
            dirs: 250,
            files_per_dir: 250,
            kind: FixtureKind::NativeReferenceHeavy,
        },
        FixtureScenario {
            id: "native_adapter_mix",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-adapter-mix-content-authoring",
            dirs: 250,
            files_per_dir: 250,
            kind: FixtureKind::NativeAdapterMix,
        },
        FixtureScenario {
            id: "native_real_project",
            source_revision: "native-fixtures-v1",
            rule_cohort: "native-real-project-content-authoring",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::NativeRealProject,
        },
    ]
}

pub(super) fn create_native_project(root: &Path, kind: FixtureKind) -> Result<(), String> {
    let (goals, specs, adrs, notes, broken_every, adapter_mix) = match kind {
        FixtureKind::NativeSmall => (25, 25, 5, 10, None, false),
        FixtureKind::NativeMedium => (250, 250, 50, 100, None, false),
        FixtureKind::NativeLarge => (2500, 2500, 500, 1000, None, false),
        FixtureKind::NativeReferenceHeavy => (250, 250, 50, 100, Some(11), false),
        FixtureKind::NativeAdapterMix => (250, 250, 50, 100, None, true),
        FixtureKind::NativeRealProject => (120, 120, 24, 60, Some(17), true),
        _ => unreachable!("native fixture expected"),
    };

    fs::create_dir_all(root.join(".assura")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("docs/goals")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("docs/decisions")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("specs")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("notes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("schemas")).map_err(|error| error.to_string())?;
    if adapter_mix {
        fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    }

    write_file(
        root.join(".assura/config.yml"),
        &native_config_yaml(adapter_mix),
    )?;
    write_file(
        root.join("schemas/content_runtime.schema.json"),
        native_schema_json(),
    )?;

    for index in 0..specs {
        write_file(
            root.join(format!("specs/spec-{index:04}.json")),
            &format!(
                "{{\n  \"id\": \"spec-{index:04}\",\n  \"title\": \"Spec {index:04}\",\n  \"status\": \"active\"\n}}\n"
            ),
        )?;
    }

    for index in 0..goals {
        let spec_index = if broken_every.is_some_and(|every| index % every == 0) {
            specs + index
        } else {
            index % specs
        };
        let adr_index = index % adrs.max(1);
        write_file(
            root.join(format!("docs/goals/goal-{index:04}.md")),
            &format!(
                "---\nid: goal-{index:04}\ntitle: Goal {index:04}\nstatus: active\nspecs:\n  - spec-{spec_index:04}\nadrs:\n  - adr-{adr_index:04}\nimplementation: crate::runtime::Goal{index:04}\n---\n# Goal {index:04}\n\nRuntime validation benchmark fixture with [Spec](../../specs/spec-{spec_index:04}.json).\n\n## Evidence\n\nContent authoring project evidence row.\n"
            ),
        )?;
    }

    for index in 0..adrs {
        write_file(
            root.join(format!("docs/decisions/adr-{index:04}.json")),
            &format!(
                "{{\n  \"id\": \"adr-{index:04}\",\n  \"title\": \"ADR {index:04}\",\n  \"status\": \"accepted\"\n}}\n"
            ),
        )?;
    }

    for index in 0..notes {
        write_file(
            root.join(format!("notes/note-{index:04}.md")),
            &format!(
                "# Note {index:04}\n\nThis note supports content search and Markdown scanning.\n\n"
            ),
        )?;
    }

    if adapter_mix {
        let mut runtime = String::new();
        for index in 0..goals {
            runtime.push_str(&format!("pub struct Goal{index:04};\n"));
        }
        write_file(root.join("src/runtime.rs"), &runtime)?;
    }

    Ok(())
}

fn native_config_yaml(adapter_mix: bool) -> String {
    let mut config = r#"structure:
  ./:
    required: false
    markdown:
      lint_common: true
      lint_trailing_spaces: true
      check_links: true
models:
  source: schemas/content-runtime.linkml.yaml
  validation_artifact: schemas/content_runtime.schema.json
collections:
  goals:
    class: Goal
    path: docs/goals/*.md
    adapter: markdown_frontmatter
    data: frontmatter
    body: markdown
    id: id
  specs:
    class: Spec
    path: specs/*.json
    adapter: json_record
    id: id
  adrs:
    class: Adr
    path: docs/decisions/*.json
    adapter: json_record
    id: id
relations:
  goals.specs:
    target: specs
    many: true
  goals.adrs:
    target: adrs
    many: true
"#
    .to_string();
    if adapter_mix {
        config.push_str(
            r#"code_symbols:
  goals.implementation:
    provider: local-rust
    many: false
"#,
        );
    }
    config
}

fn native_schema_json() -> &'static str {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Goal": {
      "type": "object",
      "required": ["id", "title", "status", "specs", "adrs"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "enum": ["active", "completed"] },
        "specs": { "type": "array", "items": { "type": "string", "minLength": 1 } },
        "adrs": { "type": "array", "items": { "type": "string", "minLength": 1 } },
        "implementation": { "type": "string" }
      },
      "additionalProperties": false
    },
    "Spec": {
      "type": "object",
      "required": ["id", "title", "status"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "enum": ["draft", "active"] }
      },
      "additionalProperties": false
    },
    "Adr": {
      "type": "object",
      "required": ["id", "title", "status"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "enum": ["proposed", "accepted"] }
      },
      "additionalProperties": false
    }
  }
}
"#
}
