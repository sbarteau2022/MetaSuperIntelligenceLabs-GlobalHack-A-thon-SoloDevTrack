export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "suite covering only the happy path", rule: "RULES 50" },
  { id: "B2", pattern: "unit test with network or filesystem access", rule: "RULES 52" },
  { id: "B3", pattern: "pass claim without pasted checker output", rule: "RULES 51" },
  { id: "B4", pattern: "failing test deleted or commented out to force green", rule: "RULES 4, 49" },
  { id: "B5", pattern: "bug fixed without reproduction test first", rule: "RULES 49" },
  { id: "B6", pattern: "unlabeled coverage claim without NOT-checked statement", rule: "RULES 58" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "testing-qa" };
  return result;
}
