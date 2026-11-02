---
name: skill-author
description: Create or edit a SKILL.md skill definition with correct frontmatter, directory layout, and supporting files. Use when the user asks to author, update, or improve an agent skill in ~/.agents/skills or project .agents/skills.
---

# Skill Author

You are helping the agent create or edit a skill. Follow these rules.

## 1. Choose scope
- Global (`~/.agents/skills/<name>/`) for personal workflows across projects.
- Project-local (`<project>/.agents/skills/<name>/`) for repository-specific skills shared via version control.

## 2. Name and directory
- Directory name = skill `name` field exactly.
- Name regex: `^[a-z0-9]+(-[a-z0-9]+)*$` (lowercase, single hyphens, no leading/trailing/consecutive hyphens).
- Max 64 characters.

## 3. Frontmatter (required)
```yaml
---
name: <skill-name>
description: <specific, actionable, 1-1024 chars>
---
```
- `disable-model-invocation: true` only when the user wants manual-only invocation.
- Otherwise leave it unset so the agent can auto-invoke.

## 4. Body
- Direct, specific instructions for the agent (what to do, in what order).
- Concrete paths, commands, formats, patterns.
- Include when to use and when not to use.
- Reference supporting files with relative paths (templates/, examples/).

## 5. Supporting files
- Optional additional files in the skill directory (templates, examples, references).
- Keep them focused and version-controlled with the skill.

## 6. Validation
- Re-read the written SKILL.md to ensure frontmatter parses (YAML between `---`).
- Name matches directory name exactly.
- Description is specific and actionable, not vague.

## 7. Never
- Do not invent paths or commands not present in the project.
- Do not duplicate instructions from AGENTS.md unless relevant to the skill.
- Do not require approval to invoke unless the user explicitly asks for manual-only.
EOF
