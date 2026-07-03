use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join(".agents/skills/project-maintenance")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
extensions:
  agent_guidance:
    - id: agent_project_guidance
      severity: high
      agents_path: AGENTS.md
      skill_paths:
        - ".agents/skills/*/SKILL.md"
      required_agents_sections:
        - Operating Rules
        - Process Docs vs Skills
        - Skills
        - Anchors
      required_skill_frontmatter:
        - name
        - description
        - applies_when
      required_skill_sections:
        - Workflow
        - Read as needed
        - Outputs
        - Guardrails
      skill_index_section: Skills
      best_practices_reference: "Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as concise indexes to deeper references."
      skill_routing_section: Skills
      allowed_skill_name_patterns:
        - "project-*"
      skill_reference_sections:
        - Read as needed
      skill_reference_prefixes:
        - references/
        - scripts/
        - assets/
        - docs/process/
      max_agents_lines: 80
      max_skill_lines: 80
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(
        root.join("AGENTS.md"),
        r#"# Agent Guidance

## Operating Rules

Use the project-local guidance.

## Process Docs vs Skills

Keep durable process docs separate from executable skills.
Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as concise indexes to deeper references.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | [`project-maintenance`](.agents/skills/project-maintenance/SKILL.md) |

## Anchors

Keep section headings stable.
"#,
    )
    .unwrap();
    fs::write(
        root.join(".agents/skills/project-maintenance/SKILL.md"),
        r#"---
name: project-maintenance
description: Maintain project guidance.
applies_when: Maintaining project guidance.
---

# Project Maintenance

## Workflow

Follow local project guidance.

## Read as needed

- `references/runbook.md`

## Outputs

- Updated guidance or a no-op explanation.

## Guardrails

- Keep the entrypoint concise.
"#,
    )
    .unwrap();
}

#[test]
fn compiled_config_cli_supports_agent_guidance_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-agent-guidance-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project);

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    fs::write(
        project.join(".agents/skills/project-maintenance/SKILL.md"),
        r#"---
name: project-maintenance
description: Maintain project guidance.
---

# Project Maintenance

## Workflow

Follow local project guidance.
"#,
    )
    .unwrap();

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("agent_guidance:agent_project_guidance"));
    assert!(stdout.contains("applies_when"));

    fs::write(
        project.join("AGENTS.md"),
        r#"# Agent Guidance

## Operating Rules

Use the project-local guidance.

## Process Docs vs Skills

Keep durable process docs separate from executable skills.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | [`project-maintenance`](.agents/skills/project-maintenance/SKILL.md) |

## Anchors

Keep section headings stable.
"#,
    )
    .unwrap();
    fs::write(
        project.join(".agents/skills/project-maintenance/SKILL.md"),
        r#"---
name: project-maintenance
description: Maintain project guidance.
applies_when: Maintaining project guidance.
---

# Project Maintenance

## Workflow

Follow local project guidance.

## Read as needed

- No deeper references yet.

## Outputs

- Updated guidance or a no-op explanation.

## Guardrails

- Keep the entrypoint concise.
"#,
    )
    .unwrap();

    let new_field_invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(new_field_invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&new_field_invalid.stdout);
    assert!(stdout.contains("progressive-disclosure guidance"));
    assert!(stdout.contains("must reference supporting docs or assets"));
}
