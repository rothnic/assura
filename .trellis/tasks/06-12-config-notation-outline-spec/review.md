# Review record

## Reviewer

- Review agent: Aristotle
- Date: 2026-06-12

## Findings and Resolution

1. Unquoted `@rule` references were invalid YAML.
   - Resolution: quoted all documented `@rule` examples and stated the stable
     documented form must remain YAML-valid.
2. Aggregate code-to-doc relation duplicated the selected package heading.
   - Resolution: made section rules validate children under the selected
     section, and added relation semantics for root-relative paths, basename
     captures, template expansion, and missing/multiple section matches.
3. Reusable rule fragments were underspecified.
   - Resolution: documented node fragments, tree fragments, mismatch errors,
     and `use` merge order.
4. The old notation source-truth document still conflicted with the new target.
   - Resolution: replaced stale next-step language with the target
     implementation order from the new Trellis spec.
5. Outline depth inference was ambiguous.
   - Resolution: specified relative matching, title-H1 handling, root-level
     choice, and skipped-level/multiple-match error behavior.
6. Constitution version metadata was inconsistent.
   - Resolution: aligned the header, footer version, update date, and next
     review date.
