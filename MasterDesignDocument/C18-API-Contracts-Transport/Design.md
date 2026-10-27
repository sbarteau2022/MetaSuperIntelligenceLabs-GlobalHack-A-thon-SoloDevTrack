# C18 — API Contracts, Transport & Streaming

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Typed API surfaces, transport, streaming, and endpoint discovery.

## Subcategories (broad topics)

- C18.1 Endpoint catalog (OpenAI-compatible chat/completions, embed, tools, workflows, holes, mind, rag)
- C18.2 Request/response schemas and validation
- C18.3 Streaming transport + TTFT propagation
- C18.4 MCP runtime (lazy sessions, stdio/HTTP, preset catalog)
- C18.5 Cross-tool import (read-only scan + explicit import)
- C18.6 Lane/endpoint discovery and registration
- C18.7 Backward compatibility & versioning
- C18.8 Error model and retry/idempotency semantics

## Open questions / gaps

- No OpenAPI/schema doc exists yet; generate from current routes.

## Contract catalog (frozen for design)

| Surface          | Transport                 | Notes                                        |
| ---------------- | ------------------------- | -------------------------------------------- |
| Chat/completions | HTTP, streaming preferred | TTFT propagated in telemetry (F10)           |
| Embeddings       | HTTP                      | Honest BM25 fallback when missing (NF6)      |
| Tools            | MCP stdio/HTTP            | Lazy sessions, preset catalog                |
| Workflows        | HTTP                      | Packet/judge/retry/escalation                |
| Holes            | HTTP                      | create/run/diff/approve/merge + Merkle roots |
| Mind/memory      | HTTP                      | deposit/tail with provenance                 |
| RAG              | HTTP                      | BM25 always; dense when configured           |
| Model serving    | OpenAI-compatible         | Kill switch to standard transformer (C10.4)  |

### Error model (minimum)

- Deterministic hashes for refs; receipt-bearing success; idempotency keys for retry (C03.5, C08 NF5).
- 409 without preregistered plan; 400 without problem statement (doc-19 pattern).

## Formal API Specification (OpenAPI 3.1 + Session Types)

### Core Types

```yaml
# openapi.yaml (generated from Rust types)
openapi: 3.1.0
info:
  title: Ares Enterprise API
  version: 1.0.0
  description: Agentic memory-computation platform

components:
  schemas:
    # Receipt (every mutating response)
    Receipt:
      type: object
      required: [id, hash, timestamp, merkle_root]
      properties:
        id: { type: string, format: uuid }
        hash: { type: string, pattern: "^sha256:[a-f0-9]{64}$" }
        timestamp: { type: integer, format: unix-ms }
        merkle_root: { type: string, pattern: "^sha256:[a-f0-9]{64}$" }
        metadata: { type: object }

    # Idempotency Key
    IdempotencyKey:
      type: object
      required: [operation, agent_id, run_id, sequence, param_hash]
      properties:
        operation:
          {
            type: string,
            enum:
              [
                tool_call,
                hole_create,
                workflow_submit,
                rag_ingest,
                mind_deposit,
              ],
          }
        agent_id: { type: string }
        run_id: { type: string }
        sequence: { type: integer, minimum: 0 }
        param_hash: { type: string, pattern: "^sha256:[a-f0-9]{64}$" }

    # Error Response
    ErrorResponse:
      type: object
      required: [code, message, request_id]
      properties:
        code:
          { type: integer, enum: [400, 401, 403, 404, 409, 422, 429, 500, 503] }
        message: { type: string }
        request_id: { type: string, format: uuid }
        details: { type: object }
```

### Chat/Completions (OpenAI-Compatible + Streaming)

```yaml
paths:
  /v1/chat/completions:
    post:
      summary: Streaming chat completion with TTFT telemetry
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/ChatCompletionRequest"
      responses:
        "200":
          description: Streaming response (SSE)
          content:
            text/event-stream:
              schema:
                $ref: "#/components/schemas/StreamEvent"
        "400": { $ref: "#/components/responses/ErrorResponse" }
        "409": { $ref: "#/components/responses/ErrorResponse" } # No prereg plan
        "429": { $ref: "#/components/responses/ErrorResponse" }
      x-streaming: true
      x-ttft-telemetry: true

components:
  schemas:
    ChatCompletionRequest:
      type: object
      required: [model, messages]
      properties:
        model: { type: string, enum: [qwen3.8-27b, standard-transformer] }
        messages:
          { type: array, items: { $ref: "#/components/schemas/Message" } }
        stream: { type: boolean, default: true }
        lane_id: { type: string } # Optional lane routing
        idempotency_key: { $ref: "#/components/schemas/IdempotencyKey" }

    StreamEvent:
      oneOf:
        - type: object
          required: [type, data]
          properties:
            type: { type: string, enum: [token] }
            data: { type: string }
            ttft_ms: { type: integer } # Only on first event
        - type: object
          required: [type, receipt]
          properties:
            type: { type: string, enum: [receipt] }
            receipt: { $ref: "#/components/schemas/Receipt" }
        - type: object
          required: [type, error]
          properties:
            type: { type: string, enum: [error] }
            error: { $ref: "#/components/schemas/ErrorResponse" }
```

### Session Types for Streaming (Linear Logic)

The **streaming protocol** is a **session type** ensuring linear usage:

```rust
// Session type for chat completion stream
type ChatStream =
    !TTFT_ms.                    // Send TTFT (once)
    +. (                          // Internal choice: success or error
        ?Token.                   // Receive token (repeated)
        & ChatStream              // Recurse
    )
    +. ?Receipt.                  // Or receive final receipt
    +. ?ErrorResponse.            // Or receive error
    End
```

**Protocol guarantees**:

- Exactly one `TTFT_ms` sent before any tokens
- Tokens sent one at a time (backpressure via `?.`)
- Exactly one terminal message (`Receipt` or `ErrorResponse`)
- No resource leaks (linear type system)

### MCP Runtime (Lazy Sessions)

```yaml
/mcp/sessions:
  post:
    summary: Create lazy MCP session
    requestBody:
      content:
        application/json:
          schema:
            type: object
            required: [preset]
            properties:
              preset: { type: string, enum: [filesystem, git, web, custom] }
              config: { type: object }
    responses:
      "201":
        content:
          application/json:
            schema:
              type: object
              properties:
                session_id: { type: string, format: uuid }
                transport: { type: string, enum: [stdio, http] }
                endpoint: { type: string } # If HTTP
```

### Lane/Endpoint Discovery

```yaml
  /v1/lanes:
    get:
      summary: Discover available lanes
      responses:
        '200':
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/LaneInfo'

components:
  schemas:
    LaneInfo:
      type: object
      required: [id, name, tools, rag_collection, approval_policy]
      properties:
        id: {type: string}
        name: {type: string}
        tools: {type: array, items: {type: string}}
        rag_collection: {type: string}
        approval_policy: {type: string, enum: [none, hole, human]}
        status: {type: string, enum: [active, idle, busy, error]}
```

### Versioning & Compatibility

```yaml
# API versioning via header
parameters:
  - name: X-API-Version
    in: header
    schema: { type: string, pattern: '^v\d+$' }
    required: false

# Compatibility rules:
# - Minor versions: backward compatible (additive only)
# - Major versions: breaking changes, parallel deployment
# - Deprecation: 2 minor versions notice
```

### Error Codes (Extended)

| Code | Condition                                   | Retryable                 | Idempotent |
| ---- | ------------------------------------------- | ------------------------- | ---------- |
| 400  | Invalid request / missing problem statement | No                        | N/A        |
| 401  | Invalid auth / secret slot                  | No                        | N/A        |
| 403  | Role lacks permission                       | No                        | N/A        |
| 404  | Resource not found                          | No                        | N/A        |
| 409  | No preregistered plan / conflict            | No                        | N/A        |
| 422  | Validation failed                           | No                        | N/A        |
| 429  | Rate limited / backpressure                 | Yes (after `Retry-After`) | Yes        |
| 500  | Internal error                              | Yes (idempotent)          | Yes        |
| 503  | Kill switch active (fallback)               | Yes                       | Yes        |

---

## Contributor Deepening — Muse (2026-10-26)

### C18.9 Streaming backpressure contract (depth: TTFT honesty needs transport teeth)

C12.8 requires (ttft_cold, ttft_hit) reporting; this section makes the transport uphold it [PROPOSED — contract is my proposal; C12.8 is the authority]:

1. The `ttft_ms` field on the first StreamEvent (schema above) is REQUIRED, not optional: a stream emitting tokens before TTFT is a protocol violation, caught by the C18 session-type checker (exactly-one-TTFT-before-tokens is already the stated guarantee — this makes it enforced).
2. Backpressure: the `?.` (receive-token) step in the session type is the flow-control point. A client that stops receiving MUST trigger server-side suspension of that stream's compute (not buffering unboundedly). Unbounded server buffering under a stalled client is a DoS vector (C16 STRIDE) — bound the per-stream buffer (PROPOSED default: 64 tokens) and emit `backpressure_applied` telemetry (counts only).
3. Kill-switch interaction: HTTP 503 "Kill switch active" (error table above) MUST carry `Retry-After` + `fallback_available: true` so clients fail over to the standard path instead of retry-storming the ACFT path. A 503 without fallback indication is a retry amplifier.
4. Idempotency + streaming: a retried stream (same idempotency key) resumes from the last receipted token offset, not from zero — otherwise retries double-charge the token budget (C21). The resume offset is part of the Receipt schema (add `resume_offset` field).

### C18.10 Discovery authenticity (depth: lane discovery is a trust decision)

/v1/lanes lets any client enumerate capabilities — that is reconnaissance surface. Frozen rule [PROPOSED — rule is my proposal; C16 role matrix is the authority]:

- Lane discovery responses are filtered by caller role: Reader sees (id, name, status) only; tool lists, RAG collections, and approval policies require Contributor+. Full LaneInfo (schema above) is Operator+.
- An undiscoverable lane is still callable if addressed directly (discovery ≠ authorization — the approval gate enforces). Discovery filtering is anti-reconnaissance, not access control; document it as such so nobody mistakes obscurity for security.

What was NOT checked here: current route implementations (outside workspace); MCP preset catalog contents (C18.4 cites presets — definitions not in workspace).

## Open questions / gaps

- No OpenAPI/schema doc exists yet; generate from current routes.
