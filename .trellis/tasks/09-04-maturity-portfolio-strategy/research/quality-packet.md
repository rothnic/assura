# Rust quality and contribution work packet

Follow [execution constraints](execution-backlog.md). These cards change maintainability through small, demonstrated improvements. Record consumer/churn evidence before structural refactors; if no benefit is demonstrated, mark `not_needed` with that evidence instead of performing cosmetic churn.

## Q01

**Outcome:** Implementers receive concise Rust-specific decision guidance. **Create:** `.agents/skills/assura-rust-quality/SKILL.md`, `references/architecture.md`, `errors-and-effects.md`, `performance.md`, `tests-and-review.md`. **Modify:** AGENTS routing and the skill allowlist in `.assura/config.yml`.

- [ ] Read the skill-creator instructions available in the execution environment. Use the following as the core skill, expanding only the reference files with actual local examples:

```markdown
---
name: assura-rust-quality
description: "Rust changes to Assura: preserve policy contracts and reduce maintenance cost."
---
# Assura Rust Quality
1. Read the changed surface's spec and one existing canonical example.
2. State the observable contract, invariant and ownership boundary.
3. For config/model work, read references/architecture.md.
4. For paths, reports or subprocesses, read references/errors-and-effects.md.
5. For traversal/cache/concurrency, read references/performance.md.
6. Add a positive, negative and valid-exception case for validation changes.
7. Keep one authoritative model per semantic role and canonical item path.
8. Preserve errors, checked scope, deterministic ordering and exit behavior.
9. Do not split files or weaken policy merely to pass a numeric limit.
10. Read references/tests-and-review.md; run focused proof and the proper tier.
Report exact checks, remaining risks and any policy/dependency change.
```

- [ ] Architecture reference: map current config loader → prepared/compiled checker → report → CLI/harness, naming the actual types. Errors reference: document cache fallback versus lost report/subprocess failure. Performance reference: existing report commands and equivalent-path rules. Tests reference: choose a real meaningful positive/negative fixture from this repo and contrast it with an experimental constant-restatement test.
- [ ] Link selected [Microsoft Rust AI guidelines](https://microsoft.github.io/rust-guidelines/guidelines/ai/index.html) and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/); record source date. Do not inject the entire external guide or install another agent router/hook system.
- [ ] Register one AGENTS routing row. Validate skill metadata and structure. Ask an independent agent to identify the right reference for config, subprocess and performance examples using only the index; correct ambiguous routing.

**Accept:** Index is at most 150 lines, each reference exists, no duplicated Trellis workflow, project check passes, and routing works without reading all references. A later Q03/Q07 change records whether the skill prevented a concrete error; installation alone is not evidence of efficacy.

## Q02

**Outcome:** A contributor can make a small trustworthy change without learning internal orchestration, and an agent cannot silently weaken the merge standard. **Own:** `CONTRIBUTING.md`, `AGENTS.md`, `.github/PULL_REQUEST_TEMPLATE.md`, issue templates, scoped CI/evidence guards. **Create:** `.github/CODEOWNERS` if absent; `docs/contributing-agent-changes.md`.

- [ ] Replace the stale main-branch/Rust-1.70/v0.1 instructions with verified values and the existing validation tiers. Link release procedures rather than repeating tag-push commands. Separate the 5-minute external contributor path from maintainer Trellis details.
- [ ] Add a concise PR section with these exact fields: user problem; behavior changed; reproducer; exact validation; known gaps; material AI assistance and verification; policy/dependency changes. Small copy fixes can mark nonapplicable fields briefly.
- [ ] Add a contribution rule: author owns understanding and verification; no fabricated tests/evidence; no private prompt transcript required. New syntax, commands, dependencies and support promises need an accepted design card. Reject bulk unrelated refactors.
- [ ] Route configuration, CI/release/install and core-check changes to `@rothnic` using CODEOWNERS after confirming the account can review. Require independent review of test deletions, exclusions, severity reductions, changed performance thresholds and CI scope changes. Reuse `cargo xtask evidence`; add a changed-path summary that flags these changes for review, not an NLP-based auto-approval system.
- [ ] Inspect branch protection read-only. Prepare exact required checks/owner-review settings and evidence that required checks cannot be skipped by path filtering. Apply settings only with repository-admin authority; local hooks are insufficient.
- [ ] Verify PR author and reviewer identities before requiring owner approval: GitHub does not let Nick approve his own PR. Use an already authorized contributor/bot PR identity with Nick reviewing, or an available independent maintainer. Do not enable an unsatisfiable sole-owner gate, invent another account, grant new credentials or silently bypass review. Record the chosen enforceable route.
- [ ] Have a fresh agent follow CONTRIBUTING for one small fixture/docs fix. Record missing prerequisites, actual test command and time. Repair instructions based on that attempt.

**Accept:** Correct branch/toolchain instructions; contributor does not need chat history; a policy-weakening test fixture is visibly flagged; admin settings are verified or explicitly pending. Agents may commit owned work under repo policy; commit permission does not imply merge/tag/deploy permission.

## Q03

**Outcome:** One authoritative current configuration interpretation, with legacy roles explicit. **Own:** `src/config/mod.rs`, `ast.rs`, `types.rs`, `parser.rs`, `config.rs`, `loader.rs`, `src/cli/commands.rs`; related tests. **Create:** `tests/config_authority_contract.rs` and a short model-role map in the existing config spec.

- [ ] Use `rg` to enumerate consumers of the three Config types and ConfigParser/ConfigLoader. Classify each as supported runtime, compatibility test, experimental command or unused. Record why any two representations must remain.
- [ ] Make `config::config::Config` + `ConfigLoader` authoritative for current `structure` notation, because check already uses them. Route current `info`/configuration inspection through the same loader and summarize actual structure/rules. Do not make inspection parse a different language from validation.
- [ ] Write an integration fixture with `structure`, a reusable rule, an invalid naming case and an explicit config path outside cwd. Assert check/status/explain/inspection agree on the loaded file and rules; malformed YAML must fail clearly. Preserve supported LS-Lint migration through its own named adapter.
- [ ] If `ast::Config` or `types::Config` remains needed, rename/document it by its actual legacy/experimental role and keep conversions at one boundary; move compatibility-only code behind its test/feature boundary. Delete only confirmed unreachable duplicates and update imports rather than adding alias ladders.
- [ ] Run `cargo test --test config_authority_contract`, `--test cli_compact_notation_tests`, `--test structure_config_notation_tests`, loader/parser unit tests, migration tests and default/minimal feature builds.

**Accept:** A contributor can identify the sole current parser; no supported command silently ignores a different schema; retained compatibility code has named consumers. **Boundary:** removing a documented user syntax or changing report schema requires an explicit P01 support decision.

## Q04

**Outcome:** A first meaningful policy-domain boundary and normal module interfaces instead of textual splitting. **Own:** one selected cohesive family under `src/cli/check` or `src/config/config/structure_notation`; `src/lib.rs`; Create `src/policy/mod.rs`, `src/policy/naming.rs` only if selecting the naming extraction.

- [ ] Use recent Git history and the Q03 consumer map to choose one family. Recommended first slice: pure naming predicates currently in `src/cli/check/case.rs`; they are separable from filesystem/CLI effects. If textual config fragments have the demonstrated higher cost, choose that family instead and record why.
- [ ] For naming: move the existing functions unchanged into `pub(crate) mod policy`, retain their signatures with `pub(crate)` visibility, and update internal consumers directly. Do not expose a new public Rust API or leave old/new public aliases. Keep report/CLI types out of the new pure module.
- [ ] For textual fragments: replace the selected `include!` family with normal `mod` declarations and explicit `pub(super)` imports. Keep tightly coupled code together when a split would require broadly public fields. No “common/utils” catchall.
- [ ] Characterize behavior before moving: snake/kebab/Pascal cases, regex alternatives, multipart extensions, path-aware patterns and valid exceptions. Add a differential integration test comparing plain and fast check normalized findings on these fixtures. Preserve ordered rule/path/severity/message data; omit only runtime timings and temp-root prefixes.
- [ ] Run relevant naming/notation tests, differential tests, default/minimal builds, and the performance row affected by the extraction. Review exported symbols for a single intended path.

**Accept:** One cohesive change demonstrably clarifies ownership, no new duplicate public path, no behavioral/performance regression. Do not move the entire checker merely to obtain a fashionable directory tree. Remaining families stay queued for evidence-led decisions, not silently considered fixed.

## Q05

**Outcome:** No implication that file presence proves project maturity. **Own:** `src/maturity/*`, its consumers in CLI/constraints, experimental docs and support manifest, `tests/maturity_tests.rs` as necessary.

- [ ] Enumerate runtime and test consumers of `MaturityReport`, `MaturityLevel`, `MaturityDecisionEngine`. Keep the published support contract visible.
- [ ] Default solution: freeze and isolate the experimental scorer; remove it from onboarding recommendations and public quality/maturity messaging. Expose observed states such as `ci_config_present` separately from `ci_execution_verified`; absence of remote evidence is `unverified`, not failure or success.
- [ ] If no runtime consumer remains, remove the scorer and its constant-restatement tests in one reviewed patch. If a supported consumer exists, retain the API until an explicit deprecation decision and document its limitations; do not perform a breaking deletion automatically.
- [ ] Add behavioral cases: empty `.github/workflows` does not imply verified CI; `pyproject.toml` without `[tool.black]` does not imply Black configured; more package-manager files do not by themselves improve quality. Use real temp files with independent expected states.
- [ ] Keep exact-value tests for real serialized public contracts; remove tests only when their behavior/contract is retired or replaced by stronger coverage.

**Accept:** No supported output presents the old proxy as proven maturity; experimental retention/removal is explicit and tested. Existing supported structural constraints still work.

## Q06

**Outcome:** Reduce the cost of one high-churn maintenance command without rebuilding xtask. **Own:** `xtask/src/main.rs` and one selected command module; alternatively one cohesive `src/cli/performance_report` input group.

- [ ] Inspect churn and recent bug fixes. Default first extraction is the release-readiness group (`run_release_readiness`, report construction, release-surface interpretation and associated tests) into Create `xtask/src/release_readiness.rs`. Keep `main.rs` as dispatch and move functions/tests unchanged first.
- [ ] Define one `pub(crate) fn run(args: &[String]) -> Result<()>` entry using the existing xtask Result alias. Keep shared process helpers at the narrowest existing module boundary; do not create a general task framework.
- [ ] Add/retain CLI contract tests: `--help` still names all commands; missing mode preserves exit 2 and unknown mode preserves the current nonzero error behavior; release-readiness produces the same schema; missing input errors are explicit. Compare prior/current sample JSON after removing only time-sensitive live fields.
- [ ] If repeated performance arguments are the selected cost, replace one repeated argument group with a named input struct whose fields are the existing values; constructors must not change defaults. Do not remove all `too_many_arguments` allowances mechanically.
- [ ] Run `cargo test -p xtask`, `cargo xtask evidence`, `target-state`, and the selected command on deterministic inputs. Behavior remains unchanged.

**Accept:** Reduced dispatch-file responsibility and a clear command-owned test boundary, or a measured reduction in argument mistakes. No blanket file-length target determines success.

## Q07

**Outcome:** Missing companion, failed spawn and output failure have distinct honest behavior. **Own:** `src/main.rs`, `src/cli/agent_onboarding_report.rs`, affected integration renderers; Create `tests/launcher_error_contract.rs`.

- [ ] Refactor `run_companion` from `Option<i32>` to `std::io::Result<Option<i32>>`. `Ok(None)` means genuinely absent companion; `Err` means a present companion could not launch. Preserve fallback to compiled-in full CLI only for absence. On spawn failure, print path plus OS error to stderr and return the established runtime-error status. Reuse/confirm the existing ExitCode contract before choosing a numeric value.
- [ ] Test a minimal packaged launcher with: good companion propagates exit; absent companion reports installation guidance for a full command; present non-executable/invalid companion reports launch failure; `check` still works through its lightweight path. Use platform-appropriate fixture executables and run package smoke too.
- [ ] Change touched report serialization from `unwrap_or_default()` to a Result propagated to the command's error path. On serialization or output failure, never emit an empty success payload. Add a test with an injected failing writer where the renderer supports writing; do not add a generic framework just to simulate impossible serialization for one fixed struct.
- [ ] Audit selected cache/cleanup `.ok()`/`let _ =` sites. Preserve intentional cache-miss fallback, but make fallback reason observable through doctor/diagnostics where needed. Invalid cache must lead to real fresh validation; failed check cannot become a cache hit.
- [ ] Run launcher/report tests, `agent_integration_cli`, `agent_onboarding_config_merge`, minimal/full builds and release-smoke.

**Accept:** Failure diagnostics identify the failed operation; missing/failed companion are distinguishable; a corrupt cache does not suppress a seeded violation; valid output schemas and check exit codes are preserved.
