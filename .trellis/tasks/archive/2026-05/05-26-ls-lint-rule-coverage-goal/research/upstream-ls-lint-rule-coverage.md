# Upstream LS-Lint Rule Coverage Notes

Date: 2026-05-26

Upstream reference checked:

- Public docs: https://ls-lint.org/2.3/configuration/the-rules.html
- Public docs: https://ls-lint.org/2.3/configuration/the-basics.html
- Public announcement: https://ls-lint.org/blog/announcements/v2.3.0.html
- Source clone: `loeffel-io/ls-lint` at `49b4e7b`

## Supported LS-Lint Surface

LS-Lint 2.3 documents these built-in rules:

- `lowercase`
- `camelcase` / `camelCase`
- `pascalcase` / `PascalCase`
- `snakecase` / `snake_case`
- `screamingsnakecase` / `SCREAMING_SNAKE_CASE`
- `kebabcase` / `kebab-case`
- `regex`
- `exists`

It also documents these configuration semantics:

- global extension and sub-extension rules such as `.ts`, `.d.ts`, `.test.ts`
- wildcard extension rules such as `.*`, `.*.js`, and `.*.*.go`
- `.dir` rules for directory names
- `|` composition for multiple rules
- path-specific rules that override broader directory rules
- glob directory scopes with `*` and `**`
- brace/alternative directory scopes such as `{src,tests}`
- glob and brace patterns in `ignore`
- multiple `--config` files merged in command order
- `--workdir`
- targeted file/directory arguments
- `--error-output-format json`
- `--warn`

## Upstream Tests Reviewed

Relevant source files:

- `internal/rule/*_test.go`
- `internal/config/config_test.go`
- `internal/linter/linter_test.go`
- `internal/rule/regex.go`
- `internal/rule/exists.go`
- `internal/glob/glob.go`
- `cmd/ls_lint/main.go`

Important upstream rule tests not fully mirrored in Assura today:

- Regex negation: `regex:![0-9]+`.
- Regex directory substitutions: `${0}`, `${1}`, and the regression shape from
  issue 307 where `${1}` is resolved against `gen/swu1/data`.
- Regex full-string anchoring semantics: upstream wraps patterns as
  `^{pattern}$`.
- Exists parser errors: `exists:`, `exists:-1`, `exists:1-`,
  oversized integers, and oversized range bounds.
- Exists default form: bare `exists` means at least one and at most max int.
- Exists count accumulation and final validation behavior.
- Exists interaction with targeted path runs, including bypassing some errors
  but still checking relevant aggregate counts.
- Glob directory scopes: `src/**/c`.
- Brace directory scopes: `src/{a,b}/*`.
- Glob ignore patterns: `src/c/*/*.jpg` and `src/c/d/*`.
- Wildcard extension precedence across `.*`, `.*.jpg`, `.service.jpg`,
  `.service.*`, `.app.test.gif`, and `.*.gif`.
- Exact casing edge cases from upstream rule tests for lowercase, camelcase,
  pascalcase, snakecase, screamingsnakecase, and kebabcase.
- Multiple config file merge behavior from the CLI.
- JSON error output shape from the CLI.

## Current Assura Coverage Observed

Assura currently has meaningful parity coverage for:

- basic LS-Lint conversion through `src/config/ls_compat.rs`
- extension and sub-extension naming conversion
- `.dir` naming conversion
- multiple naming alternatives with `|`
- ignore behavior for simple ignored directories
- direct-child `exists` counts and ranges
- exact filename `exists` as an Assura compatibility extension
- direct directory `exists` with trailing slash syntax
- unsupported glob/brace directory scopes returning clear migrate errors
- wildcard extension matching in the native Assura structure checker
- regex naming in the native Assura checker
- regex alternatives with pipes in native Assura syntax

Important caveats:

- Assura regex is currently compiled directly from the provided pattern. LS-Lint
  documents and implements full-string wrapping around the user pattern.
- Assura regex does not currently support LS-Lint negation syntax.
- Assura regex does not currently support LS-Lint directory substitutions.
- Assura migrate currently rejects glob and brace directory scopes instead of
  converting them.
- Assura has some naming conventions beyond LS-Lint, but that does not prove
  exact compatibility with LS-Lint's rule edge cases.

## Recommended Goal Scope

The next goal should be an audit-and-coverage goal, not a broad rewrite. It
should require an upstream test matrix, then add or explicitly defer parity
coverage based on product value for Assura's agentic hot path.

Priority should be:

1. Regex semantics: anchoring, negation, directory substitution, multiple regex
   rules, and documented error handling.
2. Exists semantics: parser errors, bare exists, ranges, directory exists,
   targeted path runs, and direct-child-only behavior.
3. Directory scope semantics: glob and brace patterns, especially whether they
   should remain migrate blockers or become supported through native `rules:`
   grouping.
4. Wildcard extension precedence and exact upstream rule edge cases.
5. CLI parity items that matter to agentic use: JSON output, targeted paths,
   multiple configs, and workdir.
