# Skills Directory

This directory contains reusable skill definitions for Assura agents.

## Structure

```
.agents/skills/
├── README.md              # This file
├── TEMPLATE.md            # Template for creating new skills
├── built-in/              # Project-specific skills
│   └── <skill-name>/
│       └── SKILL.md
├── custom/                # Custom skills for this project
│   └── <skill-name>/
│       └── SKILL.md
└── third-party/           # External skill imports
    └── <skill-name>/
        └── SKILL.md
```

## Skill Categories

### built-in/
Skills that come with the Assura project. These are maintained by the project team and should not be modified by users.

### custom/
Project-specific skills tailored to your workflow. Add custom skills here when you need specialized instructions that aren't in the built-in collection.

### third-party/
Skills imported from external sources (e.g., GitHub skill repositories). Use these for common patterns from the broader ecosystem.

## Using Skills

1. **Read the SKILL.md** - Each skill has a SKILL.md file with:
   - Frontmatter with name, description, and compatibility
   - Usage instructions
   - Workflow steps
   - Command examples

2. **Apply the skill** - Follow the workflow in the skill file to complete your task.

3. **Document any adaptations** - If you modify a skill's approach, note it in your work summary.

## Creating New Skills

See TEMPLATE.md for a starter template.

## Skill Registry

For discovering external skills, use the built-in `find-skills` skill or browse:
- https://github.com/vercel-labs/skills
- https://github.com/topics/agent-skills
