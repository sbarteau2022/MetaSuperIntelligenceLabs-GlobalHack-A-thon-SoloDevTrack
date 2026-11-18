export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "fat controller: model access plus domain rules plus shaping", rule: "RULES 36-39" },
  { id: "B2", pattern: "conditionals, calculations, or model access in route files", rule: "contract" },
  { id: "B3", pattern: "raw SQL string concatenation with input", rule: "RULES 36, 38" },
  { id: "B4", pattern: "req.body/query/params read without edge validator", rule: "RULES 36" },
  { id: "B5", pattern: "err.stack or driver message in production response", rule: "RULES 38" },
  { id: "B6", pattern: "server-side authorization missing (UI hiding only)", rule: "RULES 39" },
  { id: "B7", pattern: "mutating endpoint without Idempotency-Key or replay guard", rule: "RULES 40" },
  { id: "B8", pattern: "growable list endpoint without pagination", rule: "RULES 41" },
  { id: "B9", pattern: "endpoint outside /v1 with no breaking-change record", rule: "RULES 42" },
  { id: "B10", pattern: "secret or connection string in committed files, .env in VCS", rule: "RULES 9, 53" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "backend-systems" };
  return result;
}
