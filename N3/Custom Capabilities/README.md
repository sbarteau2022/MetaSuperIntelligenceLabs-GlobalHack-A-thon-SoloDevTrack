# Atoma Claude Code harness

This local marketplace packages the parts of the research-grade harness that Claude Code can consume natively: plan/judge discipline, continuity, two-strike escalation, pressure testing, release gates, and lifecycle hooks.

## Install

```sh
claude plugin marketplace add /Users/stewartbarteau/Desktop/N3/atoma-harness-marketplace
claude plugin install atoma-harness@atoma-harness --scope user
```

Restart Claude Desktop after installation. The plugin complements the inference gateway; it does not replace correct Anthropic streaming and tool-call translation.

## Verify

```sh
claude plugin validate /Users/stewartbarteau/Desktop/N3/atoma-harness-marketplace
claude plugin list
```

Each skill has exactly one YAML frontmatter document. The `SessionStart` hook announces the active policy layer without exposing credentials or mutating the workspace.
