# EDGE-CASES.md

Date: 2026-10-30. Status: PROPOSED.
Purpose: failure modes that reliably appear in this stack, and the required handling.

---

## 1. Input & Validation

| Edge case | Symptom | Required handling |
|---|---|---|
| Empty body on POST | 500 from `req.body` destructuring | Guard for undefined body; 400 with error code |
| `null` vs missing vs `""` | Different code paths, same intent | Coerce once at the edge; reject empty-after-trim |
| Array where object expected | Silent `undefined` fields | Schema validation; 422 |
| Extremely long string | DB truncation or 500 | Max length at edge + DB constraint |
| Unicode / RTL / emoji in names | Layout break, sort break | UTF-8 end-to-end; test with real samples |
| Numbers as strings (`"5"` vs `5`) | Loose comparison bugs | Validate type, not just truthiness |
| Duplicate params (`?a=1&a=2`) | Ambiguous parsing | Define policy: first, last, or reject |
| Whitespace-only input | Passes `if (value)` | Trim, then validate |

## 2. Auth & Sessions

| Edge case | Symptom | Required handling |
|---|---|---|
| Token expired mid-request | 401 only on next call | Check expiry before work; return 401 with `code: token_expired` |
| Session fixation on login | Attacker-supplied id survives | Rotate session ID on login |
| Role change while session live | Stale permissions | Re-check authorization per request, not at login |
| Clock skew between services | Token rejected early | Allow small leeway (±60s) on `exp`/`nbf` |
| Logout on one device only | User expects global logout | Document scope; offer "log out all sessions" if required |
| CSRF on cookie-auth POST | Cross-site state change | SameSite cookies + CSRF token for unsafe methods |
| `disabled` button as authorization | Server still processes request | `disabled` is UI only; enforce server-side (RULES 39) |

## 3. Data & Consistency

| Edge case | Symptom | Required handling |
|---|---|---|
| Concurrent writes to same row | Lost update | Optimistic locking (version column) or transaction lock |
| Duplicate request / double-click | Two records | Idempotency key on writes |
| Unique constraint violation | 500 | Catch and return 409 with the conflicting field |
| Migration runs while old code serves | Column missing/extra | Expand/contract; both versions work during deploy |
| Clock skew in audit timestamps | Unreliable ordering | Store UTC; order by server sequence, not client time |
| Soft-deleted row still returned | "Zombie" records | Centralize the filter in the query layer, not per-caller |
| Cache stale after write | User sees old value | Invalidate on write, or version the key |
| Timezone of "today" | Off-by-one day | Store UTC, compute in the user's timezone explicitly |

## 4. API & Network

| Edge case | Symptom | Required handling |
|---|---|---|
| Upstream timeout | Hung request | Client timeout + retry with backoff + circuit breaker |
| Retry on non-idempotent write | Duplicate side effect | Retry only idempotent ops, or use idempotency key |
| Partial page of results | Missing rows | Cursor pagination for changing datasets; offset only for stable ones |
| Response larger than memory | Crash | Stream, paginate, or cap the response size |
| Client disconnects mid-upload | Orphaned temp file | Cleanup on abort + scheduled sweep |
| 4xx retried by a proxy | Amplified load | Return 429 with `Retry-After`; make 4xx non-retryable for the proxy |
| Webhook delivered twice | Double processing | Dedupe on the provider's event ID |
| Webhook arrives out of order | State regression | Version/sequence check on the consumer |
| Payload in the wrong content type | Parse error | 415 with the expected type |

## 5. Frontend & UI

| Edge case | Symptom | Required handling |
|---|---|---|
| Content longer than expected | Layout break | `min-width: 0` on flex children; test with long strings |
| User zooms to 200% | Content clipped | Layout in `rem`/`%`; no fixed-height text containers |
| Slow network | Layout shift | Reserve space for media and async content |
| JS fails to load | Blank page | Progressive enhancement; content visible without JS where possible |
| Offline | Silent failure | Detect and show an explicit offline state |
| Dark mode with hardcoded colors | Unreadable | Only semantic tokens; test both themes |
| Forced colors mode | Invisible custom controls | Use system colors; test with the setting enabled |
| Reduced motion | Vestibular trigger | Honor `prefers-reduced-motion` |
| Container style query unsupported | Layout ignores density | `data-density` attribute fallback with `:where()` |
| Modal opens with focus left behind | Keyboard user stranded | Focus the dialog; trap and restore focus; support Esc |
| `aria-hidden` on a focusable element | Focus lands on nothing | Never combine |
| Table with spanned cells | Screen reader confusion | Simplify the table instead of adding ARIA |
| Screen reader reads duplicated list semantics | Verbose output | `role="list"` only when `list-style: none`/flex removed semantics |
| Icon-only button | No accessible name | `aria-label`; mark decorative SVG `aria-hidden="true"` |
| Form submitted before validation | Data loss | Validate on submit and on blur; never clear input on error |
| Slow network, double submit | Duplicate record | Disable the submit control while in flight |
| Timezone/clock display | Wrong local time | Format with the user's timezone explicitly |
| Empty list | Blank space with no guidance | Explicit empty state, not a blank container |

## 6. Security

| Edge case | Symptom | Required handling |
|---|---|---|
| IDOR (changing a record id) | Data leak | Authorize against the record's owner/tenant, not the request id |
| Mass assignment | Unintended field writes | Explicit allowlist of writable fields |
| Path traversal in file access | Arbitrary file read | Resolve and verify the path stays under the root |
| XSS via stored content | Script execution | Sanitize at render; never inject raw HTML |
| Open redirect | Phishing | Allowlist of redirect targets |
| Secret in logs or client bundle | Leak | Server-only secrets; scrub logs; check the bundle |
| Regex denial of service (ReDoS) | Worker exhaustion | Avoid nested quantifiers; bound input length; timeout |
| Dependency compromise | Supply-chain risk | Pin versions; lockfile; audit in CI |
| Enumeration via error differences | Info leak | Uniform error envelope for auth failures |

## 7. Performance

| Edge case | Symptom | Required handling |
|---|---|---|
| Hydration mismatch | Console errors, lost state | Render identically on server and client; no `Date.now()`/random in render |
| Layout shift from late content | CLS regression | Reserve space; skeleton with the final dimensions |
| Unbounded list render | Slow page | Virtualize or paginate |
| Long tasks on the main thread | INP regression | Break up work; move to a worker |
| Too many preconnects | Slow startup | Preconnect only to origins actually used |
| Oversized images | LCP regression | Correct dimensions, modern format, `fetchpriority` on LCP |

---

## Required Test Coverage Mapping

Every edge case above that a given feature can trigger MUST have either a test or a recorded reason it cannot occur. Coverage is tracked per feature, not per document.
