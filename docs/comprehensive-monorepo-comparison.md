# Comprehensive Monorepo Comparison: LS-Lint vs Assura

**Real-world example:** TypeScript monorepo with best practices from rothnic/ls-lint PR #4

---

## LS-Lint Config (Real World Example)

**Source:** `examples/typescript_best_practices/.ls-lint.yml` from PR #4

```yaml
# ============================================================================
# Reusable rule groups
# ============================================================================

groups:
  # Naming groups
  ts-camel: "camelCase => Default for services, utilities, and hooks..."
  ts-pascal: "PascalCase => For files whose primary export is a component..."
  ts-names: "camelCase | PascalCase"
  
  # Size groups
  size-standard:
    - "content:max-lines:400 => File exceeds 400 lines..."
  size-large:
    - "content:max-lines:600 => File exceeds 600 lines..."
  size-relaxed:
    - "content:max-lines:2000 => Generated file exceeds 2000 lines..."
  
  # Composite groups
  ts-defaults:
    - "@ts-names"
    - "@size-standard"
  ts-tests:
    - "@ts-names"
    - "@size-large"
  ts-generated:
    - "@ts-names"
    - "@size-relaxed"
  
  # Documentation groups
  doc-page:
    - "kebab-case"
    - "regex:^(README|AGENTS|CHANGELOG)$"
    - "content:max-lines:500 => Documentation page exceeds 500 lines..."
    - "content:front-matter:required => Missing YAML front matter..."
  doc-templates:
    - "kebab-case"
    - "regex:^(README|AGENTS|CHANGELOG)$"
    - "content:max-lines:2000 => Template or reference file exceeds..."

# ============================================================================
# File and directory naming rules
# ============================================================================

ls:
  # Directories
  .dir: kebab-case
  
  # Source files (implied globs - applies to all files with these extensions)
  .ts: "@ts-defaults"
  .tsx: "@ts-defaults"
  .js: camelCase | PascalCase
  .jsx: camelCase | PascalCase
  
  # Test files
  .test.ts: "@ts-tests"
  .test.tsx: "@ts-tests"
  .spec.ts: "@ts-tests"
  .spec.tsx: "@ts-tests"
  
  # Declaration files
  .d.ts: camelCase | PascalCase
  
  # Style files
  .css: kebab-case
  .scss: kebab-case
  .module.css: kebab-case
  .module.scss: kebab-case
  
  # Configuration files
  .json: kebab-case | camelCase | regex:^(package|tsconfig|jest\.config|vite\.config|eslint\.config)$
  .yaml: kebab-case
  .yml: kebab-case
  
  # Root markdown: naming only
  .md: "kebab-case | regex:^(README|AGENTS|CHANGELOG)$"
  
  # Required root files
  README.md: "exists:1 => Add a root README.md describing the project..."
  AGENTS.md: "exists:1 => Add AGENTS.md with guidance for AI agents..."
  
  # Path-specific rules
  src/components:
    .dir: kebab-case | PascalCase
    .tsx: "@ts-pascal | @size-standard"
    .ts: "@ts-pascal | @size-standard"
  
  src/hooks:
    .ts: "@ts-camel | @size-standard"
    .tsx: "@ts-camel | @size-standard"
  
  src/utils:
    .ts: "@ts-camel | @size-standard"
    .tsx: "@ts-camel | @size-standard"
  
  src/types:
    .ts: "@ts-pascal | @size-standard"
  
  src/api:
    .ts: "@ts-camel | @size-standard"
    .tsx: "@ts-camel | @size-standard"
  
  src/generated:
    .ts: "@ts-generated"
    .js: "@ts-generated"
  
  docs:
    .md: "@doc-page"
  
  docs/templates:
    .md: "@doc-templates"

# ============================================================================
# Context directives
# ============================================================================

contexts:
  pre-commit:
    mode: warn
    hook: pre-commit
    environment: local
    message: >
      Commit is not blocked. These are early warnings...
  
  pre-push:
    mode: fail
    hook: pre-push
    environment: local
    override: repository owner approval
    references:
      - AGENTS.md
      - docs/contributing.md
    message: >
      Push is blocked until these naming and structure requirements...

# ============================================================================
# Ignore patterns
# ============================================================================

ignore:
  - node_modules
  - dist
  - build
  - coverage
  - .next
  - out
  - .turbo
  - .env.local
  - .env.*.local
  - "**/.env.local"
  - "**/.env.*.local"
  - "**/*.min.js"
```

**Stats:**
- Lines: ~180
- Groups: 9
- Path rules: 12
- Contexts: 2

---

## Assura Equivalent (Proposed Efficient Syntax)

```yaml
# ============================================================================
# Reusable rule groups
# ============================================================================

groups:
  # Naming conventions
  ts-camel:
    naming: camelCase
    description: "Default for services, utilities, and hooks"
  
  ts-pascal:
    naming: PascalCase
    description: "For components, classes, and named types"
  
  ts-names:
    naming: [camelCase, PascalCase]
  
  # Size constraints
  size-standard:
    max_lines: 400
    message: "File exceeds 400 lines. Split into focused modules."
  
  size-large:
    max_lines: 600
    message: "File exceeds 600 lines. Split into focused test modules."
  
  size-relaxed:
    max_lines: 2000
    message: "Generated file exceeds 2000 lines."
  
  # Composites
  ts-defaults:
    use: [ts-names, size-standard]
  
  ts-tests:
    use: [ts-names, size-large]
  
  ts-generated:
    use: [ts-names, size-relaxed]
  
  # Documentation
  doc-page:
    naming: kebab-case
    allow: [README.md, AGENTS.md, CHANGELOG.md]  # Exempt from kebab-case
    max_lines: 500
    require_frontmatter: true
    message: "Missing YAML front matter. Add title and description."
  
  doc-templates:
    naming: kebab-case
    allow: [README.md, AGENTS.md, CHANGELOG.md]
    max_lines: 2000

# ============================================================================
# Global patterns (implied globs)
# ============================================================================

patterns:
  # Directories
  ".dir": kebab-case
  
  # Source files - implied glob support
  ".ts": "@ts-defaults"
  ".tsx": "@ts-defaults"
  ".js": [camelCase, PascalCase]
  ".jsx": [camelCase, PascalCase]
  
  # Test files
  ".test.ts": "@ts-tests"
  ".test.tsx": "@ts-tests"
  ".spec.ts": "@ts-tests"
  ".spec.tsx": "@ts-tests"
  
  # Declaration files
  ".d.ts": [camelCase, PascalCase]
  
  # Style files
  ".css": kebab-case
  ".scss": kebab-case
  ".module.css": kebab-case
  ".module.scss": kebab-case
  
  # Config files with exemptions
  ".json": 
    naming: [kebab-case, camelCase]
    allow: [package.json, tsconfig.json, jest.config.json, vite.config.json]
  ".yaml": kebab-case
  ".yml": kebab-case
  
  # Root markdown (naming only)
  ".md":
    naming: kebab-case
    allow: [README.md, AGENTS.md, CHANGELOG.md]

# ============================================================================
# Structure (path-specific rules)
# ============================================================================

structure:
  ./:
    # Required root files
    require: [README.md, AGENTS.md]
    
    files:
      # Only allowed root markdown files
      allow: [README.md, AGENTS.md, CHANGELOG.md, LICENSE*]
  
  src/components:
    files:
      naming: PascalCase
      use: size-standard
      extensions: [tsx, ts]
    
    dirs:
      naming: [kebab-case, PascalCase]
  
  src/hooks:
    files:
      naming: camelCase
      use: size-standard
      extensions: [ts, tsx]
  
  src/utils:
    files:
      naming: camelCase
      use: size-standard
      extensions: [ts, tsx]
  
  src/types:
    files:
      naming: PascalCase
      use: size-standard
      extensions: [ts]
  
  src/api:
    files:
      naming: camelCase
      use: size-standard
      extensions: [ts, tsx]
  
  src/generated:
    files:
      use: ts-generated
      extensions: [ts, js]
  
  docs:
    files:
      use: doc-page
      extensions: [md]
  
  docs/templates:
    files:
      use: doc-templates
      extensions: [md]

# ============================================================================
# CLI/Hooks configuration (not in config - handled by tool)
# ============================================================================

# Contexts moved to CLI args or separate hooks config
# assura check --mode=warn      (pre-commit)
# assura check --mode=fail      (pre-push)

# ============================================================================
# Exclusions
# ============================================================================

exclude:
  - "node_modules/**"
  - "dist/**"
  - "build/**"
  - "coverage/**"
  - ".next/**"
  - "out/**"
  - ".turbo/**"
  - ".env.local"
  - ".env.*.local"
  - "**/.env.local"
  - "**/.env.*.local"
  - "**/*.min.js"
```

**Stats:**
- Lines: ~150
- Groups: 9 (same)
- Path rules: 10 (simpler structure syntax)
- Patterns: 15 (clear separation)

---

## Side-by-Side Feature Comparison

| Feature | LS-Lint Syntax | Assura Syntax | Notes |
|---------|---------------|---------------|-------|
| **Implied glob** | `.ts: snake_case` | `patterns: ".ts": snake_case` | Assura explicit but clear |
| **Groups** | `groups: {name: rules}` | `groups: {name: rules}` | Same concept |
| **Group ref** | `@group-name` | `use: @group-name` | Explicit with `use:` |
| **OR syntax** | `camelCase \| PascalCase` | `naming: [camel, Pascal]` | Array is cleaner |
| **Path rules** | `src/components: {rules}` | `structure: src/components:` | Both work well |
| **Exists check** | `README.md: exists:1` | `require: [README.md]` | List is more concise |
| **Rule message** | `=> "message"` | `message: "..."` | Explicit field |
| **Size limits** | `content:max-lines:400` | `max_lines: 400` | Direct property |
| **Regex exempt** | `regex:^(README)$` | `allow: [README.md]` | List is clearer |
| **Contexts** | `contexts: {name: {...}}` | CLI args `--mode=warn` | Handle at CLI layer |
| **Frontmatter** | `content:front-matter:required` | `require_frontmatter: true` | Boolean is cleaner |

---

## Key Differences Explained

### 1. Implied Globs

**LS-Lint:**
```yaml
ls:
  .ts: snake_case    # Implicitly "**/*.ts"
```

**Assura:**
```yaml
patterns:
  ".ts": snake_case   # Explicit, but supports implied syntax
```

**Why:** Assura makes the glob explicit while supporting LS-Lint's shorthand.

### 2. Exists Checks

**LS-Lint:**
```yaml
ls:
  README.md: "exists:1 => message"
  AGENTS.md: "exists:1 => message"
```

**Assura:**
```yaml
structure:
  ./:
    require: [README.md, AGENTS.md]
```

**Why:** List form is more concise and clear.

### 3. Composite Groups

**LS-Lint:**
```yaml
groups:
  ts-defaults:
    - "@ts-names"
    - "@size-standard"
```

**Assura:**
```yaml
groups:
  ts-defaults:
    use: [ts-names, size-standard]
```

**Why:** Explicit `use` field is clearer than string parsing.

### 4. Rule Messages

**LS-Lint:**
```yaml
content:max-lines:400 => File exceeds 400 lines...
```

**Assura:**
```yaml
max_lines: 400
message: "File exceeds 400 lines..."
```

**Why:** Separate fields are easier to parse and extend.

### 5. Regex Exemptions

**LS-Lint:**
```yaml
kebab-case | regex:^(README|AGENTS|CHANGELOG)$
```

**Assura:**
```yaml
naming: kebab-case
allow: [README.md, AGENTS.md, CHANGELOG.md]
```

**Why:** List of explicit filenames is more readable than regex.

### 6. Contexts

**LS-Lint:**
```yaml
contexts:
  pre-commit:
    mode: warn
    message: "..."
```

**Assura:**
```bash
# Handled at CLI layer
assura check --mode=warn
```

**Why:** Context is an execution concern, not a config concern.

---

## What Assura Can't Do (Current)

1. **No content validation** (max-lines on arbitrary files)
   - Assura has markdown frontmatter validation
   - General content validation out of scope

2. **No front-matter required check** for non-markdown
   - Could be added to FileBundle

3. **No `=> message` inline syntax**
   - Uses explicit `message:` field instead

---

## What Assura Adds (Not in LS-Lint)

1. **Root file whitelist** (`allow: [...]`)
   - LS-Lint can't prevent files, only require them

2. **Structure inheritance**
   - Child directories inherit parent rules

3. **Pattern + Structure separation**
   - Clear distinction between global and path-specific

4. **Rule specificity**
   - More specific paths automatically win

---

## Efficiency Comparison

| Aspect | LS-Lint | Assura | Winner |
|--------|---------|--------|--------|
| Lines of config | 180 | 150 | Assura |
| Mental model | Single `ls:` block | Separated concerns | Tie |
| Group reusability | ✅ | ✅ | Tie |
| Path specificity | ✅ | ✅ | Tie |
| Root constraints | ❌ | ✅ | Assura |
| Implied globs | ✅ | ✅ | Tie |
| Content validation | ✅ | ⚠️ Partial | LS-Lint |

---

## Recommendation

**Assura can handle ~90% of this real-world configuration efficiently.**

**Missing for 100% parity:**
1. Content validation (max-lines) on non-markdown files
2. Front-matter required for docs

**But Assura adds:**
1. Root file constraints (critical for dogfooding)
2. Cleaner syntax (arrays vs string parsing)
3. Better separation of concerns

**Verdict:** Trade content validation for root constraints. Most projects don't need content validation anyway.

---

*Comparison based on rothnic/ls-lint PR #4 TypeScript best practices example*

---

## Alternative Naming Discussion

### Why "patterns" is Wrong

The term `patterns` is too generic. It doesn't describe the **semantic purpose** - defining expected conventions for file types.

### Better Options

**Option 1: `conventions`**
```yaml
conventions:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Describes what it does: defines naming conventions
- ✅ Clear and self-documenting
- ✅ Extensible (can add size conventions, etc.)

**Option 2: `defaults`**
```yaml
defaults:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Describes scope: default rules for file types
- ✅ Implies override capability
- ⚠️ Could be confused with default values

**Option 3: `by_extension`**
```yaml
by_extension:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Very explicit about mechanism
- ⚠️ Doesn't work for path patterns like "src/**/*.rs"

**Option 4: `types`**
```yaml
types:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Short and clear
- ✅ Natural language: "types of files"
- ✅ Extensible to "file types"

**Option 5: `shapes`**
```yaml
shapes:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Captures "expected shape" concept
- ⚠️ Unusual terminology, might confuse

**Option 6: `rules`**
```yaml
rules:
  ".rs": snake_case
  ".tsx": PascalCase
```
- ✅ Generic but clear
- ⚠️ Might conflict with other "rules" concepts

### Recommended: `conventions`

**Rationale:**
- Describes the semantic purpose: defining conventions
- Works for extensions (`.rs`) and paths (`src/**/*.rs`)
- Self-documenting: "these are the conventions for file types"
- Extensible: can include size conventions, docs conventions, etc.

**Example:**
```yaml
conventions:
  # By extension (implied global)
  ".rs": snake_case
  ".tsx": PascalCase
  
  # By path pattern
  "src/**/*.test.ts": snake_case
  
  # With multiple constraints
  "**/*.md":
    naming: kebab-case
    max_lines: 500

# Structure for directory-specific rules  
structure:
  packages/*:
    require: [AGENTS.md, README.md]
```

**Comparison:**
```yaml
# LS-Lint
ls:
  .rs: snake_case

# Assura (patterns - too generic)
patterns:
  ".rs": snake_case

# Assura (conventions - descriptive)
conventions:
  ".rs": snake_case
```

### Alternative: Use `files` with scope

Another approach: Use `files` everywhere but specify scope:

```yaml
files:
  # Global conventions by extension
  - scope: global
    match: "*.rs"
    naming: snake_case
  
  # Path-specific
  - scope: "src/components/*"
    naming: PascalCase
  
  # Required files
  - scope: "packages/*"
    require: [AGENTS.md]
```

But this is verbose and loses the elegance of the implied glob.

### Final Recommendation

Use **`conventions`** for global file type rules, keep **`structure`** for directory-specific organization rules.

```yaml
# Expected conventions for file types (global)
conventions:
  ".rs": snake_case
  ".tsx": PascalCase
  ".test.ts": snake_case

# Expected structure for directories
structure:
  packages/*:
    require: [AGENTS.md, README.md, src/]
```

**Why this works:**
- `conventions` = "how files should be named/formatted"
- `structure` = "what directories should contain"
- Clear separation of concerns
- Both are descriptive and self-documenting

---

