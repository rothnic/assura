# Find-Skills Installation Documentation

## Overview

**find-skills** is a MicroClaw-compatible agent skill that enables discovery and recommendation of reusable skills from the [vercel-labs/skills](https://github.com/vercel-labs/skills) registry.

## What It Does

- Searches the vercel-labs/skills repository for skills matching task keywords
- Evaluates skill fit based on requirements and platform compatibility
- Provides installation and adaptation guidance for MicroClaw
- Recommends best-fit skills and alternatives

## Installation Location

```
/workspace/repos/research/assura/skills/built-in/find-skills/
└── SKILL.md
```

## Dependencies

- `curl` - For GitHub API access
- Internet connectivity to access github.com

## Verification

### 1. File Structure
✓ SKILL.md installed at `/workspace/repos/research/assura/skills/built-in/find-skills/SKILL.md`

### 2. API Connectivity Test
```bash
curl -s "https://api.github.com/repos/vercel-labs/skills/contents"
```
✓ Successfully retrieves repository contents

### 3. Sample Usage
To find skills for a specific task (e.g., "database migration"):
```bash
curl -sL "https://raw.githubusercontent.com/vercel-labs/skills/main/README.md" | grep -i "database"
```

## How to Use

1. **Activate the skill** when the agent asks:
   - "Do we have a skill for X?"
   - "Find skills for [task]"
   - "What existing skill can I reuse?"

2. **Skill workflow**:
   - Clarify the target task
   - Search registry by keywords
   - Extract skill metadata (name, dependencies, platform)
   - Recommend best-fit and fallback options
   - Provide MicroClaw adaptation steps

## Integration with Assura

This skill is designed for the AI agent coordination system in the Assura project. When activated:
- Skill metadata is loaded into the system prompt
- Full instructions are available on demand via `activate_skill`
- Compatible with the Anthropic Agent Skills standard

## Notes

- The skill uses the GitHub Contents API (unauthenticated) which has rate limits
- For production use, consider using a GitHub token for higher rate limits
- Some GitHub Search API features require authentication
