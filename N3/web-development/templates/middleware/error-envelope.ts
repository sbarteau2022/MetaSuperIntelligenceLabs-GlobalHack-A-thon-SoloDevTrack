/**
 * error-envelope.ts — pipeline stage 9/9 (error envelope).
 *
 * Status: PROPOSED. Date: 2026-10-30.
 * Signature: Express-style `(err, req, res, next)`; framework-agnostic minimal
 * structural types below — zero dependencies, no framework import.
 * Runs LAST: maps ALL errors to one envelope {code, message, requestId, details?}.
 * Fail-closed: 500s carry a generic message; stack traces, SQL, and internal
 * paths NEVER reach the client (RULES.md rule 38).
 * Conformance: GLOBAL-LOGIC.md §8, RULES.md rules 37/38/43, middleware/SKILL.md §4.
 */

export interface ErrorEnvelope {
  code: string;
  message: string;
  requestId?: string;
  details?: unknown;
}

export interface EnvelopeResult {
  status: number;
  body: { error: ErrorEnvelope };
}

export interface RequestIdCarrier {
  id?: string;
}

export interface ErrorCarrier {
  status?: number;
  code?: string;
  message?: string;
}

export interface ResponseWriter {
  status(code: number): { json(body: unknown): void };
}

export type ErrorNextFunction = (err?: unknown) => void;

function build(status: number, code: string, message: string, requestId?: string, details?: unknown): EnvelopeResult {
  const error: ErrorEnvelope = { code, message };
  if (requestId !== undefined) error.requestId = requestId;
  if (details !== undefined) error.details = details;
  return { status, body: { error } };
}

export function badRequest(message: string, requestId?: string, details?: unknown): EnvelopeResult {
  return build(400, "BAD_REQUEST", message, requestId, details);
}

export function unauthorized(requestId?: string): EnvelopeResult {
  return build(401, "UNAUTHORIZED", "Authentication required.", requestId);
}

export function forbidden(requestId?: string): EnvelopeResult {
  return build(403, "FORBIDDEN", "You do not have access.", requestId);
}

export function notFound(requestId?: string): EnvelopeResult {
  return build(404, "NOT_FOUND", "Resource not found.", requestId);
}

export function internal(requestId?: string): EnvelopeResult {
  return build(500, "INTERNAL", "Something went wrong.", requestId);
}

export function errorMiddleware(err: ErrorCarrier, req: RequestIdCarrier, res: ResponseWriter, _next: ErrorNextFunction): void {
  const status = err.status ?? 500;
  const result = status === 500
    ? internal(req.id)
    : build(status, err.code ?? "UNKNOWN", err.message ?? "Request failed.", req.id);
  res.status(result.status).json(result.body);
}
