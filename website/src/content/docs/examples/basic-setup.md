---
title: Basic Project Setup
description: Set up Assura in a small Rust project
template: doc
sidebar:
  order: 1
---

This example uses the current supported CLI surface.

## Project Shape

```text
my-rust-project/
  .assura/
    config.yml
  src/
    main.rs
  Cargo.toml
  README.md
```

1. **Create a Rust project**

   ```bash
   cargo new my-rust-project
   cd my-rust-project
   ```

2. **Install Assura**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
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

## CI Snippet

```yaml
name: Assura
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
      - run: assura check --format text
```
