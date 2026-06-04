---
title: LS-Lint Compatibility Adversarial Review Template
status: active
---

# LS-Lint Compatibility Adversarial Review Template

Use this template for each adversarial LS-Lint compatibility review. The
reviewer must try to find at least one semantic gap. If no gap is found, the
review must say which gap hypotheses were disproved and cite current tests or
native LS-Lint behavior.

## Source Pointers

- Latest LS-Lint docs landing page and notation example:
  `https://ls-lint.org/`
- Official LS-Lint 2.3 basics:
  `https://ls-lint.org/2.3/configuration/the-basics.html`
- Official LS-Lint 2.3 rules:
  `https://ls-lint.org/2.3/configuration/the-rules.html`
- Official LS-Lint 2.3 CLI config merge docs:
  `https://ls-lint.org/2.3/configuration/the-command-line-interface.html`
- Official LS-Lint 2.3 changelog:
  `https://ls-lint.org/2.3/prologue/changelog.html`
- Upstream source clone used by this task:
  `/tmp/ls-lint-upstream`
- Upstream source files to inspect first:
  `/tmp/ls-lint-upstream/internal/config/config.go`
  `/tmp/ls-lint-upstream/internal/rule/rule.go`
  `/tmp/ls-lint-upstream/internal/rule/exists.go`
  `/tmp/ls-lint-upstream/internal/rule/regex.go`
  `/tmp/ls-lint-upstream/internal/linter/linter.go`
  `/tmp/ls-lint-upstream/cmd/ls_lint/main.go`
- Upstream tests to inspect first:
  `/tmp/ls-lint-upstream/internal/config/config_test.go`
  `/tmp/ls-lint-upstream/internal/linter/linter_test.go`
  `/tmp/ls-lint-upstream/internal/rule/*_test.go`
- Native package under test for parity fixtures:
  `@ls-lint/ls-lint@2.3.1`

## Required Feature Segments

For each segment, assess converter behavior, Assura native behavior, native
golden parity evidence, docs/matrix evidence, and edge-case risk.

1. Config shape: top-level `ls` and `ignore`; unknown top-level keys; invalid
   YAML shapes; non-string keys and non-string rule values.
2. Extensions and subextensions: `.js`, `.d.ts`, `.test.js`,
   `.local.build.js`, wildcard `.*`, wildcard subextension `.*.js`, and most
   specific match behavior; include the latest-docs examples `.d.ts`,
   `.spec.ts`, and `.mock.ts`.
3. Directory rules: `.dir`, root `.dir`, nested scopes, `models`,
   `src/templates`, current-directory and descendant override behavior.
4. Directory patterns: `packages/*/src`, `packages/**/templates`,
   `packages/*`, nested `"*"`, `packages/*/{src,tests}`, nested
   `"{src,tests}"`, latest-docs `packages/*/{src,__tests__}`, and matching
   for both direct and descendant paths.
5. Ignore patterns: exact paths, directories, `**/*.png`, `bazel-*`,
   `"**/{a,b}/*.js"`, dedupe, and merge ordering.
6. Multiple rules: exact separator ` | `, invalid `|` without spaces,
   mixed naming and `exists`, multiple regex rules, and regex alternation
   inside `regex:`.
7. Naming rules and aliases: `lowercase`, `camelcase`/`camelCase`,
   `pascalcase`/`PascalCase`, `snakecase`/`snake_case`,
   `screamingsnakecase`/`SCREAMING_SNAKE_CASE`,
   `kebabcase`/`kebab-case`; confirm removed `pointcase`/`point.case`
   rejection.
8. Regex: anchoring, negation `regex:!`, invalid regex syntax, empty regex,
   directory substitutions `${0}`, `${1}`, latest-docs `components/*` plus
   `tests/.test.ts: regex:${1}`, and substitution depth behavior.
9. Exists: bare `exists`, `exists:0`, `exists:N`, `exists:N-M`, invalid
   values, `uint16` bounds, extension/subextension file counts, `.dir`
   directory counts, and missing-directory behavior.
10. Scalar non-dot keys: `src: kebab-case`, `README.md: exists:1`,
    `src/: exists:1`; verify scalar naming keys are validated no-ops while
    scalar `exists` keys become direct counts for default validation and are
    covered by explicit target-path semantics separately.
11. Multiple configs: repeated `--config`, top-level `ls` key replacement,
    no recursive deep merge, `ignore` append/sort/dedupe behavior.
12. Explicit target-path semantics: Assura validation mode only; no CLI
    drop-in parity, no exact LS-Lint JSON parity, no `--workdir` or `--debug`
    parity claim.

## Required Response Format

```text
Review result: gaps found | no gaps found

Coverage checklist:
- Config shape: pass | gap | unproven - evidence:
- Extensions/subextensions: pass | gap | unproven - evidence:
- Directory rules: pass | gap | unproven - evidence:
- Directory patterns: pass | gap | unproven - evidence:
- Ignore patterns: pass | gap | unproven - evidence:
- Multiple rules: pass | gap | unproven - evidence:
- Naming rules/aliases: pass | gap | unproven - evidence:
- Regex: pass | gap | unproven - evidence:
- Exists: pass | gap | unproven - evidence:
- Scalar no-op keys: pass | gap | unproven - evidence:
- Multiple configs: pass | gap | unproven - evidence:
- Target-path / non-CLI scope: pass | gap | unproven - evidence:

Findings:
1. Severity: blocker | high | medium | low
   Segment:
   File/line:
   Why this may differ from LS-Lint:
   Native LS-Lint repro command or fixture:
   Suggested Assura test:

Disproved hypotheses:
- Hypothesis:
  Evidence:

Final challenge:
- The strongest remaining way to break compatibility would be:
```
