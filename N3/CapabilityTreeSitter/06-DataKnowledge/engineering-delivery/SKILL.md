---
name: engineering-delivery
description: Deliver a scoped engineering change with a written design, bounded validation, adversarial review, and evidence-backed documentation.
---

# Engineering delivery

Use this skill before changing code, configuration, prompts, MCPs, or build tooling.

1. Read the repository instructions, README, architecture docs, and the files that own the behavior. State the scope, non-goals, rollback path, and acceptance checks before editing.
2. Use `zed-engineering-toolchain.detect_toolchains` and then `plan_language_check`. Prefer Rust, MLX, Swift, C/C++, and TypeScript paths when the project supports them. Do not invent a build command.
3. Implement the smallest compatible change. Preserve existing user edits and secrets. Never claim a dependency, model, benchmark, or external system works until it has been checked.
4. Pressure-test boundaries: empty and malformed input, Unicode, cancellation, timeouts, retries, permissions, offline operation, mixed versions, large payloads, concurrent calls, and rollback.
5. Run the planned checks and relevant integration/E2E tests. Record exact commands, exit codes, and skipped checks with reasons.
6. Do an adversarial review: inspect the diff for unsafe defaults, injection paths, destructive operations, data leakage, privilege escalation, race conditions, API/version drift, and test gaps.
7. Update the README and implementation documentation. Use the local documentation standard if present. Mark evidence as observed, staged, proposed, or unverified; do not upgrade its status by wording alone.

Stop and ask before external side effects: deploys, account changes, new paid services, uploading code/data, deleting state, or storing secrets.
