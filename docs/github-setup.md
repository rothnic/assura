---
title: GitHub Setup
status: current
owner: maintainers
---

# GitHub Infrastructure Setup

## Overview
This document summarizes the GitHub infrastructure for the Assura project.

## Completed Tasks

### 2C.1 Repository Structure ✓
The following files have been created and committed:

**CI/CD Workflows:**
- `.github/workflows/ci.yml` - Rust CI pipeline (build, test, lint, fmt, coverage)
- `.github/workflows/release.yml` - Multi-platform GitHub release archive automation
- `.github/workflows/docs.yml` - Documentation deployment to GitHub Pages
- `.github/workflows/security.yml` - Security audit with cargo audit

**Issue Templates:**
- `.github/ISSUE_TEMPLATE/bug_report.md` - Bug report template
- `.github/ISSUE_TEMPLATE/feature_request.md` - Feature request template
- `.github/ISSUE_TEMPLATE/config.yml` - Issue template configuration

**Pull Request Template:**
- `.github/PULL_REQUEST_TEMPLATE.md` - PR template with checklist

**Documentation:**
- `CONTRIBUTING.md` - Comprehensive contribution guidelines
- `LICENSE` / `LICENSE-MIT` / `LICENSE-APACHE` - Dual MIT/Apache-2.0 license

## Next Steps to Complete Setup

### Step 1: Create GitHub Repository

Authenticate with GitHub CLI:
```bash
gh auth login
# OR set token:
export GH_TOKEN=your_github_token
```

Create the repository:
```bash
cd /Users/nroth/workspace/assura
gh repo create rothnic/assura \
  --public \
  --description "Structure-first repository validation CLI" \
  --source=. \
  --remote=origin \
  --push
```

### Step 2: Configure Branch Protection

After pushing, set up branch protection via GitHub CLI:

```bash
# Protect master branch with required PR reviews
gh api repos/rothnic/assura/branches/master/protection \
  --method PUT \
  --input - <<< '{
    "required_status_checks": {
      "strict": true,
      "contexts": ["Check", "Test Suite"]
    },
    "enforce_admins": false,
    "required_pull_request_reviews": {
      "required_approving_review_count": 1,
      "dismiss_stale_reviews": true,
      "require_code_owner_reviews": false
    },
    "restrictions": null
  }'
```

Or configure via GitHub web interface:
1. Go to Settings → Branches
2. Add rule for `master` branch
3. Enable:
   - Require pull request reviews before merging (1 approval)
   - Require status checks to pass for currently reliable gates
   - Include administrators

Rustfmt and Clippy are intentionally tracked as tooling-baseline cleanup work
before they become required branch-protection gates.

### Step 3: Set Up Repository Secrets

The current no-Rust release path publishes GitHub release archives and does not
require repository secrets.

Coverage reports are generated in GitHub Actions and attached as workflow
artifacts. No hosted coverage service token is required for the current CI
workflow.

## File Structure

```
assura/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml           # Main CI workflow
│   │   ├── release.yml      # Release automation
│   │   ├── docs.yml         # Documentation deployment
│   │   └── security.yml     # Security audits
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md    # Bug report template
│   │   ├── feature_request.md
│   │   └── config.yml       # Issue template config
│   └── PULL_REQUEST_TEMPLATE.md
├── CONTRIBUTING.md          # Contribution guidelines
├── LICENSE                  # MIT License
├── LICENSE-MIT              # MIT License (explicit)
└── LICENSE-APACHE           # Apache-2.0 License
```

## Workflow Details

### CI Workflow (ci.yml)
Triggers on push/PR to main/master:
- **Check**: Verifies code compiles
- **Rustfmt**: Enforces code formatting
- **Clippy**: Runs linter with warnings as errors
- **Test Suite**: Runs tests on Ubuntu and macOS
- **Coverage**: Generates code coverage reports and uploads the Cobertura XML
  report as a GitHub Actions artifact

Rustfmt and Clippy are blocking CI gates. Windows full test-suite coverage is
still tracked separately from the release installer smoke because the pre-1.0
tooling baseline has platform-specific dependency work remaining.

### Release Workflow (release.yml)
Triggers on version tags (v*):
- Creates GitHub release
- Builds binaries for 5 targets:
  - Linux (x86_64, musl)
  - Windows (x86_64)
  - macOS (x86_64, ARM64)
- Enforces the compressed archive size budget before publishing assets
- Publishes the archives to the GitHub release

### Documentation Workflow (docs.yml)
Builds documentation on pushes and pull requests to `main` or `master`:
- Builds rustdoc
- Deploys to GitHub Pages from `main`

### Security Workflow (security.yml)
Triggers on Cargo.toml/lock changes + daily schedule:
- Runs `cargo audit` to check for vulnerabilities

## Usage

### For Contributors

1. Fork the repository
2. Create a feature branch
3. Make changes following CONTRIBUTING.md guidelines
4. Submit a PR using the template

### For Maintainers

**Creating a release:**
```bash
# Update version in Cargo.toml
git add Cargo.toml
git commit -m "chore(release): bump version to 0.1.0"
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The release workflow will automatically:
- Create or update the GitHub release for the tag
- Build binaries for Linux, macOS, and Windows
- Enforce the compressed archive size budget
- Upload the prebuilt archives used by the install scripts

## Notes

- Most infrastructure jobs use `ubuntu-latest`; CI and release jobs also use
  macOS and Windows where those platforms are part of the workflow.
- Caching is enabled for cargo dependencies to speed up builds
- Security workflow runs daily to catch new vulnerabilities
- Branch protection should require only reliable checks until the tooling
  baseline is clean

## Troubleshooting

**If CI fails:**
- Check that all files are properly formatted: `cargo fmt --check`
- Run clippy locally: `cargo clippy -- -D warnings`
- Ensure tests pass: `cargo test`

**If release fails:**
- Ensure version in `Cargo.toml` matches the tag.
- Check the archive-size gate output before raising the 8 MiB budget.
- Reproduce the platform package locally with `node --run verify:release-smoke`
  on Unix or the Windows installer smoke in CI.

## References

- GitHub Actions documentation: https://docs.github.com/en/actions
- Rust CI best practices: https://doc.rust-lang.org/cargo/guide/continuous-integration.html
- GitHub releases: https://docs.github.com/en/repositories/releasing-projects-on-github
