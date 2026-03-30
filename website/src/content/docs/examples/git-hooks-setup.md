---
title: Git Hooks Setup
description: How to configure Git hooks for automatic validation on commits
template: doc
sidebar:
  order: 4
---

import { Steps, Tabs, TabItem, Aside, FileTree } from '@astrojs/starlight/components';

This example shows how to set up Git hooks to run Assura validation automatically when committing code.

## Overview

Git hooks allow you to run scripts at various points in the Git workflow:

- **pre-commit**: Run before creating a commit
- **pre-push**: Run before pushing to a remote
- **commit-msg**: Validate commit messages
- **post-checkout**: Run after switching branches

## Basic pre-commit Hook

<Steps>

1. **Create the hook file**

   Create `.git/hooks/pre-commit`:

   ```bash
   #!/bin/bash
   # Pre-commit hook for Assura validation

   echo "Running Assura validation..."

   # Check if assura is installed
   if ! command -v assura &> /dev/null; then
       echo "Error: assura is not installed"
       echo "Install with: cargo install assura"
       exit 1
   fi

   # Run validation
   assura validate --format check

   # Capture exit code
   EXIT_CODE=$?

   if [ $EXIT_CODE -ne 0 ]; then
       echo ""
       echo "❌ Validation failed!"
       echo "Please fix the issues above before committing."
       echo "To bypass this check, use: git commit --no-verify"
       exit 1
   fi

   echo "✅ Validation passed!"
   exit 0
   ```

2. **Make the hook executable**

   ```bash
   chmod +x .git/hooks/pre-commit
   ```

3. **Test the hook**

   Try to commit code that violates rules:

   ```bash
   echo "TODO: fix this" > test.txt
   git add test.txt
   git commit -m "test commit"
   ```

</Steps>

## Pre-push Hook

Create `.git/hooks/pre-push`:

```bash
#!/bin/bash
# Pre-push hook for comprehensive validation

echo "Running pre-push validation..."

# Get the list of files being pushed
files=$(git diff --name-only HEAD @{push} 2>/dev/null || git diff --name-only HEAD origin/$(git branch --show-current) 2>/dev/null)

if [ -z "$files" ]; then
    echo "No files to validate"
    exit 0
fi

echo "Validating changed files..."

# Run validation
assura validate --format check

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Validation failed!"
    echo "Please fix the issues before pushing."
    exit 1
fi

echo "✅ Validation passed!"
exit 0
```

Make it executable:

```bash
chmod +x .git/hooks/pre-push
```

## Advanced pre-commit with Staged Files Only

Validate only files that are staged for commit:

```bash
#!/bin/bash
# Pre-commit hook validating only staged files

echo "Running Assura validation on staged files..."

# Get list of staged Rust files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "No Rust files staged for commit"
    exit 0
fi

echo "Validating files:"
echo "$STAGED_FILES"

# Run validation only on staged files
assura validate $STAGED_FILES --format check

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "❌ Validation failed!"
    exit 1
fi

echo "✅ Validation passed!"
exit 0
```

## Commit Message Validation

Create `.git/hooks/commit-msg`:

```bash
#!/bin/bash
# Validate commit message format

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(head -n1 "$COMMIT_MSG_FILE")

# Check for conventional commits format
if ! echo "$COMMIT_MSG" | grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+"; then
    echo "❌ Invalid commit message format"
    echo ""
    echo "Commit message must follow conventional commits:"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo "Types: feat, fix, docs, style, refactor, test, chore"
    echo ""
    echo "Example: feat(auth): add login validation"
    exit 1
fi

# Check message length
MSG_LENGTH=${#COMMIT_MSG}
if [ $MSG_LENGTH -gt 72 ]; then
    echo "❌ Commit message too long ($MSG_LENGTH chars, max 72)"
    exit 1
fi

echo "✅ Commit message format valid"
exit 0
```

## Using pre-commit Framework

For easier management, use the [pre-commit](https://pre-commit.com/) framework:

<Steps>

1. **Install pre-commit**

   ```bash
   pip install pre-commit
   # or
   brew install pre-commit
   ```

2. **Create `.pre-commit-config.yaml`**

   ```yaml
   repos:
     - repo: local
       hooks:
         - id: assura-validate
           name: Assura Validation
           entry: assura validate --format check
           language: system
           pass_filenames: false
           always_run: true
           stages: [pre-commit]
         
         - id: assura-staged
           name: Assura Validate Staged
           entry: assura validate
           language: system
           files: '\.rs$'
           stages: [pre-commit]
         
         - id: rust-fmt
           name: Rust Format
           entry: cargo fmt -- --check
           language: system
           files: '\.rs$'
           pass_filenames: false
         
         - id: rust-clippy
           name: Rust Clippy
           entry: cargo clippy -- -D warnings
           language: system
           files: '\.rs$'
           pass_filenames: false
   ```

3. **Install the hooks**

   ```bash
   pre-commit install
   ```

4. **Run manually**

   ```bash
   pre-commit run --all-files
   ```

</Steps>

## Husky (Node.js Projects)

For Node.js projects, use Husky:

<Steps>

1. **Install Husky**

   ```bash
   npm install --save-dev husky
   npx husky init
   ```

2. **Add Assura hook**

   ```bash
   echo 'assura validate --format check' > .husky/pre-commit
   ```

3. **Add multiple checks**

   Edit `.husky/pre-commit`:

   ```bash
   #!/bin/sh
   . "$(dirname "$0")/_/husky.sh"

   echo "Running pre-commit checks..."

   # Format check
   npm run format:check

   # Lint
   npm run lint

   # Assura validation
   assura validate --format check

   echo "✅ All checks passed!"
   ```

</Steps>

## Lefthook (Alternative to Husky)

[Lefthook](https://github.com/evilmartians/lefthook) is a fast and powerful Git hooks manager:

<Steps>

1. **Install Lefthook**

   ```bash
   # macOS
   brew install lefthook

   # Linux
   npm install lefthook --save-dev
   ```

2. **Create `lefthook.yml`**

   ```yaml
   pre-commit:
     parallel: true
     commands:
       assura:
         run: assura validate --format check
       fmt:
         glob: "*.rs"
         run: cargo fmt -- --check {staged_files}
       clippy:
         run: cargo clippy -- -D warnings

   pre-push:
     commands:
       test:
         run: cargo test
       assura-full:
         run: assura validate --format check
   ```

3. **Install hooks**

   ```bash
   lefthook install
   ```

</Steps>

## Shared Git Hooks (Team Setup)

To share hooks with your team:

<Steps>

1. **Create a hooks directory in your repo**

   ```bash
   mkdir -p scripts/git-hooks
   ```

2. **Add hooks to the repo**

   Copy your hooks to `scripts/git-hooks/`:

   <FileTree>
   - scripts/
     - git-hooks/
       - pre-commit
       - pre-push
       - commit-msg
   </FileTree>

3. **Create a setup script**

   Create `scripts/setup-hooks.sh`:

   ```bash
   #!/bin/bash
   # Setup script for shared Git hooks

   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
   REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

   echo "Setting up Git hooks..."

   # Copy hooks
   cp "$SCRIPT_DIR/git-hooks/"* "$REPO_ROOT/.git/hooks/"

   # Make executable
   chmod +x "$REPO_ROOT/.git/hooks/"*

   echo "✅ Git hooks installed!"
   ```

4. **Add to README**

   ```markdown
   ## Setup

   After cloning, run:
   ```bash
   ./scripts/setup-hooks.sh
   ```
   ```

</Steps>

## Conditional Validation

Only run validation on certain conditions:

```bash
#!/bin/bash
# Conditional pre-commit hook

# Skip validation for WIP commits
if grep -q "WIP" "$1" 2>/dev/null || [ "$(git rev-parse --abbrev-ref HEAD)" = "WIP" ]; then
    echo "Skipping validation for WIP"
    exit 0
fi

# Skip validation if env var is set
if [ -n "$SKIP_VALIDATION" ]; then
    echo "Skipping validation (SKIP_VALIDATION is set)"
    exit 0
fi

# Run validation
assura validate --format check
exit $?
```

## Performance Optimization

For large repos, validate incrementally:

```bash
#!/bin/bash
# Fast pre-commit for large repositories

echo "Running incremental validation..."

# Get list of changed files since last successful commit
LAST_COMMIT=$(git rev-parse HEAD)
CHANGED_FILES=$(git diff-tree --no-commit-id --name-only -r $LAST_COMMIT 2>/dev/null || echo "")

if [ -z "$CHANGED_FILES" ]; then
    echo "No changes to validate"
    exit 0
fi

# Filter to only relevant files
FILES_TO_VALIDATE=$(echo "$CHANGED_FILES" | grep '\.rs$' || true)

if [ -z "$FILES_TO_VALIDATE" ]; then
    echo "No Rust files changed"
    exit 0
fi

# Run validation with caching
assura validate --format check --cache
exit $?
```

## Bypassing Hooks

<Aside type="caution" title="Use with Care">
  Only bypass hooks when absolutely necessary!
</Aside>

```bash
# Skip pre-commit hook
git commit --no-verify -m "emergency fix"

# Skip pre-push hook
git push --no-verify
```

## Troubleshooting

### Hook not running

Check that the hook is executable:

```bash
ls -la .git/hooks/pre-commit
# Should show -rwxr-xr-x

# If not executable:
chmod +x .git/hooks/pre-commit
```

### Assura not found

Ensure assura is in your PATH:

```bash
# Add to hook
export PATH="$HOME/.cargo/bin:$PATH"
```

### Slow commits

Use staged file validation only:

```bash
# Only validate staged files
STAGED=$(git diff --cached --name-only)
assura validate $STAGED
```

<Aside type="tip" title="Best Practices">
  - Keep hooks fast (< 5 seconds)
  - Use `--format check` for minimal output
  - Only validate relevant files
  - Cache results when possible
  - Document how to bypass in emergencies
</Aside>
