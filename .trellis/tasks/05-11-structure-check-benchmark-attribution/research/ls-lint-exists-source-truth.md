# LS-Lint Exists Source Truth

## Sources

- LS-Lint 2.3 rules: https://ls-lint.org/2.3/configuration/the-rules.html
- LS-Lint 2.3 basics: https://ls-lint.org/2.3/configuration/the-basics.html
- LS-Lint v2.3.0 exists rule:
  https://raw.githubusercontent.com/loeffel-io/ls-lint/v2.3.0/internal/rule/exists.go
- LS-Lint v2.3.0 linter:
  https://raw.githubusercontent.com/loeffel-io/ls-lint/v2.3.0/internal/linter/linter.go
- LS-Lint v2.3.0 config indexing:
  https://raw.githubusercontent.com/loeffel-io/ls-lint/v2.3.0/internal/config/config.go

## Findings

- The rules page describes `exists` as allowing or disallowing counts for a
  given extension and says it only applies to the directory itself, not
  subdirectories.
- The same page shows directory support through `.dir: exists:1` and
  `.dir: exists:0`, including directory-pattern examples.
- The basics page defines extension and sub-extension rule keys, wildcard
  extension keys, directory rule keys through `.dir`, and directory patterns.
- The v2.3.0 source stores `exists` counts inside the rule object and
  increments the count when a file or directory visits a matching extension or
  `.dir` key for the same config directory.
- The v2.3.0 source does not expose exact filename matching as a distinct
  `exists` syntax in the linter path; non-extension scalar keys are indexed as
  rule keys and do not match `README.md` as an exact file name.

## Live Fixture

Command:

```bash
tmpdir=$(mktemp -d)
cd "$tmpdir"
printf 'ls:\n  README.md: exists:1\n' > .ls-lint.yml
printf '# Readme\n' > README.md
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint
```

Observed result:

```text
ls-lint v2.3.0
. failed for `README.md` rules: exists:1 (found 0)
```

## Conclusion

Assura should keep exact filename `exists` as a compatibility extension for
the current converter, but active docs and specs should not call it native
LS-Lint parity. LS-Lint parity for `exists` covers extension, wildcard
extension, range, `exists:0`, and `.dir` directory count semantics.
