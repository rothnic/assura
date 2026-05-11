---
title: 'Monorepo Syntax Comparison - Realistic Examples'
status: historical
---

# Monorepo Syntax Comparison: Realistic Examples

Three complete approaches for the same monorepo to see which works best.

---

## The Monorepo Structure

```
my-monorepo/
├── apps/
│   ├── web/              # Next.js web app
│   │   ├── src/
│   │   │   ├── components/     # React components (PascalCase)
│   │   │   ├── hooks/          # React hooks (camelCase)
│   │   │   ├── utils/          # Utilities (camelCase)
│   │   │   └── styles/         # Global styles (kebab-case)
│   │   ├── public/
│   │   └── tests/              # E2E tests
│   │
│   └── admin/            # Admin dashboard
│       └── src/
│           └── components/
│
├── packages/
│   ├── ui/               # Shared UI library
│   │   ├── src/
│   │   │   ├── Button/
│   │   │   │   ├── Button.tsx
│   │   │   │   └── Button.test.tsx
│   │   │   └── Card/
│   │   └── package.json
│   │
│   ├── utils/            # Shared utilities
│   │   └── src/
│   │
│   └── types/            # Shared types
│       └── src/
│
├── internal/             # Internal tools
│   └── scripts/
│       └── migrations/
│
├── docs/                 # Documentation
│   ├── guides/
│   ├── api/
│   └── templates/
│
└── AGENTS.md
```

**Requirements:**
1. Apps (web, admin): 400 line limit, camelCase/PascalCase
2. Packages (ui, utils, types): 300 line limit, require AGENTS.md
3. UI library: Components use PascalCase styles too
4. Internal tools: 600 line limit, more relaxed
5. Tests: Unit tests 600 lines, E2E tests 1000 lines
6. Docs: Standard docs 500 lines, templates 2000 lines

---

## Approach A: Boolean Syntax

```yaml
# Groups define reusable rule sets
groups:
  typescript-app:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 400
  
  typescript-lib:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 300
    require_docs: true
  
  typescript-internal:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 600
  
  react-components:
    extensions: [tsx]
    naming: PascalCase
    max_lines: 400
  
  styles-global:
    extensions: [css, scss]
    naming: kebab-case
  
  styles-component:
    extensions: [css, scss, module.css, module.scss]
    naming: PascalCase
  
  unit-tests:
    extensions: [test.ts, test.tsx]
    naming: snake_case
    max_lines: 600
  
  e2e-tests:
    extensions: [test.ts, test.tsx]
    naming: snake_case
    max_lines: 1000
  
  standard-docs:
    extensions: [md]
    naming: kebab-case
    max_lines: 500
    require_frontmatter: true
  
  template-docs:
    extensions: [md]
    naming: kebab-case
    max_lines: 2000

rules:
  # Root requirements
  README.md:
    exists: true
  AGENTS.md:
    exists: true
  
  # Root markdown whitelist
  .md:
    allow: [README.md, AGENTS.md, CHANGELOG.md]
  
  # Apps
  apps/web/src/:
    @typescript-app: true
    @styles-global: true
    
    components/:
      @react-components: true
      @styles-component: true
    
    hooks/:
      @typescript-app: true
    
    utils/:
      @typescript-app: true
    
    styles/:
      @styles-global: true
  
  apps/admin/src/:
    @typescript-app: true
    
    components/:
      @react-components: true
  
  apps/web/tests/:
    @e2e-tests: true
  
  # Packages
  packages/*/:
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src]
  
  packages/ui/src/:
    @typescript-lib: true
    @styles-component: true
  
  packages/utils/src/:
    @typescript-lib: true
  
  packages/types/src/:
    @typescript-lib: true
  
  # Unit tests in packages (colocated)
  packages/*/src/**/*.test.ts:
    @unit-tests: true
  
  # Internal tools
  internal/:
    @typescript-internal: true
  
  # Documentation
  docs/:
    @standard-docs: true
  
  docs/api/:
    @template-docs: true
  
  docs/templates/:
    @template-docs: true

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/.next/**"
```

---

## Approach B: Use Array Syntax

```yaml
groups:
  typescript-app:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 400
  
  typescript-lib:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 300
    require_docs: true
  
  typescript-internal:
    extensions: [ts, tsx]
    naming: camelCase
    max_lines: 600
  
  react-components:
    extensions: [tsx]
    naming: PascalCase
    max_lines: 400
  
  styles-global:
    extensions: [css, scss]
    naming: kebab-case
  
  styles-component:
    extensions: [css, scss, module.css, module.scss]
    naming: PascalCase
  
  tests-unit:
    extensions: [test.ts, test.tsx]
    naming: snake_case
    max_lines: 600
  
  tests-e2e:
    extensions: [test.ts, test.tsx]
    naming: snake_case
    max_lines: 1000
  
  docs-standard:
    extensions: [md]
    naming: kebab-case
    max_lines: 500
    require_frontmatter: true
  
  docs-templates:
    extensions: [md]
    naming: kebab-case
    max_lines: 2000

rules:
  README.md:
    exists: true
  AGENTS.md:
    exists: true
  
  .md:
    allow: [README.md, AGENTS.md, CHANGELOG.md]
  
  # Apps - single use array for multiple groups
  apps/web/src/:
    use: [@typescript-app, @styles-global]
    
    components/:
      use: [@react-components, @styles-component]
    
    hooks/:
      use: [@typescript-app]
    
    utils/:
      use: [@typescript-app]
    
    styles/:
      use: [@styles-global]
  
  apps/admin/src/:
    use: [@typescript-app]
    
    components/:
      use: [@react-components]
  
  apps/web/tests/:
    use: [@tests-e2e]
  
  # Packages
  packages/*/:
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src]
  
  packages/ui/src/:
    use: [@typescript-lib, @styles-component]
  
  packages/utils/src/:
    use: [@typescript-lib]
  
  packages/types/src/:
    use: [@typescript-lib]
  
  packages/*/src/**/*.test.ts:
    use: [@tests-unit]
  
  internal/:
    use: [@typescript-internal]
  
  docs/:
    use: [@docs-standard]
  
  docs/api/:
    use: [@docs-templates]
  
  docs/templates/:
    use: [@docs-templates]

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/.next/**"
```

---

## Approach C: Inline Overrides

```yaml
groups:
  typescript-base:
    extensions: [ts, tsx]
    naming: camelCase
  
  typescript-lib:
    extends: typescript-base
    max_lines: 300
    require_docs: true
  
  typescript-app:
    extends: typescript-base
    max_lines: 400
  
  typescript-internal:
    extends: typescript-base
    max_lines: 600
  
  react-component:
    extends: typescript-app
    naming: PascalCase
  
  styles-base:
    extensions: [css, scss, module.css, module.scss]
  
  styles-kebab:
    extends: styles-base
    naming: kebab-case
  
  styles-pascal:
    extends: styles-base
    naming: PascalCase
  
  tests-base:
    extensions: [test.ts, test.tsx]
    naming: snake_case
  
  docs-base:
    extensions: [md]
    naming: kebab-case

rules:
  README.md:
    exists: true
  AGENTS.md:
    exists: true
  
  .md:
    allow: [README.md, AGENTS.md, CHANGELOG.md]
  
  # Apps with inline overrides
  apps/web/src/:
    @typescript-app:
      # Override severity
      severity: error
    
    @styles-kebab:
      severity: warn
    
    components/:
      @react-component: true
      @styles-pascal: true
    
    hooks/:
      @typescript-app:
        # Override just max_lines
        max_lines: 300  # Hooks should be smaller
  
  apps/admin/src/:
    @typescript-app: true
    
    components/:
      @react-component: true
  
  apps/web/tests/:
    @tests-base:
      max_lines: 1000  # E2E tests can be longer
  
  # Packages
  packages/*/:
    require:
      files: [AGENTS.md, README.md, package.json]
      dirs: [src]
  
  packages/ui/src/:
    @typescript-lib:
      severity: error  # Libraries must be strict
    @styles-pascal: true
  
  packages/utils/src/:
    @typescript-lib: true
  
  packages/types/src/:
    @typescript-lib: true
  
  packages/*/src/**/*.test.ts:
    @tests-base:
      max_lines: 600  # Unit tests shorter
  
  internal/:
    @typescript-internal:
      severity: warn  # Internal tools more lenient
  
  docs/:
    @docs-base:
      max_lines: 500
      require_frontmatter: true
  
  docs/api/:
    @docs-base:
      max_lines: 2000
  
  docs/templates/:
    @docs-base:
      max_lines: 2000
      require_frontmatter: false

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/.next/**"
```

---

## Comparison Summary

| Aspect | A: Boolean | B: Use Array | C: Inline Override |
|--------|------------|--------------|-------------------|
| **Lines of config** | 145 | 135 | 155 |
| **Clarity** | Good | Good | Good |
| **Override capability** | ❌ None | ❌ None | ✅ Full |
| **Multiple groups** | Verbose | ✅ Clean | Verbose |
| **Learning curve** | Low | Low-Med | Medium |
| **Flexibility** | Low | Low | High |

---

## Verdict by Use Case

**Use Boolean (`@group: true`) when:**
- Simple monorepos
- No need for overrides
- Groups are complete as-is
- Most common case

**Use Array (`use: [@g1, @g2]`) when:**
- Multiple groups per path
- Want to see all groups at a glance
- Composability is important

**Use Inline Override (`@group: {override}`) when:**
- Need to tweak group settings
- Different severity per path
- Fine-grained control needed

---

## Recommendation: Hybrid Approach

Support all three syntaxes:

```yaml
rules:
  # Simple: boolean
  src/components/:
    @react-components: true
  
  # Multiple: array
  src/pages/:
    use: [@typescript-app, @styles-global]
  
  # Override: inline object
  src/generated/:
    @typescript-app:
      max_lines: 5000
      severity: warn
```

This gives maximum flexibility without forcing verbosity.

---

## Complete Real-World Example (Hybrid)

```yaml
groups:
  ts-base:
    extensions: [ts, tsx]
    naming: camelCase
  
  ts-app:
    extends: ts-base
    max_lines: 400
  
  ts-lib:
    extends: ts-base
    max_lines: 300
    require_docs: true
  
  react-comp:
    extends: ts-app
    naming: PascalCase
  
  styles:
    extensions: [css, scss, module.css, module.scss]
  
  tests:
    extensions: [test.ts, test.tsx]
    naming: snake_case

rules:
  # Root
  README.md: { exists: true }
  AGENTS.md: { exists: true }
  
  # Web app - use array for multiple
  apps/web/src/:
    use: [@ts-app, @styles]
    
    components/: @react-comp: true
    hooks/: @ts-app: true
  
  # Packages - boolean
  packages/*/:
    require: [AGENTS.md, README.md]
  
  packages/ui/src/: @ts-lib: true
  
  # Unit tests - override inline
  "**/*.test.ts":
    @tests:
      max_lines: 600
  
  # E2E tests - override inline
  apps/web/e2e/:
    @tests:
      max_lines: 1000
  
  # Generated - override inline
  src/generated/:
    @ts-app:
      naming: [camelCase, PascalCase, snake_case]
      max_lines: 5000

exclude:
  - "**/node_modules/**"
```

---

Which approach do you prefer for your monorepo?
