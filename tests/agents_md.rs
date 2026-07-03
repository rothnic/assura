use std::fs;
use std::path::Path;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, severity: &str, max_agents_lines: usize) {
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
      max_agents_lines: {max_agents_lines}
      max_skill_lines: 80
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

fn write_valid_skill(project: &TempDir, name: &str) {
    let skill_dir = project.path().join(".agents/skills").join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            r#"---
name: {name}
description: Maintain the local workflow.
applies_when: Use for local maintenance.
---

# {name}

## Workflow

Run the workflow.

## Read as needed

- `references/runbook.md`

## Outputs

- Updated files.

## Guardrails

- Keep entrypoints concise.
"#
        ),
    )
    .unwrap();
}

fn valid_agents_md(skill: &str) -> String {
    format!(
        r#"# Agent Instructions

## Operating Rules

Read the onboarding packet before specializing.

## Process Docs vs Skills

Use process docs for durable background and skills for repeatable workflows.
Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as
concise indexes to deeper references.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | [`{skill}`](.agents/skills/{skill}/SKILL.md) |

## Anchors

Keep these section headings stable.
"#
    )
}

#[test]
fn agents_md_contract_validates_use_case_skill_routing_table() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 120);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        r#"# Agent Instructions

## Operating Rules

Read the onboarding packet before specializing.

## Process Docs vs Skills

Use process docs for durable background and skills for repeatable workflows.
Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as
concise indexes to deeper references.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | `missing-skill` |

## Anchors

Keep these section headings stable.
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation
                .message
                .contains("unknown skill or pattern `missing-skill`")
    }));
}

#[test]
fn agents_md_contract_requires_progressive_disclosure_reference() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 120);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        r#"# Agent Instructions

## Operating Rules

Read the onboarding packet before specializing.

## Process Docs vs Skills

Use process docs for durable background and skills for repeatable workflows.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | [`project-maintenance`](.agents/skills/project-maintenance/SKILL.md) |

## Anchors

Keep these section headings stable.
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation
                .message
                .contains("must reference `Progressive disclosure")
    }));
}

#[test]
fn agents_md_contract_requires_wildcard_skill_routes_to_be_configured() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 120);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        r#"# Agent Instructions

## Operating Rules

Read the onboarding packet before specializing.

## Process Docs vs Skills

Use process docs for durable background and skills for repeatable workflows.
Progressive disclosure: keep AGENTS.md as a use-case router and SKILL.md as
concise indexes to deeper references.

## Skills

| When | Must first load |
| --- | --- |
| Maintaining project guidance | `*` |

## Anchors

Keep these section headings stable.
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation.message.contains("unknown skill or pattern `*`")
    }));
}

#[test]
fn agents_md_contract_accepts_required_sections_and_skill_index_links() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 80);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        valid_agents_md("project-maintenance"),
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn agents_md_contract_reports_missing_sections_duplicate_anchors_and_stale_skill_links() {
    let project = TempDir::new().unwrap();
    write_config(&project, "high", 80);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        r#"# Agent Instructions

## Operating Rules

Basic guidance.

## Skills

- [Missing](.agents/skills/missing-skill/SKILL.md): stale link.

## Skills

Duplicate heading.
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation
                .message
                .contains("missing required section `Process Docs vs Skills`")
    }));
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation
                .message
                .contains("duplicate heading anchor `skills`")
    }));
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.message.contains("missing project-local skill")
    }));
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation
                .message
                .contains(".agents/skills/project-maintenance/SKILL.md")
    }));
}

#[test]
fn agents_md_contract_can_warn_without_blocking_draft_mode() {
    let project = TempDir::new().unwrap();
    write_config(&project, "low", 5);
    write_valid_skill(&project, "project-maintenance");
    fs::write(
        project.path().join("AGENTS.md"),
        valid_agents_md("project-maintenance"),
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.iter().any(|violation| {
        violation.path == Path::new("AGENTS.md")
            && violation.rule == "agent_guidance:agent_project_guidance"
            && violation.severity == "low"
            && violation.message.contains("exceeding limit 5")
    }));
}
