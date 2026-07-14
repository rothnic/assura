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
        files.allowed_names.as_deref(),
        Some(&["SKILL.md".to_string()][..])
    );
    assert_eq!(files.max_lines, None);
    assert_eq!(files.max_size, None);
    assert_eq!(
        files
            .max_lines_patterns
            .as_ref()
            .and_then(|patterns| patterns.get(".agents/skills/{skill}/SKILL.md")),
        Some(&600)
    );
    assert_eq!(
        files
            .max_size_patterns
            .as_ref()
            .and_then(|patterns| patterns.get(".agents/skills/{skill}/SKILL.md"))
            .map(String::as_str),
        Some("24KB")
    );
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
        .expect("agents directory should allow skills child");
    assert!(!skills.required);
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
    let skill_files = skill.files.as_ref().expect("skill file rules");
    assert_eq!(skill_files.max_lines, None);
    assert_eq!(skill_files.max_size, None);
    assert_eq!(
        skill_files
            .max_lines_patterns
            .as_ref()
            .and_then(|patterns| patterns.get(".agents/skills/{skill}/SKILL.md")),
        Some(&600)
    );
    assert_eq!(
        skill_files
            .max_size_patterns
            .as_ref()
            .and_then(|patterns| patterns.get(".agents/skills/{skill}/SKILL.md"))
            .map(String::as_str),
        Some("24KB")
    );
}

#[test]
fn built_in_agentic_project_rule_composes_root_guidance_and_skills() {
    let config = parse_config(
        r#"
structure:
  ./:
    use: "@agentic-project"
    extra: false
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    let files = root.files.as_ref().expect("root file rules");
    assert_eq!(
        files
            .exists
            .as_ref()
            .and_then(|exists| exists.get("AGENTS.md")),
        Some(&"1".to_string())
    );
    assert_eq!(files.allow_extra, Some(false));

    let directories = root.directories.as_ref().expect("root directory rules");
    assert_eq!(
        directories
            .exists
            .as_ref()
            .and_then(|exists| exists.get(".assura")),
        Some(&"0-1".to_string())
    );
    assert_eq!(directories.allow_extra, Some(false));

    let agents = root
        .children
        .as_ref()
        .and_then(|children| children.get(".agents"))
        .expect("agentic project should allow .agents child");
    assert!(!agents.required);
    assert!(!agents.inherit);

    let skill = agents
        .children
        .as_ref()
        .and_then(|children| children.get("skills"))
        .and_then(|skills| skills.children.as_ref())
        .and_then(|children| children.get("{skill}"))
        .expect("agentic project should compose skill directory best practices");
    assert_eq!(
        skill
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("SKILL.md")),
        Some(&"1".to_string())
    );
}

#[test]
fn built_in_agentic_project_children_merge_with_local_children() {
    let config = parse_config(
        r#"
structure:
  ./:
    use: "@agentic-project"
    children:
      docs/:
        README.md: exists:0-1
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    let children = root.children.as_ref().expect("root children");
    assert!(children.contains_key(".agents"));
    assert!(children.contains_key("docs/"));
}
