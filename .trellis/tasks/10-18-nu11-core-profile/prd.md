# NU11: Bounded core filesystem instruction profiling

## Objective

Identify which Assura functions consume modeled instruction work on the exact
accepted 800-scope full public CLI workload. Preserve the earlier strict timing
failure (14.7796545ms vs14.7001ms), its unknown cause and bounded R03
non-reproduction. Instruction counts cannot explain that failure or clear it.

## Authority

Nick authorized Beads iteration with independent GPT-6 Luna reviews,
corrections and root-owned gated merges. Bead ae-4s5 is canonical; root owns
Beads/native/PR/push/dispatch/merge/closure. Candidate source owner nu11-core-profile.
Root approved exact source scope after independent review; tools/builds/manual
runs require separate root admission. No release/history/policy promotion.

## Scope

Existing .github/workflows/r03-performance-variance.yml optional default-preserving
mode and isolated profile job; new scripts/nu11-callgrind-profile.py; existing
src/cli/performance_report/fixture_tests.rs ignored exact constructor exporter;
existing tests/r03_performance_variance_contract_tests.py and focused
tests/nu11_callgrind_profile_tests.py. Task/context/evidence as needed.
No new Rust module/public API/dependency, performance threshold or history
change.
Keep existing scripts/r03-performance-variance.sh byte-identical and default
3x16 timing behavior unchanged. Leave old plan-only r03_callgrind_attribution.py
and old A/B tests untouched and uncredited as executor evidence.

Two narrowly scoped metadata decisions are separately authorized because the
approved files require closed-world registration: add only
scripts/nu11-callgrind-profile.py to the existing scripts/ allowed-name list,
and allow reason manual_performance_fixture_export plus its one exact ignored
test mapping for
src/cli/performance_report/fixture_tests.rs::export_nu11_accepted_many_scope_fixture
under supported_surface_tests. Preserve all existing categories, mappings,
relationships, severity and other allowlists. These entries authorize neither
profile execution nor a product-policy or performance-gate change.

## Contract

- Existing workflow enables reviewed candidate-branch dispatch before merge.
  Optional mode defaults variance; old job only absent/variance, separate
  callgrind job only selected mode. Branch ref, github.sha, full input SHA and
  checkout HEAD agree before effects; refuse dirty source or overwrite.
- Dedicated owned immutable Debian container and authenticated timestamped
  snapshot/exact Valgrind package. Normal vendor apt inside that disposable
  container is explicitly allowed; no host install/config or old R03 ABI claim.
  Bind actual architecture/image/tool/compiler/package/metadata/source inputs.
- Export existing accepted constructor only to fresh absent absolute root.
  Manifest binds every path/content/config, existing metadata/config proves801
  rules/nonempty policy; public JSON proves801files800dirs/success/no violations.
  One filename negative and guaranteed exact restore plus positive recheck.
- Same optimized production and symbol-retaining twin inputs except symbols;
  freeze all binaries/tools/helpers before profiles. No build between3profiles.
- Vendor health/true/fixed no-I/O sensitivity, exact target compatibility and
  fixed malformed-profile rejection controls run first, capture raw/exit/hash.
  Pinned callgrind_annotate only; no custom parser or line-coverage fraction.
- Exactly3 identical profiles, unique raw outputs, numeric total and exclusive
  functionIr with optional source annotation. Unresolved target functions,
  missing/incomplete artifact, drift, timeout, pressure or control failure stop
  and remain unfavorable/inconclusive; never retry/top up or infer timing/noise.
- 30s tool controls/120s target profile/128MiB output/512MiB evidence/45minjob.
  Nonsecret explicit env and resource/process/timeout cleanup evidence.
- Only after profiles pass run one unchanged full strict production matrix with
  prebuilt normal stripped/static bundle and gate executable. Preserve all rows,
  thresholds/failures and no Cargo during/after profile collection.

## Acceptance

Meaningful helper/runner/default-mode/negative contract tests and independent
source/final-readiness review before publication. Root separately admits exactly
one reviewed candidate-branch mode dispatch after resource/tool readiness review.
Retain actual health/controls/threeprofiles/rawvendoroutput/strictmatrix and
independent result review; never call source-only tests actual profiler proof.
Product timing failure remains open unless separately explained by evidence;
this candidate may deliver bounded observation without causal resolution.
Root verifies merged outcome/native closure and owned checkout/ref closure.

## Evidence and phase

Base a1d2f0aaf405a8d7e6359eb4c42aaaa4251a599e. External root:
/Users/nroth/workspace/.assura-review-20261017/nu11/
Read systemic-filesystem-profiling-protocol-v5.md, executor-implementation-readiness.md,
root-selected-implementation-scope.md and selected-implementation-scope-independent-review.md.
Source-only now; no local Cargo while NU03 owns Rust lane. Python focused negative
controls/standalone formatting/cheap task/context/diff checks allowed. No package
operation/build/profile/dispatch/native/Beads by worker. Stop unexpected gates
and retain raw before root three-competitor diagnosis.


## Source checkpoint (2026-10-20)

The optional Callgrind mode, constructor-backed ignored fixture exporter and
local bounded runner are implemented. The default variance script is unchanged.
Root accepted independent review findings for combined-output accounting,
PID/PGID launch receipts, distinct overflow/deadline causes, remaining-job
deadlines and bounded owned-process cleanup, including setup/persistence faults.
The source gate now passes 13 focused Python controls and 3 existing R03
contract tests. Earlier failed assertions, cleanup errors and missing-import
receipts are retained under external `nu11/correction-20261020`; none is hidden
or counted as a pass. Standalone Rust formatting, context, diff and source-bound
structure guards pass.

Rust exporter compilation, container/Valgrind compatibility, actual three-profile
collection and the subsequent unchanged strict production matrix remain unrun.
This is a source checkpoint awaiting independent final review and root admission.
It neither explains the original timing failure nor proves a speed improvement.
