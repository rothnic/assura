# LS-Lint Comparison Benchmarks

This directory contains performance benchmarks comparing Assura with LS-Lint.

## Prerequisites

To run the full comparison benchmarks (including LS-Lint), you need to install LS-Lint:

```bash
./scripts/install_ls_lint.sh
```

Or manually:
```bash
npm install -g @ls-lint/ls-lint
```

## Running Benchmarks

### All Comparison Benchmarks
```bash
cargo bench --bench ls_lint_comparison
```

### Specific Benchmark Groups
```bash
# Cold start comparison
cargo bench --bench ls_lint_comparison cold_start

# Full validation (warm)
cargo bench --bench ls_lint_comparison full_validation

# Throughput comparison
cargo bench --bench ls_lint_comparison throughput

# Project type comparison (Rust/JS)
cargo bench --bench ls_lint_comparison project_types

# Incremental validation
cargo bench --bench ls_lint_comparison incremental

# Complex rules performance
cargo bench --bench ls_lint_comparison complex_rules

# Structure-first assura check attribution
cargo bench --bench profiling structure_check
```

## Benchmark Scenarios

### Test Fixtures
The benchmarks create test fixtures of various sizes:

- **Small**: 50 files across 10 directories
- **Medium**: 500 files across 50 directories  
- **Large**: 5000 files across 200 directories

### Metrics Captured

1. **Cold Start Time**: First run initialization overhead
2. **Warm Validation Time**: Subsequent run performance
3. **Memory Usage Peak**: (if instrumentation available)
4. **Files/Second Throughput**: Raw validation speed
5. **Incremental Validation**: Single file change performance

## Expected Performance

Assura is designed to be **2x+ faster** than LS-Lint across all scenarios:

- Written in Rust for native performance
- Efficient glob pattern matching
- Minimal allocations in hot paths
- Parallel validation support

## Interpreting Results

Benchmark results are saved in `target/criterion/` with HTML reports:

```bash
# View HTML report
open target/criterion/ls_lint_comparison/report/index.html
```

Key metrics to watch:
- **Files/second**: Higher is better
- **Speedup factor**: Assura time / LS-Lint time (should be < 0.5)
- **Consistency**: Performance across different project types

## Structure-First Profiling

`benches/profiling.rs` also includes `structure_check/...` groups for the
current `assura check` implementation. These benchmarks reuse Criterion and
cover full `run_structure_check` scenarios plus isolated attribution slices for
config load, traversal, exclusion pruning, directory count reads, and glob
pattern matching.

The structure-first production path currently uses `walkdir::WalkDir`.
Existing `jwalk` benchmarks remain as traversal and older `ConstraintEngine`
comparison context.
