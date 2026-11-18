# Middleware Pipeline Order

Status: PROPOSED. Date: 2026-10-30.
Source: `middleware/SKILL.md` §4 (rule 43: order explicit and documented).
No stage moves without updating this file AND `middleware/SKILL.md`.

| # | Stage | One-line responsibility | Fail-closed note |
|---|-------|------------------------|------------------|
| 1 | request-id | Attach/propagate `X-Request-Id` (`req.id`) for log↔error correlation. | Never throws; always emits an id when the client sends none. |
| 2 | logging | One scrubbed request log line (method, route, requestId). | Never logs secrets/tokens/cards (rule 44); a logging error is surfaced, never silent. |
| 3 | security headers | One middleware sets all response security headers. | Missing header config fails at boot, not per-request. |
| 4 | CORS | Explicit origin allowlist; never wildcard + credentials. | Unknown origin gets no CORS headers, so the browser blocks. |
| 5 | rate-limit | Uniform limits; no silent IP bypasses. | Limiter error denies (429/503); it never becomes an allow. |
| 6 | auth | Single `requireAuth` guard, server-side, every request. | Missing/invalid credential → 401 with no reason leaked. |
| 7 | validation | Schema/allowlist check at the edge, before handlers. | Unknown field, wrong type, or oversized body → 400, handler never runs. |
| 8 | handler | Route/controller domain logic (owned by backend-systems). | Uncaught throw → stage 9 as generic 500, never raw. |
| 9 | error envelope | Central handler maps ALL errors to `{code, message, requestId, details?}`. | 500s carry a generic message; no stack/SQL/paths leak (rule 38). |

What was NOT checked: runtime wiring against a live framework (template only).
