---
id: goal-assura-real-repo-ls-lint-performance-evidence
type: goal
title: Assura real-repo LS-Lint performance evidence
status: complete
created: 2026-05-26
owners:
  - assura-maintainers
related:
  - docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md
  - docs/goals/assura-ls-lint-counterexample-closure.md
  - website/src/content/docs/reference/performance.mdx
  - website/src/content/docs/reference/performance-test-cases.mdx
  - src/cli/performance_report/fixtures.rs
  - src/cli/performance_report/external_fixtures.rs
  - src/cli/performance_report/external_fixture_catalog.rs
  - src/cli/performance_report/external_fixture_scenarios.rs
  - tests/ls_lint_realistic_fixture_manifest.yml
  - docs/analysis/2026-05-26-real-repo-ls-lint-performance-candidates.md
---

# Goal: Assura Real-Repo LS-Lint Performance Evidence

## Objective

Make the public LS-Lint performance comparison real-repo-backed and faster than
native LS-Lint on every headline case.

The comparison must use at least ten pinned open-source repositories. Each
repository must have its own feature-rich LS-Lint config that a skilled LS-Lint
user would plausibly write to keep an opinionated project structure consistent
and to prevent unexpected files or folders from being committed. Generated
fixtures and adversarial regression fixtures can remain useful diagnostics, but
they must not be the main evidence behind a user-facing aggregate performance
claim.

For this goal, a real-repo headline case is complete only when Assura is faster
than native LS-Lint for both:

- the cold one-shot CLI check path, and
- the warm/editor-session check path.

## Current Finding

The real-repo performance evidence objective is complete as of the checked
2026-05-26 benchmark and website artifact. The PR source of truth is
`benches/history/current.json`, mirrored exactly to
`website/public/data/performance/current.json`; older `target/performance`
smoke artifacts are local corroborating evidence only.

- The opt-in external performance report includes ten pinned open-source
  repositories in the `real-repo-headline` cohort.
- Every headline real-repo case records `source_type="external-pinned-repo"`.
- Every real-repo headline case has passing native LS-Lint, cold Assura, cold
  `assura-check`, and warm editor-session rows.
- Assura is faster than native LS-Lint on all ten real repositories for both
  cold one-shot checks and warm/editor-session checks.
- The checked website artifact reports a 6.88x aggregate speedup for cold
  `assura-cli` and a 56.14x aggregate speedup for the warm session row.
- The stricter universal cold 2x claim remains a separate incomplete claim:
  this artifact reports 7 of 10 cold `assura-cli` rows at 2x. Warm session
  2x evidence is complete at 10 of 10.

Diagnostic classification: `assura-check-compiled-cli` remains a non-headline
diagnostic row in this goal. It is excluded from the completion claim because
the accepted evidence gates are native LS-Lint, cold `assura-cli`, and warm
editor-session rows. Five compiled-config diagnostic rows still skip with exit
2 and should be handled by a separate compiled-config hardening goal before
that mode is presented as public comparison evidence.

## Required Work

1. Define the public fixture taxonomy.
   - Add or document a headline cohort for pinned real repositories.
   - Keep generated policy fixtures as supporting evidence.
   - Keep synthetic and adversarial cases as diagnostic/regression evidence.
   - Ensure aggregate summaries can be read without benchmark jargon.

2. Promote real pinned repositories into the comparison evidence.
   - Include at least ten pinned real repositories in the performance report
     used for public evidence.
   - Cover meaningfully different shapes, such as frontend monorepos,
     framework/package libraries, Rust projects, docs-heavy projects, CLIs,
     app repos, and generated-output-heavy repos.
   - Record repository URL, immutable revision, resolved commit, file counts,
     ignored-path counts, directory counts, rule counts, and config source.

3. Use realistic LS-Lint policies.
   - Policies should use LS-Lint features a capable LS-Lint user would use:
     `.dir`, directory scopes, wildcard and multipart extensions, regex, ignore
     rules, and `exists` counts where they naturally express structure.
   - Every repository must have a distinct config tailored to that repo's
     layout, not one generic policy copied ten times.
   - Configs must include structure-control rules that prevent accidental
     unexpected files or folders where LS-Lint can express that contract, such
     as `.dir` regex whitelists, extension bans, exact scoped rules, and
     generated-output ignores.
   - Do not include Assura-only extensions in native LS-Lint headline rows.
   - Keep configs readable enough that users can understand the project policy.

4. Add real tests for the real-repo path.
   - Keep unit tests for fixture enumeration and metadata.
   - Add materialization tests that prove pinned revisions, cache reuse, symlink
     handling, and config writing.
   - Add a real-repo performance smoke that can run explicitly and produces
     rows for the pinned fixtures.
   - Add a checked artifact or reproducible command that proves the public
     aggregate includes real-repo rows.

5. Update the website experience.
   - The performance summary should say "real pinned repositories" only when the
     rendered data includes them.
   - Test-case pages should use plain labels such as "Next.js monorepo" and
     "Rust docs project" instead of internal fixture jargon.
   - Generated/adversarial cases should be clearly labeled as support cases, not
     as the headline proof.

## Acceptance Checks

Run and record:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
cargo build --release -p assura --bins
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --include-external-fixtures \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cmp -s benches/history/current.json website/public/data/performance/current.json
```

Evidence must show:

- public/headline real-repo rows include `source_type="external-pinned-repo"`;
- at least ten headline real-repo cases have both `assura-cli` and
  `ls-lint-cli` rows;
- every headline real-repo case has a warm/editor-session Assura row;
- Assura is faster than native LS-Lint on every headline real-repo case for the
  cold one-shot CLI path;
- Assura is faster than native LS-Lint on every headline real-repo case for the
  warm/editor-session path;
- aggregate comparison results are computed from the real-repo headline cohort,
  not only generated fixtures;
- generated and adversarial fixtures remain available as diagnostic support;
- website wording matches the data actually rendered.

If external fixtures are too slow for default local runs, the PR must still
include a reproducible full-evidence command and a checked or linked artifact
that reviewers can inspect without trusting conversational claims.

## Review Criteria

Block completion if:

- the headline aggregate is still generated-only;
- fewer than ten real-repo fixtures are measured;
- any headline real-repo fixture is slower than native LS-Lint on cold or warm
  checks;
- real-repo fixtures are only declared but never measured;
- LS-Lint configs look like artificial microbenchmarks rather than project
  policy;
- multiple real repos share the same generic config instead of repo-specific
  policies;
- configs fail to express closed-world or unexpected-file/folder guardrails
  where LS-Lint supports them;
- generated stress fixtures are presented as real repository proof;
- public docs hide the distinction between cold CLI, warm editor-session, and
  diagnostic rows.

## Handoff Prompt

```text
Execute docs/goals/assura-real-repo-ls-lint-performance-evidence.md.

Start by reading AGENTS.md, .agents/skills/assura-performance-reporting/SKILL.md,
docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md,
website/src/content/docs/reference/performance.mdx,
website/src/content/docs/reference/performance-test-cases.mdx,
src/cli/performance_report/fixtures.rs,
src/cli/performance_report/external_fixtures.rs,
src/cli/performance_report/fixture_metadata.rs, and
tests/ls_lint_realistic_fixture_manifest.yml.

The goal is not more synthetic benchmark coverage. Implement at least ten
pinned real repositories with distinct, feature-rich LS-Lint policies that
control project structure and prevent unexpected files/folders where LS-Lint can
express that contract. Make those real repositories drive the user-facing
LS-Lint performance comparison. Assura must be faster than native LS-Lint on
every headline real-repo case for both cold one-shot checks and warm/editor
session checks. Keep generated/adversarial fixtures as diagnostics, and produce
measurable evidence that a reviewer can inspect.
```

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-05-26 | Tightened goal scope from a small real-repo proof to the requested 10 pinned real repositories, each with a unique feature-rich LS-Lint policy, and explicit cold plus warm faster-than-LS-Lint gates. | `docs/goals/assura-real-repo-ls-lint-performance-evidence.md` |
| 2026-05-26 | Captured an initial 10-repo candidate set with immutable refs and intended policy shapes so implementation can start from concrete repositories instead of synthetic placeholders. | `docs/analysis/2026-05-26-real-repo-ls-lint-performance-candidates.md` |
| 2026-05-26 | Expanded the opt-in external performance fixture catalog to 10 pinned real repositories, switched external fixtures to generate Assura configs from each repo's LS-Lint policy, added unique policy bodies and metadata for every candidate, and introduced a `real-repo-headline` cohort that takes over claim summaries when real-repo rows are present. This is infrastructure progress only; measured cold/warm faster-than-LS-Lint evidence is still required before completion. | `src/cli/performance_report/fixtures.rs`; `src/cli/performance_report/external_fixtures.rs`; `src/cli/performance_report/fixture_metadata.rs`; `src/cli/performance_report/claim_summary.rs`; `tests/ls_lint_realistic_fixture_manifest.yml`; `cargo test --lib performance_report -- --nocapture`; `cargo test --test ls_lint_parity_regression_tests realistic_fixture_manifest_is_pinned_and_complete -- --nocapture` |
| 2026-05-26 | Split the external fixture catalog/scenario list into separate modules so Assura's own file-length policy passes, broadened repo-specific regex/ignore policies to match real framework and fixture naming conventions, rebuilt the full companion binary, and produced a 5-iteration real-repo evidence artifact. All ten real repositories pass native LS-Lint, cold Assura, cold `assura-check`, and warm session rows. Cold Assura is faster than native LS-Lint on every repo; warm session is faster on every repo. The real-repo aggregate is 3.37x for cold `assura-cli`, 3.37x for cold `assura-check-cli`, and 1483x for warm session. The stricter cold 2x claim remains incomplete, so this is performance-evidence progress rather than goal completion. | `src/cli/performance_report/external_fixture_catalog.rs`; `src/cli/performance_report/external_fixture_scenarios.rs`; `src/cli/performance_report/fixture_metadata.rs`; `target/performance/real-repo-ls-lint-evidence.json`; `target/performance/real-repo-ls-lint-evidence.jsonl`; `target/performance/real-repo-ls-lint-evidence-website/`; `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --quiet`; `cargo run --quiet -- check --format json .`; `cargo test --lib performance_report -- --nocapture`; `cargo test --test ls_lint_parity_regression_tests realistic_fixture_manifest_is_pinned_and_complete -- --nocapture`; `target/release/assura performance-report --include-external-fixtures --output target/performance/real-repo-ls-lint-evidence.json --history target/performance/real-repo-ls-lint-evidence.jsonl --website-dir target/performance/real-repo-ls-lint-evidence-website --iterations 5` |
| 2026-05-26 | Published the checked website performance data from the 10 real-repo cohort and expanded the performance test-case page with per-repo metrics, project-specific structure notes, and a policy-protection summary explaining why each LS-Lint config is a meaningful guardrail for that repository. Current checked website data shows 10 of 10 cold faster-than-LS-Lint rows and 10 of 10 warm faster-than-LS-Lint rows. | `website/public/data/performance/current.json`; `website/public/data/performance/ls-lint-comparison-history.jsonl`; `website/src/components/performance-evidence.astro`; `website/src/content/docs/reference/performance.mdx`; `website/src/content/docs/reference/performance-test-cases.mdx` |
| 2026-05-26 | Verified the updated website docs build and render the real-repo summaries from the checked data. Assura's own structure check remains clean. | `cargo run --quiet -- check --format json .`; `npx pnpm@10.25.0 build`; `website/dist/reference/performance/index.html`; `website/dist/reference/performance-test-cases/index.html` |
| 2026-05-26 | Addressed the review-agent finding that the first real-repo configs were too permissive by adding repo-specific scoped source rules, required native LS-Lint direct-count checks where they match the pinned tree, and explicit foreign-language `exists:0` bans. Regenerated the checked website evidence from rebuilt release binaries. All ten real repos are still faster than native LS-Lint on cold and warm paths; the stricter universal cold 2x claim remains incomplete. | `src/cli/performance_report/external_fixture_catalog.rs`; `src/cli/performance_report/fixture_metadata.rs`; `website/public/data/performance/current.json`; `target/release/assura performance-report --include-external-fixtures --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 5` |
| 2026-05-26 | Added a materialization regression test proving pinned revision checkout, cache reuse without the upstream remote, symlink preservation, LS-Lint config writing, generated Assura config writing, and source revision recording for external fixtures. | `src/cli/performance_report/external_fixtures.rs`; `cargo test --lib performance_report::external_fixtures -- --nocapture` |
| 2026-05-26 | Regenerated an intermediate target artifact after fresh release builds. It confirmed 10 `source_type="external-pinned-repo"` headline cases with passing LS-Lint, cold Assura, and warm session rows, but it is superseded for PR review by the checked benchmark and website artifacts below. | `target/release/assura performance-report --include-external-fixtures --output target/performance/real-repo-ls-lint-evidence.json --history target/performance/real-repo-ls-lint-evidence.jsonl --website-dir target/performance/real-repo-ls-lint-evidence-website --iterations 5` |
| 2026-05-29 | Verified the checked website artifact after PR preparation. It includes 10 pinned real repositories, all `source_type="external-pinned-repo"`, matching benchmark and website `current.json` files, 10 of 10 cold faster-than-LS-Lint rows, 7 of 10 cold 2x rows, and 10 of 10 warm 2x rows. | `jq -r ... benches/history/current.json`; `cmp -s benches/history/current.json website/public/data/performance/current.json`; `cargo fmt --all -- --check`; `git diff --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --quiet`; `cargo run --quiet -- check --format json .`; `cd website && PATH=/usr/local/bin:$PATH ASTRO_TELEMETRY_DISABLED=1 ./node_modules/.bin/astro build` |
