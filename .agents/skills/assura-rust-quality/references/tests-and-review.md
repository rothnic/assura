# Tests and review references

Read this before choosing a Rust test or preparing a review.

`tests/skill_contract.rs` demonstrates a meaningful positive/negative
configuration fixture: a valid routed skill is accepted, while missing required
frontmatter or a reference-free routing section produces a real
`agent_guidance` finding. A validation change should similarly prove a valid
case, a seeded violation, and a justified exception through the actual checker.

Avoid constant-restatement tests such as asserting a fixed policy list or a
private helper's literal output. They only detect intentional edits, not a
consumer-visible regression. Prefer an integration fixture whose expected
finding, ordering, exit status, or rendered report would change if the contract
were broken.

Run the focused test first. For Rust behavior, then run `cargo xtask fast`; use
`cargo xtask pr` and any relevant feature, OS, or release gate before review.
Record the candidate SHA, cwd, binary, command exits, negative control, and
remaining limits. An independent review checks the tested SHA, not merely the
branch tip.
