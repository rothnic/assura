//! Built-in reusable structure-rule coverage.

use super::parse_config;

#[test]
fn built_in_agent_skill_dir_rule_expands_without_local_definition() {
    let config = parse_config(
        r#"
structure:
  .agents/skills/:
    "{skill}/":
      use: "@agent-skill-dir"
"#,
    )
    .unwrap();

    let skills = config.structure.get(".agents/skills/").unwrap();
    let skill = skills
        .children
        .as_ref()
        .and_then(|children| children.get("{skill}"))
        .expect("captured skill child");
    let files = skill.files.as_ref().expect("skill file rules");
    assert_eq!(
        files
            .exists
            .as_ref()
            .and_then(|exists| exists.get("SKILL.md")),
        Some(&"1".to_string())
    );
    assert_eq!(
        files.allowed_names.as_ref().map(Vec::as_slice),
        Some(&["SKILL.md".to_string()][..])
    );
    assert_eq!(files.max_lines, Some(600));
    assert_eq!(files.max_size.as_deref(), Some("24KB"));
    assert_eq!(files.allow_extra, Some(false));
    assert!(!skill.inherit);

    let directories = skill.directories.as_ref().expect("skill directory rules");
    assert_eq!(directories.allow_extra, Some(false));

    let references = skill
        .children
        .as_ref()
        .and_then(|children| children.get("references"))
        .expect("references resource child");
    assert!(!references.required);
    assert!(!references.inherit);
    assert_eq!(
        references
            .self_directory
            .as_ref()
            .and_then(|directory| directory.naming.as_deref()),
        Some("kebab-case")
    );
    assert_eq!(
        references
            .files
            .as_ref()
            .and_then(|files| files.naming.as_deref()),
        Some("kebab-case")
    );
    assert_eq!(
        references.files.as_ref().and_then(|files| files.max_lines),
        Some(600)
    );
}

#[test]
fn built_in_agents_dir_rule_composes_skill_best_practices() {
    let config = parse_config(
        r#"
structure:
  .agents/:
    use: "@agents-dir"
"#,
    )
    .unwrap();

    let agents = config.structure.get(".agents/").unwrap();
    assert!(!agents.inherit);
    let skills = agents
        .children
        .as_ref()
        .and_then(|children| children.get("skills"))
        .expect("agents directory should require skills child");
    assert!(!skills.inherit);
    let skill = skills
        .children
        .as_ref()
        .and_then(|children| children.get("{skill}"))
        .expect("skills directory should have captured skill child");
    assert_eq!(
        skill
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("SKILL.md")),
        Some(&"1".to_string())
    );
    assert_eq!(
        skill
            .self_directory
            .as_ref()
            .and_then(|directory| directory.naming.as_deref()),
        Some("kebab-case")
    );
    assert_eq!(
        skill.files.as_ref().and_then(|files| files.max_lines),
        Some(600)
    );
    assert_eq!(
        skill
            .files
            .as_ref()
            .and_then(|files| files.max_size.as_deref()),
        Some("24KB")
    );
}
