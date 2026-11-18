# AtomaApp Skills Storage

This folder is the canonical project-local home for skill definitions.
- Skill folders live under `skills/<skill-name>/SKILL.md`.
- Dependencies are declared in each skill's frontmatter (`depends_on`).
- The backend loads skill text through the plugin system and applies the `skill_tree` model (topological order, cycle detection, lazy/eager modes).

See `AtomaApp/backend/src/skill_tree.rs` and `AtomaApp/backend/src/plugins.rs`.
EOF
