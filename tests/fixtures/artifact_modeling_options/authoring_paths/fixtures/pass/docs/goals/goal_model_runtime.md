---
id: goal_model_runtime
title: Model repository artifacts with native runtime validation
status: active
owners:
  - platform
specs:
  - spec_artifact_runtime
tasks:
  - task_profile_review
decisions:
  - decision_runtime_target
metadata:
  summary: Prove repository files can behave like typed content objects.
  risk: medium
  tags:
    - schema
    - runtime
---

The body stays as normal Markdown while the frontmatter carries the model data.

Agents should be able to update the frontmatter without rewriting this prose.
