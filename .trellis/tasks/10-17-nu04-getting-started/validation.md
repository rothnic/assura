# NU04 behavior evidence

Candidate worktree: `/Users/nroth/.codex/worktrees/assura-nu04-quickstart/assura`
Base: `origin/master` `49d85358bae85de04f1ce7faa746b377687dcaf3`
Branch: `fix/nu04-getting-started`
Native owner: `/root/nu04_quickstart` (local Trellis identity alias:
`nu04_quickstart`).

## Published install and reproduction

- Fetched the published installer from
  `https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh`;
  SHA-256 `689c5deed6af91534249f1cd7f4ae5f35e3e414b446f9d4606c64b46a7962e10`.
- Ran it with `ASSURA_VERSION=v0.4.1` and `BIN_DIR` under review scratch. The
  resulting v0.4.1 executable SHA-256 is
  `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`, matching
  `/usr/local/bin/assura`; the global binary was not changed.
- `assura init --recipe agentic-core --recipe structure-health` generated
  `.assura/config.yml`, `README.md`, `AGENTS.md`, `docs/agent-guidance.md`, and
  `docs/structure.md`.
- The exact original YAML from this guide produced a blocking root direct-file
  violation and exit 1. The report said one file matched `*` where the closed
  rule expected zero; the generated `AGENTS.md` was omitted from the model.

## Revised published example

In a fresh minimal project, the exact YAML was extracted from the revised guide
and used as `.assura/config.yml`. The generated README, AGENTS file, and both
docs files were retained unchanged; their before/after SHA-256 manifests match.

| Journey step | Command / event | Exit |
| --- | --- | ---: |
| Clean modeled project | `assura check` | 0 |
| Unexpected project-root file | Add `unexpected-root.md`; `assura check` | 1 |
| Unexpected generated-docs child | Add `docs/unexpected.md`; `assura check` | 1 |
| Recovery | Remove both files; `assura check` | 0 |

Raw command output, exit receipts, generated-file manifests, and the disposable
projects remain outside source at
`/Users/nroth/workspace/.assura-review-20261017/nu04/`. The final acceptance
project is `final-doc-example/`; the original red reproduction is
`current-doc-example/`.

No existing documentation test specifically checks this guide's first-run
YAML. No new test code was added; the published released-CLI journey validates
the behavior of the exact page snippet directly.

Trellis delivery registration uses candidate
`10-17-nu04-getting-started`, owner `nu04_quickstart`, base ref
`refs/remotes/origin/master`, branch ref `refs/heads/fix/nu04-getting-started`,
and authority `prd.md#goal`. The base ref resolves locally to
`49d85358bae85de04f1ce7faa746b377687dcaf3`. The native `/root/...` path is an
owner identity; Trellis uses the supported local alias `nu04_quickstart`.

The delivery intent is registered and local refs resolve. Full
`task.py delivery inspect` was stopped after 24 seconds because it waited on
`gh pr list --repo rothnic/assura --state all --limit 1000` without returning;
the hosted pull-request snapshot remains for coordinator validation.
