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
   assura init --recipe agentic-core --recipe structure-health
   ```

4. **Run validation**

   ```bash
   assura check
   ```

5. **Add a stricter shape**

   Edit `.assura/config.yml`:

   ```yaml
   rules:
     closed-entry:
       exists: 0
     closed:
       ./*/: $closed-entry
       ./*: $closed-entry

   structure:
     ./: $closed
     README.md: exists:1
     Cargo.toml: exists:1
     src/: exists:1
     ./*.lock: exists:0-1
     ./**/:
       .rs: snake_case
   exclude:
     - "target/**"
   ```

   The root policy mirrors the project tree. It requires the public files,
   allows one lockfile, closes direct root contents through a reusable rule,
   and checks Rust source names at every directory depth.

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
