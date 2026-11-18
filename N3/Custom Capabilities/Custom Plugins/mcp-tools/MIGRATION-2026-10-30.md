# Good tools migration 2026-10-30
Migrated, PASS per tools audit 2026-10-30:
- tool-router: route/search/execute, truncation, receipts, token cap - THE STANDARD
- language-toolchain: detect + plan + run language checks - PASS with receipt gap
- engineering-toolchain: detect + plan + run language checks - PASS with receipt gap
- ts-to-rust: TS 7.1beta strict -> Rust, ts_check + ts_to_rust - tool_write chain
- hypermind-mcp: via router only, NOT direct - no direct copy migrated
Removed, NOT migrated:
- optiq-harness / optiq bridge: harness gone, bridge already trashed
- autonomous_runbook_mcp: GAP, no bounded output/judge per loop, wrap with router first, not migrated as-is
- zed_research_sources: not PASS, not migrated
- AtomaApp mcp-servers config jsons: duplicates of OD/Zed configs, not tools
Canonical home is N3. Agents: attack these migrated good tools next - convert to TS->Rust new style, with boolean gates, before send.
