---
id: analysis-2026-05-09-project-assessment-and-alignment
type: analysis
title: Assura project assessment and alignment
status: historical
created: 2026-05-09
updated: 2026-05-09
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-05-09-trellis-governance-adr.md
  - docs/analysis/2026-05-09-documentation-cleanup-register.md
---

# Assura Project Assessment and Alignment

Date: 2026-05-09
Branch: `master`
Scope: Rust crate, CLI, config systems, validation engines, website docs, CI/release workflows, self-validation config, OpenCode plugin package, planning/status docs.

> Historical snapshot: this report captures the pre-remediation state verified
> before the 2026-05-09 self-enforcement work began. Use current command output,
> `.trellis/spec/assura/index.md`, and `.assura/config.yml` for the active
> implementation state after this date.

## Executive Summary

Assura is best understood today as a strong validation-engine prototype/library with several useful subsystems, not yet as the production-ready CLI product described by the README, website, release notes, and phase summaries.

The core Rust modules have meaningful implemented value: naming constraints, file-size constraints, LS-Lint parsing/conversion internals, markdown parsing/validation, maturity detection, graph building/querying, hooks management, and a broad test suite. The public user journey is not aligned with that code reality. The primary CLI commands advertised to users, `check`, `status`, `init`, and `watch`, currently print "not yet implemented" and return success. The code contains internal `migrate` and `info` helpers, but the actual Clap command enum does not expose `assura migrate` or `assura info`.

The highest-risk issue is trust: a user following current docs will believe they installed a working validator, but the core commands do not validate. A maintainer following current CI expectations will believe Clippy and formatting are enforced, but both fail locally. The documentation set also has multiple competing narratives: "production ready", "phase complete", "actual state audit", "implementation gaps", two incompatible config formats, website guides using unsupported commands, and release notes claiming capabilities not yet wired into the CLI.

Recommended posture until remediation: present Assura as a pre-1.0 validation engine and CLI-in-progress, make this file the canonical state report, archive or demote older phase-complete summaries, and prioritize a small honest CLI before adding new validation features.

## Verification Snapshot

Commands run from `/Users/nroth/workspace/assura` on 2026-05-09.

| Command | Result | Notes |
| --- | --- | --- |
| `cargo test --all-targets` | Pass | Library test run reported 292 passed and 3 ignored; integration suites passed; bench harness targets executed as test binaries. The run emitted many warnings. |
| `cargo fmt --all -- --check` | Fail | Rustfmt produced diffs across benches, source, and tests. |
| `cargo clippy --all-targets -- -D warnings` | Fail | Failed with unused imports/variables/dead code and Clippy lints; output reported 81 lib errors and 88 lib-test errors. |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | Pass | Rustdoc generated successfully. |
| `./target/debug/assura --help` | Pass | Exposes `check`, `status`, `init`, `watch`, `hooks`, and `help`. |
| `./target/debug/assura check` | Stub | Prints `Check command not yet implemented` and exits success. |
| `./target/debug/assura status` | Stub | Prints `Status command not yet implemented` and exits success. |
| `./target/debug/assura init` | Stub | Prints `Init command not yet implemented` and exits success. |
| `./target/debug/assura watch` | Stub | Prints `Watch command not yet implemented` and exits success. |
| `./target/debug/assura migrate --help` | Fail | Clap reports `unrecognized subcommand 'migrate'`. |
| `./target/debug/assura info --help` | Fail | Clap reports `unrecognized subcommand 'info'`. |
| `bun install --frozen-lockfile && bun test && bun run build` in `opencode-plugin/` | Not verified | `bun` is not installed in this environment. |
| `pnpm install --frozen-lockfile && pnpm build` in `website/` | Not verified | `pnpm` is not installed in this environment. |

## Product Objective

The project appears to be aiming for a dependency-aware filesystem validation tool with:

- A fast Rust CLI for project validation, watching, hooks, and CI.
- Structure-first configuration that mirrors the project tree.
- LS-Lint migration and compatibility as an adoption path.
- Built-in constraints for naming, size, line counts, required files, markdown structure, and possibly dependency analysis.
- Project maturity detection for progressive enforcement.
- Agent-oriented workflows, including OpenCode integration and self-validation.

That is a coherent product direction. The implementation is not yet at the point where the primary user journey works end to end.

Recommended product narrative for now:

> Assura is a pre-1.0 Rust validation engine with working internal validators and experimental CLI/docs. The next release should focus on a small reliable CLI: load one canonical config format, walk files, validate constraints, return correct exit codes, and print honest output.

## Current-State Truth Table

| Area | Docs claim | Code reality | Evidence | User risk |
| --- | --- | --- | --- | --- |
| CLI `check` | README and release notes say `assura check` validates a project. | Stub prints "not yet implemented" and exits success. | `src/cli/commands.rs:13-22`; CLI run. | Critical: false success in local/CI validation. |
| CLI `status` | Advertised as project status command. | Stub prints "not yet implemented" and exits success. | `src/cli/commands.rs:24-28`; CLI run. | High: no project health visibility. |
| CLI `init` | README quickstart says `assura init`. | Stub prints "not yet implemented" and exits success. | `src/cli/commands.rs:30-38`; CLI run. | High: first-run setup fails silently. |
| CLI `watch` | README/release notes/website claim watch mode. | Stub prints "not yet implemented" and exits success. | `src/cli/commands.rs:40-48`; CLI run. | High: advertised development workflow absent. |
| CLI `migrate` | Phase docs and config docs claim `assura migrate`. | Helper exists, but command is not registered in `Commands`. | `src/cli/commands.rs:80-118`; `src/cli/args.rs:22-83`; CLI run. | Medium: migration adoption path inaccessible through binary. |
| CLI `info` | Phase docs claim `assura info`. | Helper exists, but command is not registered in `Commands`. | `src/cli/commands.rs:120-149`; `src/cli/args.rs:22-83`; CLI run. | Medium: debug/documentation command inaccessible. |
| Directory validation path | Product describes project-wide validation. | There is no CLI traversal path in `check`; TODO remains. | `src/cli/commands.rs:72-75`; `docs/archive/actual-state-audit.md`. | Critical: core value proposition missing from CLI. |
| `exists` constraint | Specs and LS-Lint docs describe required/forbidden file counts. | AST validator always passes `exists`. | `src/validation/constraints.rs:59-62`; `docs/archive/implementation-gaps.md:72-85`. | High: required files can be missing without failure. |
| Pairing/cross-directory tests | Specs describe pairing by shared variables. | Pairing is standalone, not integrated; some tests ignored. | `src/validation/pairing.rs`; cargo test ignored pairing tests. | High: documented test/doc pairing enforcement not reliable. |
| Config formats | Docs describe V1, V2, AST/rules-policy, and structure-first variants. | Two incompatible parser families exist; active self-config is structure format; AST parser serves other internals/tests. | `.assura/config.yml`; `.assura/config.new.yml`; `docs/archive/actual-state-audit.md:26-41`. | High: users and implementers cannot tell which format is canonical. |
| Formatting standard | AGENTS and CI require `cargo fmt --check`. | Formatting check fails. | `.github/workflows/ci.yml:33-43`; command result. | Medium: CI would fail or local quality bar is aspirational. |
| Clippy standard | AGENTS and CI require no warnings. | Clippy with `-D warnings` fails heavily. | `.github/workflows/ci.yml:45-65`; command result. | Medium: CI would fail; warning debt hides real issues. |
| Tests | Docs say broad tests pass. | Rust tests do pass, with warnings and ignored pairing tests. | `cargo test --all-targets`. | Low: useful coverage exists, but it does not cover CLI user truth. |
| Rust docs | CI expects rustdoc with warnings denied. | Rustdoc passes. | `cargo doc` command result. | Low. |
| Website docs | Website guides use `assura validate`, completions, migration commands, and watch. | Current CLI has no `validate`, `completions`, `migrate`, or `info`; `watch` is stubbed. | `website/src/content/docs/guides/getting-started.md:58-78,147-173`; `website/src/content/docs/guides/quickstart.md:32-44`; CLI help. | Critical: public docs teach unsupported commands. |
| Release notes | v0.1.0 notes claim production-like CLI, watch, dependency analysis, self-validation. | Primary CLI commands are stubbed; self-validation cannot work through `check`. | `docs/release-notes.md:43-60,133-138`; CLI run. | High: release credibility gap. |
| OpenCode plugin | Plugin docs claim complete implementation and tests. | Source/package exists, but local verification blocked because `bun` is missing. | `opencode-plugin/package.json`; `bun` command result. | Medium: should be treated as unverified until CI/tooling confirms. |
| Website build | Website package exists. | Local build not verified because `pnpm` is missing. | `website/package.json`; `pnpm` command result. | Medium: docs site may not build in current environment. |

## User Journey Assessment

### Install

The README and website suggest a normal `cargo install assura` journey. This should not be marketed as a working validator until the CLI is fixed. The current local binary proves the core commands do not perform validation.

### Configure

The project has too many competing configuration stories:

- README uses `version: "2.0"` with `structure:`.
- `.assura/config.yml` uses the structure-first self-validation format.
- `.assura/config.new.yml` uses `rules:` and `policy:`.
- `docs/archive/configuration-spec.md` describes an AST/rules-policy syntax with direct file keys, `apply`, `exists`, `group`, `message`, and cross-directory pairing.
- Website docs include V1-style `rules:` arrays and V2-style `structure:`.

Users need one default config format, one parser path, and one migration story. Until then, configuration documentation should be explicitly labeled experimental.

### Validate

This is the largest broken journey. A user running `assura check` receives success with no validation. Some website docs use `assura validate`, which is not a command at all. This is the top priority before any public release language.

### CI and Hooks

CI config is directionally useful, but the current code does not meet it:

- `cargo fmt --all -- --check` fails.
- `cargo clippy --all-targets --all-features -- -D warnings` is configured in CI and fails locally.
- `Cargo.lock` is ignored even though CI cache/audit references `Cargo.lock`. For a binary-focused project, this policy should be revisited.
- Hook management code exists and is exposed as `assura hooks`, but hooks that call validation cannot provide value until `check` works.

### Website Docs

The website should be treated as aspirational/stale until validated. It uses unsupported commands (`assura validate`, shell completions, migration subcommands) and duplicates contradictory configuration examples.

### OpenCode Plugin

The plugin package has a coherent TypeScript structure and test scripts, but local verification could not run because `bun` is absent. Its docs also describe it as complete. Mark it "experimental/unverified" until a repeatable build/test path exists in CI.

## Engineering Readiness

### What Is Ready Enough To Build On

- Constraint primitives for naming, size, line count, LS-Lint-style conventions, directory naming, path rules, severity mapping, and triggers.
- Markdown parser/validators and schema/template support.
- Intelligence graph builder, queries, and persistence.
- Maturity detection signals and decision engine.
- Git hook management surface.
- LS-Lint parser/converter internals, subject to CLI exposure and validation semantics.
- Test suite that catches many library-level regressions.

### What Blocks A Credible User Release

- Primary CLI commands are stubs.
- Directory traversal is not wired to validation.
- `exists`, pairing, messages, `children_limit`, and some context behaviors are parsed or modeled but not consistently enforced.
- The actual CLI does not expose documented `migrate` or `info`.
- Formatting and Clippy fail despite CI policy.
- Website and release docs overclaim.
- Config systems are not clearly owned or unified.

### Architecture Concern

The project has at least two validation/config tracks:

- `src/config/*` structure-first/legacy implementation with loader, inheritance, engine, and validator.
- `src/config/ast.rs`, `src/config/parser.rs`, and `src/validation/*` AST/rules-policy implementation.

Both contain useful pieces. The immediate decision should not be "support both"; it should be "choose one path for the CLI and docs for the next milestone." Supporting both before one is end-to-end complete will compound the documentation and maintenance gap.

## Documentation Conflict Register

| Document or area | Classification | Issue | Recommended action |
| --- | --- | --- | --- |
| `docs/analysis/2026-05-09-project-assessment-and-alignment.md` | Historical pre-remediation report | Snapshot based on fresh verification before the self-enforcement implementation. | Keep in the analysis archive; refresh with a new dated report rather than editing it into a mixed timeline. |
| `docs/archive/actual-state-audit.md` | Useful but older supporting audit | Mostly consistent with current findings, but says `migrate`/`info` work as CLI commands even though not exposed now. | Keep as evidence; label superseded by this report if editing docs later. |
| `docs/archive/implementation-gaps.md` | Useful supporting gap list | Still largely accurate for feature gaps. | Keep; eventually merge high-signal items into issue tracker/backlog. |
| `README.md` | Contradictory public docs | Quickstart advertises stubbed `init/check/watch`; performance claims not currently verified against LS-Lint. | Rewrite before public release; use honest pre-1.0 wording. |
| `docs/release-notes.md` | Contradictory/stale | Claims v0.1.0 release capabilities that are not in the binary. | Archive or rewrite as planned release notes, not actual release notes. |
| `CHANGELOG.md` | Contradictory/stale | Claims 0.1.0/0.2.0 features and releases beyond current Cargo version `0.1.0`; some CLI claims are false. | Rewrite as unreleased changelog or mark historical draft. |
| Website guides/reference | Contradictory/speculative | Uses unsupported commands and multiple incompatible config examples. | Freeze or add pre-release warning; rewrite after CLI/config decision. |
| `docs/archive/documentation-summary.md` | Stale process summary | Says V2 docs and website were completed, but content conflicts with implementation. | Archive under historical planning notes. |
| `docs/PHASE*_REVIEW.md` | Mixed historical notes | Phase-complete and no-tech-debt claims conflict with current state. | Move to `docs/archive/` or mark historical, not current state. |
| `docs/archive/configuration-spec.md` | Speculative target spec | Contains desired features not implemented, including cross-directory pairing/group behavior. | Keep as proposal/spec, not user guide. |
| `docs/config-v2.md` and `docs/migration-guide.md` | Partially stale user docs | Describe commands and migration paths not exposed by CLI. | Downgrade to design drafts until CLI supports them. |
| `.assura/config.yml` | Current self-config candidate | Structure-first config exists, but `assura check` cannot enforce it. | Keep as intended self-validation config; do not claim active self-validation until CLI works. |
| `.assura/config.new.yml` | Experimental competing config | Rules/policy format competes with structure-first config. | Keep only if AST path is selected; otherwise archive or convert. |
| `opencode-plugin/IMPLEMENTATION_SUMMARY.md` | Unverified completion note | Claims 84 tests passing, but local environment lacks `bun`. | Mark unverified or add CI proof. |
| `openspec/`, `.github/skills/`, `.github/prompts/`, `specs-bak/` | Historical/planning infrastructure | Not referenced by main docs; can confuse current process. | Document as internal planning/archive, or remove if not active. |

## Recommended Direction

### Milestone 1: Restore Trust In The Repository

Goal: make local and CI quality signals truthful before adding features.

- Fix formatting so `cargo fmt --all -- --check` passes.
- Fix or explicitly scope Clippy warnings so `cargo clippy --all-targets --all-features -- -D warnings` passes, matching CI.
- Decide Cargo.lock policy. For a CLI binary, prefer committing `Cargo.lock`; if kept ignored, remove CI/audit assumptions that depend on it.
- Add a smoke test for the actual compiled CLI behavior, not just helper methods.
- Change stub commands to nonzero "not implemented" failures if they remain stubbed, so CI/users do not get false success.
- Update README top section immediately to say "pre-1.0, CLI validation in progress" until `check` is real.

Decision: mostly fix code/tooling; downgrade docs only where they currently make false public claims.

### Milestone 2: Ship A Small Honest CLI

Goal: one working path from config to validation result.

- Choose one canonical config path for CLI v0.1-next. Recommended default: existing structure-first `.assura/config.yml`, because it matches README and self-validation config.
- Wire `assura check` to:
  - discover/load config,
  - walk target paths while honoring excludes,
  - validate naming, lines, size, markdown basics, and required files that are actually supported,
  - report structured text and JSON/YAML according to existing flags,
  - return nonzero exit codes on blocking validation failures and config/runtime errors.
- Implement minimal `assura init` that writes the canonical config format, guarded by `--force`.
- Implement `assura status` as config discovery plus supported-feature summary.
- Either implement `watch` as a thin notify wrapper over `check`, or remove/downgrade it from public docs until ready.
- Register or remove `migrate` and `info`. If kept, expose them through Clap and add CLI tests.

Decision: fix code first. Do not expand docs or features until this user journey passes.

### Milestone 3: Reconcile Docs, Specs, And Roadmap

Goal: eliminate competing documentation and give users a coherent map.

- Make this report, README, and one config reference the only current-state docs.
- Move old phase summaries, implementation summaries, and completed-release drafts into `docs/archive/` or add a clear "historical draft, not current state" banner.
- Split config docs into:
  - "Current supported config",
  - "Planned config proposals",
  - "Migration notes".
- Convert true implementation gaps into tracked issues or a concise roadmap.
- Rebuild website docs from the canonical CLI/config behavior after Milestone 2.
- Add a docs CI check that searches examples for unsupported commands such as `assura validate`, `assura completions`, and undocumented subcommands.

Decision: downgrade or archive docs aggressively; only keep user-facing claims backed by commands/tests.

## Backlog By Priority

| Priority | Item | Type | User impact |
| --- | --- | --- | --- |
| P0 | Make `check` either validate or fail nonzero as not implemented. | Code | Prevents false confidence. |
| P0 | Fix README/release notes claims about production readiness and stubbed commands. | Docs | Prevents misleading first-run experience. |
| P0 | Make fmt and Clippy pass or align CI to a documented temporary standard. | Code/tooling | Restores CI trust. |
| P1 | Choose canonical config format and archive competing active examples. | Product/docs | Reduces implementation churn and user confusion. |
| P1 | Implement file traversal and result aggregation for `check`. | Code | Enables core product. |
| P1 | Implement real `exists` enforcement for required/forbidden/count patterns. | Code | Makes config correctness meaningful. |
| P1 | Expose or remove `migrate` and `info`. | Code/docs | Aligns docs with binary behavior. |
| P2 | Integrate pairing validation or remove it from current docs. | Code/docs | Avoids broken test/document pairing promises. |
| P2 | Build/test OpenCode plugin in CI or mark experimental. | Tooling/docs | Clarifies integration readiness. |
| P2 | Add website build verification and rewrite unsupported command examples. | Tooling/docs | Restores public doc credibility. |
| P3 | Revisit performance claims with reproducible LS-Lint comparison setup. | Bench/docs | Prevents unverified marketing claims. |
| P3 | Decide fate of OpenSpec/Trellis/specs-bak planning assets. | Docs/process | Reduces repo noise. |

## Acceptance Criteria For "Aligned"

Assura should not be called aligned or production-ready until:

- `assura check` validates at least the self-configured repository and returns meaningful exit codes.
- README quickstart commands all exist and perform the described behavior.
- Website quickstart uses only supported commands.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo doc` pass locally and in CI.
- One current configuration format is documented as supported; other formats are clearly marked experimental or historical.
- Release notes and changelog describe shipped behavior, not intended behavior.
- The docs conflict register has been acted on or explicitly deferred.

## Bottom Line

The project has enough useful implementation to be worth rescuing, but its current public narrative is ahead of the product. The next engineering move should be a narrowing move: pick the CLI/config spine, make it work end to end, and make every public doc tell that same truth.
