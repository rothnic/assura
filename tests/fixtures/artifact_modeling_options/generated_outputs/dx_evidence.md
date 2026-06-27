---
id: artifact_modeling_options_dx_evidence
type: evidence
---

# Artifact Modeling DX Evidence

This file records compact editor, CLI, and generated-output evidence for the
artifact-modeling comparison. These tools are authoring-time conveniences only;
the runtime proof is the Rust test over normalized checked-in schemas.

| Option | CLI evidence | Editor/DX evidence | Result |
| --- | --- | --- | --- |
| TypeSpec | `npx tsp compile` with `@typespec/json-schema` emitted `typespec_artifacts.schema.yaml`. | Official VS Code extension documents language features for TypeSpec authoring. | Strong authoring and emitter DX, needs Assura relation metadata. |
| LinkML | `uvx --from linkml gen-json-schema` emitted `linkml_artifacts.schema.json`. | Official FAQ documents YAML editing with LinkML's JSON Schema meta-model. | Strong ecosystem and semantics, needs a small Assura profile. |
| CUE | `cue export --out jsonschema -e '#Goal'` emitted `cue_goal.schema.json`. | CUE has editor extensions, but generated schema shape needs normalization. | Viable only as authoring-time source; not runtime engine. |
| Zod | `z.toJSONSchema` emitted `zod_artifacts.schema.json`. | Excellent TypeScript authoring; weak as cross-language source of truth. | Good helper package candidate, not the core model source. |
| JSON Schema/JTD | Direct model is already the validation artifact. | VS Code has built-in JSON Schema support, with draft caveats. | Best runtime target, poor primary authoring syntax. |
| Assura control | No external generator exists. | Would require Assura-owned schema, docs, and editor support. | Useful minimum-notation benchmark, not justified as core yet. |

Relevant source links are recorded in
`docs/analysis/2026-06-27-artifact-modeling-options-comparison.md`.
