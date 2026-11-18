export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "auth duplicated inline in route handlers instead of one guard", rule: "RULES 39, 43" },
  { id: "B2", pattern: "password, token, key, or card number in log line", rule: "RULES 44" },
  { id: "B3", pattern: "body parser without explicit size cap", rule: "RULES 36" },
  { id: "B4", pattern: "outbound call or handler without deadline", rule: "contract" },
  { id: "B5", pattern: "err.stack or driver message forwarded to client", rule: "RULES 38" },
  { id: "B6", pattern: "per-route security headers instead of one middleware", rule: "contract" },
  { id: "B7", pattern: "wildcard origin combined with credentialed requests", rule: "contract" },
  { id: "B8", pattern: "silent rate-limit bypass by IP trust", rule: "contract" },
  { id: "B9", pattern: "middleware importing domain services", rule: "RULES 43" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "middleware" };
  return result;
}
