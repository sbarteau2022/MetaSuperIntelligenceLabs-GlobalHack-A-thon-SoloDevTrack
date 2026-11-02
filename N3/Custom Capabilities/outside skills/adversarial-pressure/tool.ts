class AttackFinding {
  severity: string;
  reproduced: boolean;
  has_input: boolean;
  constructor(severity: string, reproduced: boolean, has_input: boolean) {
    this.severity = severity;
    this.reproduced = reproduced;
    this.has_input = has_input;
  }
}
function severity_valid(severity: string): boolean {
  if (severity === "wrong-output") {
    return true;
  }
  if (severity === "spec-violation") {
    return true;
  }
  if (severity === "test-gap") {
    return true;
  }
  if (severity === "hygiene") {
    return true;
  }
  return false;
}
function finding_counts(finding: AttackFinding): boolean {
  if (finding.reproduced) {
    if (finding.has_input) {
      if (severity_valid(finding.severity)) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function pressure_gate(findings_rounds: number, kill_rate: number): boolean {
  if (findings_rounds >= 2) {
    if (kill_rate >= 70) {
      return true;
    }
    return false;
  }
  return false;
}
function make_finding(severity: string, reproduced: boolean, has_input: boolean): AttackFinding {
  return new AttackFinding(severity, reproduced, has_input);
}
