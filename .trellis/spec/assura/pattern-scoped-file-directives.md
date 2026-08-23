# Pattern-Scoped File Directives

## Scenario: Reusable Attributes With User-Controlled Scope

### 1. Scope / Trigger

- Trigger: a file pattern inside a configured directory scope, or its reusable
  node directive, declares `max_lines` or `max_size` alongside naming.

### 2. Signatures

- Direct extension shorthand: `.ts: $source-file`; place it below `./**/` when
  the policy should rebase onto every descendant directory.
- Explicit file-glob reach: `"./*.ts"` matches direct root files while
  `"./**/*.ts"` matches root files and descendants. Inside a nested structure
  scope, `"*.ts"` and `"**/*.ts"` are resolved relative to that scope.
- Directory reach: configure hierarchy keys such as `./`,
  `packages/*/src/`, or `packages/**/generated/`.
- Normalized fields: `files.max_lines_patterns` and
  `files.max_size_patterns`, keyed by the normalized file glob.
- Portable compiled artifacts use schema version 25 or newer.
- Inspection: `assura explain <path>` reports applied directory scopes and the
  winning file-pattern attributes.

### 3. Contracts

- Extension shorthand such as `.ts` applies to direct files at its current
  anchor. Recursive reach is authored explicitly through `./**/` or a recursive
  file selector.
- Explicit file globs preserve user-selected depth. `*` matches one path
  segment and `**` crosses any number of directory separators.
- A more specific structure scope can merge local patterns with `inherit: true`
  or reset inherited patterns with `inherit: false`.
- The most specific matching file pattern wins; an exact or multi-part suffix
  beats a broader suffix, and a fixed-depth path beats a recursive path when
  both match the same file.
- Pattern values override directory-wide `files.max_lines` and
  `files.max_size` for matching files.
- Pattern values follow the same scope inheritance as naming patterns; local
  values replace inherited values for the same file pattern.
- LS-Lint migration emits `inherit: false` for nested LS-Lint directory scopes,
  preserving LS-Lint's replace-on-nested-scope behavior.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Pattern line limit is below 1 or above 100,000 | Reject config with the pattern path in the error. |
| Pattern size is malformed | Reject config with the pattern path in the error. |
| `exists` is attached to a file glob that crosses directories | Reject it and direct the user to a direct-child count in the matching structure scope. |
| File matches no size pattern | Use the containing directory default, if configured. |
| File matches a size pattern | Apply the most specific pattern value. |
| Descendant has no configured reset | Keep inherited pattern directives. |
| Inheriting descendant adds local patterns | Merge maps and replace duplicate keys locally. |
| Descendant scope sets `inherit: false` | Remove ancestor pattern directives. |
| Compiled artifact predates schema 25 | Reject it as incompatible. |

### 5. Good / Base / Bad Cases

- Good: a three-line `good-view.tsx` passes a `.tsx` limit of 3.
- Good: `packages/core/src/nested/good-file.ts` inherits the source scope.
- Base: a longer nonmatching `README.md` is unaffected by source-file limits.
- Base: a generated subtree with `inherit: false` is outside the source scope.
- Bad: `BadName.ts` reports `file_naming`, `too-long.tsx` reports
  `max_lines`, and an oversized matching source reports `max_size`.

### 6. Tests Required

- `reusable_file_directive_shorthand_matches_expanded_attributes` proves both
  authoring forms normalize to the same pattern maps.
- `pattern_scoped_file_directives` proves matching, inheritance, glob resets,
  nonmatching, naming, line, and size behavior through the public check flow.
- `explain_pattern_scope_cli` proves the CLI reports the applied directory scope
  and winning normalized file directive, including inheritance resets and an
  explicit no-match result.
- Compiled CLI coverage proves direct and recursive pattern maps survive actual
  Postcard serialization and validation.
- Website config evidence must execute the displayed shorthand policy.

### 7. Wrong vs Correct

Wrong when every file in the scope should share the limit:

```yaml
.ts:
  max_lines: 500
```

Correct scope-wide default:

```yaml
files:
  max_lines: 500
```

Wrong when generated descendants should not inherit a source policy:

```yaml
structure:
  .ts: $source-file
```

Correct explicit scope reset:

```yaml
structure:
  ./**/:
    .ts: $source-file
  ./**/generated/:
    inherit: false
```

Use the hierarchy instead of hiding reach inside a detached list:

```yaml
structure:
  packages/*/src/:
    .ts: $source-file
  packages/**/generated/:
    inherit: false
```

Choose file depth explicitly when one directory scope needs both behaviors:

```yaml
structure:
  ./*.ts: $root-source
  ./**/*.ts: $all-source
```

Use `.ts: $source-file` for direct files, or place it under `./**/` for the
same direct rule rebased across all directories. Use an explicit file glob when
one selector should carry the recursive reach itself.
