---
title: Basic Project Setup
description: Step-by-step guide to setting up Assura in a new project
template: doc
sidebar:
  order: 1
---

import { Steps, Tabs, TabItem, Aside, FileTree } from '@astrojs/starlight/components';

This example shows how to set up Assura in a basic Rust project from scratch.

## Project Structure

<FileTree>
- my-rust-project/
  - .assura/
    - config.yml
  - src/
    - main.rs
    - lib.rs
    - utils/
      - mod.rs
      - helpers.rs
  - tests/
    - integration_tests.rs
  - Cargo.toml
  - README.md
</FileTree>

## Step-by-Step Setup

<Steps>

1. **Create a new Rust project**

   ```bash
   cargo new my-rust-project
   cd my-rust-project
   ```

2. **Install Assura**

   ```bash
   cargo install assura
   ```

3. **Create the configuration directory**

   ```bash
   mkdir -p .assura
   ```

4. **Create the configuration file**

   Create `.assura/config.yml`:

   ```yaml
   name: My Rust Project
   description: A basic Rust project with Assura validation
   version: "1.0"

   settings:
     parallel: true
     max_workers: 4
     cache_enabled: true
     watch_delay: 100

   includes:
     - "src/**/*.rs"
     - "tests/**/*.rs"
     - "Cargo.toml"

   excludes:
     - "target/**/*"

   rules:
     - name: file-naming
       severity: high
       pattern: "^[a-z][a-z0-9_]*\\.rs$"
       message: "Rust files must use snake_case naming"

     - name: dependency-check
       severity: critical
       check_circular: true
       max_depth: 10

     - name: file-size
       severity: medium
       max_size: "100KB"

     - name: line-length
       severity: low
       max_length: 100
       ignore_urls: true
   ```

5. **Create sample source files**

   Create `src/utils/mod.rs`:

   ```rust
   pub mod helpers;
   ```

   Create `src/utils/helpers.rs`:

   ```rust
   /// Helper function for common operations
   pub fn format_message(name: &str) -> String {
       format!("Hello, {}!", name)
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_format_message() {
           assert_eq!(format_message("World"), "Hello, World!");
       }
   }
   ```

   Update `src/lib.rs`:

   ```rust
   pub mod utils;

   pub use utils::helpers::format_message;
   ```

   Update `src/main.rs`:

   ```rust
   use my_rust_project::format_message;

   fn main() {
       let message = format_message("Assura");
       println!("{}", message);
   }
   ```

6. **Run validation**

   ```bash
   assura validate
   ```

   Expected output:
   
   ```
   [INFO] Loading configuration from .assura/config.yml
   [INFO] Validating 5 files...
   [SUCCESS] All validations passed
   ```

</Steps>

## Testing Violations

Let's test that our rules work by creating a file with naming issues:

```bash
touch src/BadFile.rs
```

Run validation again:

```bash
assura validate
```

You should see:

```
[HIGH] src/BadFile.rs: File name doesn't match pattern: ^[a-z][a-z0-9_]*\.rs$
```

Fix the issue:

```bash
mv src/BadFile.rs src/bad_file.rs
```

Validation should now pass.

## Watch Mode Development

Enable continuous validation during development:

```bash
assura watch
```

Now when you edit files, Assura will automatically re-validate:

```
[WATCH] Watching for changes...
[WATCH] File changed: src/main.rs
[INFO] Re-validating...
[SUCCESS] All validations passed
```

Press `Ctrl+C` to stop watching.

## Integration with Cargo

Add Assura to your project's checks:

<Tabs>
<TabItem label="Makefile">
```makefile
.PHONY: check validate test

check:
	cargo check
	cargo clippy
	assura validate

validate:
	assura validate

test:
	cargo test
	assura validate --format check
```
</TabItem>
<TabItem label="Justfile">
```just
validate:
    assura validate

watch:
    assura watch

check: validate
    cargo check
    cargo clippy
    cargo test
```
</TabItem>
</Tabs>

## Next Steps

- Learn about [custom constraints](/examples/custom-constraints/)
- Set up [CI/CD integration](/examples/ci-cd-integration/)
- Configure [Git hooks](/examples/git-hooks-setup/)

<Aside type="tip" title="Quick Validation">
  Add an alias to your shell for quick validation:
  
  ```bash
  alias av='assura validate'
  alias aw='assura watch'
  ```
</Aside>
