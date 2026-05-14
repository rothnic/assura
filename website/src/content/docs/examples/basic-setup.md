---
title: Basic Project Setup
description: Set up Assura in a small Rust project
template: doc
sidebar:
  order: 1
---

import { Steps, FileTree } from '@astrojs/starlight/components';

This example uses the current supported CLI surface.

## Project Shape

<FileTree>
- my-rust-project/
  - .assura/
    - config.yml
  - src/
    - main.rs
  - Cargo.toml
  - README.md
</FileTree>

<Steps>

1. **Create a Rust project**

   ```bash
   cargo new my-rust-project
   cd my-rust-project
   ```

2. **Install Assura**

   ```bash
   cargo install assura
   ```

3. **Initialize Assura**

   ```bash
   assura init
   ```

4. **Run validation**

   ```bash
   assura check
   ```

5. **Add a stricter shape**

   Edit `.assura/config.yml`:

   ```yaml
   structure:
     ./:
       files:
         allowed_names:
           - Cargo.toml
           - README.md
         allow_extra: false
       directories:
         allowed_names:
           - src
         allow_extra: false
       children:
         src/:
           files:
             naming_patterns:
               "*.rs": snake_case
   exclude:
     - "target/**"
   ```

6. **Check again**

   ```bash
   assura check --format text
   ```

</Steps>

## CI Snippet

```yaml
name: Assura
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install assura
      - run: assura check --format text
```
