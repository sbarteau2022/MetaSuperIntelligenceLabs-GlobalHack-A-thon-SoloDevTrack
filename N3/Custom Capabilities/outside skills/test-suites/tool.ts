class SuitePlan {
  worked_done: boolean;
  boundary_done: boolean;
  oracle_done: boolean;
  kill_rate: number;
  constructor(worked_done: boolean, boundary_done: boolean, oracle_done: boolean, kill_rate: number) {
    this.worked_done = worked_done;
    this.boundary_done = boundary_done;
    this.oracle_done = oracle_done;
    this.kill_rate = kill_rate;
  }
}
function kill_ok(kill_rate: number): boolean {
  if (kill_rate >= 90) {
    return true;
  }
  return false;
}
function grade_cap(has_wrong_output: boolean, kill_rate: number, spec_violated: boolean): string {
  if (has_wrong_output) {
    return "C-cap";
  }
  if (spec_violated) {
    return "B-cap";
  }
  if (kill_rate >= 70) {
    return "pass";
  }
  return "C-plus-cap";
}
function suite_gate(cfg: SuitePlan): boolean {
  if (cfg.worked_done) {
    if (cfg.boundary_done) {
      if (cfg.oracle_done) {
        if (kill_ok(cfg.kill_rate)) {
          return true;
        }
        return false;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_suite(worked_done: boolean, boundary_done: boolean, oracle_done: boolean, kill_rate: number): SuitePlan {
  return new SuitePlan(worked_done, boundary_done, oracle_done, kill_rate);
}
