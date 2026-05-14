---
title: Custom Constraints
description: Current options and roadmap for project-specific validation
template: doc
sidebar:
  order: 2
---

import { Aside } from '@astrojs/starlight/components';

Assura v0.1 does not expose a stable custom constraint plugin API. Use the
supported structure configuration first, and run language-specific or custom
tools beside Assura in CI when you need checks outside the current rule set.

<Aside type="caution" title="Not a v0.1 plugin surface">
  Rust constraint traits, TypeScript plugins, and runtime extension hooks are
  roadmap items. Do not rely on examples from older docs or internal modules as
  public APIs.
</Aside>

## Supported Customization Today

Use `.assura/config.yml` to express project shape:

```yaml
structure:
  ./:
    files:
      naming: kebab-case
      allowed_names:
        - README.md
        - Cargo.toml
    directories:
      naming: kebab-case
      allowed_names:
        - src
        - tests
    children:
      src:
        files:
          naming: snake_case
          extensions:
            rs: snake_case
exclude:
  - "target/**"
  - "node_modules/**"
```

Supported rule families include naming conventions, allowed names, direct-child
existence counts, extension rules, directory naming, markdown rules, and
exclusions.

## Pairing With Other Tools

For checks outside Assura's current scope, run tools side by side:

```bash
assura check --format text .
cargo clippy --all-targets --all-features -- -D warnings
pnpm lint
```

This keeps Assura responsible for repository shape while language-specific
tools handle language semantics.

## Future Direction

Future work is expected to add agent-facing feedback and quality measurement
before exposing broad plugin contracts. Until that is implemented and tested,
document custom behavior as external CI checks rather than Assura plugins.
