export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "landed migration edited in place", rule: "RULES 45" },
  { id: "B2", pattern: "breaking schema change without expand/contract dual-compat", rule: "RULES 46" },
  { id: "B3", pattern: "hand-edited production rows instead of reviewed script", rule: "RULES 47" },
  { id: "B4", pattern: "hot-path filter/sort/join without index plus EXPLAIN record", rule: "RULES 48" },
  { id: "B5", pattern: "local-time storage without offset, server-local timezone reliance", rule: "contract" },
  { id: "B6", pattern: "ad-hoc deleted_at filter instead of shared query-layer scope", rule: "contract" },
  { id: "B7", pattern: "secret or credential inside migration or seed script", rule: "RULES 44" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "database-data" };
  return result;
}
