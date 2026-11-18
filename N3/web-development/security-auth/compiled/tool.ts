export interface BanRule { id: string; pattern: string; rule: string; }
export interface GateInput { bannedHitCount: number; contractOnDisk: boolean; auditClean: boolean; }
export interface GateResult { pass: boolean; skill: string; }
export const BAN_RULES: BanRule[] = [
  { id: "B1", pattern: "client-side auth check as sole gate", rule: "contract" },
  { id: "B2", pattern: "authorization missing on server request path (UI hiding only)", rule: "RULES 39" },
  { id: "B3", pattern: "plaintext, reversible, fast-hash, or globally-salted password", rule: "RULES 54" },
  { id: "B4", pattern: "session identifier not rotated on login, logout, or role change", rule: "RULES 55" },
  { id: "B5", pattern: "session cookie missing HttpOnly, Secure, or SameSite", rule: "RULES 56" },
  { id: "B6", pattern: "cookie-auth state-changing request without anti-CSRF token", rule: "contract" },
  { id: "B7", pattern: "post-login redirect not validated against allowlist", rule: "contract" },
  { id: "B8", pattern: "untrusted input rendered as raw HTML without sanitizer", rule: "RULES 36" },
  { id: "B9", pattern: "secret, token, or hash in logs or error responses", rule: "RULES 44, 38" },
  { id: "B10", pattern: "request body bound directly to model without allowlist", rule: "contract" },
  { id: "B11", pattern: "unvalidated user input in filesystem path", rule: "contract" },
  { id: "B12", pattern: "public release without recorded OWASP review", rule: "RULES 57" }
];
export function evaluate(input: GateInput): GateResult {
  let pass: boolean = true;
  if (input.bannedHitCount > 0) { pass = false; }
  if (input.contractOnDisk) { } else { pass = false; }
  if (input.auditClean) { } else { pass = false; }
  const result: GateResult = { pass: pass, skill: "security-auth" };
  return result;
}
