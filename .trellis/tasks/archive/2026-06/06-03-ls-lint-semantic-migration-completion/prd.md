# LS-Lint 2.3 Semantic Migration Completion

## Objective

Make Assura a complete semantic migration target for LS-Lint 2.3 config users.
Every LS-Lint 2.3 config feature must convert into equivalent Assura behavior,
be documented as natively expressible in Assura, or fail migration with a
precise unsupported-feature error when the input is invalid or outside LS-Lint
2.3 config semantics.

CLI drop-in parity is explicitly out of scope.

## Scope

- Define a checked LS-Lint 2.3 feature coverage matrix from upstream docs.
- Make `src/config/ls_compat.rs` the authoritative migration converter and
  report source.
- Stop using the stale `src/ls_compat/parser.rs` path for migration reports or
  public migration behavior.
- Reject unknown rule names, invalid rule syntax, malformed YAML shapes, and
  invalid multi-rule separators.
- Validate converted Assura config before writing migration output.
- Add native LS-Lint golden parity tests for small deterministic fixtures.
- Keep existing realistic fixtures as regression/performance evidence.
- Update docs to claim complete LS-Lint 2.3 config feature coverage while
  clearly excluding CLI drop-in parity.

## Non-Goals

- No LS-Lint CLI drop-in replacement.
- No support requirement for LS-Lint flags such as `--workdir`, `--debug`, or
  exact JSON output.
- No hosted migration service.
- No LLM-based migration.
- No weakening Assura-only features.

## Definition Of Done

- A checked feature matrix lists every LS-Lint 2.3 config feature and maps it
  to converter support, Assura native support, parity test evidence, and docs.
- `assura migrate` rejects LS-Lint-invalid rules instead of producing bad
  Assura config.
- Migration report counts are generated from the authoritative converter.
- Native LS-Lint golden parity tests pass for all feature families.
- Existing realistic fixture benchmarks/tests still pass.
- Docs say Assura has complete LS-Lint config feature coverage and exclude CLI
  drop-in parity.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --test ls_lint_rule_coverage_tests --quiet
cargo test --test ls_lint_parity_regression_tests --quiet
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- migrate tests/fixtures/ls-lint/full-coverage/.ls-lint.yml --output target/full-coverage-assura.yml
cargo run --quiet -- check --config target/full-coverage-assura.yml tests/fixtures/ls-lint/full-coverage --format json
cargo run --quiet -- check --format json .
node --run verify:docs
git diff --check
```

## Reviewer Blocking Criteria

Block completion if any LS-Lint 2.3 config feature lacks matrix coverage,
native LS-Lint and Assura disagree on a golden fixture without a documented
reason, migration silently drops unsupported or malformed config, the stale
parser remains authoritative for reports or behavior, or docs imply CLI
drop-in parity.
