/**
 * validate.ts — pipeline stage 7/9 (validation).
 *
 * Status: PROPOSED. Date: 2026-10-30.
 * Signature: Express-style `(req, _res, next)` via `validateBody(schema)`;
 * framework-agnostic minimal types — zero dependencies, usable without any
 * schema library. Runs AFTER auth (6), BEFORE handler (8).
 * Guard model: explicit field allowlist + primitive type checks.
 * Fail-closed: unknown fields, missing required fields, wrong types, or a
 * non-object body REJECT with 400 before the handler ever runs (rule 36).
 * Conformance: GLOBAL-LOGIC.md §8, RULES.md rules 36/37/43, middleware/SKILL.md §4.
 */

export type FieldType = "string" | "number" | "boolean";

export interface FieldRule {
  type: FieldType;
  required?: boolean;
}

export type Schema = Record<string, FieldRule>;

export interface ValidationSuccess {
  ok: true;
  value: Record<string, string | number | boolean>;
}

export interface ValidationFailure {
  ok: false;
  errors: string[];
}

export function validate(body: unknown, schema: Schema): ValidationSuccess | ValidationFailure {
  const errors: string[] = [];
  if (typeof body !== "object" || body === null || Array.isArray(body)) {
    return { ok: false, errors: ["body must be a JSON object"] };
  }
  const input = body as Record<string, unknown>;
  for (const key of Object.keys(input)) {
    if (!(key in schema)) errors.push(`unknown field: ${key}`);
  }
  const value: Record<string, string | number | boolean> = {};
  for (const [key, rule] of Object.entries(schema)) {
    const v = input[key];
    if (v === undefined) {
      if (rule.required) errors.push(`missing required field: ${key}`);
      continue;
    }
    if (typeof v !== rule.type) {
      errors.push(`field ${key} must be ${rule.type}`);
      continue;
    }
    value[key] = v as string | number | boolean;
  }
  return errors.length > 0 ? { ok: false, errors } : { ok: true, value };
}

export function validateBody(schema: Schema): (req: { body?: unknown }, _res: unknown, next: (err?: unknown) => void) => void {
  return (req, _res, next): void => {
    const result = validate(req.body, schema);
    if (!result.ok) {
      next({ status: 400, code: "BAD_REQUEST", message: result.errors.join("; ") });
      return;
    }
    req.body = result.value;
    next();
  };
}
