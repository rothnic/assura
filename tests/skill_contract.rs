use std::fs;
use std::path::Path;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, severity: &str, max_skill_lines: usize) {
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
extensions:
  agent_guidance:
    - id: agent_project_guidance
      severity: {severity}
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
      max_agents_lines: 120
      max_skill_lines: {max_skill_lines}
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - target/**
"#
        ),
    )
    .unwrap();
}

fn write_agents_md(project: &TempDir, skill: &str) {
    fs::write(
        project.path().join("AGENTS.md"),
        format!(
            r#"# Agent Instructions

## Operating Rules

Read project guidance before acting.

## Process Docs vs Skills

Keep process docs durable and skills executable.
Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as
concise indexes to deeper references.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | [`{skill}`](.agents/skills/{skill}/SKILL.md) |

## Anchors

Keep headings stable.
"#
        ),
    )
    .unwrap();
}

fn write_skill(project: &TempDir, skill: &str, content: &str) {
    let skill_dir = project.path().join(".agents/skills").join(skill);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

fn valid_skill_content() -> &'static str {
    r#"---
name: project-maintenance
description: Maintain project-local Assura guidance.
applies_when: Use when maintaining the project-local Assura baseline.
version: 1
---

# Project Maintenance

## Workflow

Run the workflow.

## Read as needed

- `references/runbook.md`

## Outputs

- Updated baseline files.

## Guardrails

- Keep the entrypoint concise.
"#
}

#[test]
fn skill_contract_accepts_required_frontmatter_sections_and_progressive_disclosure() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 80);
    write_agents_md(&project, "project-maintenance");
    write_skill(&project, "project-maintenance", valid_skill_content());

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn skill_contract_reports_missing_frontmatter_and_required_sections() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 80);
    write_agents_md(&project, "project-maintenance");
    write_skill(
        &project,
        "project-maintenance",
        r#"---
name: project-maintenance
description: Maintain project-local Assura guidance.
---

# Project Maintenance

## Workflow

Run the workflow.
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new(".agents/skills/project-maintenance/SKILL.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation
                .message
                .contains("missing frontmatter field `applies_when`")
    }));
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new(".agents/skills/project-maintenance/SKILL.md")
            && violation
                .message
                .contains("missing required section `Read as needed`")
    }));
}

#[test]
fn skill_contract_warns_when_skill_entrypoint_should_move_detail_to_references() {
    let project = TempDir::new().unwrap();
    write_config(&project, "low", 10);
    write_agents_md(&project, "project-maintenance");
    write_skill(&project, "project-maintenance", valid_skill_content());

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new(".agents/skills/project-maintenance/SKILL.md")
            && violation.severity == "low"
            && violation.message.contains("references/")
            && violation.message.contains("docs/process/")
    }));
}

#[test]
fn skill_contract_requires_reference_sections_to_point_to_deeper_material() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 80);
    write_agents_md(&project, "project-maintenance");
    write_skill(
        &project,
        "project-maintenance",
        r#"---
name: project-maintenance
description: Maintain project-local Assura guidance.
applies_when: Use when maintaining the project-local Assura baseline.
---

# Project Maintenance

## Workflow

Run the workflow.

## Read as needed

- No deeper references yet.

## Outputs

- Updated baseline files.

## Guardrails

- Keep the entrypoint concise.
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new(".agents/skills/project-maintenance/SKILL.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation
                .message
                .contains("must reference supporting docs or assets")
    }));
}
