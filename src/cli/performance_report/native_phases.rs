//! Assura-native in-process attribution rows for the native performance suite.

use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
};
use crate::config::loader::ConfigLoader;
use crate::content_repository::{ContentFinding, ContentRepository, RepositoryModel};
use crate::intelligence::{FactIngestor, FactSet, InMemoryFactStore};
use std::hint::black_box;
use std::time::Instant;

// allow-reason: native attribution rows use the shared performance row metadata shape.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_native_phase_rows(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
) -> Vec<PerformanceResultRow> {
    let mut samples = NativePhaseSamples::with_capacity(iterations);
    let mut failure = None;
    for _ in 0..iterations {
        match measure_native_phase_sample(fixture) {
            Ok(sample) => samples.push(sample),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }
    samples.into_rows(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        failure,
        baseline_id,
    )
}

fn measure_native_phase_sample(fixture: &MaterializedFixture) -> Result<NativePhaseSample, String> {
    let config_path = fixture.root.join(".assura/config.yml");

    let started = Instant::now();
    let config = ConfigLoader::load(&config_path).map_err(|error| error.to_string())?;
    let model = RepositoryModel::from_config(&fixture.root, &config).map_err(format_findings)?;
    let config_model_load_ms = elapsed_ms(started);

    let started = Instant::now();
    let repository = ContentRepository::try_new(model.clone()).map_err(format_findings)?;
    let schema_compile_ms = elapsed_ms(started);

    let (validation, profile) = repository.validate_profiled(&fixture.root);

    let started = Instant::now();
    let facts = facts_for_generation(&model, &validation, "native-phase-a");
    let mut store = InMemoryFactStore::load(facts);
    black_box(store.stats());
    let fact_ingest_load_ms = elapsed_ms(started);

    let replacement = facts_for_generation(&model, &validation, "native-phase-b");
    let started = Instant::now();
    store.replace_generation("native-phase-a", replacement);
    black_box(store.stats());
    let incremental_replace_generation_ms = elapsed_ms(started);

    let started = Instant::now();
    black_box(store.keyword_search("runtime").len());
    let warm_keyword_query_ms = elapsed_ms(started);

    let started = Instant::now();
    black_box(
        serde_json::to_vec::<FactSet>(store.facts())
            .map_err(|error| format!("failed to serialize native fact set: {error}"))?,
    );
    let factset_serialize_json_ms = elapsed_ms(started);

    Ok(NativePhaseSample {
        config_model_load_ms,
        schema_compile_ms,
        file_index_ms: profile.file_index_ms,
        object_load_validate_ms: profile.object_load_ms,
        edge_collect_ms: profile.edge_collect_ms,
        reference_validate_ms: profile.reference_validate_ms,
        repository_validate_total_ms: profile.total_ms,
        fact_ingest_load_ms,
        incremental_replace_generation_ms,
        warm_keyword_query_ms,
        factset_serialize_json_ms,
    })
}

fn facts_for_generation(
    model: &RepositoryModel,
    validation: &crate::content_repository::RepositoryValidation,
    generation: &str,
) -> FactSet {
    let mut ingestor = FactIngestor::new(generation);
    ingestor.ingest_repository_model(model);
    ingestor.ingest_repository_validation(validation);
    ingestor.finish()
}

fn format_findings(findings: Vec<ContentFinding>) -> String {
    findings
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("; ")
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

struct NativePhaseSample {
    config_model_load_ms: f64,
    schema_compile_ms: f64,
    file_index_ms: f64,
    object_load_validate_ms: f64,
    edge_collect_ms: f64,
    reference_validate_ms: f64,
    repository_validate_total_ms: f64,
    fact_ingest_load_ms: f64,
    incremental_replace_generation_ms: f64,
    warm_keyword_query_ms: f64,
    factset_serialize_json_ms: f64,
}

struct NativePhaseSamples {
    config_model_load_ms: Vec<f64>,
    schema_compile_ms: Vec<f64>,
    file_index_ms: Vec<f64>,
    object_load_validate_ms: Vec<f64>,
    edge_collect_ms: Vec<f64>,
    reference_validate_ms: Vec<f64>,
    repository_validate_total_ms: Vec<f64>,
    fact_ingest_load_ms: Vec<f64>,
    incremental_replace_generation_ms: Vec<f64>,
    warm_keyword_query_ms: Vec<f64>,
    factset_serialize_json_ms: Vec<f64>,
}

impl NativePhaseSamples {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            config_model_load_ms: Vec::with_capacity(capacity),
            schema_compile_ms: Vec::with_capacity(capacity),
            file_index_ms: Vec::with_capacity(capacity),
            object_load_validate_ms: Vec::with_capacity(capacity),
            edge_collect_ms: Vec::with_capacity(capacity),
            reference_validate_ms: Vec::with_capacity(capacity),
            repository_validate_total_ms: Vec::with_capacity(capacity),
            fact_ingest_load_ms: Vec::with_capacity(capacity),
            incremental_replace_generation_ms: Vec::with_capacity(capacity),
            warm_keyword_query_ms: Vec::with_capacity(capacity),
            factset_serialize_json_ms: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, sample: NativePhaseSample) {
        self.config_model_load_ms.push(sample.config_model_load_ms);
        self.schema_compile_ms.push(sample.schema_compile_ms);
        self.file_index_ms.push(sample.file_index_ms);
        self.object_load_validate_ms
            .push(sample.object_load_validate_ms);
        self.edge_collect_ms.push(sample.edge_collect_ms);
        self.reference_validate_ms
            .push(sample.reference_validate_ms);
        self.repository_validate_total_ms
            .push(sample.repository_validate_total_ms);
        self.fact_ingest_load_ms.push(sample.fact_ingest_load_ms);
        self.incremental_replace_generation_ms
            .push(sample.incremental_replace_generation_ms);
        self.warm_keyword_query_ms
            .push(sample.warm_keyword_query_ms);
        self.factset_serialize_json_ms
            .push(sample.factset_serialize_json_ms);
    }

    // allow-reason: native attribution rows share the report's explicit metadata surface.
    #[allow(clippy::too_many_arguments)]
    fn into_rows(
        self,
        fixture: &MaterializedFixture,
        timestamp: &str,
        commit_sha: &str,
        branch: &str,
        environment: &PerformanceEnvironment,
        failure: Option<String>,
        baseline_id: &str,
    ) -> Vec<PerformanceResultRow> {
        [
            (
                "assura-native config and model load",
                "native:phase:config-model-load",
                self.config_model_load_ms,
            ),
            (
                "assura-native schema validator compile",
                "native:phase:schema-compile",
                self.schema_compile_ms,
            ),
            (
                "assura-native content file index",
                "native:phase:file-index",
                self.file_index_ms,
            ),
            (
                "assura-native object load and validation",
                "native:phase:object-load-validate",
                self.object_load_validate_ms,
            ),
            (
                "assura-native relation edge collect",
                "native:phase:edge-collect",
                self.edge_collect_ms,
            ),
            (
                "assura-native relation validation",
                "native:phase:reference-validate",
                self.reference_validate_ms,
            ),
            (
                "assura-native repository validation total",
                "native:phase:repository-validate-total",
                self.repository_validate_total_ms,
            ),
            (
                "assura-native project facts ingest and index load",
                "native:phase:fact-ingest-load",
                self.fact_ingest_load_ms,
            ),
            (
                "assura-native incremental generation replace",
                "native:phase:incremental-replace-generation",
                self.incremental_replace_generation_ms,
            ),
            (
                "assura-native warm keyword query",
                "native:phase:warm-keyword-query",
                self.warm_keyword_query_ms,
            ),
            (
                "assura-native fact set JSON serialization",
                "native:phase:factset-serialize-json",
                self.factset_serialize_json_ms,
            ),
        ]
        .into_iter()
        .map(|(tool_name, row_family, samples)| {
            row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                "not-applicable",
                RowMeasurement::new(tool_name, row_family),
                samples,
                failure.clone(),
                baseline_id,
            )
        })
        .collect()
    }
}
