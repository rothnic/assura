# Assura capability, efficiency and ecosystem review

Second pass, 2026-10-17. Inputs: integration `f3540d4a`, review checkout
`66c1fe16`, published `v0.4.1`. Runtime source is unchanged between those
reviewed revisions. This extends the [first pass](../../../../docs/analysis/2026-10-17-new-user-review.md).
It revises priorities and evidence requirements; it authorizes no product fix,
benchmark campaign, evaluator change, outreach or Beads mutation.

Assura has a credible role as an executable repository-convention layer:
project-owned paths, naming, cardinalities, file limits, captured artifact
relationships and selected documentation rules, with shared local/CI/agent
feedback. Its advantage must be demonstrated through dependable coverage and
cheaper policy maintenance or repair. Feature count and synthetic runtime
ratios do not establish that advantage.

The most important new capability observation is a selected-file relationship
omission. The most useful systemic performance work is one bounded investigation
of repeated filesystem work in the rich checker. The backlog now contains
**14 new proposals, 10 carried outcomes and 22 historical terminal records**.
Only NU13 and SP01 are new; four architectural hypotheses remain conditional
inside SP01. The [native graph](beads-graph.json) has 25 nodes including its
epic, 15 blocking edges and no cycles. It remains unapplied.

## 1. Core capability: evaluate what a passing result proves

The prior trials proved configured naming, closed-scope, local-link and line
checks, preservation, and local integration lifecycle. This pass added eight
small published-binary checks of a captured counterpart relationship:

```yaml
structure:
  ./:
    extra: true
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
```

| Case | Actual result and interpretation |
| --- | --- |
| `Button.tsx` exists, counterpart absent; full Check | Exit 1; missing counterpart correctly names producer and required path. |
| Same tree, Check targets only `Button.tsx` | Exit 0, one file checked, no relationship finding; no LS-Lint target-semantics flag supplied. |
| Add a comment-only `Button.test.tsx`; full Check | Exit 0; the rule proves presence, not execution or test adequacy. |
| Cached full pass, then delete counterpart | Exit 1; deletion invalidation works. |
| Relax policy, then restore required counterpart | Cached full result changes pass then fail; policy invalidation works. |
| Delete the producer | Exit 0; no source remains to require a counterpart. |

The initial review assertion expected file-target exit 1 and failed. That
expectation was a hypothesis, not an established public scope contract; the
unfavorable receipt is retained. Independent source inspection explains the
omission: ordinary Check constructs a checker directly and passes the selected
file to extension traversal; `relationship_entries` skips the entry equal to
the selected path, including a file. Prepared changed-path checks separately
fall back for relationship policy and have a regression assertion for that
fallback. These are distinct execution paths.

NU13 must define and enforce the ordinary targeted-check coverage contract.
At minimum, a declared relationship must be enforced or explicitly omitted
with a full-check follow-up. A smaller checked scope cannot substantiate
equivalent full-policy performance. The cached full results above remain
positive controls; this observation does not demonstrate a stale cache or a
session failure. See [probe analysis](core-capability-probe.json), raw receipts
in the [second-pass manifest](second-pass-evidence-manifest.json), and
[source analysis](systemic-filesystem-review.md).

## 2. Systemic performance: core filesystem linting

A dedicated agent examined traversal, scope resolution, metadata/content reads,
extension walkers, caching, prepared checks and watch eligibility. The existing
engine already has substantial optimization: compiled patterns, per-directory
rule reuse, static-scope indexing, exact-suffix lookup, exclusion pruning,
shared per-file content, prepared plans, and one full fallback per changed-path
batch. Reproposing those capabilities would duplicate completed work.

The remaining opportunities are specific mechanisms, with unmeasured impact:

| Mechanism | Source-established work | Smallest discriminator and possible response |
| --- | --- | --- |
| Demand-driven metadata | Full-engine `file_limits.rs:19-25` evaluates `fs::metadata(path)` while constructing a tuple even when the size limit is absent; called for eligible file bundles. | Count attributed stat-family calls with and without size policy. If meaningful, guard the absent-size case before fetching metadata. Start with the local guard; no cache or new abstraction is needed. |
| Repeated enumeration | Core traversal is followed by independent extension walks. Test relationships enumerate checked scope and project root; root checks repeat the same scope. Docs lifecycle discovers active/status files separately. Direct-child counting rereads directories. | Count walks, directory opens, content reads/bytes by consumer. Start with equal-scope reuse or a combined selector pass. A shared check-lifetime inventory is conditional on demonstrated broader cost and equal traversal semantics. |
| Scope candidate lookup | Full engine scans all S scopes per uncached directory D, approximately O(D*S). A magical scope disables the fast engine's static index, which otherwise already works. | Vary scopes independently of files/depth and measure matcher calls, setup, wall time and memory. Index literal-prefix candidates only if this work dominates; preserve wildcard fallback, precedence and inheritance. |
| Safe incremental eligibility | Rich Markdown and many extensions conservatively force full checking. Watch and prepared batches have distinct eligibility rules. | Attribute actual edit/burst fallback cost. Expand only a proven local dependency class, preserving parent/relationship/link/provider/config invalidation and full fallback for unknown dependencies. Resolve NU13 first for any claimed coverage expansion. |

Source shape implies at least six enumeration passes for core plus selected
enabled extension families in Assura's root policy. Their filters and purposes
differ; this is not a sixfold wall-time claim. Existing phase timers also leave
custom-extension work without a separate phase field, so retained phase medians
cannot explain all rich-policy cost. Separate phase medians are not an additive
decomposition of public-command runtime.

**SP01 is the only new performance work item:** bind source, release binary,
policy, tree and host; profile ordinary public Check with rich realistic policy
and a controlled wide-tree case; capture CPU, metadata/enumeration/read counts,
bytes, wall time and peak memory. First test eager metadata, then identical-scope
enumeration. Produce one smallest keep/defer decision. Each hypothesis's
measured time fraction is currently unknown; even eliminating fraction f is
bounded by `1/(1-f)`. No percentage improvement is claimed here.

Any later optimization must preserve exact findings/severities/exits, root and
file semantics, exclusions, symlinks, unreadable paths, deterministic ordering,
count/relationship behavior and fail-fast behavior. Use uninstrumented balanced
same-host before/after timings. Retain the [existing keep/stop policy](../../../../docs/analysis/2026-07-05-performance-decision-matrix.md):
target public gain at least 5% and 0.75 ms, exact quiet-command gain at least 3%,
no accepted spillover beyond 5% or 0.25 ms, unchanged required gates, and two
directionally consistent runs with order reversal. No eager whole-tree inventory
or persistent invalidation scheme should be added to the cheap naming path.

NU11 remains a separate current-head gate investigation. The latest hosted run
still fails at [37214120576](https://github.com/rothnic/assura/actions/runs/37214120576):
11.812 versus 11.650 ms on the naming-equivalent many-scope case. The rich-engine
mechanisms above do not explain that fast-path result. Source regression versus
runner variability remains unproven. Avoid unchanged retries until green,
threshold relaxation, parser replacement, binary splitting, unconditional
parallel traversal and another daemon as substitutes for attribution.

## 3. Efficiency: execution, attention and policy ownership

One agent made 80 completed timed calls, five fresh-process samples per command
per project, serially on the shared Intel Mac. Same published executable and
configuration before/after; existing OS caches; no cold-disk or p95 claim.
Review used `--base HEAD`, with disposable trial history. Project configurations
differ, so this is operation-cost evidence rather than a language comparison.

| Operation | TypeScript median | Python-led median | Relevant coverage/output |
| --- | ---: | ---: | --- |
| Full Check JSON | 37.8 ms | 56.1 ms | 273/449 files; 18/129 findings; Python retains one blocking setup conflict. |
| Review JSON | 212.9 ms | 530.8 ms | About 5.6x/9.5x Check; approximately 11/12 KB; active low findings still omitted from finding history. |
| Doctor JSON | 38.0 ms | 58.4 ms | About 2.5/2.8 KB; setup/diagnostic operation. |
| Explain one source file | 17.2 ms | 19.6 ms | Approximately 4.7 KB; explains a specific path. |
| Check agent output, five findings | 36.7 ms | 59.9 ms | Approximately 3.2 KB, versus 7.9/68.3 KB full Check JSON. |

The explicit five-finding option already saves 58.6%/95.3% of Check JSON bytes
in these conditions. Those are bytes, not measured model tokens or dollars.
Routine automatic feedback has a separate implemented 256-byte ceiling; explicit
reports need not share it. Quiet mode preserves failure diagnostics correctly.
NU07 should improve actual counts/offender paths and repeated repair guidance,
so a maintainer can identify the intended repair in one bounded result.

A calculated scenario of 100 JSON Reviews costs 21.29/53.08 seconds and roughly
1.11/1.20 MB of stdout, before reading/model/native-check costs. This multiplies
sampled medians; 100-call workloads were not executed. It supports sensible
cadence: Check as the policy gate, Review at meaningful checkpoints, Doctor for
setup/troubleshooting, Explain for a confusing path. Repeating all reports per
tool call is not an efficient workflow. Installed-hook latency was not measured.

Setup and repair cost remain the bigger unmeasured economics: generated configs
are 145/132 lines in these two cases, but setup-command runtime is not time to
an owner-approved policy. Actual user minutes, manual decisions, extra repair
commands, policy churn, nuisance findings, observable model cost, and retained
use must be collected through existing public-case evaluation and F02. The old
65/80 trial command receipts are research instrumentation, not a normal-user
command-count estimate. See [measurements](efficiency-measurements.json) and
[full analysis](efficiency-review.md).

## 4. Ecosystem fit and limits

Assura should own concrete repository conventions and repair context, composed
with existing specialist tools. Its primitives alone are not a unique moat:

| Existing job | Existing tool capability | Assura's useful role |
| --- | --- | --- |
| Filesystem naming/cardinality | [LS-Lint](https://ls-lint.org/2.3/configuration/the-rules.html) already supports naming, regex/directory substitutions and file/directory counts. | Switch only for additional intended conventions or a demonstrably cheaper workflow. A naming-only repository can keep LS-Lint. |
| Language correctness/style | [ESLint](https://eslint.org/docs/latest/use/core-concepts/) and [Biome](https://biomejs.dev/linter/) analyze source with their own rule/language scope. | Select the real native checks; do not substitute file layout for language correctness or compare unequal workloads as a speed advantage. |
| Actual import architecture/dead code | [dependency-cruiser](https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-reference.md) validates dependency rules; [Knip](https://knip.dev/reference/issue-types) identifies unused files, exports and dependencies. | Captured file pairs do not establish dependency direction, dead-code status or semantic architecture. Delegate those jobs. |
| Documentation style/links | [markdownlint](https://github.com/DavidAnson/markdownlint) covers Markdown rules; [lychee](https://lychee.cli.rs/guides/cli/) supports offline local files and fragment checking. | Retain a useful specialist; replace overlap only after an explicit rule/diagnostic/exclusion comparison. Local links alone are not unique. |
| Hook execution and arbitrary policy | [pre-commit](https://pre-commit.com/) composes multi-language hooks; [Conftest](https://www.conftest.dev/) evaluates structured data with Rego. | Be an invoked repository check. Preserve the existing runner/native failures and avoid a second general execution or policy platform. |

The fit test is concrete: identify costly recurring conventions before setup,
record what the current stack already enforces, and require a material unmet
convention or cheaper repair/maintenance outcome. Preserve equivalent-policy
negative controls. If current tools suffice, or nobody owns the intended
conventions, another policy language has no demonstrated benefit. No new generic
runner, ecosystem adapter, AST/dependency engine, semantic quality score,
marketplace, hosted platform or broad repair feature is justified by this review.

The release story also needs existing W02's attention: the live latest release
and support ledger say v0.4.1, while `release-surfaces.json` still says current
published v0.3.0. Other support rows retain next-release wording. Likewise the
July performance matrix says cold 2x=2/8 while the September 30 checked report
says 1/8. Its cold aggregate 1.308x and warm session 24.798x have different
execution models: persistent startup/initial full validation sit outside warm
requests. Eight generated fixtures are not eight independent user projects.
The website already labels the persistent-session comparison; no new false
marketing allegation or intrinsic-engine speed claim follows from it.

The next evidence is a small paired workflow pilot under existing F02:
configuration/maintenance minutes, missed/false findings, one-repair outcome,
whole-workflow resource/attention cost and continued use, against the maintainer's
existing stack. Preserve its participant, duration and retention criteria. These
internal unblinded trials do not satisfy independent A07 or external F02 credit.
See [ecosystem research](ecosystem-fit-review.md) and [primary sources](ecosystem-sources.json).

## 5. Backlog decisions and actual dependencies

- **NU13, new P1:** targeted captured-relationship coverage. Keep ordinary,
  explicit LS-Lint, full, cached and prepared modes distinct; define the contract
  and preserve meaningful negatives before any coverage-expanding optimization.
- **SP01, new P1 research spike:** attribute core filesystem cost and choose
  one supported improvement. It can run beside correctness fixes. Only a future
  incremental-expansion candidate needs NU13 resolved first; no artificial
  dependency blocks the initial profiling work.
- **NU01/NU02/NU03 remain P1:** configured finding visibility, compatible
  onboarding policy and actual native-source coverage. NU03 adds a composition
  acceptance case under one existing hook runner, retaining native execution,
  failures/order and removal behavior without duplicate scans or a new runner.
- **NU07 remains a focused P2:** offender names, actual limits and bounded
  nonrepetitive repair context. Its attention value is separate from raw speed.
- **NU11/W02 gain sharper evidence criteria:** date/source/platform/profile/
  scope/startup labels, current failed gate, actual release JSON and honest
  fit/non-fit. Historical faster rows cannot clear today's required gate.
- **A07/F02 retain their owners and prerequisites:** propose cost measurements
  only for public nonsealed follow-ups and the existing external pilot. No
  private protocol, holdout, allocation, credit or outreach change is authorized.
- **CF01-CF04 remain historical done:** all four merge SHAs are ancestral to
  refreshed master. Caching, batch fallback, compact scheduling and async delivery
  already shipped; none is recreated as a generic efficiency feature card.

The [updated backlog](backlog.json) gives problem, evidence, acceptance, priority
rationale, related work and recommended lanes. The [graph validation](backlog-validation.json)
keeps 25 unique nodes, 15 prerequisite edges, and no cycles/missing references.
Conditional architectures are excluded from the open-node graph. Historical
terminal dependencies remain observations, not newly open blockers.

Local Beads is 1.2.2; no authoritative store resolved in the first pass and none
has been selected or initialized here. Before importing, select that store,
deduplicate against its existing IDs and reconcile lifecycle/owners. Native
graph creation starts nodes open and is not an idempotent synchronization tool.
The installed-version graph schema and static integrity were checked; native
store dry-run/import remain unrun.

## Proof limits and handoff

This pass executed bounded diagnostic measurements and capability probes,
inspected source and retained benchmarks, and researched primary ecosystem docs.
It measured no architectural speed improvement, competitor performance, actual
installed host delivery, model cost, maintainer demand or retention. No product
source or original projects changed. No full benchmark/OS matrix, private
evaluation or Beads operation ran. Required correctness and performance gates
remain unchanged; the latest hosted performance failure remains unresolved.
