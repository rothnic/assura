# Validation placement and proof reuse

Inspect `xtask/src/main.rs`, `scripts/ci-scope.sh` and current workflows for the
changed surface. Record command elapsed time, queue time, exit, host, source,
toolchain, test count and failed test/job name. Optimize measured repetition;
never change required coverage to manufacture a green result.

| Change/phase | Pre-PR work | Merge proof |
| --- | --- | --- |
| Docs, skills, Trellis only | workflow gate, diff check, structure, `cargo xtask evidence`; instruction decision scenarios | Independent process review plus applicable hosted scope/evidence jobs |
| Any website or docs path | Above plus `cargo xtask docs` and affected browser/link checks | Applicable docs/website hosted checks |
| Rust behavior | Focused red/green contract; `cargo xtask fast` at meaningful implementation boundaries; `cargo xtask pr` once candidate settles | Required final-candidate OS/features/install/performance jobs and scoped independent review |
| CI, scripts or validation logic | Positive and negative scope/behavior controls; affected tier and representative execution | Review coverage changes explicitly; all affected required hosted jobs |
| Performance | Correctness proof first; comparable baseline/candidate raw rows on one idle host | Existing required native, warm and no-slower gates on final candidate |

`pr` currently calls `fast`, then target-state, Clippy and docs. Do not run
unchanged `fast` immediately before `pr` merely to satisfy two command names;
the nested invocation supplies that proof. `pr` does not include the complete
hosted release/install/OS/performance matrix. Never call it equivalent to CI.
`cargo xtask changed` is triage only; it is not a merge-gate replacement.

Run cheap structure/scope/target-state checks before long builds where relevant.
Install pinned website dependencies once per isolated checkout when docs run.
Review a coherent committed candidate while independent validation runs, then
review any fixes as a delta. Push a prepared candidate once; avoid successive
evidence-only pushes that invalidate live CI. Existing final-head requirements
remain authoritative, including metadata that alters check selection.

Proof reuse requires unchanged relevant files, dependencies, configuration,
toolchain, environment and invocation. Record what was compared. Rebase or a
changed dependency invalidates affected proof; required final-head CI still
runs. Do not rerun an unchanged failing test/benchmark until it happens to pass.

For agent-driven evaluation, read the detailed
[runner-isolation](runner-isolation.md) contract first. Bind the candidate
executable by absolute path and record its source SHA, version, and SHA-256
before launch. Verify `command -v assura`, `assura --version`, and the hash from
the initializer's actual command environment as well; `/bin/zsh -lc` or another
login-shell tool may replace a parent `PATH` prefix. A mismatch, ambient global
binary fallback, private-evaluator exposure or missing child-context proof
makes the run invalid and earns no screening, holdout, or acceptance credit.
Use a disposable login-shell-safe launcher or command shim rather than changing
global binaries or startup files. The preferred binding is the regular
candidate executable itself; a shim must `exec` one fixed absolute target.
Record the resolved command path and target path, canonicalize and hash the
actual target, and reject aliases, functions, opaque symlink chains, or a shim
that delegates elsewhere. Then run focused wrong-target and exact-target
controls plus a fresh identity canary before repeating the evaluation protocol.

## VPS routing

Read the remote section of `assura-local-build` before offloading. Use remote
Linux for sustained builds/tests and comparable benchmarks when capacity and
disk allow; keep local checks bounded. Linux cannot prove macOS file watching,
Windows installation, host permission approval or browser behavior by proxy.
Limit heavy work to one job per host initially and record observed utilization.
Benchmarks run without concurrent compilation; compare both binaries there.

Before changing CI infrastructure, collect at least three prepared candidate
runs: queue/execution time, first-pass success, retries and duplicate suites.
Then propose one measured cache/parallelism improvement with identical coverage
and cold/warm validation. Self-hosted GitHub runners, secrets changes and
protection changes are separate proposals, not implied by SSH access.
