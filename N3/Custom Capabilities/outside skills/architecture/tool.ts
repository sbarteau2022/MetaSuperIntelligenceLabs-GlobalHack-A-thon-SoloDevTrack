class ArchDecision {
  requirements_set: boolean;
  options_count: number;
  decided: boolean;
  risky_sliced: boolean;
  constructor(requirements_set: boolean, options_count: number, decided: boolean, risky_sliced: boolean) {
    this.requirements_set = requirements_set;
    this.options_count = options_count;
    this.decided = decided;
    this.risky_sliced = risky_sliced;
  }
}
function options_enough(options_count: number): boolean {
  if (options_count >= 2) {
    return true;
  }
  return false;
}
function arch_gate(cfg: ArchDecision): boolean {
  if (cfg.requirements_set) {
    if (options_enough(cfg.options_count)) {
      if (cfg.decided) {
        if (cfg.risky_sliced) {
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
function review_verdict(blockers: number, should_fix: number): string {
  if (blockers > 0) {
    return "do-not-merge";
  }
  if (should_fix > 0) {
    return "approve-with-changes";
  }
  return "approve";
}
function make_arch(requirements_set: boolean, options_count: number, decided: boolean, risky_sliced: boolean): ArchDecision {
  return new ArchDecision(requirements_set, options_count, decided, risky_sliced);
}
