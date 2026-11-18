---
description: Run the Atoma harness loop until the promise is true
argument-hint: PROMPT [--max-iterations N] [--completion-promise TEXT]
allowed-tools: [Bash, TaskCreate, TaskUpdate, Read, Write, Edit, Glob, Grep]
---

# Harness Run

Work the task with tools, not text. Rules:

1. TaskCreate first.
2. Read/Glob the working folder before writing.
3. Every turn must emit at least one tool_use block until done.

Prompt: $ARGUMENTS
