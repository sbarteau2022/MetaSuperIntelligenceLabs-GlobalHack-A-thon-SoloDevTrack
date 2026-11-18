export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "latest tag or unpinned image/action/dependency", rule: "GLOBAL-LOGIC 8" },
  { id: "B2", pattern: "required env var missing from documented env matrix", rule: "contract" },
  { id: "B3", pattern: "default value for a secret", rule: "RULES 53" },
  { id: "B4", pattern: ".env or secret material committed to VCS", rule: "RULES 9" },
  { id: "B5", pattern: "build script without typecheck step", rule: "contract" },
  { id: "B6", pattern: "release without health-check, log destination, alert, rollback notes", rule: "contract" },
  { id: "B7", pattern: "dependency/secret audit missing from pipeline invocation", rule: "contract" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "devops-deployment" };
  return result;
}
