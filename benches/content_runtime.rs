use assura::cli::run_structure_check;
use assura::config::loader::ConfigLoader;
use assura::content_repository::ContentRepository;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::{Builder, TempDir};

fn content_runtime_benchmarks(c: &mut Criterion) {
    let fixture = ContentRuntimeFixture::new(240, 240, 240);
    let config_path = fixture.root().join(".assura/config.yml");
    let config = ConfigLoader::load(&config_path).expect("content runtime config");
    let repository =
        ContentRepository::from_config(fixture.root(), &config).expect("content repository");

    let mut group = c.benchmark_group("content_runtime");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(480));

    group.bench_function("repository_validate_warm/240_goals_240_specs", |b| {
        b.iter(|| {
            let validation = repository.validate(fixture.root());
            assert_eq!(validation.findings.len(), 0);
            black_box(validation.snapshot.objects.len())
        })
    });

    group.bench_function("repository_cold_in_process/240_goals_240_specs", |b| {
        b.iter(|| {
            let config = ConfigLoader::load(&config_path).expect("content runtime config");
            let repository =
                ContentRepository::from_config(fixture.root(), &config).expect("repository");
            let validation = repository.validate(fixture.root());
            assert_eq!(validation.findings.len(), 0);
            black_box(validation.snapshot.objects.len())
        })
    });

    group.finish();

    let mut check_group = c.benchmark_group("content_runtime_check");
    check_group.sample_size(10);
    check_group.warm_up_time(Duration::from_millis(500));
    check_group.throughput(Throughput::Elements(480));

    check_group.bench_with_input(
        BenchmarkId::new(
            "assura_check_cold_in_process",
            "no_content_runtime_baseline",
        ),
        fixture.baseline_root(),
        |b, root| {
            b.iter(|| {
                let report =
                    run_structure_check(Some(root.to_path_buf()), None, false).expect("check");
                assert!(report.violations.is_empty());
                black_box(report.files_checked)
            })
        },
    );

    check_group.bench_with_input(
        BenchmarkId::new("assura_check_cold_in_process", "with_content_runtime"),
        fixture.root(),
        |b, root| {
            b.iter(|| {
                let report =
                    run_structure_check(Some(root.to_path_buf()), None, false).expect("check");
                assert!(report.violations.is_empty());
                black_box(report.files_checked)
            })
        },
    );

    check_group.finish();
}

struct ContentRuntimeFixture {
    runtime: TempDir,
    baseline: TempDir,
}

impl ContentRuntimeFixture {
    fn new(goals: usize, specs: usize, unrelated_notes: usize) -> Self {
        let runtime = Builder::new()
            .prefix("assura_content_runtime_bench_")
            .tempdir()
            .expect("runtime fixture");
        let baseline = Builder::new()
            .prefix("assura_content_runtime_baseline_")
            .tempdir()
            .expect("baseline fixture");

        write_repo(runtime.path(), goals, specs, unrelated_notes, true);
        write_repo(baseline.path(), goals, specs, unrelated_notes, false);

        Self { runtime, baseline }
    }

    fn root(&self) -> &Path {
        self.runtime.path()
    }

    fn baseline_root(&self) -> &Path {
        self.baseline.path()
    }
}

fn write_repo(root: &Path, goals: usize, specs: usize, unrelated_notes: usize, with_runtime: bool) {
    fs::create_dir_all(root.join(".assura")).expect("assura dir");
    fs::create_dir_all(root.join("docs/goals")).expect("goals dir");
    fs::create_dir_all(root.join("specs")).expect("specs dir");
    fs::create_dir_all(root.join("notes")).expect("notes dir");
    fs::create_dir_all(root.join("schemas")).expect("schemas dir");

    fs::write(root.join(".assura/config.yml"), config_yaml(with_runtime)).expect("config");
    fs::write(
        root.join("schemas/content_runtime.schema.json"),
        runtime_schema_json(),
    )
    .expect("schema");

    for index in 0..specs {
        fs::write(
            root.join(format!("specs/spec-{index:04}.json")),
            format!(
                "{{\n  \"id\": \"spec-{index:04}\",\n  \"title\": \"Spec {index:04}\",\n  \"status\": \"active\"\n}}\n"
            ),
        )
        .expect("spec");
    }

    for index in 0..goals {
        let spec_index = index % specs;
        fs::write(
            root.join(format!("docs/goals/goal-{index:04}.md")),
            format!(
                "---\nid: goal-{index:04}\ntitle: Goal {index:04}\nstatus: active\nspecs:\n  - spec-{spec_index:04}\n---\n# Goal {index:04}\n\nRuntime validation benchmark fixture.\n"
            ),
        )
        .expect("goal");
    }

    for index in 0..unrelated_notes {
        fs::write(
            root.join(format!("notes/note-{index:04}.md")),
            format!("# Note {index:04}\n\nNot part of a content collection.\n"),
        )
        .expect("note");
    }
}

fn config_yaml(with_runtime: bool) -> String {
    let mut config = "structure:\n  ./:\n    required: false\n".to_string();
    if with_runtime {
        config.push_str(
            r#"
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

relations:
  goals.specs:
    target: specs
    many: true
"#,
        );
    }
    config
}

fn runtime_schema_json() -> &'static str {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Goal": {
      "type": "object",
      "required": ["id", "title", "status", "specs"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "enum": ["active", "completed"] },
        "specs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }
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
    }
  }
}
"#
}

criterion_group!(content_runtime, content_runtime_benchmarks);
criterion_main!(content_runtime);
