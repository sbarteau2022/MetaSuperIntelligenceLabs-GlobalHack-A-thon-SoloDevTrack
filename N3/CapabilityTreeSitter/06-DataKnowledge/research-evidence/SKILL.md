---
name: research-evidence
description: Find primary technical sources, papers, models, and documentation while preserving provenance, license checks, and uncertainty.
---

# Research and evidence

Use `research-sources.search_arxiv` for paper metadata, `research-sources.search_huggingface_models` for public Hub discovery, Context7 for version-specific library documentation, and the Hugging Face MCP for authenticated Hub work.

1. Start with a precise question and inclusion/exclusion criteria. Prefer original papers, official docs, source repositories, and pinned releases.
2. For each candidate, record source URL, version/date, license, owner, hardware/runtime assumptions, and the exact claim it supports.
3. Treat model cards, download counts, benchmark tables, and search results as leads—not proof of compatibility, security, permission, or performance.
4. Before importing a whole repository or a large framework, review its license, maintenance, dependency tree, network/data behavior, sandbox boundaries, and a minimal local smoke test. Pin revisions and document the reason for inclusion.
5. Separate observed results from proposals. Cite the source beside every material claim, and say what was not verified.

Never submit, publish, deploy, create paid infrastructure, or send private project data to a research tool without explicit authorization.
