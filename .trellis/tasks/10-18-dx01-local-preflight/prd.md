# DX01 local validation preflight

## Goal

Prevent known invalid long local validation runs with one opt-in skill-local launcher. The coordinator accepted the bounded design in the external review packet; no further feature discovery is needed.

## Scope and ownership

This worker owns the local-build skill, its scripts/local-preflight.py and scripts/local-preflight-tests.py controls, this task and its own journal. The coordinator owns Beads, review, PR, push and merge. No product Rust/API/policy/dependency changes or Cargo/website builds.

## Requirements

- Bind canonical caller/repository root, HEAD/tree, dirty status, effective target and selected environment before launch.
- Reject a target inside another Assura checkout; permit checkout-local and explicitly selected private owned external targets. Never redirect target/TMPDIR implicitly.
- Preserve the checkout-local target restriction for xtask routes that read target/debug/assura-full.
- Socket-sensitive opt-in checks require an existing private owned disk-backed TMPDIR and check filesystem-encoded Darwin socket length including NUL. The Darwin bound must not leak to other platforms.
- Optional Bash profile checks the effective child resolver and mapfile capability. Ordinary Cargo routes do not require Bash.
- Optional website profile preserves pnpm/npm/node fallback, checks selected pnpm against packageManager, and records package-manager child Node identity without claiming arbitrary nested-build proof.
- Failed preflight emits measured evidence and never invokes the supplied command. Success execs the exact argv with inherited environment and exit behavior.

## Acceptance

- [x] Fake-command negative controls demonstrate pre-launch rejection for wrong checkout/target, unsafe/long TMPDIR, missing mapfile and selected website-tool mismatch.
- [x] Local/relative/explicit private external targets pass and preserve argv, cwd, environment, output and nonzero exit.
- [x] Dirty source is reported and original fixture bytes stay unchanged.
- [x] Website fallback/identity controls preserve the current build route and label probe limits honestly.
- [x] Focused Python controls, formatting/diff, workflow, structure and applicable evidence gates recorded outside Git; no Cargo/website build.
- [x] Owned coherent implementation committed cleanly before independent review handoff.

## Design evidence

Accepted coordinator packet (external): /Users/nroth/workspace/.assura-review-20261017/dx01-source-research.md and dx01-independent-design-review.md. Relevant repository contracts: tooling-stabilization.md and existing local-build skill.

## Out of scope

No generic controller/schema framework, installs, shell/global PATH changes, runtime optimization claims, full Rust tier, CI/release or publication actions.

## Validation and handoff

17 focused Python controls passed (27 helper invocations); source/config bytes
are checked before/after each invocation. Formatting, diff and structure passed;
structure retains only 16 existing low line-count advisories. Evidence passed
using coordinator-approved prebuilt xtask SHA
`af14a910341cddfd49a3d2ae7a18213e4a2d4363fbfe5f3c73948c9068cf1ea3`,
with byte-identical xtask/Cargo/config inputs and recorded Bash 5.3.8 selection.
This is binary reuse, not a fresh Cargo build. Raw command/identity/exit receipts
are under `/Users/nroth/workspace/.assura-review-20261017/dx01/` outside Git.
No Rust/website build, install or full Rust tier ran. Independent review and
integration remain coordinator-owned; task stays in progress until delivery.

## Independent review correction

Coordinator accepted three blockers in the independent review of b3647c8d:
Cargo/application argument boundaries, equivalent xtask routes, and literal
Rust socket temp-root/spelling. Added eight focused controls before changing the
helper: all 25 ran against the unchanged b3647c8d helper and produced 14 expected
failing subcases. Failed receipts are retained outside Git. The same 25 controls
then passed after the bounded fixes; 42 helper invocations distinguish metadata
probes from supplied-command execution and preserve tracked fixture bytes.

Cargo parsing stops at `--`, consumes the supported option pairs and recognizes
common local xtask run/package/manifest forms. External manifests, -C source
routing, unknown command aliases and non-Cargo wrappers are explicitly rejected.
The socket profile requires nonempty inherited TMPDIR and measures its literal
byte spelling; it never substitutes Python TEMP/TMP/defaults or canonicalizes
the socket address. Ordinary canonical filesystem aliases remain permitted;
Node identity remains observational with no invented engine/version policy.

Current correction receipts: `/Users/nroth/workspace/.assura-review-20261017/dx01/correction-b364/`.
Scoped independent rereview of the committed correction remains coordinator-owned.

The first correction structure check rejected a literal command placeholder in
the skill. The failed receipt is preserved. Three read-only diagnoses supported
removing only that example and describing the equivalent xtask launches in
prose; canonical `cargo xtask pr` and `cargo xtask docs` examples remain. Helper
and control hashes still match the 25-control GREEN run. Final document checks
are recorded separately in `correction-b364/doc-recovery/` to retain the failure.
