---
id: analysis-2026-05-26-ls-lint-counterexample-challenge
title: LS-Lint Counterexample Challenge Review
date: 2026-05-26
status: implemented
related:
  - docs/goals/assura-ls-lint-rule-coverage-audit.md
  - docs/goals/assura-ls-lint-counterexample-closure.md
  - docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md
---

# LS-Lint Counterexample Challenge Review

## Scope

This review challenged the current Assura LS-Lint compatibility work with an
adversarial review agent plus local verification. The goal was to find concrete
LS-Lint configs where current Assura diverges from `@ls-lint/ls-lint@2.3.0` or
is materially slower on the same tree.

These findings are implementation blockers for full LS-Lint compatibility and
performance claims. They should be closed with regression tests and measured
release evidence, not by narrowing public claims.

## Environment

- Assura release binaries rebuilt locally before performance verification:
  `cargo build --release -p assura --bin assura` and
  `cargo build --release -p assura-check-cli`
- `assura-check --version`: `assura-check 0.1.0`
- LS-Lint package: `@ls-lint/ls-lint@2.3.0`
- Native LS-Lint binary: `ls-lint v2.3.0`
- Host: `Darwin 25.2.0 x86_64`
- Rust: `rustc 1.94.1 (e408947bf 2026-03-25)`
- Node/npm used for package install: `node v25.6.0`, `npm 11.8.0`

## Correctness Findings

### 1. Non-extension scalar `exists` keys are an Assura extension

Fixture: `/private/tmp/assura-lslint-exact-exists-fMYa1p`

`.ls-lint.yml`:

```yaml
ls:
  README.md: exists:1
```

Tree:

```text
README.md
.ls-lint.yml
```

Observed LS-Lint:

```bash
cd /private/tmp/assura-lslint-exact-exists-fMYa1p
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json
```

Exit status: `1`

Observed diagnostic:

```json
{".":{"README.md":["exists:1 (found 0)"]}}
```

Observed Assura after migration:

```bash
target/release/assura migrate .ls-lint.yml --output .assura/config.yml
target/release/assura check --config .assura/config.yml --format json .
```

Exit status: `0`

Assura converted the key into an exact file count:

```yaml
structure:
  ./:
    files:
      exists:
        README.md: '1'
    inherit: true
exclude:
- .assura/**
```

Related inverse fixture: `/private/tmp/assura-lslint-exact-exists-zero-LZCnZh`

For `README.md: exists:0` with `README.md` present, LS-Lint exits `0` while
Assura exits `1`.

Related scalar naming fixture:
`/private/tmp/assura-lslint-string-path-ctQyqw`

For:

```yaml
ls:
  src: kebab-case
```

with `src/BadName.js`, LS-Lint exits `0`. Assura now matches that upstream
no-op behavior for non-`exists` scalar path keys; this is covered by
`converted_lslint_non_dot_scalar_naming_keys_match_upstream_noop`.

Updated interpretation:

- Assura intentionally extends LS-Lint notation so scalar exact `exists` keys
  can require a file or directory, for example package-scoped `AGENTS.md`.
- This must be documented and tested as an Assura extension rather than removed
  or softened.
- Non-`exists` scalar path keys remain LS-Lint-compatible no-ops.
- Upstream LS-Lint comparison evidence should still make clear that this is
  stronger than LS-Lint 2.3 behavior.

Required regression coverage:

- Converted Assura-extended `README.md: exists:1` must require the file.
- Converted Assura-extended `src/: exists:1` must require the directory.
- Converted Assura-extended package scopes must support rules such as
  `packages/*: AGENTS.md: exists:1`.
- Non-`exists` scalar path keys such as `src: kebab-case` must not become
  Assura child file-naming policies during LS-Lint migration.
- Docs and evidence must separate Assura-extended exact `exists` from upstream
  LS-Lint 2.3 parity.

### 2. Targeted directory runs need explicit compatibility semantics

Fixture: `/private/tmp/assura-lslint-target-dir-e2mGSL`

`.ls-lint.yml`:

```yaml
ls:
  .ts: kebabcase
```

Tree:

```text
src/BadName.ts
```

Observed LS-Lint full-tree run:

```bash
cd /private/tmp/assura-lslint-target-dir-e2mGSL
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json
```

Exit status: `1`; diagnostic reports `src/BadName.ts`.

Observed LS-Lint targeted directory run:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json src
```

Exit status: `0`.

Observed Assura targeted directory run:

```bash
target/release/assura migrate .ls-lint.yml --output .assura/config.yml
target/release/assura check --config .assura/config.yml --format json src
```

Exit status: `1`; diagnostic reports `src/BadName.ts`.

Impact:

- Full-tree behavior matches.
- Assura's native changed-path behavior should recurse for agent feedback.
- `--ls-lint-target-semantics` checks only the target like LS-Lint, including
  targeted directories with direct count rules.

Required regression coverage:

- Native `assura check <directory>` must keep recursive feedback.
- Explicit LS-Lint target semantics must match LS-Lint's non-recursive target
  behavior.
- Explicit LS-Lint target semantics must not enforce direct child `exists`
  counts for a targeted directory when LS-Lint would skip those child checks.

### 3. Root `.dir` `exists` rules are skipped by Assura

Fixture: `/private/tmp/assura-lslint-root-dir-exists1-xnDgzc`

`.ls-lint.yml`:

```yaml
ls:
  .dir: exists:1
```

Observed LS-Lint with no child directories:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json
```

Exit status: `1`; diagnostic:

```json
{".":{".dir":["exists:1 (found 0)"]}}
```

Observed Assura after migration:

```bash
target/release/assura migrate .ls-lint.yml --output .assura/config.yml
target/release/assura check --config .assura/config.yml --format json .
```

Exit status: `0`; no violations.

Adding `good-dir/` did not change the result: LS-Lint still failed at `.`, and
Assura still passed.

Impact:

- Non-root `.dir: exists:0` behavior was confirmed to match in a child scope,
  but root `.dir` existence is not covered.
- The current Assura checker skips root self-directory validation.

Required regression coverage:

- Root `.dir: exists:1` and `.dir: exists:0` must be tested against LS-Lint
  behavior.
- The full checker and any LS-Lint fast/compiled paths must agree on root
  `.dir` semantics.

## Confirmed Performance Gaps

### 4. Multipart extension rules trigger exponential candidate generation

Fixture: `/private/tmp/assura-lslint-perf-multipart-MHWQc5/project`

`.ls-lint.yml`:

```yaml
ignore:
  - .assura
ls:
  .a.b.c.d.e.f.g.h.i.j.k.js: kebabcase
```

Tree: `1,500` files named like:

```text
src/file-0001.a.b.c.d.e.f.g.h.i.j.k.js
```

Both tools exited `0`.

Rebuilt `assura-check` release timings:

```text
3.63s
3.59s
3.50s
```

Native LS-Lint timings:

```text
0.02s
0.02s
0.02s
```

Likely cause:

- `src/cli/check/patterns.rs` generates every wildcard combination in
  `lslint_extension_candidates`.
- A filename with eleven extension segments produces `2^11` candidates per
  file before lookup.
- LS-Lint appears to avoid paying that cost for this shape.

Required regression coverage:

- Add a performance fixture with long multipart extensions.
- Add a unit test or benchmark guard proving candidate generation is linear or
  bounded for long suffixes.
- Re-run release evidence after the fix and compare against native LS-Lint from
  the pinned package.

### 5. Many configured child scopes can be slower than LS-Lint

Fixture: `/private/tmp/assura-lslint-many-scopes-QpnXT9/project`

`.ls-lint.yml` shape:

```yaml
ignore:
  - .assura
ls:
  .dir: kebab-case
  pkg-0000:
    .js: kebab-case
  pkg-0001:
    .js: kebab-case
  # ... through pkg-0799
```

Tree: `800` package directories with one matching JS file each.

Both tools exited `0`.

Rebuilt `assura-check` release timings:

```text
0.14s
0.14s
0.14s
0.15s
0.15s
```

Native LS-Lint timings:

```text
0.06s
0.06s
0.06s
0.06s
0.06s
```

Impact:

- The checked-in standard performance report still shows Assura faster on the
  current fixture suite.
- This synthetic fixture shows that the current suite does not cover every
  LS-Lint-valid shape that matters for compatibility claims.
- Root `.dir` disables the LS-Lint fast path in the current implementation,
  which likely contributes to this gap.

Required regression coverage:

- Add a many-scope fixture to the performance suite.
- Keep the claim gate based on release binaries and pinned native LS-Lint,
  not debug/cargo timings.
- Verify whether `.dir` can remain fast-path-compatible without losing root and
  descendant correctness.

## Review Agent Findings

The review agent independently found:

- Targeted directory mismatch for `.ts: kebabcase`.
- Multipart extension performance gap for
  `.a.b.c.d.e.f.g.h.i.j.k.js`.

Local verification confirmed both, corrected the command shape to
`assura check --config ...`, and added the scalar-key and root `.dir exists`
correctness gaps above.

A follow-up independent review after the first closure pass found two additional
compatibility blockers:

- Non-`exists` scalar path keys such as `src: kebab-case` were still converted
  into child file-naming policy even though LS-Lint treats them as a no-op.
- `--ls-lint-target-semantics` still enforced direct child count rules for a
  targeted directory even though LS-Lint skips those child checks for a directory
  path argument.

Both findings are now covered by
`tests/ls_lint_rule_coverage_tests.rs`.

## Closure Evidence

The checked closure evidence lives in `benches/history/current.json`, mirrored
exactly to `website/public/data/performance/current.json`. It was generated by
the same performance-report path used for the public website artifact:

```bash
cargo build --release -p assura --bins -p assura-check-cli
target/release/assura performance-report \
  --include-external-fixtures \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cmp -s benches/history/current.json website/public/data/performance/current.json
```

The counterexample rows in
`benches/history/current.json` now show:

```text
multipart_extension_regression: assura-cli 8.682 ms, ls-lint-cli 11.904 ms
many_configured_scopes_regression: assura-cli 25.513 ms, ls-lint-cli 42.834 ms
```

The same checked report has row-level warm 2x evidence for all eight
realistic-equivalent fixtures, including `0.100 ms` for the multipart-extension
counterexample and `1.939 ms` for the many-scope counterexample. The public
headline claim summary is driven by the real-repo cohort: cold `assura-cli` is
faster than LS-Lint on 10 of 10 real repositories, the universal cold 2x gate
remains explicitly `not-complete` at 7 of 10, and the warm/editor-session 2x
gate is complete at 10 of 10.

## Completion Criteria For Closure

The compatibility/performance closure work is complete when:

- All confirmed correctness gaps above have failing regression tests before the
  fix and passing tests after the fix.
- Assura-extended exact file/directory policy remains available through
  converted LS-Lint-style notation and native config, with evidence clearly
  labeling that behavior as stronger than upstream LS-Lint 2.3 semantics.
- Targeted file and directory checks have explicit LS-Lint compatibility tests.
- Release timing evidence includes the multipart-extension and many-scope
  fixtures, and Assura is not slower than native LS-Lint on those fixtures.
- Existing standard performance fixtures remain green and preserve the checked
  real-time/editor-session 2x claim gate. The local macOS cold CLI summary
  remains a separate non-complete diagnostic until a dedicated cold-start goal
  closes it with implementation evidence.
