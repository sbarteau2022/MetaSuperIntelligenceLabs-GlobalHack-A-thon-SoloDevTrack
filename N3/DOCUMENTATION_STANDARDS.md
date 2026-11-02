# Documentation Standards

**Version:** 1.0.0
**Effective Date:** 2026-09-17
**Author:** Documentation Governance
**Status:** MANDATORY — All new implementations MUST comply

---

## 1. Purpose

This document establishes the **required** documentation standards for every implementation in this repository. No pull request, commit, or deployment will be accepted without compliant documentation. These standards exist to ensure:

- **Traceability** — Every change can be traced to its origin, rationale, and evidence
- **Reproducibility** — Any implementation can be rebuilt from documentation alone
- **Accountability** — Decisions are justified with evidence, not opinion
- **Knowledge Preservation** — Learnings survive personnel changes
- **Plain Language** — Documentation is readable by engineers, researchers, and stakeholders alike

---

## 2. Required Metadata Header (EVERY Document)

Every documentation file MUST begin with this exact header structure:

```markdown
---
title: "[Descriptive Title — What Was Implemented]"
document_type: "[implementation | bug_fix | feature | architecture | research | migration | deprecation]"
version: "1.0.0"
date: "YYYY-MM-DDTHH:MM:SSZ"          # ISO 8601 UTC — when this doc was written
implementation_date: "YYYY-MM-DDTHH:MM:SSZ"  # When the code change was merged
author: "[Full Name / GitHub Handle]"
reviewers: ["[Name/Handle]", "[Name/Handle]"]
status: "[draft | review | approved | deprecated | superseded]"
related_files:
  - path: "relative/path/to/file.ext"
    lines: "START-END"                 # Exact line range of the implementation
    role: "[primary | supporting | test | config]"
related_issues: ["#123", "#456"]       # GitHub issue/PR numbers
related_documents: ["doc-name.md"]     # Other docs this relates to
tags: ["tag1", "tag2", "domain"]       # Searchable categorization
---
```

**Rules:**

- `date` = when you write the documentation (ISO 8601 UTC)
- `implementation_date` = when the code was actually merged/deployed
- `lines` = exact line numbers (e.g., "42-157") — use `git blame` or IDE to verify
- At least **two reviewers** required for `approved` status
- `related_files` MUST include every file touched by the implementation

---

## 3. Required Document Sections (ALL Mandatory)

### 3.1 Executive Summary (Plain Language)

**2–4 sentences. No jargon.** Answer: *What did we do? Why does it matter? What changed for the user?*

> **Example:** "We replaced the tokenizer with a Rust implementation that reduces latency by 40%. Users will see faster first-token response. The change was necessary because the Python tokenizer was a bottleneck under load."

### 3.2 Problem Statement (Plain Language)

**What was broken, missing, or suboptimal?** Describe the pain point as a user or operator would experience it. Include metrics if available (latency, error rate, cost, usability).

> **Example:** "Under concurrent load (>50 req/s), the Python tokenizer added 200ms p99 latency and caused OOM kills on 8GB GPUs."

### 3.3 Solution Overview (Plain Language)

**What did we build/change?** High-level description. No code yet — just the approach.

> **Example:** "Ported the tokenizer to Rust using the `tokenizers` crate, exposed via PyO3 bindings. Added SIMD-optimized normalization. Kept the same public API so zero downstream changes required."

### 3.4 Implementation Details

#### 3.4.1 Files Modified (with Line References)

| File | Lines | Change Type | Description |
| ------ | ------- | ------------- | ------------- |
| `src/tokenizer.rs` | 1-342 | new | Rust tokenizer core |
| `src/bindings.py` | 15-89 | modified | PyO3 bindings, replaced Python impl |
| `tests/test_tokenizer.py` | 1-156 | new | Parity + perf tests |

**Every row must reference exact line numbers.** Use `git diff --stat` and `git blame` to verify.

#### 3.4.2 Languages Chosen and Why

| Language | Component | Rationale |
| ---------- | ----------- | ----------- |
| Rust | Tokenizer core | Memory safety, zero-cost abstractions, SIMD via `packed_simd`, no GIL |
| Python | Bindings / API | Ecosystem compatibility, zero downstream migration |
| C | SIMD intrinsics (fallback) | Platform-specific optimization where Rust SIMD unstable |

**If you would push the design further, where?** Document the next evolutionary step:

> **Example:** "Next: Migrate the entire preprocessing pipeline to Rust (normalization, truncation, padding) — would eliminate remaining Python overhead and enable batch tokenization on GPU via `candle`."

#### 3.4.3 Grounding in Verifiable Literature

**Was this implementation grounded in valid, verifiable preprint/peer-reviewed literature?**

If **YES**, provide full citations:

| Paper | Venue/Year | What Was Used | Link/DOI |
|-------|------------|---------------|----------|
| "Fast Byte-Level BPE Tokenization" | arXiv:2305.12345 / 2023 | Algorithm 2 (parallel BPE merge), SIMD normalization | <https://arxiv.org/abs/2305.12345> |
| "SIMD-Accelerated String Processing" | PLDI '22 | Section 4.3 (vectorized UTF-8 validation) | <https://doi.org/10.1145/3519939.3523712> |

**Citation format:** Use IEEE or APA consistently. Include DOI or arXiv ID. If the paper is not open-access, note access method (institutional, author copy, etc.).

If **NO**, state explicitly: *"This implementation was not directly grounded in published literature. It applies established engineering patterns (Rust/PyO3 bindings, SIMD optimization) to a known bottleneck."*

**No hand-waving.** "Based on research" is not a citation.

#### 3.4.4 Configuration Changes

List any config files, env vars, feature flags, or defaults changed:

```yaml
# config/tokenizer.yaml (NEW FILE)
tokenizer:
  backend: "rust"          # was "python"
  simd_enabled: true
  batch_size: 32
  cache_dir: "/tmp/tokenizer_cache"
```

### 3.5 Testing Evidence (MANDATORY)

**Every implementation MUST include testing evidence.** No exceptions.

#### 3.5.1 Test Files and Locations

| Test File | Lines | Test Type | Coverage |
| ----------- | ------- | ----------- | ---------- |
| `tests/test_tokenizer_parity.py` | 1-89 | correctness | 100% parity with Python output |
| `tests/test_tokenizer_perf.py` | 1-112 | performance | Latency, throughput, memory |
| `tests/test_tokenizer_edge_cases.py` | 1-67 | robustness | Empty, unicode, huge inputs |

#### 3.5.2 What the Tests Showed (Plain Language)

**Summarize results in plain language. Include numbers.**

> **Example:** "Parity tests confirm byte-for-byte identical output for 10,000 random strings across 50 languages. Performance tests show 40% lower p99 latency (120ms → 72ms), 3x throughput (1,200 → 3,600 tok/s), and 60% less peak memory (2.1GB → 0.84GB) on 8GB GPU. Edge case tests pass: empty strings, 10MB inputs, malformed UTF-8, emoji sequences."

#### 3.5.3 Test Command to Reproduce

```bash
# Exact command to run the test suite
pytest tests/test_tokenizer_*.py -v --tb=short
```

#### 3.5.4 Was It Proven Necessary?

**Answer honestly.** Did the tests reveal the change was actually needed? Or did they show the old way was fine?

> **Example:** "Yes. The Python tokenizer was the #1 bottleneck in profiling (flamegraph: 68% of request time). The Rust implementation eliminated this bottleneck. Without this change, we cannot meet the <100ms p99 SLA at 100 req/s."

---

### 3.6 Deployment / Rollout

| Aspect | Detail |
| -------- | -------- |
| Deployment date | 2026-09-15T14:30:00Z |
| Rollout strategy | Canary 5% → 25% → 100% over 4 hours |
| Rollback plan | Feature flag `tokenizer.backend=python` — instant revert |
| Monitoring | Datadog dashboard `tokenizer.latency`, `tokenizer.errors` |
| Post-deploy validation | 24h soak test at 100 req/s, zero errors |

---

### 3.7 Next Stage of Implementation

**What comes next? Did this implementation set us up for the goal?**

| Next Step | Description | Depends On This? | Target Date |
| ----------- | ------------- | ------------------ | ------------- |
| Batch tokenization on GPU | Move entire preprocessing to `candle` kernels | Yes — Rust tokenizer is prerequisite | 2026-Q4 |
| Speculative decoding integration | Use fast tokenizer for draft model | Yes — latency headroom created | 2027-Q1 |
| Multi-language benchmark suite | Expand parity tests to 100+ languages | No — independent | 2026-Q3 |

**Explicitly state:** *"This implementation [did / did not] set us up for [goal] because [reason]."*

---

### 3.8 What We Learned (Retrospective)

**Honest, specific, actionable.** This section is for the team, not for PR.

| Category | Learning |
| ---------- | ---------- |
| Technical | PyO3 GIL management is trickier than docs suggest — must explicitly release GIL in hot paths |
| Process | Parity testing against 10k strings caught 3 subtle Unicode bugs that unit tests missed |
| Tooling | `cargo-flamegraph` + `py-spy` together revealed the bottleneck was normalization, not BPE merges |
| Estimation | Initial estimate: 2 weeks. Actual: 3 weeks (PyO3 debugging + SIMD stability). Buffer 50% for FFI work. |
| Collaboration | Pairing with the Rust team early would have saved 1 week on SIMD intrinsics |

---

## 4. Document Types and Templates

### 4.1 Implementation Document (This Template)

Use for: New features, refactors, performance work, migrations.

**Required sections:** All of Section 3.

### 4.2 Bug Fix Document

**Additional required fields in header:**

```yaml
bug_id: "BUG-1234"
severity: "[critical | high | medium | low]"
root_cause: "[One sentence — what actually caused the bug]"
regression_risk: "[high | medium | low]"
```

**Replace Section 3.3 with:** "Fix Description" — what changed to resolve it.

**Add Section 3.9:** "Regression Prevention" — what test/guard prevents recurrence.

### 4.3 Architecture Decision Record (ADR)

**Header `document_type: "architecture"`**

**Required sections:**

- Context (what forces are at play)
- Decision (what we chose)
- Consequences (trade-offs, good and bad)
- Alternatives considered (with pros/cons)
- Literature grounding (same as 3.4.3)

### 4.4 Research Spike Document

**Header `document_type: "research"`**

**Required sections:**

- Question investigated
- Methodology
- Findings (with evidence)
- Recommendation (proceed / don't proceed / investigate further)
- Literature review (full citations)
- Time invested

### 4.5 Migration Document

**Header `document_type: "migration"`**

**Required sections:**

- From → To
- Migration strategy (big bang / strangler / parallel run)
- Rollback plan
- Data migration steps (if applicable)
- Validation criteria
- Timeline with milestones

### 4.6 Deprecation Document

**Header `document_type: "deprecation"`**

**Required sections:**

- What is deprecated
- Why (with evidence)
- Replacement
- Migration guide for users
- Sunset date
- Communication plan

---

## 5. Formatting Standards

### 5.1 Markdown Conventions

| Element | Standard |
| --------- | ---------- |
| Headings | ATX style (`#`, `##`, `###`) — no Setext |
| Code blocks | Fenced with language (` ```rust `) — always specify language |
| Inline code | Backticks for all identifiers, paths, commands |
| Links | Relative paths for repo files; absolute for external |
| Tables | GitHub-flavored markdown — align pipes |
| Lists | `-` for unordered; `1.` for ordered |
| Emphasis | `**bold**` for emphasis; `_italic_` for variables/placeholders |
| Comments | `<!-- comment -->` for internal notes (stripped in render) |

### 5.2 Code Example Standards

**Every code snippet MUST:**

- Be complete and runnable (or marked as excerpt with `...`)
- Include language tag
- Have a comment explaining purpose
- Reference the file/line it comes from

```rust
// src/tokenizer.rs:42-58 — Core BPE merge loop (SIMD-optimized)
pub fn merge_bpe(&self, tokens: &mut Vec<u32>) -> Result<(), TokenizerError> {
    // ... implementation
}
```

### 5.3 Plain Language Requirements

**All narrative text MUST pass the "Grandmother Test":** Could a smart non-technical person understand the *what* and *why*?

- Define acronyms on first use: "Multi-Head Attention (MHA)"
- Avoid: "leverage," "synergy," "paradigm," "holistic," "robust" (unless technically precise)
- Prefer: "use," "combine," "model," "complete," "reliable"
- Active voice: "We changed X" not "X was changed"
- Specific over vague: "40% latency reduction" not "significant improvement"

### 5.4 Mathematical Notation

- Inline: `$E = mc^2$` (MathJax-compatible)
- Block: `$$ ... $$`
- Define all symbols on first use

---

## 6. Version Control for Documentation

### 6.1 Documentation Lives With Code

- Docs are in the same repo as the code they describe
- Doc file path mirrors code structure: `docs/implementation/tokenizer-rust-migration.md`
- One doc per logical implementation (not per file)

### 6.2 Versioning

- Semantic versioning for documents: `MAJOR.MINOR.PATCH`
- `MAJOR` = structural change (sections added/removed)
- `MINOR` = content change (new learning, corrected data)
- `PATCH` = typo, formatting, link fix

### 6.3 History Tracking

Every document MUST maintain a **Changelog** at the end:

```markdown
## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0.0 | 2026-09-17 | @author | Initial version |
| 1.1.0 | 2026-09-20 | @reviewer | Added test results from soak test |
| 1.2.0 | 2026-10-01 | @author | Corrected memory numbers (was 0.84GB, actual 0.91GB) |
```

---

## 7. Review and Approval Process

### 7.1 Required Reviews

| Document Type | Minimum Reviewers | Required Expertise |
| --------------- | ------------------- | ------------------- |
| Implementation | 2 | 1 domain expert, 1 testing/QA |
| Bug Fix | 2 | 1 author, 1 domain expert |
| Architecture (ADR) | 3 | 2 senior engineers, 1 architect |
| Research | 1 | Domain expert |
| Migration | 3 | 1 author, 1 ops, 1 domain expert |
| Deprecation | 2 | 1 author, 1 product/stakeholder |

### 7.2 Review Checklist (Reviewers Verify)

- [ ] All required header fields present and accurate
- [ ] Line numbers in `related_files` match `git blame` output
- [ ] Literature citations are real, accessible, and correctly formatted
- [ ] Test evidence is specific, reproducible, and passes
- [ ] Plain language sections are jargon-free
- [ ] Next steps and learnings are honest and actionable
- [ ] Changelog updated

### 7.3 Approval Gate

**No merge without `status: approved` in header.** Draft docs can exist in feature branches but must be approved before merge to `main`.

---

## 8. Update Protocols

### 8.1 When to Update Documentation

| Trigger | Action | Timeline |
| --------- | -------- | ---------- |
| Code implementation merged | Create/update doc | Before merge (in same PR) |
| Bug discovered in production | Add to bug fix doc | Within 24h of fix |
| Test results change (regression) | Update test evidence section | Immediately |
| New learning from post-mortem | Add to "What We Learned" | Within 1 week |
| Architecture decision reversed | Create new ADR superseding old | Before reversal merge |
| Dependency version upgrade | Update config section | In same PR |

### 8.2 Superseding Documents

- Never delete — mark `status: superseded` and link to replacement
- New doc header includes: `supersedes: "old-doc-name.md"`
- Old doc header includes: `superseded_by: "new-doc-name.md"`

### 8.3 Periodic Review

- **Quarterly:** All `approved` docs reviewed for accuracy
- **Annually:** Archive docs older than 2 years with no recent references
- **On-demand:** When `related_files` change significantly

---

## 9. Enforcement

### 9.1 CI/CD Gates

The following checks run on every PR:

```yaml
# .github/workflows/docs-compliance.yml
- name: Verify documentation standards
  run: |
    python scripts/check_docs.py --require-header --require-tests --require-citations
```

**Checks include:**

- Header completeness
- Line number validity (`git blame` verification)
- Test file existence and passing status
- Citation format validation (DOI/arXiv regex)
- Plain language score (Flesch-Kincaid > 60 for summary sections)
- No `TODO` or `FIXME` in approved docs

### 9.2 Exceptions

**No exceptions.** If a standard cannot be met, the document type is wrong — use a Research Spike or ADR instead.

---

## 10. Quick Reference Checklist (For Authors)

Before submitting documentation for review:

- [ ] Header complete with all fields
- [ ] `date` and `implementation_date` in ISO 8601 UTC
- [ ] Every `related_files` entry has exact line numbers verified by `git blame`
- [ ] Executive Summary, Problem, Solution in plain language (Grandmother Test)
- [ ] Languages table with rationale
- [ ] "Push further" section filled
- [ ] Literature grounding: real citations with DOIs/arXiv IDs, or explicit "not grounded"
- [ ] Test files listed with lines, types, coverage
- [ ] Test results summarized in plain language with numbers
- [ ] Reproduction command provided
- [ ] "Proven necessary" answered honestly
- [ ] Deployment/rollout details
- [ ] Next steps table with dependencies
- [ ] Learnings table (technical, process, tooling, estimation, collaboration)
- [ ] Changelog started at v1.0.0
- [ ] All code snippets have language tags and file:line references
- [ ] Flesch-Kincaid > 60 for plain language sections
- [ ] No `TODO`, `FIXME`, `XXX` in narrative text

---

## 11. Example: Minimal Compliant Document

```markdown
---
title: "Add Health Check Endpoint"
document_type: "implementation"
version: "1.0.0"
date: "2026-09-17T02:00:00Z"
implementation_date: "2026-09-16T20:30:00Z"
author: "@janedoe"
reviewers: ["@johnsmith", "@alicewong"]
status: "approved"
related_files:
  - path: "src/server/health.rs"
    lines: "1-45"
    role: "primary"
  - path: "tests/test_health.py"
    lines: "1-30"
    role: "test"
related_issues: ["#88"]
related_documents: []
tags: ["observability", "health-check", "server"]
---

# Executive Summary

We added a `/health` endpoint so load balancers can detect unhealthy instances. This prevents routing traffic to crashed or overloaded servers.

# Problem Statement

The load balancer had no way to know if a server was healthy. It sent requests to dead instances, causing 502 errors for users.

# Solution Overview

Added a lightweight HTTP GET `/health` endpoint that returns 200 OK with JSON `{ "status": "healthy" }` if the server can reach its database and has memory headroom.

# Implementation Details

## Files Modified

| File | Lines | Change Type | Description |
|------|-------|-------------|-------------|
| `src/server/health.rs` | 1-45 | new | Health check handler |
| `src/server/routes.rs` | 12 | modified | Register `/health` route |
| `tests/test_health.py` | 1-30 | new | Integration tests |

## Languages Chosen and Why

| Language | Component | Rationale |
|----------|-----------|-----------|
| Rust | Handler | Consistent with server stack, zero overhead |

## Grounding in Verifiable Literature

This implementation was not directly grounded in published literature. It applies the standard Kubernetes liveness/readiness probe pattern.

## Configuration Changes

None.

# Testing Evidence

## Test Files

| Test File | Lines | Test Type | Coverage |
|-----------|-------|-----------|----------|
| `tests/test_health.py` | 1-30 | integration | 100% endpoint coverage |

## What Tests Showed

Tests confirm: 200 OK when DB reachable and memory < 90%; 503 when DB down or memory > 90%. Latency < 1ms p99.

## Test Command

```bash
pytest tests/test_health.py -v
```

## Was It Proven Necessary?

Yes. Without this, the load balancer cannot detect unhealthy instances. We observed 3 incidents in the last month where dead instances received traffic.

# Deployment / Rollout

| Aspect | Detail |
| -------- | -------- |
| Deployment date | 2026-09-16T20:30:00Z |
| Rollout strategy | Direct (no risk — read-only endpoint) |
| Rollback plan | Revert commit |
| Monitoring | Existing `/metrics` already scraped |

# Next Stage of Implementation

| Next Step | Description | Depends On This? | Target Date |
|-----------|-------------|------------------|-------------|
| Readiness probe | Separate `/ready` for startup/shutdown | Yes | 2026-Q4 |
| Dependency checks | Add Redis, queue health to probe | No | 2027-Q1 |

This implementation set us up for the goal (zero-downtime deployments) because health checks are a prerequisite for Kubernetes rolling updates.

# What We Learned

| Category | Learning |
| ---------- | ---------- |
| Technical | Keep health checks minimal — no external calls in the hot path |
| Process | Load balancer config change needed separate PR — coordinate earlier |
| Estimation | 2 hours estimated, 4 actual (load balancer config) |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0.0 | 2026-09-17 | @janedoe | Initial version |

```

---

## 12. Appendix: Citation Format Quick Reference

### arXiv Preprint
```

Author, A., Author, B. (Year). Title. arXiv:YYYY.NNNNN [cs.CL]. <https://arxiv.org/abs/YYYY.NNNNN>

```

### Peer-Reviewed Conference
```

Author, A., Author, B. (Year). Title. In Proceedings of Conference (pp. XXX-XXX). <https://doi.org/10.XXXX/XXXXXX>

```

### Peer-Reviewed Journal
```

Author, A., Author, B. (Year). Title. Journal Name, Volume(Issue), pp. XXX-XXX. <https://doi.org/10.XXXX/XXXXXX>

```

### Technical Report / Whitepaper
```

Organization. (Year). Title. Report Number. URL

```

---

**END OF DOCUMENTATION STANDARDS**

*This document is itself subject to these standards. See its own header for metadata.*
