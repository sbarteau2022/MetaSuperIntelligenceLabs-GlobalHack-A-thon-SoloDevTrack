---
name: cortex-typescript
description: "Apply these opinionated TypeScript conventions whenever writing or reviewing TypeScript: branded types for domain modeling, discriminated unions for state, `satisfies` over `as`, const objects instead of enums, type-only imports, and the strict tsconfig baseline beyond `strict: true`. Also use for monorepo project references and incremental build setup."
---

# cortex-typescript (interface)

## Purpose
TypeScript 5+ house conventions for code being written or changed: branded types
for domain modeling, discriminated unions for state, `satisfies` over `as`, const
objects instead of enums, type-only imports, strict tsconfig baseline beyond
`strict: true`, project references for monorepos.

## When to use
Writing or reviewing TypeScript. Monorepo project-reference and incremental-build
setup. Do not refactor untouched files to match unless asked.

## Out of scope
`enum` (use `as const` objects). `any` (use `unknown` and narrow; isolate any
genuinely unavoidable boundary case in one adapter function).

## Type signature
Compiled gate: `tool.rs` (`ts-conventions-gate`, schema: `tool.json`).
- `import_valid(import_style: String) -> bool` — "type-only" | "satisfies"
- `ts_gate(cfg: TsConventions) -> bool` — no_enum AND no_any AND brand_ok AND import_valid
- `make_ts(no_enum, no_any, brand_ok, import_style) -> TsConventions`

## Source
Trash source: `skills-desktop-web-development-20261030/kimi-frontend-kit/skills/cortex-typescript/SKILL.md`
(`71c37d1f7a9fadeb889731d998e11cfc6476e1616effecd1b7780bce4`). Full prose shrunk
to this interface 2026-10-31 (Agent A2). No tool/server claims in source; nothing
removed. See `RECEIPT.md`.
