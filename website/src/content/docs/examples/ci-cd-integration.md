---
title: CI/CD Integration
description: How to integrate Assura into your continuous integration and deployment pipelines
template: doc
sidebar:
  order: 3
---

import { Tabs, TabItem, Aside, Steps, Card, CardGrid } from '@astrojs/starlight/components';

This example shows how to integrate Assura into various CI/CD platforms to enforce code quality checks automatically.

## Overview

Integrating Assura into your CI/CD pipeline ensures that code quality checks run automatically on every commit, preventing issues from reaching production.

<CardGrid>
  <Card title="GitHub Actions" icon="github">
    Native integration with GitHub workflows
  </Card>
  <Card title="GitLab CI" icon="gitlab">
    GitLab CI/CD pipeline configuration
  </Card>
  <Card title="CircleCI" icon="circleci">
    CircleCI configuration examples
  </Card>
  <Card title="Jenkins" icon="jenkins">
    Jenkins pipeline and freestyle jobs
  </Card>
</CardGrid>

## GitHub Actions

### Basic Setup

Create `.github/workflows/assura.yml`:

```yaml
name: Assura Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  validate:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Cache Assura
        uses: actions/cache@v3
        with:
          path: ~/.cargo/bin/assura
          key: ${{ runner.os }}-assura
      
      - name: Install Assura
        run: |
          if ! command -v assura &> /dev/null; then
            cargo install assura
          fi
      
      - name: Run validation
        run: assura validate --format check
```

### Advanced Setup with Reporting

```yaml
name: Assura Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  validate:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for better analysis
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
      
      - name: Install Assura
        run: cargo install assura
      
      - name: Run validation
        id: validate
        run: |
          assura validate --format json > assura-report.json
          echo "exit_code=$?" >> $GITHUB_OUTPUT
        continue-on-error: true
      
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: assura-report
          path: assura-report.json
      
      - name: Comment PR
        if: github.event_name == 'pull_request' && failure()
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('assura-report.json', 'utf8'));
            
            const issues = report.results || [];
            const critical = issues.filter(i => i.severity === 'critical').length;
            const high = issues.filter(i => i.severity === 'high').length;
            
            const body = `## Assura Validation Report
            
            ❌ **${issues.length}** issues found
            
            | Severity | Count |
            |----------|-------|
            | Critical | ${critical} |
            | High | ${high} |
            | Medium | ${issues.filter(i => i.severity === 'medium').length} |
            | Low | ${issues.filter(i => i.severity === 'low').length} |
            
            Please fix the reported issues before merging.`;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
      
      - name: Fail on validation errors
        if: steps.validate.outputs.exit_code != '0'
        run: exit 1
```

## GitLab CI

### Basic Configuration

Add to `.gitlab-ci.yml`:

```yaml
stages:
  - validate
  - test
  - build

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/bin/
    - target/

assura:validate:
  stage: validate
  image: rust:latest
  before_script:
    - |
      if ! command -v assura &> /dev/null; then
        cargo install assura
      fi
  script:
    - assura validate --format check
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

### With Merge Request Reports

```yaml
assura:validate:
  stage: validate
  image: rust:latest
  before_script:
    - cargo install assura
  script:
    - assura validate --format json > assura-report.json || true
    - |
      # Convert to GitLab code quality format
      cat assura-report.json | jq '[.results[] | {
        description: .message,
        check_name: .rule,
        fingerprint: "\(.file):\(.line):\(.rule)",
        severity: (if .severity == "critical" then "blocker"
                   elif .severity == "high" then "critical"
                   elif .severity == "medium" then "major"
                   else "minor" end),
        location: {
          path: .file,
          lines: {
            begin: (.line // 1)
          }
        }
      }]' > codequality-report.json
  artifacts:
    reports:
      codequality: codequality-report.json
    paths:
      - assura-report.json
    expire_in: 1 week
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

## CircleCI

### Basic Configuration

Add to `.circleci/config.yml`:

```yaml
version: 2.1

jobs:
  validate:
    docker:
      - image: cimg/rust:1.70
    steps:
      - checkout
      - restore_cache:
          keys:
            - v1-assura-{{ checksum "Cargo.lock" }}
            - v1-assura-
      - run:
          name: Install Assura
          command: |
            if ! command -v assura &> /dev/null; then
              cargo install assura
            fi
      - save_cache:
          paths:
            - ~/.cargo/bin/assura
          key: v1-assura-{{ checksum "Cargo.lock" }}
      - run:
          name: Run Assura validation
          command: assura validate --format check

workflows:
  version: 2
  build-and-validate:
    jobs:
      - validate
```

## Jenkins

### Pipeline (Jenkinsfile)

```groovy
pipeline {
    agent any
    
    tools {
        rust 'rust-1.70'
    }
    
    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }
        
        stage('Install Assura') {
            steps {
                sh '''
                    if ! command -v assura &> /dev/null; then
                        cargo install assura
                    fi
                '''
            }
        }
        
        stage('Validate') {
            steps {
                sh 'assura validate --format check'
            }
        }
    }
    
    post {
        failure {
            mail to: team@example.com,
                 subject: "Validation Failed: ${env.JOB_NAME} - ${env.BUILD_NUMBER}",
                 body: "Assura validation failed. Check ${env.BUILD_URL}"
        }
    }
}
```

## Azure DevOps

### Azure Pipelines

Create `azure-pipelines.yml`:

```yaml
trigger:
  - main
  - develop

pr:
  - main
  - develop

pool:
  vmImage: 'ubuntu-latest'

steps:
  - task: Cache@2
    inputs:
      key: 'assura | "$(Agent.OS)"'
      path: $(HOME)/.cargo/bin/assura
    displayName: Cache Assura

  - script: |
      if ! command -v assura &> /dev/null; then
        cargo install assura
      fi
    displayName: Install Assura

  - script: assura validate --format check
    displayName: Run Assura validation
```

## Bitbucket Pipelines

Add to `bitbucket-pipelines.yml`:

```yaml
image: rust:1.70

pipelines:
  default:
    - step:
        name: Validate
        caches:
          - cargo
        script:
          - |
            if ! command -v assura &> /dev/null; then
              cargo install assura
            fi
          - assura validate --format check
  
  pull-requests:
    '**':
      - step:
          name: Validate
          script:
            - cargo install assura
            - assura validate --format check
```

## Best Practices

<Steps>

1. **Use --format check for CI**

   The check format provides minimal output and appropriate exit codes:
   
   ```bash
   assura validate --format check
   ```

2. **Cache the Assura binary**

   Always cache the installed binary to speed up builds:
   
   ```yaml
   - uses: actions/cache@v3
     with:
       path: ~/.cargo/bin/assura
       key: ${{ runner.os }}-assura
   ```

3. **Run on pull requests**

   Ensure validation runs on PRs to catch issues early:
   
   ```yaml
   on:
     pull_request:
       branches: [main]
   ```

4. **Generate reports**

   Generate JSON reports for further processing:
   
   ```bash
   assura validate --format json > report.json
   ```

5. **Fail fast**

   For quick feedback in CI, enable fail-fast mode:
   
   ```yaml
   settings:
     fail_fast: true
   ```

</Steps>

## Integration with Other Tools

### Combine with Clippy

```yaml
- name: Run all checks
  run: |
    cargo fmt --check
    cargo clippy -- -D warnings
    assura validate --format check
```

### Generate Markdown Reports

```yaml
- name: Generate report
  run: assura validate --format markdown > VALIDATION_REPORT.md
  
- name: Upload report
  uses: actions/upload-artifact@v3
  with:
    name: validation-report
    path: VALIDATION_REPORT.md
```

### Slack Notifications

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Assura validation failed'
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

<Aside type="tip" title="Exit Codes">
  Assura returns these exit codes:
  - `0` - No issues found
  - `1` - Issues found (when using --format check)
  - `2` - Configuration error
  - `3` - Runtime error
</Aside>

## Troubleshooting

### Issue: Assura not found

**Solution**: Ensure cargo bin directory is in PATH:

```yaml
- run: echo "$HOME/.cargo/bin" >> $GITHUB_PATH
```

### Issue: Slow installation

**Solution**: Use caching more aggressively:

```yaml
- uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      ~/.cargo/bin/assura
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### Issue: False positives on generated files

**Solution**: Update your `.assura/config.yml`:

```yaml
excludes:
  - "**/generated/**/*"
  - "**/*.gen.rs"
```

<Aside type="note" title="Enterprise Support">
  For enterprise CI/CD integrations, contact us through [GitHub Discussions](https://github.com/anomalyco/assura/discussions).
</Aside>
