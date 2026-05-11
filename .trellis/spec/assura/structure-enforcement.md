# Structure Enforcement Contract

## Scenario: Closed-World Direct Contents

### 1. Scope / Trigger

- Trigger: Assura must reject unexpected well-named files and directories, not
  only naming mismatches.
- Applies to the structure-first config loaded from `.assura/config.yml` and
  represented by `src/config/config.rs`.

### 2. Signatures

- Config fields:
  - `DirectoryNode.files.allow_extra: Option<bool>`
  - `DirectoryNode.files.allowed_patterns: Option<Vec<String>>`
  - `DirectoryNode.files.forbidden_patterns: Option<Vec<String>>`
  - `DirectoryNode.directories: Option<DirectoryBundle>`
- `DirectoryBundle` fields:
  - `naming: Option<String>`
  - `required: Option<Vec<String>>`
  - `allowed_names: Option<Vec<String>>`
  - `allowed_patterns: Option<Vec<String>>`
  - `forbidden_patterns: Option<Vec<String>>`
  - `allow_extra: Option<bool>`
  - `severity: Option<Severity>`
  - `exists: Option<HashMap<String, String>>`

### 3. Contracts

- Missing new fields preserve the historical non-strict behavior.
- `files.allow_extra: false` checks only direct child files of the configured
  directory.
- `directories.allow_extra: false` checks only direct child directories of the
  configured directory.
- Direct child files are allowed by exact `allowed_names`, glob-like
  `allowed_patterns`, or extension/naming entries in `files.extensions`.
- Direct child directories are allowed by `children`, exact
  `directories.allowed_names`, or glob-like `directories.allowed_patterns`.
- `forbidden_patterns` override broad allowed patterns for both files and
  directories.
- Direct-content policy does not inherit into descendants. Inherited naming,
  size, docs, and markdown rules may still apply.

### 4. Validation & Error Matrix

| Condition | Violation rule |
| --- | --- |
| Unknown direct file with `files.allow_extra: false` | `unexpected_file` |
| Direct file matching `files.forbidden_patterns` | `forbidden_file` |
| Unknown direct directory with `directories.allow_extra: false` | `unexpected_directory` |
| Direct directory matching `directories.forbidden_patterns` | `forbidden_directory` |
| `exists` count outside expected range | `exists_count` |

### 5. Good/Base/Bad Cases

- Good: root config lists `src`, `tests`, and `docs` as children, sets
  `directories.allow_extra: false`, and rejects a stray `scratch/` directory.
- Base: config omits `allow_extra`; Assura continues applying naming and file
  rules without rejecting additional direct entries.
- Bad: a root `notes.md` file passes because it is kebab-case while the root
  contract intends to allow only declared project files.

### 6. Tests Required

- CLI fixture rejects an unknown direct file.
- CLI fixture rejects an unknown direct directory.
- CLI fixture accepts exact names, configured children, and allowed patterns.
- CLI fixture proves forbidden patterns override broad allowed patterns.
- CLI fixture covers `exists:0`, `exists:1`, and `exists:N-M`.
- LS-Lint compatibility tests cover `.dir`, wildcard extension rules, and
  direct-current-directory existence checks.

### 7. Wrong vs Correct

#### Wrong

```yaml
structure:
  files:
    naming: kebab-case
```

This only rejects badly named files. It does not define what files should
exist, so unexpected well-named files can accumulate.

#### Correct

```yaml
structure:
  files:
    allowed_names:
      - README.md
      - Cargo.toml
    allow_extra: false
  directories:
    allowed_names:
      - src
      - tests
    allow_extra: false
```

This states the expected direct contents and rejects undeclared drift.
