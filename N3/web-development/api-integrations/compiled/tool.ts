export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "route outside /v1 with no breaking-change record", rule: "RULES 42" },
  { id: "B2", pattern: "response missing envelope code/message/requestId", rule: "RULES 37" },
  { id: "B3", pattern: "stack trace, SQL, or internal path in error response", rule: "RULES 38" },
  { id: "B4", pattern: "input reaches logic without edge validation", rule: "RULES 36" },
  { id: "B5", pattern: "growable list without cursor pagination", rule: "RULES 41" },
  { id: "B6", pattern: "webhook without provider event-id dedupe", rule: "RULES 40" },
  { id: "B7", pattern: "unsupported content type not rejected with 415", rule: "contract" },
  { id: "B8", pattern: "429 without Retry-After header", rule: "contract" },
  { id: "B9", pattern: "hand-written duplicate of generated contract types", rule: "RULES 7" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "api-integrations" };
  return result;
}
