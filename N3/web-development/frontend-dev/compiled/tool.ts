export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "div with role button", rule: "RULES 11, 12" },
  { id: "B2", pattern: "redundant role on native button", rule: "RULES 12" },
  { id: "B3", pattern: "redundant role list on ul", rule: "RULES 12" },
  { id: "B4", pattern: "fake heading: non-h element styled as heading", rule: "RULES 13, 14" },
  { id: "B5", pattern: "table used for layout", rule: "RULES 15" },
  { id: "B6", pattern: "aria-hidden true on focusable element or ancestor", rule: "RULES 18" },
  { id: "B7", pattern: "!important outside utilities cascade layer", rule: "RULES 24" },
  { id: "B8", pattern: "global reset selector", rule: "RULES 32" },
  { id: "B9", pattern: "literal tier custom property referenced in component code", rule: "RULES 25, 26" },
  { id: "B10", pattern: "hardcoded color, spacing, radius, or duration in component styles", rule: "RULES 27" },
  { id: "B11", pattern: "BEM naming to manage specificity", rule: "RULES 31" },
  { id: "B12", pattern: "nontrivial inline style", rule: "RULES 20" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "frontend-dev" };
  return result;
}
