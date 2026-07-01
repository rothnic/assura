# Markdown Linter Options

## Sources

- <https://rumdl.dev/>
- <https://rumdl.dev/comparison/>
- <https://docs.rs/rumdl>
- <https://github.com/akiomik/mado>
- <https://crates.io/crates/markdownlint-rs>
- Local prior worktree:
  `/Users/nroth/workspace/assura-markdown-linter-spec/.trellis/tasks/06-14-markdown-document-graph-linter/`

## Findings

The prior local markdown-linter spec chose "document graph first" and treated a
markdownlint-compatible style baseline as secondary. That was appropriate before
the beta release, but the post-beta ask is stronger: Assura should use or
integrate the fastest practical Rust markdown linter/fixer that remains
consistent with markdownlint.

`rumdl` is the current strongest default candidate. Its public docs describe a
Rust-native Markdown linter and formatter with markdownlint-compatible rule
coverage, `check --fix`/`fmt`, config discovery/conversion, a library crate, LSP
support, and benchmark claims against `markdownlint-cli` and
`markdownlint-cli2`. Its comparison page says it implements all markdownlint
rules plus additional rules and supports auto-fix for most rules. These claims
must be verified in Assura fixtures before adoption.

`mado` is a fast Rust Markdown linter with CommonMark/GFM compatibility and
benchmark tooling. It appears useful as a performance comparison candidate, but
current public comparison material says it has fewer rules and no auto-fix, so
it is weaker for Assura's requested linter/fixer goal.

`markdownlint-rs`/`mdlint` style projects are relevant comparison candidates.
They may provide fast Rust linting and fixing, but the public surface appears
less established than `rumdl` for markdownlint compatibility plus fixer coverage
as of this planning pass.

## Planning Decision

The post-beta Markdown goal should default to a `rumdl` evaluation/adoption
path, not a from-scratch linter rewrite. The exit bar must require local proof:

- markdownlint rule/config compatibility fixture matrix;
- fix safety and idempotence fixtures;
- Assura integration over existing severity/suppression/finding contracts;
- benchmark comparison against `rumdl`, `markdownlint-cli2`, current Assura
  Markdown checks, and any retained alternative such as `mado`;
- a fallback decision record if `rumdl` cannot satisfy Assura's API, MSRV,
  binary-size, licensing, or performance constraints.
