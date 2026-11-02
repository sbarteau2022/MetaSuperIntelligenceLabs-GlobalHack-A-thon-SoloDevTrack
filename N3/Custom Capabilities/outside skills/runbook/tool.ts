class RunbookPacket {
  judge_set: boolean;
  pass_set: boolean;
  reversible: boolean;
  budget_set: boolean;
  constructor(judge_set: boolean, pass_set: boolean, reversible: boolean, budget_set: boolean) {
    this.judge_set = judge_set;
    this.pass_set = pass_set;
    this.reversible = reversible;
    this.budget_set = budget_set;
  }
}
function packet_complete(cfg: RunbookPacket): boolean {
  if (cfg.judge_set) {
    if (cfg.pass_set) {
      if (cfg.reversible) {
        if (cfg.budget_set) {
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
function stop_valid(stop: string): boolean {
  if (stop === "unparseable_input") {
    return true;
  }
  if (stop === "spin_detected") {
    return true;
  }
  if (stop === "cap_exhausted") {
    return true;
  }
  if (stop === "judge_unavailable") {
    return true;
  }
  if (stop === "scope_boundary") {
    return true;
  }
  if (stop === "irreversible_needs_commander") {
    return true;
  }
  return false;
}
function repair_budget_valid(passes: number): boolean {
  if (passes >= 0) {
    if (passes <= 5) {
      return true;
    }
    return false;
  }
  return false;
}
function make_packet(judge_set: boolean, pass_set: boolean, reversible: boolean, budget_set: boolean): RunbookPacket {
  return new RunbookPacket(judge_set, pass_set, reversible, budget_set);
}
