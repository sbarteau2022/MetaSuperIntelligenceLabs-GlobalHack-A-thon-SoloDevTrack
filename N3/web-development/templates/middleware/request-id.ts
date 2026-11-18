/**
 * request-id.ts — pipeline stage 1/9 (request-id).
 *
 * Status: PROPOSED. Date: 2026-10-30.
 * Signature: Express-style `(req, res, next)`; framework-agnostic minimal
 * structural types below — zero dependencies, no framework import.
 * Runs FIRST: attaches `req.id` and echoes `X-Request-Id` so stages 2–9
 * (logging … error envelope) can correlate on one id.
 * Fail-closed: never throws; always generates an id when the client sends none.
 * Logic budget: 8 statement lines (3 in newId + 5 in requestId).
 * Conformance: GLOBAL-LOGIC.md §8, RULES.md rules 37/43/44, middleware/SKILL.md §4.
 */

export interface RequestLike {
  headers: Record<string, string | string[] | undefined>;
  id?: string;
}

export interface ResponseLike {
  setHeader(name: string, value: string): void;
}

export type NextFunctionLike = (err?: unknown) => void;

function newId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID();
  return `req-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e9).toString(36)}`;
}

export function requestId(req: RequestLike, res: ResponseLike, next: NextFunctionLike): void {
  const incoming = req.headers["x-request-id"];
  const id = Array.isArray(incoming) ? incoming[0] : (incoming ?? newId());
  req.id = id;
  res.setHeader("X-Request-Id", id);
  next();
}
