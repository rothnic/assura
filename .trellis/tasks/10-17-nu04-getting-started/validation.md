# NU04 behavior evidence

Candidate worktree: `/Users/nroth/.codex/worktrees/assura-nu04-quickstart/assura`
Base: `origin/master` `49d85358bae85de04f1ce7faa746b377687dcaf3`
Branch: `fix/nu04-getting-started`

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
