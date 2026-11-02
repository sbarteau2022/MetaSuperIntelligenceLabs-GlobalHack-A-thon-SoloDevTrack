class PolyglotFinding {
  cwe: string;
  entropy_high: boolean;
  compiled: boolean;
  constructor(cwe: string, entropy_high: boolean, compiled: boolean) {
    this.cwe = cwe;
    this.entropy_high = entropy_high;
    this.compiled = compiled;
  }
}
function cwe_valid(cwe: string): boolean {
  if (cwe === "CWE-78") {
    return true;
  }
  if (cwe === "CWE-89") {
    return true;
  }
  if (cwe === "CWE-798") {
    return true;
  }
  return false;
}
function polyglot_gate(cfg: PolyglotFinding): boolean {
  if (cwe_valid(cfg.cwe)) {
    if (cfg.entropy_high) {
      if (cfg.compiled) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_polyglot(cwe: string, entropy_high: boolean, compiled: boolean): PolyglotFinding {
  return new PolyglotFinding(cwe, entropy_high, compiled);
}
