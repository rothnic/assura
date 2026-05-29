---
id: analysis-2026-05-26-real-repo-ls-lint-performance-candidates
title: Real-Repo LS-Lint Performance Candidate Set
date: 2026-05-26
status: draft
related:
  - docs/goals/assura-real-repo-ls-lint-performance-evidence.md
  - src/cli/performance_report/external_fixtures.rs
  - tests/ls_lint_realistic_fixture_manifest.yml
---

# Real-Repo LS-Lint Performance Candidate Set

## Purpose

This note captures the first 10 pinned repository candidates for
`docs/goals/assura-real-repo-ls-lint-performance-evidence.md`.

The final benchmark suite must prove that Assura is faster than native LS-Lint
on each headline real-repo case for both cold one-shot checks and warm/editor
session checks. This candidate list is only the starting catalog; completion
still requires materialization, repo-specific configs, measurements, and review.

## Selection Criteria

Each candidate should support a distinct, feature-rich LS-Lint policy that
controls project structure rather than only naming a few extensions. Good policy
surface includes:

- root directory/file guardrails with `.dir` and `.*` regex rules,
- scoped rules for app, package, docs, tests, examples, scripts, crates, or
  tooling directories,
- generated-output ignores such as `node_modules`, `dist`, `target`, coverage,
  cache, and framework build folders,
- extension bans where the project convention disallows a file family in a
  source subtree,
- `exists` counts only where the real repository shape makes that contract
  natural and native LS-Lint-compatible.

## Initial 10-Repo Candidate Catalog

| Fixture id | Repository | Pinned ref checked | Resolved commit or tag object | Intended policy shape |
| --- | --- | --- | --- | --- |
| `pinned_nextjs` | `https://github.com/vercel/next.js` | `refs/tags/v15.0.0^{}` | `51bfe3c1863b191f4b039bc230e8ed5c57b0baf3` | Large frontend monorepo with packages, examples, tests, build outputs, config exceptions. |
| `pinned_mdbook` | `https://github.com/rust-lang/mdBook` | `refs/tags/v0.4.48` | `b7a27d2759e80d804a33a4bc9c31b2b6863a5cb2` | Rust docs/library project with source, docs, book assets, tests, target ignores. |
| `pinned_vite` | `https://github.com/vitejs/vite` | `refs/tags/v5.4.0` | `cf50e548d4ae7f6ed9ade507d70acbd39b4e0b93` | Frontend tooling monorepo with packages, playgrounds, docs, config and generated-output rules. |
| `pinned_tailwindcss` | `https://github.com/tailwindlabs/tailwindcss` | `refs/tags/v3.4.1` | `7361468f77500105b0559e879e121f34306e8da2` | JS package repo with source, scripts, tests, fixtures, docs, build artifacts. |
| `pinned_prettier` | `https://github.com/prettier/prettier` | `refs/tags/3.3.0^{}` | `c4ab460357478d2b847c60a1efb40098b1181931` | JS tooling repo with tests, docs, scripts, website, and generated output. |
| `pinned_pnpm` | `https://github.com/pnpm/pnpm` | `refs/tags/v9.0.0^{}` | `dd6b0b62d80340655ac1cf4c4365fcfffe1e3f2c` | Package-manager monorepo with packages, config files, tests, docs, and lockfile conventions. |
| `pinned_rustlings` | `https://github.com/rust-lang/rustlings` | `refs/tags/v6.0.0^{}` | `28d2bb04326d7036514245d73f10fb72b9ed108c` | Rust educational repo with exercises, solutions, tests, book/docs, and target ignores. |
| `pinned_clap` | `https://github.com/clap-rs/clap` | `refs/tags/v4.5.0^{}` | `a751c5fe65cd33cb09e85ff3039b4fd0182cdb6e` | Rust workspace with crates, examples, tests, benches, docs, and generated target output. |
| `pinned_ripgrep` | `https://github.com/BurntSushi/ripgrep` | `refs/tags/14.1.1^{}` | `4649aa9700619f94cf9c66876e9549d83420e16c` | Rust CLI repo with crates, tests, docs, scripts, fixtures, and target ignores. |
| `pinned_tokio` | `https://github.com/tokio-rs/tokio` | `refs/tags/tokio-1.38.0` | `14c17fc09656a30230177b600bacceb9db33e942` | Rust async workspace with crates, tests, benches, examples, docs, and target ignores. |

## Commands Used To Resolve Candidates

```bash
git ls-remote https://github.com/vercel/next.js refs/tags/v15.0.0 refs/tags/v15.0.0^{}
git ls-remote https://github.com/rust-lang/mdBook refs/tags/v0.4.48
git ls-remote https://github.com/vitejs/vite refs/tags/v5.4.0
git ls-remote https://github.com/tailwindlabs/tailwindcss refs/tags/v3.4.1
git ls-remote https://github.com/prettier/prettier refs/tags/3.3.0^{}
git ls-remote https://github.com/pnpm/pnpm refs/tags/v9.0.0^{}
git ls-remote https://github.com/rust-lang/rustlings refs/tags/v6.0.0^{}
git ls-remote https://github.com/clap-rs/clap refs/tags/v4.5.0^{}
git ls-remote https://github.com/BurntSushi/ripgrep refs/tags/14.1.1^{}
git ls-remote https://github.com/tokio-rs/tokio refs/tags/tokio-1.38.0
```

## Open Implementation Work

- Add all 10 candidates to the performance fixture catalog.
- Materialize each repository through the external fixture cache path.
- Author a distinct LS-Lint config for each repo and the equivalent Assura
  config.
- Verify native LS-Lint and Assura have the expected exit status on every repo.
- Measure both cold and warm rows with at least five iterations.
- Promote only cases where Assura is faster than native LS-Lint on both cold
  and warm rows into the headline real-repo aggregate.
