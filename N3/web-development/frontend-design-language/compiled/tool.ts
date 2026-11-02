export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "tier-1 literal value in component code", rule: "RULES 26" },
  { id: "B2", pattern: "hardcoded color, spacing, radius, or duration in component styles", rule: "RULES 27" },
  { id: "B3", pattern: "new literal added to component without token registration first", rule: "RULES 7" },
  { id: "B4", pattern: "!important outside utilities cascade layer", rule: "RULES 24" },
  { id: "B5", pattern: "global reset selector", rule: "RULES 32" },
  { id: "B6", pattern: "BEM naming to manage specificity", rule: "RULES 31" },
  { id: "B7", pattern: "redundant ARIA forwarded from spec", rule: "RULES 11, 12" },
  { id: "B8", pattern: "physical property where a logical property exists", rule: "RULES 29" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "frontend-design-language" };
  return result;
}
