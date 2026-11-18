export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "placeholder copy presented as final spec text", rule: "contract" },
  { id: "B2", pattern: "flow missing happy, error, empty, or loading state", rule: "RULES 50" },
  { id: "B3", pattern: "mockup motion not achievable with registered tokens", rule: "contract" },
  { id: "B4", pattern: "component spec without a11y notes block", rule: "RULES 11-24" },
  { id: "B5", pattern: "non-native control pattern in wireframes or prototypes", rule: "RULES 11, 12" },
  { id: "B6", pattern: "table for layout, tabular data without table plus th scope", rule: "RULES 15" },
  { id: "B7", pattern: "skipped heading level or div styled as heading", rule: "RULES 13, 14" },
  { id: "B8", pattern: "interactive target below 24x24 CSS px", rule: "RULES 22" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "ui-ux-flow" };
  return result;
}
